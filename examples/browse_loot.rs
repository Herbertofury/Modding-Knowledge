//! Browse attributed LOOT source evidence without resolving or executing it.
use athena_mod_knowledge::{
    Result, Store,
    loot::{BrowseCursor, SourceRole},
};
use serde_json::json;

fn main() -> Result<()> {
    const USAGE: &str = "Usage: browse_loot DATABASE ROLE [--fallout4] [--target EXACT_LABEL] [--after UNIX OBSERVATION_ID]\nRoles: requires, incompatible, load-after-if-present";
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [db, role, rest @ ..] = args.as_slice() else {
        return Err(USAGE.into());
    };
    let role_filter = match role.as_str() {
        "requires" => SourceRole::Requires,
        "incompatible" => SourceRole::Incompatible,
        "load-after-if-present" => SourceRole::LoadAfterIfPresent,
        _ => return Err(USAGE.into()),
    };
    let (fallout4, rest) = match rest {
        [flag, rest @ ..] if flag == "--fallout4" => (true, rest),
        _ => (false, rest),
    };
    let (label, rest) = match rest {
        [flag, label, rest @ ..] if flag == "--target" => (Some(label.as_str()), rest),
        _ => (None, rest),
    };
    let after = match rest {
        [] => None,
        [flag, time, id] if flag == "--after" => Some(BrowseCursor {
            observed_unix: time.parse()?,
            observation_id: id.clone(),
        }),
        _ => return Err(USAGE.into()),
    };

    let store = Store::open_read_only(db)?;
    let (page, scope) = if fallout4 {
        (
            store.browse_loot_fallout4(role_filter, label, after.as_ref())?,
            "Pinned Fallout 4 LOOT source only, not Fallout 4 VR. Conditions and constraints are not evaluated.",
        )
    } else {
        (
            store.browse_loot(role_filter, label, after.as_ref())?,
            "Pinned Oblivion-family LOOT source only. Conditions and constraints are not evaluated.",
        )
    };
    let matches: Vec<_> = page
        .matches
        .iter()
        .map(|m| {
            let r = &m.observation;
            json!({
                "id": r.id, "entity_id": r.entity_id, "source_id": r.source_id,
                "source_url": r.source_url, "predicate": r.predicate, "value": r.value,
                "observed_unix": r.observed_unix, "applicable_build": r.applicable_build,
                "mod_version": r.mod_version, "completeness": r.completeness,
                "policy_revision": r.policy_revision,
                "relationships": m.relationships_json(),
            })
        })
        .collect();
    let next = page
        .next
        .map(|k| json!({"observed_unix": k.observed_unix, "observation_id": k.observation_id}));
    let output = json!({
        "schema": 1,
        "scope": scope,
        "coverage_note": "Fewer matches or an empty page does not establish absence or compatibility.",
        "role": role, "exact_target_label": label,
        "count": matches.len(), "scanned": page.scanned, "matches": matches, "next": next,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
