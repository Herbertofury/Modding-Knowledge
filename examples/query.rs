//! Read-only JSON query for consumers; does not create/migrate databases or fetch.
use athena_mod_knowledge::{Observation, Result, Store, loot};
use serde_json::json;

enum Selection<'a> {
    All,
    Predicate(&'a str),
    LootRelations,
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
                "Usage: query DATABASE ENTITY_ID [OFFSET] [--predicate EXACT_PREDICATE | --loot-relations | --loot-fallout4-relations]".into()
            );
        }
    };
    let predicate = match selection {
        Selection::All => None,
        Selection::Predicate(p) => Some(p),
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
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
