//! Read-only JSON query for consumers; does not create/migrate databases or fetch.
use athena_mod_knowledge::{Observation, Result, Store, loot};
use serde_json::{Value, json};

const MOD_RELATIONSHIP_PREDICATE: &str = "mod.relationships.v1";

enum Selection<'a> {
    All,
    Predicate(&'a str),
    LootRelations,
    ModRelations,
    Fallout4LootRelations,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let (database, entity, offset, selection) = match &args[1..] {
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
        [db, entity, flag] if flag == "--mod-relations" => (db, entity, 0, Selection::ModRelations),
        [db, entity, offset, flag] if flag == "--mod-relations" => {
            (db, entity, offset.parse::<u32>()?, Selection::ModRelations)
        }
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
                "Usage: query DATABASE ENTITY_ID [OFFSET] [--predicate EXACT_PREDICATE | --loot-relations | --loot-fallout4-relations | --mod-relations]".into()
            );
        }
    };
    let predicate = match selection {
        Selection::All => None,
        Selection::Predicate(p) => Some(p),
        Selection::ModRelations => Some(MOD_RELATIONSHIP_PREDICATE),
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
    if matches!(selection, Selection::ModRelations) {
        output["mod_relationships"] = json!({
            "schema": 1,
            "coverage": "unknown",
            "scope": "Recorded evidence on this page only. Roles quote source claims; conditions, builds and compatibility are not evaluated. An empty column does not establish absence.",
            "observations": records.iter().map(mod_relationships).collect::<Vec<_>>(),
        });
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// Parse the fields used for classification strictly. In particular, Serde
// rejects duplicate keys for these fields instead of letting the last win.
#[derive(serde_derive::Deserialize)]
struct RelationEnvelope {
    version: u64,
    review: String,
    coverage: RoleCoverage,
    relationships: Vec<RecordedRole>,
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

// The raw observation remains in the original output. This view does not
// infer roles from prose or make candidate/unversioned statements actionable.
fn mod_relationships(row: &Observation) -> Value {
    let labels = ["required", "optional", "compatible", "unknown"];
    let stored_labels = ["required", "optional", "works_with"];
    let mut items: [Vec<Value>; 4] = std::array::from_fn(|_| Vec::new());
    let mut coverage = ["unknown"; 4];
    let mut review = None;
    let mut status = "unrecognized_payload";
    if let Ok(envelope) = serde_json::from_str::<RelationEnvelope>(&row.value)
        && envelope.version == 1
        && matches!(envelope.review.as_str(), "reviewed" | "candidate")
        && let Ok(mut payload) = serde_json::from_str::<Value>(&row.value)
        && let Some(relationships) = payload
            .get_mut("relationships")
            .and_then(Value::as_array_mut)
    {
        status = "recorded_evidence";
        for (relationship, recorded) in std::mem::take(relationships)
            .into_iter()
            .zip(envelope.relationships)
        {
            let role = recorded.relation.as_deref();
            let index = if recorded.basis.as_deref() == Some("explicit") {
                stored_labels.iter().position(|r| Some(*r) == role)
            } else {
                None
            };
            items[index.unwrap_or(3)].push(relationship);
        }
        review = Some(envelope.review);
        for (i, reported) in [
            envelope.coverage.required,
            envelope.coverage.optional,
            envelope.coverage.works_with,
        ]
        .into_iter()
        .enumerate()
        {
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
        "columns": labels.into_iter().zip(coverage).zip(items).map(|((role, coverage), evidence)| {
            json!({"role": role, "coverage": coverage, "evidence": evidence})
        }).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(value: &str) -> Value {
        mod_relationships(&Observation {
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
        })
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
}
