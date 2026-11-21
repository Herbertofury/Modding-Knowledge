//! Read-only JSON query for consumers; does not create/migrate databases or fetch.
use athena_mod_knowledge::{Result, Store};
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let (database, entity, offset, predicate) = match &args[1..] {
        [db, entity] => (db, entity, 0, None),
        [db, entity, offset] => (db, entity, offset.parse::<u32>()?, None),
        [db, entity, flag, predicate] if flag == "--predicate" => {
            (db, entity, 0, Some(predicate.as_str()))
        }
        [db, entity, offset, flag, predicate] if flag == "--predicate" => {
            (db, entity, offset.parse::<u32>()?, Some(predicate.as_str()))
        }
        _ => {
            return Err(
                "Usage: query DATABASE ENTITY_ID [OFFSET] [--predicate EXACT_PREDICATE]".into(),
            );
        }
    };
    let store = Store::open_read_only(database)?;
    let records = match predicate {
        Some(p) => store.observations_page_for_predicate(entity, p, offset)?,
        None => store.observations_page(entity, offset)?,
    };
    let rows:Vec<_>=records.iter().map(|r|serde_json::json!({"id":r.id,"entity_id":r.entity_id,"source_id":r.source_id,"source_url":r.source_url,"predicate":r.predicate,"value":r.value,"observed_unix":r.observed_unix,"applicable_build":r.applicable_build,"mod_version":r.mod_version,"completeness":r.completeness,"policy_revision":r.policy_revision})).collect();
    let mut output = serde_json::json!({"schema":1,"entity_id":entity,"offset":offset,"page_limit":1000,"count":rows.len(),"observations":rows});
    if let Some(p) = predicate {
        output["predicate"] = p.into();
        output["coverage_note"] =
            "An empty page does not establish absence; evidence can be unrecorded, unavailable, or outside this page.".into();
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
