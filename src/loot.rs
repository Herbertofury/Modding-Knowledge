//! Read-only relationship projection for the reviewed Oblivion-family LOOT tranche.
//! These are source facts, not evaluated behavior, dependencies or load-order advice.
use crate::Observation;
use serde_json::{Value, json};

/// Pass this exact predicate to [crate::Store::observations_page_for_predicate].
pub const OBSERVATION_PREDICATE: &str = "conditional_plugin_metadata";
const COMMIT: &str = "b3ffc84f6d730f6efb5083448d78611df16e8d78";
const BLOB: &str = "8c613acedf019890b950e4c4b2450151cead1025";
const SOURCE: &str = "https://github.com/loot/oblivion/blob/b3ffc84f6d730f6efb5083448d78611df16e8d78/masterlist.yaml";

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

/// Exact source roles kept separate. Load-after entries are not requirements.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceFields {
    /// This may be a pattern as written by the source, not a resolved identity.
    pub plugin_source_label: String,
    pub requirements: SourceFileSet,
    pub incompatibilities: SourceFileSet,
    pub load_after_if_present: SourceFileSet,
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
            Some(fields) => json!({
                "observation_id": id,
                "status": "reviewable_source_fields",
                "plugin_source_label": fields.plugin_source_label,
                "conditions_and_constraints": "not_evaluated",
                "source_declared_requirements": fields.requirements.to_json(),
                "source_declared_incompatibilities": fields.incompatibilities.to_json(),
                "source_load_after_if_present": fields.load_after_if_present.to_json(),
            }),
            None => json!({"observation_id": id, "status": "malformed_or_unsupported"}),
        }
    }
}

/// Inspect one existing observation without evaluating its conditions or following its links.
/// This does not authorize the input; read through the Store for its current source-policy check.
pub fn project(observation: &Observation) -> RelationshipEvidence<'_> {
    RelationshipEvidence {
        observation,
        fields: parse_fields(observation),
    }
}

fn parse_fields(r: &Observation) -> Option<SourceFields> {
    if r.source_id != "loot-oblivion"
        || r.source_url != SOURCE
        || r.predicate != OBSERVATION_PREDICATE
    {
        return None;
    }
    let mut value: Value = serde_json::from_str(&r.value).ok()?;
    if value["source_commit_sha"] != COMMIT
        || value["source_blob_sha"] != BLOB
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
