//! Build a new reviewed snapshot; no external JSON may self-grant publication permission.
use athena_mod_knowledge::{Observation, Result, Store};
use serde_json::Value;
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err("Expected new SQLite output path".into());
    }
    if std::path::Path::new(&args[1]).exists() {
        return Err("Refusing overwrite".into());
    }
    let data: Value = serde_json::from_str(include_str!("../data/reviewed-bundle.json"))?;
    // Build beside target so final hard-link publication is same-filesystem and
    // refuses an existing target atomically. Preserve failed staging for diagnosis.
    let stage = std::path::PathBuf::from(format!("{}.pending-{}", args[1], std::process::id()));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&stage)?;
    let mut store = Store::open(&stage)?;
    for (id, name) in data["games"].as_object().ok_or("games missing")? {
        store.register_game(id, name.as_str().ok_or("game name missing")?)?;
    }
    let revision = "reviewed-public-facts-2026-11-14";
    for (id, url) in data["sources"].as_object().ok_or("sources missing")? {
        store.set_source_policy(
            id,
            url.as_str().ok_or("source URL missing")?,
            true,
            true,
            data["source_policy_revisions"][id]
                .as_str()
                .unwrap_or(revision),
        )?;
    }
    let entities = data["entities"].as_array().ok_or("entities missing")?;
    for e in entities {
        store.register_entity(s(e, "id")?, s(e, "game_id")?, s(e, "kind")?, s(e, "name")?)?;
    }
    let mut rows = Vec::new();
    for r in data["observations"]
        .as_array()
        .ok_or("observations missing")?
    {
        rows.push(Observation {
            id: s(r, "id")?.into(),
            entity_id: s(r, "entity_id")?.into(),
            source_id: s(r, "source_id")?.into(),
            source_url: s(r, "source_url")?.into(),
            predicate: s(r, "predicate")?.into(),
            value: s(r, "value")?.into(),
            observed_unix: r["observed_unix"].as_i64().ok_or("timestamp missing")?,
            applicable_build: s(r, "applicable_build")?.into(),
            mod_version: s(r, "mod_version")?.into(),
            completeness: s(r, "completeness")?.into(),
            policy_revision: s(r, "policy_revision")?.into(),
        });
    }
    store.import(&rows)?;
    store.import(&rows)?; // byte-identical replay must not add duplicates
    drop(store);
    let reopened = Store::open(&stage)?;
    let mut found = 0;
    for e in entities {
        let id = s(e, "id")?;
        let actual = reopened.observations(id)?;
        for row in &actual {
            assert!(rows.contains(row), "Unexpected reopened row");
        }
        assert_eq!(
            actual.len(),
            rows.iter().filter(|r| r.entity_id == id).count()
        );
        found += actual.len();
    }
    assert_eq!(found, rows.len());
    drop(reopened);
    // Windows FlushFileBuffers requires a writable handle; this is our own staging file.
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&stage)?
        .sync_all()?;
    std::fs::hard_link(&stage, &args[1])?;
    std::fs::remove_file(&stage)?;
    println!(
        "Native Rust Store import + identical replay + close/reopen exact equality passed: {} entities, {} observations",
        entities.len(),
        found
    );
    Ok(())
}
fn s<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .ok_or_else(|| format!("missing string {k}").into())
}
