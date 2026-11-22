//! Read-only relationship projection for separately pinned LOOT source tranches.
//! These are source facts, not evaluated behavior, dependencies or load-order advice.
use crate::{Observation, Result, Store, observation_from_row};
use rusqlite::params;
use serde_json::{Value, json};

/// Pass this exact predicate to [crate::Store::observations_page_for_predicate].
pub const OBSERVATION_PREDICATE: &str = "conditional_plugin_metadata";
const COMMIT: &str = "b3ffc84f6d730f6efb5083448d78611df16e8d78";
const BLOB: &str = "8c613acedf019890b950e4c4b2450151cead1025";
const SOURCE_ID: &str = "loot-oblivion";
const SOURCE: &str = "https://github.com/loot/oblivion/blob/b3ffc84f6d730f6efb5083448d78611df16e8d78/masterlist.yaml";
const FALLOUT4_COMMIT: &str = "4c7858a84bc65b34904b53c5e6252d3f6f9b1247";
const FALLOUT4_BLOB: &str = "eed7994e786c896463887d03bcb3f14426b37753";
const FALLOUT4_SOURCE: &str = "https://github.com/loot/fallout4/blob/4c7858a84bc65b34904b53c5e6252d3f6f9b1247/masterlist.yaml";
const FALLOUT4_PREFIX: &str = "loot:fallout4:plugin-metadata:";

#[derive(Clone, Copy)]
struct Pin {
    id: &'static str,
    url: &'static str,
    commit: &'static str,
    blob: &'static str,
}

const OBLIVION: Pin = Pin {
    id: SOURCE_ID,
    url: SOURCE,
    commit: COMMIT,
    blob: BLOB,
};
const FALLOUT4: Pin = Pin {
    id: "loot-fallout4",
    url: FALLOUT4_SOURCE,
    commit: FALLOUT4_COMMIT,
    blob: FALLOUT4_BLOB,
};

/// Distinguishes what the source recorded from what has been verified in a game.
#[derive(Debug, PartialEq, Eq)]
pub enum SourceFileSet {
    /// The source observation does not contain this field.
    NotRecorded,
    /// An explicit empty array; not proof of compatibility or no relationships.
    RecordedEmpty,
    /// Recognized shapes; all original entries and conditions are still unevaluated.
    RecordedUnevaluated(Vec<Value>),
    /// The exact raw value is still available, including explicit null.
    MalformedOrUnsupported(Value),
}

impl SourceFileSet {
    fn from_source(field: Option<Value>) -> Self {
        match field {
            None => Self::NotRecorded,
            Some(Value::Array(entries)) if entries.is_empty() => Self::RecordedEmpty,
            Some(Value::Array(entries)) if entries.iter().all(supported_file_entry) => {
                Self::RecordedUnevaluated(entries)
            }
            Some(raw) => Self::MalformedOrUnsupported(raw),
        }
    }

    fn to_json(&self) -> Value {
        match self {
            Self::NotRecorded => json!({"status": "not_recorded", "raw": null}),
            Self::RecordedEmpty => json!({"status": "recorded_empty", "raw": []}),
            Self::RecordedUnevaluated(entries) => {
                json!({"status": "recorded_unevaluated", "raw": entries})
            }
            Self::MalformedOrUnsupported(raw) => {
                json!({"status": "malformed_or_unsupported", "raw": raw})
            }
        }
    }
}

/// Role as recorded in the pinned source, not as interpreted by the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceRole {
    Requires,
    Incompatible,
    LoadAfterIfPresent,
}

/// Resume the same role/label query on the same immutable snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowseCursor {
    pub observed_unix: i64,
    pub observation_id: String,
}

/// The full observation is retained for provenance and access to the original raw row.
#[derive(Debug, PartialEq, Eq)]
pub struct BrowseMatch {
    pub observation: Observation,
    pub fields: SourceFields,
}

impl BrowseMatch {
    /// Role details in the same shape used by the existing entity relationship view.
    pub fn relationships_json(&self) -> Value {
        self.fields.to_relationships_json(&self.observation.id)
    }
}

/// One bounded work slice, which may be empty with a nonempty continuation.
#[derive(Debug, PartialEq, Eq)]
pub struct BrowsePage {
    pub matches: Vec<BrowseMatch>,
    pub scanned: usize,
    pub next: Option<BrowseCursor>,
}

pub const BROWSE_PAGE_LIMIT: usize = 100;
pub const BROWSE_SCAN_LIMIT: usize = 256;

impl Store {
    /// Browse evidenced source labels across the reviewed Oblivion-family source.
    ///
    /// Public policy and its current revision are checked before scanning. The optional label
    /// is literal and case-sensitive, never an identity or regex lookup. Only recognized,
    /// nonempty role fields match. A short/empty page is not proof of absence; it may carry
    /// a continuation. No condition, constraint, or runtime behavior is evaluated.
    pub fn browse_loot(
        &self,
        role: SourceRole,
        exact_target_label: Option<&str>,
        after: Option<&BrowseCursor>,
    ) -> Result<BrowsePage> {
        if exact_target_label.is_some_and(|s| s.trim().is_empty() || s.len() > 1024) {
            return Err("Expected an exact nonblank target label of at most 1024 bytes".into());
        }
        if after.is_some_and(|c| c.observed_unix < 0 || c.observation_id.is_empty()) {
            return Err("Expected a nonnegative timestamp and a nonempty observation ID".into());
        }
        // Limit the key-only sort before reading raw JSON (up to 64 KiB per observation).
        // The existing schema has no global source/time index; the database may scan keys
        // across the snapshot, but snapshot size does not increase JSON projection per call.
        macro_rules! browse_sql {
            ($after:literal) => {
                concat!(
                    "WITH visible AS MATERIALIZED (SELECT o.id,o.observed_unix ",
                    "FROM observations o JOIN sources s ON s.id=o.source_id ",
                    "WHERE s.can_retain=1 AND s.can_publish=1 AND o.policy_revision=s.policy_revision ",
                    "AND o.source_id=?1 AND o.source_url=?2 AND o.predicate=?3",
                    $after,
                    " ORDER BY o.observed_unix DESC,o.id LIMIT ?4) ",
                    "SELECT o.id,o.entity_id,o.source_id,o.source_url,o.predicate,o.value,",
                    "o.observed_unix,o.applicable_build,o.mod_version,o.completeness,o.policy_revision ",
                    "FROM visible v JOIN observations o ON o.id=v.id ORDER BY v.observed_unix DESC,v.id"
                )
            };
        }
        let mut query = self.db.prepare_cached(if after.is_some() {
            browse_sql!(" AND (o.observed_unix<?5 OR (o.observed_unix=?5 AND o.id>?6))")
        } else {
            browse_sql!("")
        })?;
        let capacity = (BROWSE_SCAN_LIMIT + 1) as i64;
        let mut rows = match after {
            Some(c) => query.query(params![
                SOURCE_ID,
                SOURCE,
                OBSERVATION_PREDICATE,
                capacity,
                c.observed_unix,
                c.observation_id,
            ])?,
            None => query.query(params![SOURCE_ID, SOURCE, OBSERVATION_PREDICATE, capacity])?,
        };
        let mut matches = Vec::new();
        let mut scanned = 0;
        let mut last = None;
        while let Some(row) = rows.next()? {
            if scanned == BROWSE_SCAN_LIMIT || matches.len() == BROWSE_PAGE_LIMIT {
                return Ok(BrowsePage {
                    matches,
                    scanned,
                    next: last,
                });
            }
            let observation = observation_from_row(row)?;
            scanned += 1;
            let observed_unix = observation.observed_unix;
            let observation_id = if let Some(fields) = parse_fields(&observation, OBLIVION)
                && fields.has_target(role, exact_target_label)
            {
                let id = observation.id.clone();
                matches.push(BrowseMatch {
                    observation,
                    fields,
                });
                id
            } else {
                observation.id
            };
            last = Some(BrowseCursor {
                observed_unix,
                observation_id,
            });
        }
        Ok(BrowsePage {
            matches,
            scanned,
            next: None,
        })
    }
}

/// Exact source roles kept separate. Load-after entries are not requirements.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceFields {
    /// This may be a pattern as written by the source, not a resolved identity.
    pub plugin_source_label: String,
    pub requirements: SourceFileSet,
    pub incompatibilities: SourceFileSet,
    pub load_after_if_present: SourceFileSet,
}

impl SourceFields {
    fn to_relationships_json(&self, id: &str) -> Value {
        json!({
            "observation_id": id,
            "status": "reviewable_source_fields",
            "plugin_source_label": self.plugin_source_label,
            "conditions_and_constraints": "not_evaluated",
            "source_declared_requirements": self.requirements.to_json(),
            "source_declared_incompatibilities": self.incompatibilities.to_json(),
            "source_load_after_if_present": self.load_after_if_present.to_json(),
        })
    }

    fn has_target(&self, role: SourceRole, exact_label: Option<&str>) -> bool {
        let field = match role {
            SourceRole::Requires => &self.requirements,
            SourceRole::Incompatible => &self.incompatibilities,
            SourceRole::LoadAfterIfPresent => &self.load_after_if_present,
        };
        let SourceFileSet::RecordedUnevaluated(entries) = field else {
            return false;
        };
        exact_label.is_none_or(|label| {
            entries.iter().any(|entry| {
                entry
                    .as_str()
                    .or_else(|| entry.get("name").and_then(Value::as_str))
                    == Some(label)
            })
        })
    }
}

/// Projected roles stay attached to the actual provenance-bearing observation.
#[derive(Debug, PartialEq, Eq)]
pub struct RelationshipEvidence<'a> {
    observation: &'a Observation,
    fields: Option<SourceFields>,
}

impl<'a> RelationshipEvidence<'a> {
    /// Policy-checked input when the observation comes directly from the Store.
    pub fn observation(&self) -> &'a Observation {
        self.observation
    }

    /// None means malformed or out-of-scope source data, not no relationships.
    pub fn fields(&self) -> Option<&SourceFields> {
        self.fields.as_ref()
    }

    /// The v1 row shape emitted by the read-only query example.
    pub fn to_json(&self) -> Value {
        let id = &self.observation.id;
        match &self.fields {
            Some(fields) => fields.to_relationships_json(id),
            None => json!({"observation_id": id, "status": "malformed_or_unsupported"}),
        }
    }
}

/// Inspect one existing observation without evaluating its conditions or following its links.
/// This does not authorize the input; read through the Store for its current source-policy check.
pub fn project(observation: &Observation) -> RelationshipEvidence<'_> {
    RelationshipEvidence {
        observation,
        fields: parse_fields(observation, OBLIVION),
    }
}

/// Read a separately pinned Fallout 4 LOOT observation, not Fallout 4 VR.
/// A caller-supplied row does not verify provenance or grant admission; obtain
/// observations through the policy-filtered Store when reading a snapshot.
pub fn project_fallout4(observation: &Observation) -> RelationshipEvidence<'_> {
    let exact_plugin = observation
        .entity_id
        .strip_prefix(FALLOUT4_PREFIX)
        .is_some_and(|suffix| {
            suffix.len() == 20
                && suffix
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        });
    RelationshipEvidence {
        observation,
        fields: if exact_plugin {
            parse_fields(observation, FALLOUT4)
        } else {
            None
        },
    }
}

fn parse_fields(r: &Observation, pin: Pin) -> Option<SourceFields> {
    if r.source_id != pin.id || r.source_url != pin.url || r.predicate != OBSERVATION_PREDICATE {
        return None;
    }
    let mut value: Value = serde_json::from_str(&r.value).ok()?;
    if value["source_commit_sha"] != pin.commit
        || value["source_blob_sha"] != pin.blob
        || value["condition_evaluation"] != "not_evaluated"
    {
        return None;
    }
    let plugin = value.get_mut("plugin_metadata")?.as_object_mut()?;
    let label = plugin.get("name")?.as_str()?.to_owned();
    if label.trim().is_empty() {
        return None;
    }
    Some(SourceFields {
        plugin_source_label: label,
        requirements: SourceFileSet::from_source(plugin.remove("req")),
        incompatibilities: SourceFileSet::from_source(plugin.remove("inc")),
        load_after_if_present: SourceFileSet::from_source(plugin.remove("after")),
    })
}

// Shape validation only. Nothing here checks existence or evaluates a LOOT expression.
fn supported_file_entry(entry: &Value) -> bool {
    match entry {
        Value::String(name) => !name.trim().is_empty(),
        Value::Object(fields) => {
            fields
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty())
                && fields.iter().all(|(key, value)| match key.as_str() {
                    "name" | "display" | "condition" | "constraint" | "detail" => value.is_string(),
                    _ => false,
                })
        }
        _ => false,
    }
}
