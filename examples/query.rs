//! Read-only JSON query for consumers; does not create/migrate databases or fetch.
use athena_mod_knowledge::{Observation, Result, Store};
use serde_json::{Value, json};

const LOOT_PREDICATE: &str = "conditional_plugin_metadata";
const LOOT_COMMIT: &str = "b3ffc84f6d730f6efb5083448d78611df16e8d78";
const LOOT_BLOB: &str = "8c613acedf019890b950e4c4b2450151cead1025";
const LOOT_SOURCE: &str = "https://github.com/loot/oblivion/blob/b3ffc84f6d730f6efb5083448d78611df16e8d78/masterlist.yaml";

enum Selection<'a> {
    All,
    Predicate(&'a str),
    LootRelations,
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
                "Usage: query DATABASE ENTITY_ID [OFFSET] [--predicate EXACT_PREDICATE | --loot-relations]".into()
            );
        }
    };
    let predicate = match selection {
        Selection::All => None,
        Selection::Predicate(p) => Some(p),
        Selection::LootRelations => Some(LOOT_PREDICATE),
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
    if matches!(selection, Selection::LootRelations) {
        output["loot_relationships"] = json!({
            "schema": 1,
            "scope": "Pinned Oblivion-family LOOT source only; conditions and constraints are not evaluated; not a load-order recommendation.",
            "observations": records.iter().map(project_loot).collect::<Vec<_>>(),
        });
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn project_loot(r: &Observation) -> Value {
    let unsupported = || json!({"observation_id": r.id, "status": "malformed_or_unsupported"});
    if r.source_id != "loot-oblivion"
        || r.source_url != LOOT_SOURCE
        || r.predicate != LOOT_PREDICATE
    {
        return unsupported();
    }
    let Ok(value) = serde_json::from_str::<Value>(&r.value) else {
        return unsupported();
    };
    if value["source_commit_sha"] != LOOT_COMMIT
        || value["source_blob_sha"] != LOOT_BLOB
        || value["condition_evaluation"] != "not_evaluated"
    {
        return unsupported();
    }
    let Some(plugin) = value["plugin_metadata"].as_object() else {
        return unsupported();
    };
    let Some(name) = plugin
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
    else {
        return unsupported();
    };
    json!({
        "observation_id": r.id,
        "status": "reviewable_source_fields",
        "plugin_source_label": name,
        "conditions_and_constraints": "not_evaluated",
        "source_declared_requirements": file_set(plugin.get("req")),
        "source_declared_incompatibilities": file_set(plugin.get("inc")),
        "source_load_after_if_present": file_set(plugin.get("after")),
    })
}

fn file_set(field: Option<&Value>) -> Value {
    let status = match field {
        None => "not_recorded",
        Some(Value::Array(entries)) if entries.is_empty() => "recorded_empty",
        Some(Value::Array(entries)) if entries.iter().all(supported_file_entry) => {
            "recorded_unevaluated"
        }
        Some(_) => "malformed_or_unsupported",
    };
    json!({"status": status, "raw": field})
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_sets_keep_empty_unknown_and_unsupported_separate() {
        assert_eq!(file_set(None)["status"], "not_recorded");
        assert_eq!(file_set(Some(&json!([])))["status"], "recorded_empty");
        for malformed in [
            json!(null),
            json!(["file.esm", 5]),
            json!([{"name":"F.esm","condition":false}]),
            json!([{"name":"F.esm","future":true}]),
            json!([{"name":"F.esm","detail":[{"lang":"en","text":"unprojected"}]}]),
        ] {
            assert_eq!(
                file_set(Some(&malformed)),
                json!({"status":"malformed_or_unsupported","raw":malformed})
            );
        }
        let raw = json!(["A.esp",{"name":"Other.dll","condition":"file(\"A.esp\")","constraint":"version(\"Other.dll\", ==, \"1\")","display":"Do not interpret"}]);
        assert_eq!(
            file_set(Some(&raw)),
            json!({"status":"recorded_unevaluated","raw":raw})
        );
    }

    #[test]
    fn pin_and_role_names_do_not_infer_mod_identity_or_runtime() {
        let mut relation = Observation {
            id:"synthetic".into(), entity_id:"fixture".into(), source_id:"loot-oblivion".into(),
            source_url: LOOT_SOURCE.into(), predicate: LOOT_PREDICATE.into(),
            value:json!({
                "source_commit_sha":LOOT_COMMIT,"source_blob_sha":LOOT_BLOB,
                "condition_evaluation":"not_evaluated",
                "plugin_metadata":{"name":"Fixture(.*)\\.esp","req":["A.esp"],"after":["B.esp"],"inc":[]}
            }).to_string(),
            observed_unix:1, applicable_build:"conditional".into(), mod_version:"unverified".into(),
            completeness:"partial".into(), policy_revision:"test".into()
        };
        let view = project_loot(&relation);
        assert_eq!(view["observation_id"], "synthetic");
        assert_eq!(view["plugin_source_label"], "Fixture(.*)\\.esp");
        assert_eq!(view["conditions_and_constraints"], "not_evaluated");
        assert_eq!(
            view["source_declared_requirements"]["raw"],
            json!(["A.esp"])
        );
        assert_eq!(
            view["source_load_after_if_present"]["raw"],
            json!(["B.esp"])
        );
        assert_eq!(
            view["source_declared_incompatibilities"]["status"],
            "recorded_empty"
        );
        relation.source_url = "https://example.org/not-the-pinned-author".into();
        assert_eq!(
            project_loot(&relation)["status"],
            "malformed_or_unsupported"
        );
    }
}
