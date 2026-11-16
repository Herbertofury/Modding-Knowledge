//! Narrow Skyrim SE plugin decoder: real WEAP DATA values + EDID, not full xEdit.
//! WEAP DATA layout verified against TES5Edit Core/wbDefinitionsTES5.pas.
//! Unknown subrecords remain opaque; compressed records are explicitly unparsed.
use crate::{Entity, Snapshot, Value};
use std::collections::BTreeMap;
const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_RECORDS: usize = 100_000;
fn u32_at(b: &[u8], p: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        b.get(p..p + 4).ok_or("truncated u32")?.try_into().unwrap(),
    ))
}
fn tag(b: &[u8]) -> Result<String, String> {
    if b.len() != 4
        || !b
            .iter()
            .all(|v| v.is_ascii_uppercase() || v.is_ascii_digit() || *v == b'_')
    {
        return Err("invalid signature".into());
    }
    Ok(String::from_utf8(b.to_vec()).unwrap())
}
fn text(b: &[u8]) -> Result<String, String> {
    let x = b.strip_suffix(&[0]).ok_or("unterminated string")?;
    if x.contains(&0) {
        return Err("embedded null".into());
    }
    String::from_utf8(x.to_vec()).map_err(|_| "unsupported string encoding".into())
}
fn subs<'a>(
    mut b: &'a [u8],
    cancel: &impl Fn() -> bool,
    budget: &mut usize,
) -> Result<Vec<(String, &'a [u8])>, String> {
    let mut out = vec![];
    let mut extended = None;
    while !b.is_empty() {
        if cancel() {
            return Err("cancelled".into());
        }
        if *budget == 0 {
            return Err("subrecord count bound".into());
        }
        *budget -= 1;
        if b.len() < 6 {
            return Err("truncated subrecord".into());
        }
        let name = tag(&b[..4])?;
        let small = u16::from_le_bytes([b[4], b[5]]) as usize;
        b = &b[6..];
        if name == "XXXX" {
            if extended.is_some() || small != 4 {
                return Err("invalid XXXX".into());
            }
            extended = Some(u32_at(b, 0)? as usize);
            b = b.get(4..).ok_or("truncated XXXX")?;
            continue;
        }
        let size = extended.take().unwrap_or(small);
        let payload = b.get(..size).ok_or("truncated subrecord payload")?;
        out.push((name, payload));
        b = &b[size..];
    }
    if extended.is_some() {
        return Err("orphan XXXX".into());
    }
    Ok(out)
}
struct Parser<'a, F: Fn() -> bool> {
    out: Snapshot,
    masters: Vec<String>,
    name: String,
    cancel: &'a F,
    count: usize,
    subrecord_budget: usize,
}
impl<F: Fn() -> bool> Parser<'_, F> {
    fn records(&mut self, mut b: &[u8], depth: usize) -> Result<(), String> {
        if depth > 32 {
            return Err("group nesting limit".into());
        }
        while !b.is_empty() {
            if (self.cancel)() {
                return Err("cancelled".into());
            }
            if b.len() < 24 {
                return Err("truncated record header".into());
            }
            let sig = tag(&b[..4])?;
            let size = u32_at(b, 4)? as usize;
            if sig == "GRUP" {
                if size < 24 {
                    return Err("invalid group size".into());
                }
                let group = b.get(24..size).ok_or("truncated group")?;
                self.records(group, depth + 1)?;
                b = &b[size..];
                continue;
            }
            let end = 24usize.checked_add(size).ok_or("record overflow")?;
            let payload = b.get(24..end).ok_or("truncated record payload")?;
            self.count += 1;
            if self.count > MAX_RECORDS {
                return Err("record count limit".into());
            }
            let flags = u32_at(b, 8)?;
            let id = u32_at(b, 12)?;
            if sig == "TES4" {
                return Err("unexpected nested/repeated TES4".into());
            }
            let index = (id >> 24) as usize;
            let origin = if index < self.masters.len() {
                &self.masters[index]
            } else if index == self.masters.len() {
                &self.name
            } else {
                return Err("unresolved FormID master index".into());
            };
            let key = format!("{}:{:06X}", origin.to_lowercase(), id & 0xffffff);
            let mut e = Entity {
                fields: BTreeMap::from([
                    ("RecordType".into(), Value::Text(sig.clone())),
                    ("RecordFlags".into(), Value::Unsigned(flags as u64)),
                ]),
                opaque: BTreeMap::from([("RecordMetadata".into(), b[16..24].to_vec())]),
                limitations: vec![],
            };
            if flags & 0x40000 != 0 {
                e.opaque
                    .insert("compressed_payload".into(), payload.to_vec());
                e.limitations
                    .push("compressed record body not decoded".into());
            } else {
                let mut occurrences: BTreeMap<String, usize> = BTreeMap::new();
                for (name, data) in subs(payload, self.cancel, &mut self.subrecord_budget)? {
                    if (self.cancel)() {
                        return Err("cancelled".into());
                    }
                    let i = occurrences.entry(name.clone()).or_default();
                    let path = format!("{name}[{i}]");
                    *i += 1;
                    if name == "EDID" {
                        match text(data) {
                            Ok(v) => {
                                e.fields.insert(path, Value::Text(v));
                            }
                            Err(reason) => {
                                e.opaque.insert(path, data.to_vec());
                                e.limitations.push(reason);
                            }
                        }
                    } else if sig == "WEAP" && name == "DATA" && data.len() == 10 {
                        e.fields.insert(
                            format!("{path}.Value"),
                            Value::Unsigned(u32_at(data, 0)? as u64),
                        );
                        e.fields
                            .insert(format!("{path}.Weight"), Value::FloatBits(u32_at(data, 4)?));
                        e.fields.insert(
                            format!("{path}.Damage"),
                            Value::Unsigned(u16::from_le_bytes([data[8], data[9]]) as u64),
                        );
                    } else {
                        if sig == "WEAP" && name == "DATA" {
                            e.limitations
                                .push("WEAP DATA shape unsupported; fields not compared".into());
                        }
                        e.opaque.insert(path, data.to_vec());
                    }
                }
            }
            if self.out.entities.insert(key, e).is_some() {
                return Err("duplicate canonical record identity".into());
            }
            b = &b[end..];
        }
        Ok(())
    }
}
/// Explicit Skyrim SE caller context required: TES4 header alone cannot identify a game.
/// Input borrowed immutably; malformed, oversized and cancelled inputs produce no snapshot.
pub fn parse(
    bytes: &[u8],
    plugin_name: &str,
    cancel: impl Fn() -> bool,
) -> Result<Snapshot, String> {
    if cancel() {
        return Err("cancelled".into());
    }
    if bytes.len() > MAX_BYTES {
        return Err("input exceeds 64 MiB analysis bound".into());
    }
    if plugin_name.is_empty() || plugin_name.chars().any(|c| "/\\:\0".contains(c)) {
        return Err("invalid plugin basename".into());
    }
    if bytes.get(..4) != Some(b"TES4") || bytes.len() < 24 {
        return Err("not a TES4 plugin".into());
    }
    if u32_at(bytes, 8)? & 0x40000 != 0 {
        return Err("compressed TES4 unsupported".into());
    }
    let end = 24usize
        .checked_add(u32_at(bytes, 4)? as usize)
        .ok_or("header overflow")?;
    if u32_at(bytes, 12)? != 0 {
        return Err("TES4 FormID must be zero".into());
    }
    let mut masters = vec![];
    let mut subrecord_budget = 100_000;
    let headers = subs(
        bytes.get(24..end).ok_or("truncated TES4")?,
        &cancel,
        &mut subrecord_budget,
    )?;
    if headers
        .first()
        .is_none_or(|(name, payload)| name != "HEDR" || payload.len() != 12)
        || headers.iter().filter(|(n, _)| n == "HEDR").count() != 1
    {
        return Err("required HEDR missing or malformed".into());
    }
    for (name, payload) in headers {
        if name == "MAST" {
            let m = text(payload)?;
            if m.is_empty()
                || m.chars().any(|c| "/\\:\0".contains(c))
                || m.eq_ignore_ascii_case(plugin_name)
                || masters.len() >= 253
                || masters.iter().any(|x: &String| x.eq_ignore_ascii_case(&m))
            {
                return Err("ambiguous master list".into());
            }
            masters.push(m);
        }
    }
    let out = Snapshot {
        format: "skyrim-se-plugin".into(),
        parser: "narrow-weap-data-v1".into(),
        entities: BTreeMap::from([("@TES4".into(), Entity {fields:BTreeMap::from([("HeaderFlags".into(),Value::Unsigned(u32_at(bytes,8)? as u64))]),opaque:BTreeMap::from([("HeaderMetadata".into(),bytes[16..24].to_vec()),("HeaderSubrecords".into(),bytes[24..end].to_vec())]),limitations:vec![]} )]),
        inventory_complete: true,
        limitations: vec!["Narrow projection: TES4 metadata, group context, record version and all non-WEAP-DATA fields are not fully interpreted; not full-plugin/xEdit equivalence".into()],
    };
    let mut p = Parser {
        out,
        masters,
        name: plugin_name.into(),
        cancel: &cancel,
        count: 0,
        subrecord_budget,
    };
    p.records(&bytes[end..], 0)?;
    Ok(p.out)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Change, compare};
    fn sub(sig: &[u8; 4], b: &[u8]) -> Vec<u8> {
        [sig.as_slice(), &(b.len() as u16).to_le_bytes(), b].concat()
    }
    fn rec(sig: &[u8; 4], id: u32, flags: u32, b: &[u8]) -> Vec<u8> {
        let body = if sig == b"TES4" {
            [sub(b"HEDR", &[0; 12]), b.to_vec()].concat()
        } else {
            b.to_vec()
        };
        let b = body.as_slice();
        let mut h = vec![0; 24];
        h[..4].copy_from_slice(sig);
        h[4..8].copy_from_slice(&(b.len() as u32).to_le_bytes());
        h[8..12].copy_from_slice(&flags.to_le_bytes());
        h[12..16].copy_from_slice(&id.to_le_bytes());
        h.extend_from_slice(b);
        h
    }
    fn weapon(damage: u16, id: u32) -> Vec<u8> {
        let data = [
            7u32.to_le_bytes().as_slice(),
            2.0f32.to_le_bytes().as_slice(),
            damage.to_le_bytes().as_slice(),
        ]
        .concat();
        rec(
            b"WEAP",
            id,
            0,
            &[sub(b"EDID", b"IronSword\0"), sub(b"DATA", &data)].concat(),
        )
    }
    fn plugin(damage: u16) -> Vec<u8> {
        [rec(b"TES4", 0, 0, &[]), weapon(damage, 0x800)].concat()
    }
    #[test]
    fn equal_size_damage_change_is_field_delta() {
        let a = plugin(8);
        let b = plugin(12);
        assert_eq!(a.len(), b.len());
        let report = compare(
            &parse(&a, "Sword.esp", || false).unwrap(),
            &parse(&b, "Sword.esp", || false).unwrap(),
        )
        .unwrap();
        assert_eq!(
            report.changes,
            vec![Change::Field {
                entity: "sword.esp:000800".into(),
                path: "DATA[0].Damage".into(),
                before: Some(Value::Unsigned(8)),
                after: Some(Value::Unsigned(12))
            }]
        );
        assert!(!report.semantic_complete);
    }
    #[test]
    fn master_reorder_keeps_canonical_identity() {
        let a = [
            rec(b"TES4", 0, 0, &sub(b"MAST", b"Skyrim.esm\0")),
            weapon(8, 0x800),
        ]
        .concat();
        let b = [
            rec(
                b"TES4",
                0,
                0,
                &[sub(b"MAST", b"Other.esm\0"), sub(b"MAST", b"Skyrim.esm\0")].concat(),
            ),
            weapon(8, 0x1000800),
        ]
        .concat();
        assert!(
            compare(
                &parse(&a, "S.esp", || false).unwrap(),
                &parse(&b, "S.esp", || false).unwrap()
            )
            .unwrap()
            .changes
            .iter()
            .all(|c| matches!(c,Change::Opaque{entity,..} if entity=="@TES4"))
        );
    }
    #[test]
    fn compressed_is_not_clean_semantics() {
        let x = [
            rec(b"TES4", 0, 0, &[]),
            rec(b"WEAP", 0x800, 0x40000, b"opaque"),
        ]
        .concat();
        let s = parse(&x, "S.esp", || false).unwrap();
        let r = compare(&s, &s).unwrap();
        assert!(!r.semantic_complete);
        assert!(
            r.changes
                .iter()
                .any(|c| matches!(c, Change::Unknown { .. }))
        );
    }
    #[test]
    fn nested_group_decodes() {
        let w = weapon(8, 0x800);
        let mut h = vec![0; 24];
        h[..4].copy_from_slice(b"GRUP");
        h[4..8].copy_from_slice(&((w.len() + 24) as u32).to_le_bytes());
        let p = [rec(b"TES4", 0, 0, &[]), h, w].concat();
        assert_eq!(parse(&p, "S.esp", || false).unwrap().entities.len(), 2);
    }
    #[test]
    fn malformed_refused() {
        for n in [0, 3, 23, 25, 40] {
            let p = plugin(8);
            assert!(parse(&p[..n], "S.esp", || false).is_err());
        }
    }
    #[test]
    fn cancellation_refused() {
        assert!(
            parse(&plugin(8), "S.esp", || true)
                .unwrap_err()
                .contains("cancelled")
        );
    }
    #[test]
    fn duplicate_identity_refused() {
        let p = [plugin(8), weapon(9, 0x800)].concat();
        assert!(
            parse(&p, "S.esp", || false)
                .unwrap_err()
                .contains("duplicate")
        );
    }
    #[test]
    fn unresolved_master_refused() {
        let p = [rec(b"TES4", 0, 0, &[]), weapon(8, 0x2000800)].concat();
        assert!(parse(&p, "S.esp", || false).is_err());
    }
    #[test]
    fn short_data_not_removed_fields() {
        let a = plugin(8);
        let b = [
            rec(b"TES4", 0, 0, &[]),
            rec(b"WEAP", 0x800, 0, &sub(b"DATA", &[0; 9])),
        ]
        .concat();
        let r = compare(
            &parse(&a, "S.esp", || false).unwrap(),
            &parse(&b, "S.esp", || false).unwrap(),
        )
        .unwrap();
        assert!(
            !r.changes
                .iter()
                .any(|c| matches!(c,Change::Field{path,after:None,..} if path.starts_with("DATA")))
        );
        assert!(!r.semantic_complete);
    }
    #[test]
    fn header_flags_are_reported_and_scope_incomplete() {
        let a = plugin(8);
        let mut b = a.clone();
        b[8] = 1;
        let r = compare(
            &parse(&a, "S.esp", || false).unwrap(),
            &parse(&b, "S.esp", || false).unwrap(),
        )
        .unwrap();
        assert!(r.changes.iter().any(
            |c| matches!(c,Change::Field{entity,path,..}if entity=="@TES4"&&path=="HeaderFlags")
        ));
        assert!(!r.semantic_complete);
    }
    #[test]
    fn cancelled_empty_plugin_refused() {
        assert!(parse(&rec(b"TES4", 0, 0, &[]), "S.esp", || true).is_err());
    }
    #[test]
    fn missing_hedr_refused() {
        let mut b = vec![0; 24];
        b[..4].copy_from_slice(b"TES4");
        assert!(parse(&b, "S.esp", || false).is_err());
    }
    #[test]
    fn subrecord_budget_is_bounded() {
        let b = sub(b"NULL", &[]);
        assert!(subs(&b, &|| false, &mut 0).is_err());
    }
    #[test]
    fn cancellation_during_subrecords() {
        use std::cell::Cell;
        let n = Cell::new(0);
        let b = [sub(b"NULL", &[]), sub(b"NULL", &[])].concat();
        assert!(
            subs(
                &b,
                &|| {
                    n.set(n.get() + 1);
                    n.get() > 1
                },
                &mut 10
            )
            .is_err()
        );
    }
    #[test]
    fn invalid_self_master_refused() {
        let b = rec(b"TES4", 0, 0, &sub(b"MAST", b"S.esp\0"));
        assert!(parse(&b, "S.esp", || false).is_err());
    }
}
