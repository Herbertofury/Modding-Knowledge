//! Review-only FOMOD evidence. No XML interpretation, file writes or installer execution.
//! Adapter statements are not authenticated proof; retain original evidence in the vault.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    RecordedInteraction,
    InferredFiles,
    ImportedUnknown,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Binding {
    pub archive_sha256: String,
    pub module_config_sha256: String,
    /// Digest of canonical external dependency inputs, including game/profile identity.
    /// None means unknown, not an empty environment.
    pub context_sha256: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Selection {
    /// Document-local occurrence coordinates, NOT a transferable name or global ID.
    pub step: u32,
    pub group: u32,
    pub plugin: u32,
    pub selected: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChoiceEvidence {
    pub binding: Binding,
    pub origin: Origin,
    pub tool: String,
    pub tool_version: String,
    pub observed_unix: i64,
    /// SHA256 of separately retained raw log/inference result; validated by host/vault.
    pub raw_evidence_sha256: String,
    pub complete: bool,
    pub ambiguous: bool,
    pub selections: Vec<Selection>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewReason {
    InferredNotRecorded,
    UnknownOrigin,
    Incomplete,
    Ambiguous,
    ArchiveChanged,
    InstallerChanged,
    ContextUnknown,
    ContextChanged,
}
#[derive(Debug, Serialize)]
pub struct Review {
    pub reasons: Vec<ReviewReason>,
    /// Only an attended UI may offer same-artifact recorded choices as preselection.
    /// Host must still evaluate current XML constraints and show all relevant steps.
    pub may_offer_recorded_preselection: bool,
    pub requires_user_review: bool,
    pub automatic_replay_allowed: bool,
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn valid_binding(b: &Binding) -> bool {
    digest(&b.archive_sha256)
        && digest(&b.module_config_sha256)
        && b.context_sha256.as_ref().is_none_or(|s| digest(s))
}
pub fn review(e: &ChoiceEvidence, target: &Binding) -> Result<Review, String> {
    if !valid_binding(&e.binding)
        || !valid_binding(target)
        || !digest(&e.raw_evidence_sha256)
        || e.tool.trim().is_empty()
        || e.tool_version.trim().is_empty()
        || e.observed_unix <= 0
    {
        return Err(
            "Invalid binding or evidence provenance; original evidence must remain retained".into(),
        );
    }
    let mut coordinates = HashSet::new();
    for s in &e.selections {
        if !coordinates.insert((s.step, s.group, s.plugin)) {
            return Err("Duplicate selection coordinates; do not choose one silently".into());
        }
    }
    let mut reasons = Vec::new();
    match e.origin {
        Origin::RecordedInteraction => {}
        Origin::InferredFiles => reasons.push(ReviewReason::InferredNotRecorded),
        Origin::ImportedUnknown => reasons.push(ReviewReason::UnknownOrigin),
    }
    if !e.complete {
        reasons.push(ReviewReason::Incomplete);
    }
    if e.ambiguous {
        reasons.push(ReviewReason::Ambiguous);
    }
    if e.binding.archive_sha256 != target.archive_sha256 {
        reasons.push(ReviewReason::ArchiveChanged);
    }
    if e.binding.module_config_sha256 != target.module_config_sha256 {
        reasons.push(ReviewReason::InstallerChanged);
    }
    match (&e.binding.context_sha256, &target.context_sha256) {
        (Some(a), Some(b)) if a != b => reasons.push(ReviewReason::ContextChanged),
        (Some(_), Some(_)) => {}
        _ => reasons.push(ReviewReason::ContextUnknown),
    }
    Ok(Review {
        may_offer_recorded_preselection: reasons.is_empty(),
        reasons,
        requires_user_review: true,
        automatic_replay_allowed: false,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn evidence() -> ChoiceEvidence {
        ChoiceEvidence {
            binding: Binding {
                archive_sha256: "a".repeat(64),
                module_config_sha256: "b".repeat(64),
                context_sha256: Some("c".repeat(64)),
            },
            origin: Origin::RecordedInteraction,
            tool: "fixture".into(),
            tool_version: "1".into(),
            observed_unix: 1,
            raw_evidence_sha256: "d".repeat(64),
            complete: true,
            ambiguous: false,
            selections: vec![Selection {
                step: 0,
                group: 0,
                plugin: 0,
                selected: false,
            }],
        }
    }
    #[test]
    fn exact_recorded_is_attended_only() {
        let e = evidence();
        let r = review(&e, &e.binding).unwrap();
        assert!(r.may_offer_recorded_preselection && r.requires_user_review);
        assert!(!r.automatic_replay_allowed);
    }
    #[test]
    fn inference_never_becomes_historical_clicks() {
        let mut e = evidence();
        e.origin = Origin::InferredFiles;
        let r = review(&e, &e.binding).unwrap();
        assert!(!r.may_offer_recorded_preselection);
        assert_eq!(r.reasons, vec![ReviewReason::InferredNotRecorded]);
    }
    #[test]
    fn repack_same_config_still_needs_review() {
        let e = evidence();
        let mut b = e.binding.clone();
        b.archive_sha256 = "f".repeat(64);
        assert_eq!(
            review(&e, &b).unwrap().reasons,
            vec![ReviewReason::ArchiveChanged]
        );
    }
    #[test]
    fn changed_installer_cannot_reuse_positions_or_names() {
        let e = evidence();
        let mut b = e.binding.clone();
        b.module_config_sha256 = "e".repeat(64);
        assert!(!review(&e, &b).unwrap().may_offer_recorded_preselection);
    }
    #[test]
    fn unknown_and_changed_context_are_not_equivalent() {
        let e = evidence();
        let mut b = e.binding.clone();
        b.context_sha256 = None;
        assert_eq!(
            review(&e, &b).unwrap().reasons,
            vec![ReviewReason::ContextUnknown]
        );
        b.context_sha256 = Some("e".repeat(64));
        assert_eq!(
            review(&e, &b).unwrap().reasons,
            vec![ReviewReason::ContextChanged]
        );
    }
    #[test]
    fn partial_ambiguous_and_unknown_keep_all_reasons() {
        let mut e = evidence();
        e.complete = false;
        e.ambiguous = true;
        e.origin = Origin::ImportedUnknown;
        assert_eq!(review(&e, &e.binding).unwrap().reasons.len(), 3);
    }
    #[test]
    fn duplicate_coordinates_rejected() {
        let mut e = evidence();
        e.selections.push(e.selections[0].clone());
        assert!(review(&e, &e.binding).is_err());
    }
    #[test]
    fn malformed_provenance_rejected() {
        let mut e = evidence();
        e.raw_evidence_sha256 = "not-a-hash".into();
        assert!(review(&e, &e.binding).is_err());
    }
    #[test]
    fn unknown_serialized_origin_rejected() {
        assert!(serde_json::from_str::<Origin>("\"certain_inference\"").is_err());
    }
}
