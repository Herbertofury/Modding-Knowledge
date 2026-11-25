//! Build a new reviewed snapshot; no external JSON may self-grant publication permission.
use athena_mod_knowledge::{Observation, Result, Store};
use serde_json::Value;
use std::collections::HashMap;
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let (path, embedded) = match &args[1..] {
        [path] if !path.starts_with("--") => (path, include_str!("../data/reviewed-bundle.json")),
        [flag, path] if flag == "--scumbumbo-sims4" => {
            (path, include_str!("../data/scumbumbo-sims4-reviewed.json"))
        }
        [flag, path] if flag == "--sims4-creator-relationships" => (
            path,
            include_str!("../data/sims4-frankk-simsonian-reviewed.json"),
        ),
        [flag, path] if flag == "--cyberpunk2077-core" => (
            path,
            include_str!("../data/cyberpunk2077-core-reviewed.json"),
        ),
        [flag, path] if flag == "--hytale-file-channels" => (
            path,
            include_str!("../data/hytale-file-channels-reviewed.json"),
        ),
        [flag, path] if flag == "--skyrim-native-runtime" => (
            path,
            include_str!("../data/skyrim-native-runtime-reviewed.json"),
        ),
        _ => {
            return Err("Expected optional reviewed-pack flag and a NEW-SQLITE-PATH".into());
        }
    };
    if std::path::Path::new(path).exists() {
        return Err("Refusing overwrite".into());
    }
    let data: Value = serde_json::from_str(embedded)?;
    // Build beside target so final hard-link publication is same-filesystem and
    // refuses an existing target atomically. Preserve failed staging for diagnosis.
    let stage = std::path::PathBuf::from(format!("{}.pending-{}", path, std::process::id()));
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
    for row in &rows {
        if row.predicate == "health_claim_v1" {
            athena_mod_knowledge::health::validate_claim(row)?;
        }
    }
    // Each transaction stays bounded; incomplete staging is never published.
    for chunk in rows.chunks(500) {
        store.import(chunk)?;
    }
    for chunk in rows.chunks(500) {
        store.import(chunk)?; // byte-identical replay must not add duplicates
    }
    drop(store);
    let reopened = Store::open_read_only(&stage)?;
    let found = verify_reopened(&reopened, entities.iter().map(|e| s(e, "id")), &rows)?;
    drop(reopened);
    // Windows FlushFileBuffers requires a writable handle; this is our own staging file.
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&stage)?
        .sync_all()?;
    std::fs::hard_link(&stage, path)?;
    std::fs::remove_file(&stage)?;
    println!(
        "Native Rust Store import + identical replay + close/reopen exact equality passed: {} entities, {} observations",
        entities.len(),
        found
    );
    Ok(())
}

fn verify_reopened<'a>(
    store: &Store,
    entity_ids: impl Iterator<Item = Result<&'a str>>,
    expected: &[Observation],
) -> Result<usize> {
    let mut missing: HashMap<_, _> = expected.iter().map(|r| (r.id.as_str(), r)).collect();
    if missing.len() != expected.len() {
        return Err("Duplicate expected observation ID".into());
    }
    for entity_id in entity_ids {
        let entity_id = entity_id?;
        let mut offset = 0u32;
        loop {
            // The staging file is unpublished and unchanged. Check every page, including
            // when the expected count lands exactly on a page boundary.
            let page = store.observations_page(entity_id, offset)?;
            if page.is_empty() {
                break;
            }
            for row in &page {
                if missing.remove(row.id.as_str()) != Some(row) {
                    return Err("Reopened observation differs from reviewed input".into());
                }
            }
            offset = offset
                .checked_add(u32::try_from(page.len())?)
                .ok_or("Snapshot exceeds observation pagination range")?;
        }
    }
    if !missing.is_empty() {
        return Err("Reviewed observations missing from reopened snapshot".into());
    }
    Ok(expected.len())
}

fn s<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .ok_or_else(|| format!("missing string {k}").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    const A: &str = "fixture:mod:a";
    const B: &str = "fixture:mod:b";

    fn row(entity_id: &str, index: usize) -> Observation {
        Observation {
            id: format!("{entity_id}:{index:04}"),
            entity_id: entity_id.into(),
            source_id: "fixture".into(),
            source_url: "https://example.org/fixture".into(),
            predicate: [
                "fixture.requires",
                "fixture.load_after_if_present",
                "fixture.works_with",
            ][index % 3]
                .into(),
            value: format!("Framework {index}"),
            observed_unix: (index / 3) as i64,
            applicable_build: "fixture-build".into(),
            mod_version: format!("opaque-{index}"),
            completeness: "partial".into(),
            policy_revision: "fixture-review".into(),
        }
    }

    fn store_with(rows: &[Observation]) -> Store {
        let mut store = Store::open(":memory:").unwrap();
        store.register_game("fixture-game", "Fixture").unwrap();
        store
            .set_source_policy(
                "fixture",
                "https://example.org/terms",
                true,
                true,
                "fixture-review",
            )
            .unwrap();
        for id in [A, B] {
            store
                .register_entity(id, "fixture-game", "mod", "Same display name")
                .unwrap();
        }
        for rows in rows.chunks(500) {
            store.import(rows).unwrap();
        }
        store
    }

    #[test]
    fn verifies_all_pages_and_keeps_equal_display_names_separate() {
        let expected: Vec<_> = (0..2001)
            .map(|i| row(A, i))
            .chain((0..1000).map(|i| row(B, i)))
            .collect();
        let store = store_with(&expected);
        assert_eq!(store.observations(A).unwrap().len(), 1000);
        assert_eq!(
            verify_reopened(&store, [B, A].into_iter().map(Ok), &expected).unwrap(),
            3001
        );
    }

    #[test]
    fn refuses_missing_extra_or_changed_oldest_rows_after_the_first_page() {
        let mut rows: Vec<_> = (0..1001).map(|i| row(A, i)).collect();
        let store = store_with(&rows);
        assert_eq!(
            verify_reopened(&store, [A, B].into_iter().map(Ok), &rows[1..])
                .unwrap_err()
                .to_string(),
            "Reopened observation differs from reviewed input"
        );
        rows[0].predicate = "fixture.works_with".into();
        assert_eq!(
            verify_reopened(&store, [A, B].into_iter().map(Ok), &rows)
                .unwrap_err()
                .to_string(),
            "Reopened observation differs from reviewed input"
        );
        let shorter = store_with(&rows[1..]);
        assert_eq!(
            verify_reopened(&shorter, [A, B].into_iter().map(Ok), &rows)
                .unwrap_err()
                .to_string(),
            "Reviewed observations missing from reopened snapshot"
        );
    }
}
