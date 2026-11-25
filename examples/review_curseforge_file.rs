//! Offline projection of exact-file relationship evidence from a local API response.
use athena_mod_knowledge::Result;
use serde_derive::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Read};

const MAX_BYTES: usize = 512 * 1024;
const MAX_RELATIONS: usize = 1024;
const DOCS: &str = "https://docs.curseforge.com/rest-api/#get-mod-file";

#[derive(Clone, Copy)]
struct FileIdentity {
    game_id: i32,
    mod_id: i32,
    id: i32,
}

impl FileIdentity {
    fn is_valid(self) -> bool {
        self.game_id > 0 && self.mod_id > 0 && self.id > 0
    }
}

#[derive(Deserialize)]
struct Envelope {
    data: FileRecord,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileRecord {
    game_id: i32,
    mod_id: i32,
    id: i32,
    is_available: Option<bool>,
    dependencies: Option<Vec<Dependency>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Dependency {
    mod_id: i32,
    relation_type: i32,
}

fn role(code: i32) -> &'static str {
    match code {
        1 => "embedded_library",
        2 => "optional",
        3 => "required",
        4 => "tool",
        5 => "incompatible",
        6 => "include",
        _ => "unknown",
    }
}

fn project(bytes: &[u8], expected: FileIdentity) -> Result<Value> {
    if bytes.is_empty() || bytes.len() > MAX_BYTES || !expected.is_valid() {
        return Err("Expected bounded JSON and positive game, mod and file IDs".into());
    }
    // Typed selected fields reject duplicates; no captured provider URL or secrets are retained.
    let record = serde_json::from_slice::<Envelope>(bytes)
        .map_err(|error| {
            format!(
                "Invalid exact-file JSON at line {}, column {}",
                error.line(),
                error.column()
            )
        })?
        .data;
    if (record.game_id, record.mod_id, record.id)
        != (expected.game_id, expected.mod_id, expected.id)
    {
        return Err("Response game, mod or file ID does not match the reviewed source".into());
    }
    let availability = match record.is_available {
        Some(true) => "reported_available",
        Some(false) => "reported_unavailable",
        None => "unknown",
    };
    let coverage = match &record.dependencies {
        None => "not_reported",
        Some(rows) if rows.is_empty() => "reported_empty",
        Some(_) => "reported_rows",
    };
    let supplied = record.dependencies.unwrap_or_default();
    if supplied.len() > MAX_RELATIONS {
        return Err("Dependency list exceeds review budget; no partial projection".into());
    }
    let mut roles = BTreeMap::<i32, Vec<i32>>::new();
    let mut rows = Vec::with_capacity(supplied.len());
    for dep in supplied {
        if dep.mod_id <= 0 || dep.relation_type < 0 {
            return Err("Invalid provider target or relation code; no partial projection".into());
        }
        let target_roles = roles.entry(dep.mod_id).or_default();
        if !target_roles.contains(&dep.relation_type) {
            target_roles.push(dep.relation_type);
        }
        rows.push(json!({
            "target_mod_id":dep.mod_id, "role":role(dep.relation_type),
            "provider_relation_code":dep.relation_type, "target_file_id":null
        }));
    }
    let unresolved = roles
        .into_iter()
        .filter_map(|(id, codes)| {
            (codes.len() > 1).then(|| json!({"target_mod_id":id,"provider_relation_codes":codes}))
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "schema":1,"provider":"curseforge","scope":"exact_file_reported_metadata",
        "source":{"game_id":expected.game_id,"mod_id":expected.mod_id,"file_id":expected.id,
            "documentation":DOCS},
        "availability":availability,"coverage":coverage,"relations":rows,
        "targets_with_multiple_roles":unresolved,"installer_ready":false,
        "notes":"Not a target file resolution, universal absence claim, compatibility check, download permission or install decision."
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [game, module, file] = args.as_slice() else {
        return Err(
            "Usage: review_curseforge_file GAME_ID MOD_ID FILE_ID < local-authorized-response.json"
                .into(),
        );
    };
    let expected = FileIdentity {
        game_id: game.parse()?,
        mod_id: module.parse()?,
        id: file.parse()?,
    };
    let mut bytes = Vec::new();
    std::io::stdin()
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&project(&bytes, expected)?)?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const EXACT: FileIdentity = FileIdentity {
        game_id: 99,
        mod_id: 111,
        id: 222,
    };
    fn sample(deps: &str) -> Vec<u8> {
        format!(r#"{{"data":{{"gameId":99,"modId":111,"id":222,"isAvailable":true{deps}}}}}"#)
            .into_bytes()
    }
    #[test]
    fn retains_all_distinct_roles_and_unknowns_without_resolving_target_files() {
        let input = sample(
            r#","dependencies":[{"modId":300,"relationType":1},{"modId":300,"relationType":2},{"modId":400,"relationType":3},{"modId":500,"relationType":4},{"modId":600,"relationType":5},{"modId":700,"relationType":6},{"modId":800,"relationType":92},{"modId":400,"relationType":3}]"#,
        );
        let snapshot = project(&input, EXACT).unwrap();
        let rows = snapshot["relations"].as_array().unwrap();
        assert_eq!(rows.len(), 8);
        let actual = rows
            .iter()
            .map(|row| row["role"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            [
                "embedded_library",
                "optional",
                "required",
                "tool",
                "incompatible",
                "include",
                "unknown",
                "required"
            ]
        );
        assert!(rows.iter().all(|row| row["target_file_id"].is_null()));
        assert_eq!(rows[6]["provider_relation_code"], 92);
        assert_eq!(
            snapshot["targets_with_multiple_roles"],
            json!([{"target_mod_id":300,"provider_relation_codes":[1,2]}])
        );
        assert_eq!(snapshot["installer_ready"], false);
    }
    #[test]
    fn empty_array_and_unreported_are_not_interchangeable() {
        for (suffix, expected) in [
            ("", "not_reported"),
            (r#","dependencies":null"#, "not_reported"),
            (r#","dependencies":[]"#, "reported_empty"),
        ] {
            let value = project(&sample(suffix), EXACT).unwrap();
            assert_eq!(value["coverage"], expected);
            assert_eq!(value["relations"], json!([]));
            assert_eq!(value["installer_ready"], false);
        }
    }
    #[test]
    fn rejects_mismatch_missing_fields_duplicate_scope_and_malformed_relations() {
        for adjusted in [
            FileIdentity { id: 223, ..EXACT },
            FileIdentity {
                mod_id: 112,
                ..EXACT
            },
            FileIdentity {
                game_id: 100,
                ..EXACT
            },
        ] {
            assert!(project(&sample(""), adjusted).is_err());
        }
        for payload in [
            br#"{"data":{"gameId":99,"modId":111,"id":222,"id":223}}"#.as_slice(),
            br#"{"data":{"gameId":99,"modId":111}}"#.as_slice(),
            &sample(r#","dependencies":[{"modId":0,"relationType":3}]"#),
            &sample(
                r#","dependencies":[{"modId":42,"relationType":3},{"modId":43,"relationType":"4"}]"#,
            ),
            &sample(r#","dependencies":[],"dependencies":[]"#),
        ] {
            assert!(project(payload, EXACT).is_err());
        }
    }
    #[test]
    fn refuses_over_budget_without_partial_rows_and_ignores_sensitive_fields() {
        assert!(project(&vec![b' '; MAX_BYTES + 1], EXACT).is_err());
        let dep = vec![json!({"modId":123,"relationType":3}); MAX_RELATIONS + 1];
        assert!(project(&sample(&format!(",\"dependencies\":{}", json!(dep))), EXACT).is_err());
        let value=project(&sample(r#","downloadUrl":"https://fixture.example/?token=DO-NOT-COPY","key":"secret-marker""#),EXACT).unwrap();
        assert!(!value.to_string().contains("secret-marker"));
        assert!(!value.to_string().contains("DO-NOT-COPY"));
        let error = project(
            &sample(r#","dependencies":[{"modId":"private-marker","relationType":3}]"#),
            EXACT,
        )
        .unwrap_err()
        .to_string();
        assert!(!error.contains("private-marker"));
    }
}
