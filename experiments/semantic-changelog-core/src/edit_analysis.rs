//! Bind a recorded local edit to exact immutable snapshot bytes before analysis.
//! No I/O, retained file bytes, or automatic tool-identity inference.
use crate::{
    Report, dbpf,
    history::{Edit, Revision},
    skyrim,
};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub enum Evidence {
    SkyrimFields(Report),
    /// Resource payload comparison only, not CAS/swatches/tuning field semantics.
    Sims4Resources(Vec<dbpf::Change>),
}
#[derive(Debug)]
pub struct AnalyzedEdit {
    pub edit: Edit,
    pub parser: &'static str,
    pub evidence: Evidence,
    /// This experiment never implies whole-format or gameplay completeness.
    pub limitations: Vec<String>,
}
fn verify(bytes: &[u8], revision: &Revision, cancel: &impl Fn() -> bool) -> Result<(), String> {
    if cancel() {
        return Err("cancelled".into());
    }
    if bytes.len() > 64 * 1024 * 1024 || bytes.len() as u64 != revision.length {
        return Err("revision length mismatch or snapshot exceeds bound".into());
    }
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(64 * 1024) {
        if cancel() {
            return Err("cancelled".into());
        }
        digest.update(chunk);
    }
    let hash: [u8; 32] = digest.finalize().into();
    if hash != revision.sha256 {
        return Err("revision content hash mismatch".into());
    }
    Ok(())
}
fn verify_pair(
    edit: &Edit,
    before: &[u8],
    after: &[u8],
    cancel: &impl Fn() -> bool,
) -> Result<(), String> {
    if edit.before.observed_at_ms > edit.after.observed_at_ms
        || edit.before.sha256 == edit.after.sha256
    {
        return Err("invalid edit revision pair".into());
    }
    verify(before, &edit.before, cancel)?;
    verify(after, &edit.after, cancel)
}
/// Caller supplies explicit Skyrim SE game context and canonical plugin basename.
/// Hash binding validates content, not the authenticity of caller-supplied Edit
/// provenance. Preserve history's known-gap status separately in UI/export.
pub fn skyrim_edit(
    edit: &Edit,
    before: &[u8],
    after: &[u8],
    plugin_name: &str,
    cancel: impl Fn() -> bool,
) -> Result<AnalyzedEdit, String> {
    verify_pair(edit, before, after, &cancel)?;
    let a = skyrim::parse(before, plugin_name, &cancel)?;
    let b = skyrim::parse(after, plugin_name, &cancel)?;
    let report = crate::compare(&a, &b).map_err(str::to_owned)?;
    if cancel() {
        return Err("cancelled".into());
    }
    let limitations = report.limitations.clone();
    Ok(AnalyzedEdit {
        edit: edit.clone(),
        parser: "narrow-weap-data-v1",
        evidence: Evidence::SkyrimFields(report),
        limitations,
    })
}
/// Explicit Sims4 caller context required. No swatch/mesh field decoding claimed.
pub fn sims4_edit(
    edit: &Edit,
    before: &[u8],
    after: &[u8],
    cancel: impl Fn() -> bool,
) -> Result<AnalyzedEdit, String> {
    verify_pair(edit, before, after, &cancel)?;
    let a = dbpf::parse(before, &cancel)?;
    let b = dbpf::parse(after, &cancel)?;
    let changes = dbpf::compare(&a, &b);
    if cancel() {
        return Err("cancelled".into());
    }
    Ok(AnalyzedEdit { edit: edit.clone(), parser: "dbpf2-index3-resource-v1", evidence: Evidence::Sims4Resources(changes), limitations: vec!["Resource payload evidence only; unsupported compression stays NotCompared; CAS/swatches/tuning/mesh fields, gameplay effects and author intent not interpreted".into()] })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{Attribution, History};
    fn edit(a: &[u8], b: &[u8]) -> Edit {
        let mut h = History::new("opaque-artifact".into(), 2).unwrap();
        h.set_enabled(true);
        h.observe(a, 1, Attribution::Unknown, None).unwrap();
        h.observe(
            b,
            2,
            Attribution::Inferred {
                tool: "xEdit".into(),
                evidence: "launch-1".into(),
            },
            Some("workflow-1".into()),
        )
        .unwrap();
        h.query(None, 0, 9).next().unwrap().clone()
    }
    fn plugin(damage: u16) -> Vec<u8> {
        let mut h = vec![0; 24];
        h[..4].copy_from_slice(b"TES4");
        h[4..8].copy_from_slice(&18u32.to_le_bytes());
        h.extend_from_slice(b"HEDR\x0c\x00");
        h.extend_from_slice(&[0; 12]);
        let mut w = vec![0; 24];
        w[..4].copy_from_slice(b"WEAP");
        w[4..8].copy_from_slice(&16u32.to_le_bytes());
        w[12..16].copy_from_slice(&0x800u32.to_le_bytes());
        h.extend(w);
        h.extend_from_slice(b"DATA\x0a\x00");
        h.extend_from_slice(&[0; 8]);
        h.extend_from_slice(&damage.to_le_bytes());
        h
    }
    #[test]
    fn real_field_diff_bound_to_recorded_bytes() {
        let (a, b) = (plugin(8), plugin(12));
        let e = edit(&a, &b);
        let result = skyrim_edit(&e, &a, &b, "Sword.esp", || false).unwrap();
        assert_eq!(result.edit, e);
        assert!(matches!(
            result.edit.attribution,
            Attribution::Inferred { .. }
        ));
        let Evidence::SkyrimFields(r) = result.evidence else {
            panic!()
        };
        assert!(!r.semantic_complete);
        assert!(!result.limitations.is_empty());
        assert!(
            r.changes
                .iter()
                .any(|c| matches!(c,crate::Change::Field{path,..}if path=="DATA[0].Damage"))
        );
    }
    #[test]
    fn wrong_equal_length_bytes_rejected_before_parse() {
        let (a, b) = (plugin(8), plugin(12));
        let e = edit(&a, &b);
        assert!(
            skyrim_edit(&e, &a, &plugin(13), "Sword.esp", || false)
                .unwrap_err()
                .contains("hash")
        );
        assert!(
            skyrim_edit(&e, &b, &a, "Sword.esp", || false)
                .unwrap_err()
                .contains("hash")
        );
    }
    #[test]
    fn cancellation_during_hash_refuses_result() {
        use std::cell::Cell;
        let a = vec![1; 130000];
        let b = vec![2; 130000];
        let e = edit(&a, &b);
        let count = Cell::new(0);
        assert!(
            sims4_edit(&e, &a, &b, || {
                count.set(count.get() + 1);
                count.get() > 2
            })
            .unwrap_err()
            .contains("cancelled")
        );
    }
    #[test]
    fn invalid_edit_pair_refused() {
        let (a, b) = (plugin(8), plugin(12));
        let mut e = edit(&a, &b);
        e.before.observed_at_ms = 3;
        assert!(skyrim_edit(&e, &a, &b, "Sword.esp", || false).is_err());
        e.before = e.after.clone();
        assert!(sims4_edit(&e, &b, &b, || false).is_err());
    }
    #[test]
    fn malformed_but_hash_matching_is_not_accepted() {
        let (a, b) = (b"bad1", b"bad2");
        let e = edit(a, b);
        assert!(sims4_edit(&e, a, b, || false).is_err());
    }
}

#[cfg(test)]
mod sims_binding_tests {
    use super::*;
    use crate::history::{Attribution, History};
    fn package(data: &[u8], compression: u32) -> Vec<u8> {
        let mut b = vec![0; 96];
        b[..4].copy_from_slice(b"DBPF");
        for (p, v) in [
            (4, 2u32),
            (36, 1),
            (44, 36),
            (60, 3),
            (64, 96 + data.len() as u32),
        ] {
            b[p..p + 4].copy_from_slice(&v.to_le_bytes());
        }
        b.extend(data);
        for v in [
            0u32,
            0x034aeecb,
            0,
            1,
            2,
            96,
            data.len() as u32,
            data.len() as u32,
            compression,
        ] {
            b.extend(v.to_le_bytes());
        }
        b
    }
    fn analyze(compression: u32) -> AnalyzedEdit {
        let (a, b) = (package(b"abc", compression), package(b"abd", compression));
        let mut h = History::new("opaque".into(), 2).unwrap();
        h.set_enabled(true);
        h.observe(&a, 1, Attribution::Unknown, None).unwrap();
        h.observe(&b, 2, Attribution::Unknown, None).unwrap();
        sims4_edit(h.query(None, 0, 3).next().unwrap(), &a, &b, || false).unwrap()
    }
    #[test]
    fn verified_package_edit_remains_resource_only() {
        let r = analyze(0);
        assert!(!r.limitations.is_empty());
        let Evidence::Sims4Resources(changes) = r.evidence else {
            panic!()
        };
        assert!(matches!(
            changes.as_slice(),
            [dbpf::Change::PayloadChanged(_)]
        ));
    }
    #[test]
    fn unknown_compression_is_still_not_compared() {
        let r = analyze(0xffff);
        assert!(!r.limitations.is_empty());
        let Evidence::Sims4Resources(changes) = r.evidence else {
            panic!()
        };
        assert!(matches!(changes.as_slice(), [dbpf::Change::NotCompared(_)]));
    }
}
