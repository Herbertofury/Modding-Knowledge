//! Read-only cursor pagination; separate from the existing offset JSON contract.
use athena_mod_knowledge::{ObservationAnchor, Result, Store};
use serde_json::json;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        return Err(
            "Usage: query_cursor DATABASE ENTITY_ID [--predicate EXACT] [--after TIME ID]".into(),
        );
    }
    let (database, entity) = (&args[0], &args[1]);
    let (mut predicate, mut anchor) = (None, None);
    let mut remaining = &args[2..];
    while !remaining.is_empty() {
        remaining = match remaining {
            [flag, exact, rest @ ..] if flag == "--predicate" && predicate.is_none() => {
                predicate = Some(exact.as_str());
                rest
            }
            [flag, time, id, rest @ ..] if flag == "--after" && anchor.is_none() => {
                anchor = Some(ObservationAnchor {
                    observed_unix: time.parse()?,
                    id,
                });
                rest
            }
            _ => return Err("Use at most one --predicate EXACT and one --after TIME ID".into()),
        };
    }
    let store = Store::open_read_only(database)?;
    let observations = store.observations_after(entity, predicate, anchor)?;
    let cursor = if observations.len() == 1000 {
        observations
            .last()
            .map(|row| json!({"observed_unix":row.observed_unix,"id":row.id}))
    } else {
        None
    };
    let rows: Vec<_> = observations
        .iter()
        .map(|r| {
            json!({
                "id":r.id,"entity_id":r.entity_id,"source_id":r.source_id,"source_url":r.source_url,
                "predicate":r.predicate,"value":r.value,"observed_unix":r.observed_unix,
                "applicable_build":r.applicable_build,"mod_version":r.mod_version,
                "completeness":r.completeness,"policy_revision":r.policy_revision
            })
        })
        .collect();
    let output = json!({
        "schema":1,"pagination":"live-exclusive-cursor","entity_id":entity,
        "predicate":predicate,"count":rows.len(),"page_limit":1000,"next_cursor":cursor,
        "observations":rows,"coverage_note":"Missing evidence does not establish absence."
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
