//! Read-only JSON query for consumers; does not create/migrate databases or fetch.
use athena_mod_knowledge::{Result, Store};
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        return Err("Usage: query DATABASE ENTITY_ID [OFFSET]".into());
    }
    let offset = if args.len() == 4 {
        args[3].parse::<u32>()?
    } else {
        0
    };
    let store = Store::open_read_only(&args[1])?;
    let records = store.observations_page(&args[2], offset)?;
    let rows:Vec<_>=records.iter().map(|r|serde_json::json!({"id":r.id,"entity_id":r.entity_id,"source_id":r.source_id,"source_url":r.source_url,"predicate":r.predicate,"value":r.value,"observed_unix":r.observed_unix,"applicable_build":r.applicable_build,"mod_version":r.mod_version,"completeness":r.completeness,"policy_revision":r.policy_revision})).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"schema":1,"entity_id":args[2],"offset":offset,"page_limit":1000,"count":rows.len(),"observations":rows})
        )?
    );
    Ok(())
}
