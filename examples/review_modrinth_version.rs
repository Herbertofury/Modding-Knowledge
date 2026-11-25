//! Review local, exact Modrinth version metadata without resolving or installing dependencies.
use athena_mod_knowledge::Result;
use serde_derive::Deserialize;
use serde_json::{Value, json};
use std::io::Read;

const MAX_BYTES: usize = 512 * 1024;
const MAX_RELATIONS: usize = 1024;
const DOCS: &str = "https://docs.modrinth.com/api/operations/getversionfromidornumber/";

#[derive(Deserialize)]
struct Version {
    id: String,
    project_id: String,
    dependencies: Option<Vec<Dependency>>,
    game_versions: Option<Vec<String>>,
    loaders: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Dependency {
    version_id: Option<String>,
    project_id: Option<String>,
    file_name: Option<String>,
    dependency_type: String,
}

fn id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 32 && value.bytes().all(|b| b.is_ascii_alphanumeric())
}

fn label(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
}

fn context(values: &Option<Vec<String>>) -> bool {
    values
        .as_ref()
        .is_none_or(|items| items.len() <= 512 && items.iter().all(|value| label(value, 128)))
}

fn project(bytes: &[u8], expected_project: &str, expected_version: &str) -> Result<Value> {
    if bytes.is_empty() || bytes.len() > MAX_BYTES || !id(expected_project) || !id(expected_version)
    {
        return Err("Expected bounded JSON and two reviewed Modrinth IDs".into());
    }
    // Selected typed fields reject duplicates and malformed arrays. Do not echo payloads in errors.
    let version: Version = serde_json::from_slice(bytes).map_err(|error| {
        format!(
            "Invalid version JSON at line {}, column {}",
            error.line(),
            error.column()
        )
    })?;
    if version.project_id != expected_project || version.id != expected_version {
        return Err("Response project or version ID does not match the reviewed release".into());
    }
    if !context(&version.game_versions) || !context(&version.loaders) {
        return Err("Invalid or oversized source-version compatibility list".into());
    }
    let coverage = match &version.dependencies {
        None => "not_reported",
        Some(rows) if rows.is_empty() => "reported_empty",
        Some(_) => "reported_rows",
    };
    let supplied = version.dependencies.unwrap_or_default();
    if supplied.len() > MAX_RELATIONS {
        return Err("Dependency list exceeds review budget; no partial projection".into());
    }
    let mut rows = Vec::with_capacity(supplied.len());
    for dep in supplied {
        if dep.project_id.as_deref().is_some_and(|value| !id(value))
            || dep.version_id.as_deref().is_some_and(|value| !id(value))
            || dep
                .file_name
                .as_deref()
                .is_some_and(|value| !label(value, 512))
            || !label(&dep.dependency_type, 64)
        {
            return Err("Invalid provider target or role text; no partial projection".into());
        }
        let role = match dep.dependency_type.as_str() {
            "required" => "required",
            "optional" => "optional",
            "incompatible" => "incompatible",
            "embedded" => "embedded",
            _ => "unknown",
        };
        let target = match (&dep.project_id, &dep.version_id, &dep.file_name) {
            (Some(_), Some(_), _) => "project_and_version_reported",
            (Some(_), None, _) => "project_only",
            (None, Some(_), _) => "version_only",
            (None, None, Some(_)) => "filename_only_unresolved",
            (None, None, None) => "not_reported",
        };
        // Filenames are literal display evidence, never a path or resolved download.
        rows.push(json!({
            "role":role,"provider_dependency_type":dep.dependency_type,
            "target_status":target,"target_project_id":dep.project_id,
            "target_version_id":dep.version_id,"reported_filename":dep.file_name
        }));
    }
    Ok(json!({
        "schema":1,"provider":"modrinth","scope":"exact_version_reported_metadata",
        "source":{"project_id":expected_project,"version_id":expected_version,
            "version_page":format!("https://modrinth.com/version/{expected_version}"),
            "documentation":DOCS, "game_versions": version.game_versions, "loaders": version.loaders},
        "coverage":coverage,"relations":rows,"installer_ready":false,
        "notes":"Provider statements for one version only. No target compatibility, identity join, download, installation or project-wide absence is established."
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [project_id, version_id] = args.as_slice() else {
        return Err(
            "Usage: review_modrinth_version PROJECT_ID VERSION_ID < reviewed-response.json".into(),
        );
    };
    let mut bytes = Vec::new();
    std::io::stdin()
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&project(&bytes, project_id, version_id)?)?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const P: &str = "Sample01";
    const V: &str = "Version1";
    fn format_input(fields: &str) -> String {
        format!(r#"{{"project_id":"{P}","id":"{V}"{fields}}}"#)
    }
    fn read(fields: &str) -> Result<Value> {
        project(format_input(fields).as_bytes(), P, V)
    }

    #[test]
    fn preserves_distinct_roles_targets_and_compatibility_context() {
        let input = include_bytes!("research/modrinth_relationships/synthetic-response.json");
        let parsed = project(input, P, V).unwrap();
        assert_eq!(parsed["source"]["game_versions"], json!(["1.21.1"]));
        assert_eq!(parsed["source"]["loaders"], json!(["fabric"]));
        let rows = parsed["relations"].as_array().unwrap();
        let roles = rows
            .iter()
            .map(|x| x["role"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            roles,
            [
                "required",
                "optional",
                "incompatible",
                "embedded",
                "unknown"
            ]
        );
        assert_eq!(rows[0]["target_status"], "project_and_version_reported");
        assert_eq!(rows[1]["target_status"], "project_only");
        assert_eq!(rows[2]["target_status"], "version_only");
        assert_eq!(rows[3]["target_status"], "filename_only_unresolved");
        assert_eq!(rows[3]["reported_filename"], "external-example.jar");
        assert_eq!(rows[4]["target_status"], "not_reported");
        assert_eq!(rows[4]["provider_dependency_type"], "future-role");
        assert_eq!(parsed["installer_ready"], false);
        assert!(parsed.get("author_id").is_none() && parsed.get("files").is_none());
    }

    #[test]
    fn absence_null_and_empty_are_not_universal_compatibility() {
        assert_eq!(read("").unwrap()["coverage"], "not_reported");
        assert_eq!(
            read(r#","dependencies":null"#).unwrap()["coverage"],
            "not_reported"
        );
        let explicit = read(r#","dependencies":[]"#).unwrap();
        assert_eq!(explicit["coverage"], "reported_empty");
        assert_eq!(explicit["installer_ready"], false);
        assert!(explicit["source"]["game_versions"].is_null());
    }

    #[test]
    fn exact_ids_reject_invalid_paths_mismatches_and_duplicate_identity_keys() {
        let input = format_input("");
        assert!(project(input.as_bytes(), "Other01", V).is_err());
        assert!(project(input.as_bytes(), P, "../wrong").is_err());
        assert!(project(input.as_bytes(), P, "Version2").is_err());
        assert!(read(r#","id":"Version1""#).is_err());
        assert!(read(r#","dependencies":null,"dependencies":[]"#).is_err());
    }

    #[test]
    fn malformed_rows_do_not_turn_into_empty_or_partial_evidence() {
        for bad in [
            r#","dependencies":{}"#,
            r#","dependencies":[{}]"#,
            r#","dependencies":[{"dependency_type":"required","version_id":3}]"#,
            r#","dependencies":[{"dependency_type":"required","project_id":"a/b"}]"#,
            r#","dependencies":[{"dependency_type":"required","file_name":"bad\nlabel"}]"#,
            r#","dependencies":[{"dependency_type":"required","dependency_type":"optional"}]"#,
            r#","loaders":"fabric""#,
        ] {
            assert!(read(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn record_and_row_budgets_are_enforced() {
        assert!(project(&vec![b' '; MAX_BYTES + 1], P, V).is_err());
        let one = r#"{"dependency_type":"required"}"#;
        let rows = vec![one; MAX_RELATIONS + 1].join(",");
        assert!(read(&format!(",\"dependencies\":[{rows}]")).is_err());
    }
}
