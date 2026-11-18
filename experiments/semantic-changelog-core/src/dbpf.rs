//! DBPF2/index3 TGI resource diff, independently implemented from Sims4Tools format docs.
//! Decoded payload equality is resource evidence, NOT CAS/tuning/mesh field interpretation.
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub kind: u32,
    pub group: u32,
    pub instance: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Payload {
    Decoded(Vec<u8>),
    Unknown { compression: u16, reason: String },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub resources: BTreeMap<Key, Payload>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Added(Key),
    Removed(Key),
    PayloadChanged(Key),
    NotCompared(Key),
}
fn word(b: &[u8], p: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        b.get(p..p + 4)
            .ok_or("truncated index word")?
            .try_into()
            .unwrap(),
    ))
}
/// Refuses malformed/ambiguous indexes rather than returning an apparently empty package.
/// Unknown compression stays explicit even if its stored bytes happen to match.
pub fn parse(b: &[u8], cancel: impl Fn() -> bool) -> Result<Package, String> {
    if cancel() {
        return Err("cancelled".into());
    }
    if b.len() < 96
        || b.len() > 64 * 1024 * 1024
        || b.get(..4) != Some(b"DBPF")
        || word(b, 4)? != 2
        || word(b, 8)? != 0
        || word(b, 60)? != 3
    {
        return Err("unsupported DBPF header/version or size".into());
    }
    if word(b, 68)? != 0 {
        return Err("extended index offset unsupported".into());
    }
    let count = word(b, 36)? as usize;
    let size = word(b, 44)? as usize;
    let pos = word(b, 64)? as usize;
    if count > 100_000 {
        return Err("resource count bound".into());
    }
    if count == 0 {
        if size != 0 || pos != 0 {
            return Err("inconsistent empty index".into());
        }
        return Ok(Package {
            resources: BTreeMap::new(),
        });
    }
    if pos < 96 {
        return Err("index overlaps header".into());
    }
    let end = pos.checked_add(size).ok_or("index overflow")?;
    let index = b.get(pos..end).ok_or("index outside package")?;
    let flags = word(index, 0)?;
    if flags & !7 != 0 {
        return Err("unsupported shared index fields".into());
    }
    let mut cursor = 4;
    let mut shared = [None; 3];
    for (i, slot) in shared.iter_mut().enumerate() {
        if flags & (1 << i) != 0 {
            *slot = Some(word(index, cursor)?);
            cursor += 4;
        }
    }
    let expected = cursor + count * (32 - 4 * flags.count_ones() as usize);
    if expected != size {
        return Err("index length/count mismatch".into());
    }
    let mut resources = BTreeMap::new();
    let mut expanded = 0usize;
    for _ in 0..count {
        if cancel() {
            return Err("cancelled".into());
        }
        let mut row = [0u32; 8];
        for i in 0..8 {
            row[i] = if i < 3 && shared[i].is_some() {
                shared[i].unwrap()
            } else {
                let w = word(index, cursor)?;
                cursor += 4;
                w
            };
        }
        let key = Key {
            kind: row[0],
            group: row[1],
            instance: ((row[2] as u64) << 32) | row[3] as u64,
        };
        let start = row[4] as usize;
        let stored = (row[5] & 0x7fff_ffff) as usize;
        let mem = row[6] as usize;
        let compression = (row[7] & 0xffff) as u16;
        let finish = start.checked_add(stored).ok_or("resource overflow")?;
        if start < 96 || (start < end && finish > pos) {
            return Err("resource overlaps header/index".into());
        }
        let payload = b.get(start..finish).ok_or("resource outside package")?;
        expanded = expanded.checked_add(mem).ok_or("expansion overflow")?;
        if mem > 16 * 1024 * 1024 || expanded > 64 * 1024 * 1024 {
            return Err("resource expansion bound".into());
        }
        let decoded = match compression {
            0 => {
                if stored != mem {
                    return Err("uncompressed length mismatch".into());
                }
                Payload::Decoded(payload.to_vec())
            }
            x if matches!(x, 0x5a42 | 0xffff)
                && payload.len() >= 2
                && payload[0] & 15 == 8
                && payload[0] >> 4 <= 7
                && u16::from_be_bytes([payload[0], payload[1]]) % 31 == 0 =>
            {
                let mut decoder = flate2::read::ZlibDecoder::new(payload);
                let mut out = Vec::new();
                let mut buf = [0; 8192];
                loop {
                    if cancel() {
                        return Err("cancelled".into());
                    }
                    let n = decoder
                        .read(&mut buf)
                        .map_err(|_| "invalid zlib resource")?;
                    if n == 0 {
                        break;
                    }
                    if out.len() + n > mem {
                        return Err("resource exceeds declared decoded size".into());
                    }
                    out.extend_from_slice(&buf[..n]);
                }
                if out.len() != mem || decoder.total_in() != payload.len() as u64 {
                    return Err("zlib size/trailing-byte mismatch".into());
                }
                Payload::Decoded(out)
            }
            _ => Payload::Unknown {
                compression,
                reason: "resource compression is not supported by this decoder".into(),
            },
        };
        if resources.insert(key, decoded).is_some() {
            return Err("duplicate TGI identity".into());
        }
    }
    Ok(Package { resources })
}
pub fn compare(a: &Package, b: &Package) -> Vec<Change> {
    let keys: BTreeSet<_> = a
        .resources
        .keys()
        .chain(b.resources.keys())
        .copied()
        .collect();
    let mut out = vec![];
    for key in keys {
        match (a.resources.get(&key), b.resources.get(&key)) {
            (None, Some(_)) => out.push(Change::Added(key)),
            (Some(_), None) => out.push(Change::Removed(key)),
            (Some(Payload::Decoded(x)), Some(Payload::Decoded(y))) => {
                if x != y {
                    out.push(Change::PayloadChanged(key));
                }
            }
            _ => out.push(Change::NotCompared(key)),
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn package(data: &[u8], compress: bool, pad: usize) -> Vec<u8> {
        let mut b = vec![0u8; 96];
        b[..4].copy_from_slice(b"DBPF");
        for (pos, v) in [(4, 2u32), (36, 1), (44, 36), (60, 3)] {
            b[pos..pos + 4].copy_from_slice(&v.to_le_bytes());
        }
        let payload = if compress {
            let mut w = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            w.write_all(data).unwrap();
            w.finish().unwrap()
        } else {
            data.to_vec()
        };
        b.extend_from_slice(&payload);
        b.resize(b.len() + pad, 0);
        let pos = b.len() as u32;
        b[64..68].copy_from_slice(&pos.to_le_bytes());
        for v in [
            0,
            0x034aeecb,
            0,
            1,
            2,
            96,
            payload.len() as u32,
            data.len() as u32,
            if compress { 0x5a42 } else { 0 },
        ] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b
    }
    #[test]
    fn equal_size_resource_change() {
        let a = package(b"abc", false, 0);
        let b = package(b"abd", false, 0);
        assert_eq!(a.len(), b.len());
        assert!(matches!(
            compare(&parse(&a, || false).unwrap(), &parse(&b, || false).unwrap())[0],
            Change::PayloadChanged(_)
        ));
    }
    #[test]
    fn repacked_same_payload_not_change() {
        let a = package(b"unchanged resource", false, 0);
        let b = package(b"unchanged resource", true, 35);
        assert_ne!(a.len(), b.len());
        assert!(compare(&parse(&a, || false).unwrap(), &parse(&b, || false).unwrap()).is_empty());
    }
    #[test]
    fn unsupported_compression_not_clean() {
        let mut p = package(b"abc", false, 0);
        let len = p.len();
        p[len - 4..len - 2].copy_from_slice(&0xffffu16.to_le_bytes());
        let a = parse(&p, || false).unwrap();
        assert!(matches!(compare(&a, &a)[0], Change::NotCompared(_)));
    }
    #[test]
    fn bad_index_rejected() {
        let mut p = package(b"abc", false, 0);
        p[44..48].copy_from_slice(&0u32.to_le_bytes());
        assert!(parse(&p, || false).is_err());
    }
    #[test]
    fn cancellation() {
        assert!(parse(&package(b"abc", false, 0), || true).is_err());
    }
    #[test]
    fn expansion_mismatch_rejected() {
        let mut p = package(b"abc", true, 0);
        let n = p.len();
        p[n - 8..n - 4].copy_from_slice(&1u32.to_le_bytes());
        assert!(parse(&p, || false).is_err());
    }
    #[test]
    fn shared_tgi_header() {
        let mut p = package(b"abc", false, 0);
        let pos = word(&p, 64).unwrap() as usize;
        p[pos..pos + 4].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(parse(&p, || false).unwrap().resources.len(), 1);
    }
    #[test]
    fn duplicate_tgi_rejected() {
        let mut p = package(b"abc", false, 0);
        let pos = word(&p, 64).unwrap() as usize;
        let row = p[pos + 4..].to_vec();
        p.extend_from_slice(&row);
        p[36..40].copy_from_slice(&2u32.to_le_bytes());
        p[44..48].copy_from_slice(&68u32.to_le_bytes());
        assert!(parse(&p, || false).unwrap_err().contains("duplicate"));
    }
    #[test]
    fn truncated_payload_rejected() {
        let mut p = package(b"abc", false, 0);
        let pos = word(&p, 64).unwrap() as usize;
        p[pos + 20..pos + 24].copy_from_slice(&99999u32.to_le_bytes());
        assert!(parse(&p, || false).is_err());
    }
    #[test]
    fn zlib_with_alternative_flag_detected_by_header() {
        let mut p = package(b"same data", true, 0);
        let n = p.len();
        p[n - 4..n - 2].copy_from_slice(&0xffffu16.to_le_bytes());
        let plain = package(b"same data", false, 0);
        assert!(
            compare(
                &parse(&p, || false).unwrap(),
                &parse(&plain, || false).unwrap()
            )
            .is_empty()
        );
    }
}
