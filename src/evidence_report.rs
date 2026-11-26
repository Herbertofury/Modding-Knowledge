//! Read-only JSON query for consumers; does not create/migrate databases or fetch.
use crate::{Observation, Result, Store, loot};
use serde_json::{Value, json};

const MOD_RELATIONSHIP_PREDICATE: &str = "mod.relationships.v1";

enum Selection<'a> {
    All,
    Predicate(&'a str),
    LootRelations,
    ModRelations(ModProjection),
    Fallout4LootRelations,
}

#[derive(Clone, Copy)]
enum ModProjection {
    Legacy,
    V2,
}

impl ModProjection {
    fn from_flag(flag: &str) -> Option<Self> {
        match flag {
            "--mod-relations" => Some(Self::Legacy),
            "--mod-relations-v2" => Some(Self::V2),
            _ => None,
        }
    }

    fn columns(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Legacy => &[
                ("required", "required"),
                ("optional", "optional"),
                ("works_with", "compatible"),
                ("", "unknown"),
            ],
            Self::V2 => &[
                ("required", "required"),
                ("optional", "optional"),
                ("works_with", "compatible"),
                ("incompatible", "conflict"),
                ("alternative", "alternative"),
                ("", "unknown"),
            ],
        }
    }

    fn schema(self) -> u8 {
        match self {
            Self::Legacy => 1,
            Self::V2 => 2,
        }
    }
}

/// Reusable implementation of the exact read-only query CLI argument contract.
pub fn report(args: &[String]) -> Result<Value> {
    let (database, entity, offset, selection) = match args {
        [db, entity] => (db, entity, 0, Selection::All),
        [db, entity, flag] if flag == "--loot-relations" => {
            (db, entity, 0, Selection::LootRelations)
        }
        [db, entity, offset, flag] if flag == "--loot-relations" => {
            (db, entity, offset.parse::<u32>()?, Selection::LootRelations)
        }
        [db, entity, flag] if flag == "--loot-fallout4-relations" => {
            (db, entity, 0, Selection::Fallout4LootRelations)
        }
        [db, entity, offset, flag] if flag == "--loot-fallout4-relations" => (
            db,
            entity,
            offset.parse::<u32>()?,
            Selection::Fallout4LootRelations,
        ),
        [db, entity, flag] if ModProjection::from_flag(flag).is_some() => (
            db,
            entity,
            0,
            Selection::ModRelations(ModProjection::from_flag(flag).unwrap()),
        ),
        [db, entity, offset, flag] if ModProjection::from_flag(flag).is_some() => (
            db,
            entity,
            offset.parse::<u32>()?,
            Selection::ModRelations(ModProjection::from_flag(flag).unwrap()),
        ),
        [db, entity, offset] => (db, entity, offset.parse::<u32>()?, Selection::All),
        [db, entity, flag, predicate] if flag == "--predicate" => {
            (db, entity, 0, Selection::Predicate(predicate))
        }
        [db, entity, offset, flag, predicate] if flag == "--predicate" => (
            db,
            entity,
            offset.parse::<u32>()?,
            Selection::Predicate(predicate),
        ),
        _ => {
            return Err(
                "Usage: query DATABASE ENTITY_ID [OFFSET] [--predicate EXACT_PREDICATE | --loot-relations | --loot-fallout4-relations | --mod-relations | --mod-relations-v2]".into()
            );
        }
    };
    let predicate = match selection {
        Selection::All => None,
        Selection::Predicate(p) => Some(p),
        Selection::ModRelations(_) => Some(MOD_RELATIONSHIP_PREDICATE),
        Selection::LootRelations | Selection::Fallout4LootRelations => {
            Some(loot::OBSERVATION_PREDICATE)
        }
    };
    let store = Store::open_read_only(database)?;
    let records = match predicate {
        Some(p) => store.observations_page_for_predicate(entity, p, offset)?,
        None => store.observations_page(entity, offset)?,
    };
    let rows:Vec<_>=records.iter().map(|r|json!({"id":r.id,"entity_id":r.entity_id,"source_id":r.source_id,"source_url":r.source_url,"predicate":r.predicate,"value":r.value,"observed_unix":r.observed_unix,"applicable_build":r.applicable_build,"mod_version":r.mod_version,"completeness":r.completeness,"policy_revision":r.policy_revision})).collect();
    let mut output = json!({"schema":1,"entity_id":entity,"offset":offset,"page_limit":1000,"count":rows.len(),"observations":rows});
    if let Some(p) = predicate {
        output["predicate"] = p.into();
        output["coverage_note"] =
            "An empty page does not establish absence; evidence can be unrecorded, unavailable, or outside this page.".into();
    }
    type Project = for<'a> fn(&'a Observation) -> loot::RelationshipEvidence<'a>;
    let loot_scope: Option<(&str, Project)> = match selection {
        Selection::LootRelations => Some((
            "Pinned Oblivion-family LOOT source only; conditions and constraints are not evaluated; not a load-order recommendation.",
            loot::project,
        )),
        Selection::Fallout4LootRelations => Some((
            "Pinned Fallout 4 LOOT source only, not Fallout 4 VR; conditions and constraints are not evaluated; not a load-order recommendation.",
            loot::project_fallout4,
        )),
        _ => None,
    };
    if let Some((scope, project)) = loot_scope {
        output["loot_relationships"] = json!({
            "schema": 1,
            "scope": scope,
            "observations": records.iter().map(|row| project(row).to_json()).collect::<Vec<_>>(),
        });
    }
    if let Selection::ModRelations(mode) = selection {
        output["mod_relationships"] = json!({
            "schema": mode.schema(),
            "coverage": "unknown",
            "scope": "Recorded evidence on this page only. Roles quote source claims; conditions, builds and compatibility are not evaluated. An empty column does not establish absence.",
            "observations": records.iter().map(|row| mod_relationships(row, mode)).collect::<Vec<_>>(),
        });
    }
    Ok(output)
}

// Parse the fields used for classification strictly. In particular, Serde
// rejects duplicate keys for these fields instead of letting the last win.
#[derive(serde_derive::Deserialize)]
struct RelationEnvelope<C, R = RecordedRole> {
    version: u64,
    review: String,
    coverage: C,
    relationships: Vec<R>,
}
#[derive(serde_derive::Deserialize)]
struct RoleCoverage {
    required: Option<String>,
    optional: Option<String>,
    works_with: Option<String>,
}
#[derive(serde_derive::Deserialize)]
struct RecordedRole {
    relation: Option<String>,
    basis: Option<String>,
}

#[derive(serde_derive::Deserialize)]
struct RoleCoverageV2 {
    required: Option<String>,
    optional: Option<String>,
    works_with: Option<String>,
    incompatible: Option<String>,
    alternative: Option<String>,
}

// V2 must not place a conflict into a dedicated column if ambiguous version
// or scope keys could turn the source's narrow claim into a blanket warning.
#[derive(serde_derive::Deserialize)]
struct RecordedRoleV2 {
    relation: Option<String>,
    basis: Option<String>,
    #[serde(rename = "condition")]
    _condition: Option<String>,
    #[serde(rename = "target_id")]
    _target_id: Option<String>,
    #[serde(rename = "version_constraint")]
    _version_constraint: Option<String>,
    #[serde(rename = "target_scope")]
    _target_scope: Option<RecordedScope>,
    #[serde(rename = "raw_relation")]
    _raw_relation: Option<String>,
    #[serde(rename = "label")]
    _label: Option<String>,
    #[serde(rename = "evidence")]
    _evidence: Option<RecordedEvidence>,
}
#[derive(serde_derive::Deserialize)]
struct RecordedEvidence {
    #[serde(rename = "section")]
    _section: Option<String>,
    #[serde(rename = "quote")]
    _quote: Option<String>,
    #[serde(rename = "location")]
    _location: Option<String>,
}
#[derive(serde_derive::Deserialize)]
struct RecordedScope {
    #[serde(rename = "game_id")]
    _game_id: Option<String>,
    #[serde(rename = "mod_version")]
    _mod_version: Option<String>,
    #[serde(rename = "file_id")]
    _file_id: Option<String>,
    #[serde(rename = "runtime")]
    _runtime: Option<String>,
}

type ClassifiedEnvelope = (u64, String, [Option<String>; 5], Vec<RecordedRole>);

fn classify_envelope(raw: &str, mode: ModProjection) -> Option<ClassifiedEnvelope> {
    match mode {
        ModProjection::Legacy => {
            let r: RelationEnvelope<RoleCoverage> = serde_json::from_str(raw).ok()?;
            let c = r.coverage;
            Some((
                r.version,
                r.review,
                [c.required, c.optional, c.works_with, None, None],
                r.relationships,
            ))
        }
        ModProjection::V2 => {
            let r: RelationEnvelope<RoleCoverageV2, RecordedRoleV2> =
                serde_json::from_str(raw).ok()?;
            let c = r.coverage;
            Some((
                r.version,
                r.review,
                [
                    c.required,
                    c.optional,
                    c.works_with,
                    c.incompatible,
                    c.alternative,
                ],
                r.relationships
                    .into_iter()
                    .map(|r| RecordedRole {
                        relation: r.relation,
                        basis: r.basis,
                    })
                    .collect(),
            ))
        }
    }
}

// The raw observation remains in the original output. This view does not
// infer roles from prose or make candidate/unversioned statements actionable.
fn mod_relationships(row: &Observation, mode: ModProjection) -> Value {
    let columns = mode.columns();
    let unknown = columns.len() - 1;
    let mut items: [Vec<Value>; 6] = std::array::from_fn(|_| Vec::new());
    let mut coverage = ["unknown"; 6];
    let mut review = None;
    let mut status = "unrecognized_payload";
    if let Some((1, review_label, reported, roles)) = classify_envelope(&row.value, mode)
        && matches!(review_label.as_str(), "reviewed" | "candidate")
        && let Ok(mut payload) = serde_json::from_str::<Value>(&row.value)
        && let Some(relationships) = payload
            .get_mut("relationships")
            .and_then(Value::as_array_mut)
            .filter(|relationships| {
                matches!(mode, ModProjection::Legacy)
                    || relationships.iter().all(|value| {
                        value.is_object()
                            && ["target_scope", "evidence"].iter().all(|&field| {
                                value
                                    .get(field)
                                    .is_none_or(|v| v.is_null() || v.is_object())
                            })
                    })
            })
    {
        status = "recorded_evidence";
        for (relationship, recorded) in std::mem::take(relationships).into_iter().zip(roles) {
            let role = recorded.relation.as_deref();
            let index = if recorded.basis.as_deref() == Some("explicit") {
                columns[..unknown]
                    .iter()
                    .position(|(r, _)| Some(*r) == role)
            } else {
                None
            };
            items[index.unwrap_or(unknown)].push(relationship);
        }
        review = Some(review_label);
        for (i, reported) in reported.into_iter().take(unknown).enumerate() {
            if !items[i].is_empty() && reported.as_deref() == Some("partial") {
                coverage[i] = "partial";
            }
        }
    }
    json!({
        "observation_id": row.id,
        "source_id": row.source_id,
        "source_url": row.source_url,
        "policy_revision": row.policy_revision,
        "observed_unix": row.observed_unix,
        "applicable_build": row.applicable_build,
        "mod_version": row.mod_version,
        "review": review,
        "status": status,
        "columns": columns.iter().zip(coverage).zip(items).map(|(((_, role), coverage), evidence)| {
            json!({"role": role, "coverage": coverage, "evidence": evidence})
        }).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(value: &str) -> Value {
        project_mode(value, ModProjection::Legacy)
    }

    fn project_mode(value: &str, mode: ModProjection) -> Value {
        mod_relationships(
            &Observation {
                id: "observation".into(),
                entity_id: "project".into(),
                source_id: "source".into(),
                source_url: "https://example.org/source".into(),
                predicate: MOD_RELATIONSHIP_PREDICATE.into(),
                value: value.into(),
                observed_unix: 42,
                applicable_build: "unverified".into(),
                mod_version: "unverified".into(),
                completeness: "partial".into(),
                policy_revision: "fixture".into(),
            },
            mode,
        )
    }

    #[test]
    fn known_role_claims_keep_conditions_and_unknown_is_not_none() {
        let row = project(
            r#"{
            "version":1,"review":"candidate","coverage":{"required":"partial", "optional":"partial"},
            "relationships":[
                {"relation":"required","basis":"explicit","condition":"one-of","target_id":null},
                {"relation":"works_with","basis":"explicit","evidence":{"quote":"core only"}},
                {"relation":"required","basis":"inferred","label":"not a requirement"},
                {"relation":"future_role","basis":"explicit","label":"Requires"}
            ]
        }"#,
        );
        assert_eq!(row["review"], "candidate");
        assert_eq!(row["applicable_build"], "unverified");
        assert_eq!(row["source_url"], "https://example.org/source");
        let c = row["columns"].as_array().unwrap();
        assert_eq!(c[0]["coverage"], "partial");
        assert_eq!(c[0]["evidence"][0]["condition"], "one-of");
        assert!(c[0]["evidence"][0]["target_id"].is_null());
        assert_eq!(c[1]["coverage"], "unknown");
        assert_eq!(c[2]["role"], "compatible");
        assert_eq!(c[2]["evidence"][0]["relation"], "works_with");
        assert_eq!(c[2]["evidence"][0]["evidence"]["quote"], "core only");
        assert_eq!(c[3]["evidence"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn unsupported_and_ambiguous_classification_never_guess() {
        for value in [
            "not JSON",
            r#"{"version":2,"review":"reviewed","coverage":{},"relationships":[]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"works_with","relation":"required","basis":"explicit"}]}"#,
        ] {
            let row = project(value);
            assert_eq!(row["status"], "unrecognized_payload");
            assert!(row["columns"].as_array().unwrap().iter().all(
                |x| x["coverage"] == "unknown" && x["evidence"].as_array().unwrap().is_empty()
            ));
        }
    }

    #[test]
    fn v2_moves_only_exact_explicit_roles_out_of_unknown() {
        let value = r#"{
            "version":1,"review":"candidate",
            "coverage":{"incompatible":"partial","alternative":"partial"},
            "relationships":[
                {"relation":"incompatible","basis":"explicit","label":"setting only",
                 "condition":"one setting is enabled","version_constraint":">= 3",
                 "target_scope":{"game_id":"sims4","file_id":"one-file"}},
                {"relation":"alternative","basis":"explicit","condition":"different pack"},
                {"relation":"incompatible","basis":"inferred"},
                {"relation":"conflict","basis":"explicit"},
                {"relation":"related","basis":"explicit"}
            ]
        }"#;
        let row = project_mode(value, ModProjection::V2);
        assert_eq!(row["review"], "candidate");
        let c = row["columns"].as_array().unwrap();
        assert_eq!(
            c.iter()
                .map(|v| v["role"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "required",
                "optional",
                "compatible",
                "conflict",
                "alternative",
                "unknown"
            ]
        );
        let expected = serde_json::from_str::<Value>(value).unwrap();
        assert_eq!(c[3]["coverage"], "partial");
        assert_eq!(c[3]["evidence"][0], expected["relationships"][0]);
        assert_eq!(c[4]["coverage"], "partial");
        assert_eq!(c[4]["evidence"][0], expected["relationships"][1]);
        assert_eq!(c[5]["coverage"], "unknown");
        assert_eq!(c[5]["evidence"].as_array().unwrap().len(), 3);
        let legacy = project(value);
        assert_eq!(legacy["columns"][3]["role"], "unknown");
        assert_eq!(legacy["columns"][3]["evidence"], expected["relationships"]);
    }

    #[test]
    fn v2_refuses_ambiguous_display_label_and_source_evidence() {
        for raw in [
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"required","basis":"explicit","label":"A","label":"B"}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"required","basis":"explicit","raw_relation":"one-of","raw_relation":"required"}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","basis":"explicit","evidence":{"location":"A"},"evidence":null}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","basis":"explicit","evidence":{"quote":"conditional","quote":"always","location":"A"}}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","basis":"explicit","evidence":{"location":"A","location":"B"}}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","basis":"explicit","evidence":["requirements","conditional","A"]}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","basis":"explicit","target_scope":["skyrim-se",null,null,null]}]}"#,
        ] {
            let r = project_mode(raw, ModProjection::V2);
            assert_eq!(r["status"], "unrecognized_payload");
            assert_eq!(r["source_url"], "https://example.org/source");
            assert!(
                r["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|column| column["evidence"].as_array().unwrap().is_empty())
            );
            assert_eq!(project(raw)["status"], "recorded_evidence");
        }
    }

    #[test]
    fn v2_projects_verified_modhold_optional_and_conditional_relationships() {
        let pack: Value = serde_json::from_str(include_str!(
            "../data/skyrim-se-modhold-seth-relationships-reviewed.json"
        ))
        .unwrap();
        let observations = pack["observations"].as_array().unwrap();
        let recorded = |id: &str, predicate: &str| {
            observations
                .iter()
                .find(|row| row["entity_id"] == id && row["predicate"] == predicate)
                .unwrap()["value"]
                .as_str()
                .unwrap()
        };
        let oskar = project_mode(
            recorded("modhold:mod:13", MOD_RELATIONSHIP_PREDICATE),
            ModProjection::V2,
        );
        assert_eq!(oskar["status"], "recorded_evidence");
        assert_eq!(oskar["columns"][0]["role"], "required");
        let required = oskar["columns"][0]["evidence"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert_eq!(required[0]["evidence"]["nexus_mod_id"], 63772);
        assert_eq!(required[1]["target_scope"]["runtime"], "<1.6.1130");
        assert!(required[1]["version_constraint"].is_null());
        let compatible = oskar["columns"][2]["evidence"].as_array().unwrap();
        assert_eq!(compatible.len(), 2);
        assert_eq!(compatible[0]["target_id"], "modhold:mod:11");
        assert_eq!(compatible[1]["target_id"], "modhold:mod:6");
        assert_eq!(compatible[1]["version_constraint"], ">=1.2.7");
        assert_eq!(compatible[1]["evidence"]["provider_required"], false);
        assert!(compatible[1]["target_scope"]["runtime"].is_null());
        let warband = project_mode(
            recorded("modhold:mod:6", MOD_RELATIONSHIP_PREDICATE),
            ModProjection::V2,
        );
        assert_eq!(warband["columns"][0]["coverage"], "unknown");
        assert_eq!(warband["columns"][3]["coverage"], "unknown");
        let alternatives = warband["columns"][4]["evidence"].as_array().unwrap();
        assert_eq!(alternatives.len(), 2);
        assert_eq!(alternatives[1]["version_constraint"], ">=1.6");
        let files_source = recorded("modhold:mod:6", "reviewed.modhold.file_metadata");
        let files: Value = serde_json::from_str(files_source).unwrap();
        assert_eq!(files["files"][0]["file_id"], 2937);
        assert_eq!(files["files"][0]["kind"], "main");
        assert_eq!(files["files"][1]["file_id"], 560);
        assert_eq!(files["files"][1]["kind"], "optional");
        assert_eq!(files["files"][1]["verified_archive_bytes"], false);
    }

    #[test]
    fn v2_retains_complete_source_fields_from_actual_reviewed_packs() {
        let mut checked = 0;
        for pack in [
            include_str!("../data/skyrim-se-distributors-reviewed.json"),
            include_str!("../data/sims4-utility-relationships-reviewed.json"),
        ] {
            let bundle: Value = serde_json::from_str(pack).unwrap();
            for row in bundle["observations"].as_array().unwrap() {
                if row["predicate"] != MOD_RELATIONSHIP_PREDICATE {
                    continue;
                }
                let raw = row["value"].as_str().unwrap();
                let expected: Value = serde_json::from_str(raw).unwrap();
                let actual = project_mode(raw, ModProjection::V2);
                assert_eq!(actual["status"], "recorded_evidence");
                let projected: Vec<&Value> = actual["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|c| c["evidence"].as_array().unwrap())
                    .collect();
                let expected = expected["relationships"].as_array().unwrap();
                assert_eq!(projected.len(), expected.len());
                assert!(expected.iter().all(|claim| projected.contains(&claim)));
                checked += 1;
            }
        }
        assert!(checked >= 20, "expected both source packs, found {checked}");
    }

    #[test]
    fn v2_rejects_ambiguous_scope_and_never_claims_absence() {
        for raw in [
            r#"{"version":1,"review":"reviewed","coverage":{"incompatible":"partial","incompatible":"unknown"},"relationships":[]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","basis":"explicit","condition":"scope","condition":null}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","basis":"explicit","target_scope":{"file_id":"f","file_id":null}}]}"#,
            r#"{"version":1,"review":"reviewed","coverage":{},"relationships":[{"relation":"incompatible","relation":"alternative","basis":"explicit"}]}"#,
            r#"{"version":2,"review":"reviewed","coverage":{},"relationships":[]}"#,
        ] {
            let r = project_mode(raw, ModProjection::V2);
            assert_eq!(r["status"], "unrecognized_payload");
            assert!(
                r["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|c| c["coverage"] == "unknown")
            );
        }
        let r = project_mode(
            r#"{"version":1,"review":"reviewed","coverage":{"incompatible":"complete"},"relationships":[]}"#,
            ModProjection::V2,
        );
        assert_eq!(r["columns"][3]["coverage"], "unknown");
    }
}
