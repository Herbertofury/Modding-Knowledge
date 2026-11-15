//! Reproducible public metadata import; does not perform network requests.
use athena_mod_knowledge::{Observation, Result, Store};
use serde_json::Value;
fn text<'a>(r: &'a Value, key: &str) -> Result<&'a str> {
    r.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Missing string {key}").into())
}
fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Supply a new output SQLite path")?;
    if std::path::Path::new(&path).exists() {
        return Err("Output exists; refusing overwrite".into());
    }
    let mut s = Store::open(&path)?;
    populate(&mut s)?;
    println!(
        "Imported 3 Modrinth projects with provider-attributed observations, not verified compatibility."
    );
    Ok(())
}
fn populate(s: &mut Store) -> Result<()> {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("../data/modrinth-public-2026-11-14.json"))?;
    s.register_game("minecraft-java", "Minecraft Java Edition")?;
    let revision = "reviewed-public-metadata-2026-11-14";
    s.set_source_policy(
        "modrinth-public-metadata",
        "https://docs.modrinth.com/api/",
        true,
        true,
        revision,
    )?;
    for r in rows {
        let id = format!("modrinth:project:{}", text(&r, "id")?);
        let source = text(&r, "source_url_api")?;
        let at = r
            .get("observed_unix")
            .and_then(Value::as_i64)
            .ok_or("Missing observation time")?;
        s.register_entity(&id, "minecraft-java", "mod", text(&r, "title")?)?;
        let mut observations = Vec::new();
        for field in [
            "slug",
            "team",
            "organization",
            "source_url",
            "issues_url",
            "wiki_url",
            "license",
            "published",
            "updated",
            "status",
            "loaders",
            "client_side",
            "server_side",
            "scope",
            "observed_at",
        ] {
            let value = r.get(field).ok_or("Missing projected field")?;
            if value.is_null() {
                continue;
            } // absence remains unknown, never an empty success
            observations.push(Observation {
                id: format!("{id}:{at}:{field}"),
                entity_id: id.clone(),
                source_id: "modrinth-public-metadata".into(),
                source_url: source.into(),
                predicate: format!("provider_claim.{field}"),
                value: serde_json::to_string(value)?,
                observed_unix: at,
                applicable_build: "unverified".into(),
                mod_version: "unverified".into(),
                completeness: "partial".into(),
                policy_revision: revision.into(),
            });
        }
        s.import(&observations)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_records_are_populated_with_attribution() {
        let mut s = Store::open(":memory:").unwrap();
        populate(&mut s).unwrap();
        let r = s.observations("modrinth:project:AANobbMI").unwrap();
        assert!(r.len() > 10);
        assert!(r.iter().any(
            |x| x.predicate == "provider_claim.license" && x.value.contains("Polyform-Shield")
        ));
        assert!(
            r.iter()
                .all(|x| x.completeness == "partial" && x.applicable_build == "unverified")
        );
        assert!(
            !s.observations("modrinth:project:YL57xq9U")
                .unwrap()
                .is_empty()
        );
    }
}
