//! Opt-in, in-memory history of explicitly supplied artifact observations.
//! No watcher, process inspection, file writes, or inferred authorship.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Attribution {
    Unknown,
    /// A launch/time correlation is NOT evidence that this tool wrote the file.
    Inferred {
        tool: String,
        evidence: String,
    },
    /// Host must supply a direct write event bound to this artifact and process.
    ObservedWrite {
        tool: String,
        evidence: String,
    },
    UserReported {
        tool: String,
    },
}
impl Attribution {
    pub fn tool(&self) -> Option<&str> {
        match self {
            Self::Unknown => None,
            Self::Inferred { tool, .. }
            | Self::ObservedWrite { tool, .. }
            | Self::UserReported { tool } => Some(tool),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let valid =
            |s: &str| !s.trim().is_empty() && s.len() <= 256 && !s.chars().any(char::is_control);
        match self {
            Self::Unknown => true,
            Self::UserReported { tool } => valid(tool),
            Self::Inferred { tool, evidence } | Self::ObservedWrite { tool, evidence } => {
                valid(tool) && valid(evidence)
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    pub sha256: [u8; 32],
    pub length: u64,
    /// Host observation time, NOT a claimed exact edit time or filesystem mtime.
    pub observed_at_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    pub before: Revision,
    pub after: Revision,
    pub attribution: Attribution,
    pub workflow: Option<String>,
}
/// One artifact's explicit, stable identity; no file bytes retained.
/// Restore only through the validated checkpoint API, never raw deserialization.
/// ```compile_fail
/// let _: semantic_changelog_core::history::History = serde_json::from_str("{}").unwrap();
/// ```
/// Host must use opaque IDs/sanitized labels: arbitrary strings can contain paths.
/// Supply snapshots only after stable-read/race checks. Bounded and fails closed:
/// full history requires explicit export/retention policy, never silent eviction.
#[derive(Serialize)]
pub struct History {
    pub(crate) artifact: String,
    pub(crate) current: Option<Revision>,
    pub(crate) edits: Vec<Edit>,
    pub(crate) capacity: usize,
    pub(crate) enabled: bool,
    pub(crate) gap: bool,
    pub(crate) continuity_lost: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Observation {
    Baseline,
    Unchanged,
    Changed,
    RebasedAfterGap,
}
impl History {
    pub fn new(artifact: String, capacity: usize) -> Result<Self, &'static str> {
        if artifact.trim().is_empty()
            || artifact.len() > 256
            || artifact.chars().any(char::is_control)
            || !(1..=100_000).contains(&capacity)
        {
            return Err("invalid artifact identity or history capacity");
        }
        Ok(Self {
            artifact,
            current: None,
            edits: Vec::new(),
            capacity,
            enabled: false,
            gap: false,
            continuity_lost: false,
        })
    }
    /// Whether supplied observations have no known continuity gap. This never
    /// guarantees every intermediate edit was captured (notifications coalesce).
    /// False permanently after a pause/overflow. Exports and UI must retain this
    /// status; a later rebase does not restore evidence of missed observations.
    pub fn continuity_preserved(&self) -> bool {
        !self.continuity_lost
    }
    /// Host calls this after dropped watcher events/read failures.
    pub fn mark_gap(&mut self) {
        self.gap = true;
        self.continuity_lost = true;
    }
    pub fn artifact(&self) -> &str {
        &self.artifact
    }
    /// UI must obtain explicit user opt-in for the selected artifact/scope.
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled && !enabled && self.current.is_some() {
            self.gap = true;
            self.continuity_lost = true;
        }
        self.enabled = enabled;
    }
    pub fn observe(
        &mut self,
        bytes: &[u8],
        observed_at_ms: u64,
        attribution: Attribution,
        workflow: Option<String>,
    ) -> Result<Observation, &'static str> {
        if !self.enabled {
            return Err("history disabled");
        }
        if bytes.len() > 64 * 1024 * 1024 {
            return Err("snapshot exceeds bound");
        }
        if !attribution.valid()
            || workflow.as_ref().is_some_and(|s| {
                s.trim().is_empty() || s.len() > 256 || s.chars().any(char::is_control)
            })
        {
            return Err("invalid provenance label");
        }
        if self
            .current
            .as_ref()
            .is_some_and(|r| observed_at_ms < r.observed_at_ms)
        {
            return Err("observation time regressed");
        }
        let next = Revision {
            sha256: Sha256::digest(bytes).into(),
            length: bytes.len() as u64,
            observed_at_ms,
        };
        let Some(previous) = self.current.as_ref() else {
            self.current = Some(next);
            self.gap = false;
            return Ok(Observation::Baseline);
        };
        if self.gap {
            self.current = Some(next);
            self.gap = false;
            return Ok(Observation::RebasedAfterGap);
        }
        if previous.sha256 == next.sha256 {
            self.current = Some(next);
            return Ok(Observation::Unchanged);
        }
        if self.edits.len() == self.capacity {
            self.continuity_lost = true;
            return Err("history full; explicit retention/export required");
        }
        self.edits.push(Edit {
            before: previous.clone(),
            after: next.clone(),
            attribution,
            workflow,
        });
        self.current = Some(next);
        Ok(Observation::Changed)
    }
    /// Tool filtering preserves attribution tier in every result. Inclusive time
    /// bounds refer to detection time, not an invented exact modification time.
    pub fn query(
        &self,
        tool: Option<&str>,
        from_ms: u64,
        through_ms: u64,
    ) -> impl Iterator<Item = &Edit> {
        self.edits.iter().filter(move |e| {
            e.after.observed_at_ms >= from_ms
                && e.after.observed_at_ms <= through_ms
                && tool.is_none_or(|t| e.attribution.tool() == Some(t))
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn history() -> History {
        let mut h = History::new("artifact-1".into(), 3).unwrap();
        h.set_enabled(true);
        h
    }
    #[test]
    fn opt_in_required() {
        let mut h = History::new("a".into(), 1).unwrap();
        assert_eq!(
            h.observe(b"a", 1, Attribution::Unknown, None),
            Err("history disabled")
        );
    }
    #[test]
    fn equal_size_change_and_touch() {
        let mut h = history();
        assert_eq!(
            h.observe(b"a", 1, Attribution::Unknown, None),
            Ok(Observation::Baseline)
        );
        assert_eq!(
            h.observe(b"a", 2, Attribution::Unknown, None),
            Ok(Observation::Unchanged)
        );
        assert_eq!(
            h.observe(b"b", 3, Attribution::Unknown, None),
            Ok(Observation::Changed)
        );
        let e = h.query(None, 0, 9).next().unwrap();
        assert_eq!(e.before.observed_at_ms, 2);
        assert_eq!(e.attribution, Attribution::Unknown);
    }
    #[test]
    fn tool_tiers_and_filters() {
        let mut h = history();
        h.observe(b"a", 1, Attribution::Unknown, None).unwrap();
        h.observe(
            b"b",
            2,
            Attribution::Inferred {
                tool: "xEdit".into(),
                evidence: "launch-session-1".into(),
            },
            Some("cleaning".into()),
        )
        .unwrap();
        h.observe(
            b"c",
            3,
            Attribution::ObservedWrite {
                tool: "Sims4Studio".into(),
                evidence: "write-event-2".into(),
            },
            None,
        )
        .unwrap();
        assert_eq!(h.query(Some("xEdit"), 2, 2).count(), 1);
        assert!(matches!(
            h.query(None, 2, 2).next().unwrap().attribution,
            Attribution::Inferred { .. }
        ));
        assert_eq!(h.query(None, 3, 2).count(), 0);
    }
    #[test]
    fn gap_never_claims_observed_edit() {
        let mut h = history();
        h.observe(b"a", 1, Attribution::Unknown, None).unwrap();
        h.set_enabled(false);
        h.set_enabled(true);
        assert_eq!(
            h.observe(b"b", 4, Attribution::Unknown, None),
            Ok(Observation::RebasedAfterGap)
        );
        assert_eq!(h.query(None, 0, 9).count(), 0);
        assert!(!h.continuity_preserved());
    }
    #[test]
    fn no_silent_eviction_or_failed_state_advance() {
        let mut h = History::new("a".into(), 1).unwrap();
        h.set_enabled(true);
        h.observe(b"a", 1, Attribution::Unknown, None).unwrap();
        h.observe(b"b", 2, Attribution::Unknown, None).unwrap();
        assert!(h.observe(b"c", 3, Attribution::Unknown, None).is_err());
        assert_eq!(
            h.observe(b"b", 3, Attribution::Unknown, None),
            Ok(Observation::Unchanged)
        );
        assert_eq!(h.query(None, 0, 9).count(), 1);
    }
    #[test]
    fn time_and_labels_validated_without_advancing() {
        let mut h = history();
        h.observe(b"a", 5, Attribution::Unknown, None).unwrap();
        assert!(h.observe(b"b", 4, Attribution::Unknown, None).is_err());
        assert!(
            h.observe(
                b"b",
                6,
                Attribution::ObservedWrite {
                    tool: "xEdit".into(),
                    evidence: "".into()
                },
                None
            )
            .is_err()
        );
        assert_eq!(h.query(None, 0, 9).count(), 0);
    }
}

#[cfg(test)]
mod gap_tests {
    use super::*;
    #[test]
    fn watcher_gap_is_persistently_visible() {
        let mut h = History::new("opaque".into(), 2).unwrap();
        h.set_enabled(true);
        h.observe(b"a", 1, Attribution::Unknown, None).unwrap();
        h.mark_gap();
        assert_eq!(
            h.observe(b"b", 2, Attribution::Unknown, None),
            Ok(Observation::RebasedAfterGap)
        );
        h.observe(b"c", 3, Attribution::Unknown, None).unwrap();
        assert!(!h.continuity_preserved());
        assert_eq!(h.query(None, 0, 9).count(), 1);
    }
}

#[cfg(test)]
mod first_gap_tests {
    use super::*;
    #[test]
    fn initial_gap_does_not_discard_next_edit() {
        let mut h = History::new("opaque".into(), 2).unwrap();
        h.set_enabled(true);
        h.mark_gap();
        assert_eq!(
            h.observe(b"a", 1, Attribution::Unknown, None),
            Ok(Observation::Baseline)
        );
        assert_eq!(
            h.observe(b"b", 2, Attribution::Unknown, None),
            Ok(Observation::Changed)
        );
        assert!(!h.continuity_preserved());
        assert_eq!(h.query(None, 0, 9).count(), 1);
    }
}

#[cfg(test)]
mod watcher_recovery_contract_tests {
    use super::*;
    // Platform adapters must map known notification loss to mark_gap BEFORE
    // reconciliation. These are contract tests, not actual OS watcher tests.
    #[test]
    fn unchanged_rescan_cannot_restore_lost_intermediate_history() {
        let mut h = History::new("artifact".into(), 4).unwrap();
        h.set_enabled(true);
        h.observe(b"A", 1, Attribution::Unknown, None).unwrap();
        // The file might have changed A -> B -> A while notifications were lost.
        h.mark_gap();
        assert_eq!(
            h.observe(b"A", 10, Attribution::Unknown, None),
            Ok(Observation::RebasedAfterGap)
        );
        assert!(!h.continuity_preserved());
        assert_eq!(h.query(None, 0, 10).count(), 0);
        assert_eq!(
            h.observe(b"C", 11, Attribution::Unknown, None),
            Ok(Observation::Changed)
        );
        assert!(!h.continuity_preserved());
        assert_eq!(
            h.query(None, 0, 11).next().unwrap().attribution,
            Attribution::Unknown
        );
    }
    #[test]
    fn repeated_loss_and_rescans_never_invent_a_writer_or_edits() {
        let mut h = History::new("artifact".into(), 4).unwrap();
        h.set_enabled(true);
        h.observe(b"A", 1, Attribution::Unknown, None).unwrap();
        for (t, bytes) in [(2, b"B"), (3, b"C"), (4, b"D")] {
            h.mark_gap();
            assert_eq!(
                h.observe(bytes, t, Attribution::Unknown, None),
                Ok(Observation::RebasedAfterGap)
            );
        }
        assert!(!h.continuity_preserved());
        assert_eq!(h.query(None, 0, 10).count(), 0);
    }
}
