//! Offline research-only file relationship shape review. No transport or Store.
use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt,
    io::{self, Read, Write},
    marker::PhantomData,
};

// Serde's default struct visitor also accepts position-only arrays. Restrict
// records to JSON objects; the typed map visitor still checks duplicate keys.
#[derive(Debug, PartialEq, Eq)]
struct Object<T>(T);

fn object<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    d: D,
) -> Result<T, D::Error> {
    struct ObjectOnly<T>(PhantomData<T>);
    impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for ObjectOnly<T> {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a JSON object")
        }

        fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            T::deserialize(MapAccessDeserializer::new(map))
        }
    }
    d.deserialize_map(ObjectOnly(PhantomData))
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Object<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        object(d).map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Project {
    provider: String,
    game: String,
    id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(deserialize_with = "object")]
    project: Project,
    id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Basis {
    AuthorStatement,
    ProviderStructured,
    Inferred,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Evidence {
    #[serde(deserialize_with = "object")]
    subject: File,
    source_id: String,
    source_url: String,
    locator: String,
    basis: Basis,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum Compatibility {
    Unknown {},
    Unavailable {},
    ProviderReported {
        #[serde(deserialize_with = "object")]
        evidence: Evidence,
        builds: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    Requires,
    OptionalDependency,
    WorksWith,
    Conflicts,
    LoadAfterIfPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Applies {
    AsDeclared,
    IfTargetPresent,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Constraint {
    Unspecified {},
    ExactTargetFile {
        #[serde(deserialize_with = "object")]
        file: File,
    },
    MinimumLabel {
        label: String,
        applies: Applies,
    },
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Relationship {
    #[serde(deserialize_with = "object")]
    target: Project,
    role: Role,
    #[serde(deserialize_with = "object")]
    constraint: Constraint,
    #[serde(deserialize_with = "object")]
    evidence: Evidence,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    #[serde(deserialize_with = "object")]
    subject: File,
    #[serde(deserialize_with = "object")]
    compatibility: Compatibility,
    relationships: Vec<Object<Relationship>>,
}

#[derive(Debug, Serialize)]
struct Review<'a> {
    status: &'static str,
    subject: &'a File,
    compatibility: &'a Compatibility,
    unverified_attributed_source_statements: Vec<&'a Relationship>,
    unverified_candidates: Vec<&'a Relationship>,
}

const MAX_INPUT: usize = 64 * 1024;

fn label(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn opaque(value: &str) -> bool {
    label(value, 128) && !value.chars().any(char::is_whitespace)
}

fn project_is_valid(project: &Project) -> bool {
    [&project.provider, &project.game, &project.id]
        .into_iter()
        .all(|s| opaque(s))
}

fn file_is_valid(file: &File) -> bool {
    project_is_valid(&file.project) && opaque(&file.id)
}

fn validate_evidence(expected: &File, evidence: &Evidence) -> Result<(), &'static str> {
    if &evidence.subject != expected
        || !opaque(&evidence.source_id)
        || !label(&evidence.locator, 256)
        || !label(&evidence.source_url, 2048)
        || evidence.source_url.chars().any(char::is_whitespace)
    {
        return Err("Invalid evidence scope or locator");
    }
    let url =
        url::Url::parse(&evidence.source_url).map_err(|_| "Expected canonical HTTPS source")?;
    if url.as_str() != evidence.source_url
        || url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Expected canonical HTTPS source");
    }
    Ok(())
}

/// Validate shape and exact identity only, not attribution or authorization.
fn review<'a>(expected: &File, input: &'a Input) -> Result<Review<'a>, &'static str> {
    if !file_is_valid(expected) || &input.subject != expected {
        return Err("Input subject does not match the expected exact file");
    }
    if input.relationships.len() > 128 {
        return Err("Too many relationships");
    }
    if let Compatibility::ProviderReported { evidence, builds } = &input.compatibility {
        validate_evidence(expected, evidence)?;
        if evidence.basis != Basis::ProviderStructured
            || builds.is_empty()
            || builds.len() > 128
            || builds
                .iter()
                .enumerate()
                .any(|(i, build)| !label(build, 128) || builds[..i].contains(build))
        {
            return Err("Expected distinct published build labels, not a compatibility inference");
        }
    }
    let mut result = Review {
        status: "offline_shape_review_only",
        subject: &input.subject,
        compatibility: &input.compatibility,
        unverified_attributed_source_statements: Vec::new(),
        unverified_candidates: Vec::new(),
    };
    for (index, record) in input.relationships.iter().enumerate() {
        let Object(row) = record;
        validate_evidence(expected, &row.evidence)?;
        if !project_is_valid(&row.target) || row.target.game != expected.project.game {
            return Err("Invalid or mismatched target project");
        }
        match &row.constraint {
            Constraint::ExactTargetFile { file }
                if !file_is_valid(file) || file.project != row.target =>
            {
                return Err("A target file must belong to the exact target project");
            }
            Constraint::MinimumLabel {
                label: minimum,
                applies,
            } if !label(minimum, 128)
                || (*applies == Applies::IfTargetPresent && row.role == Role::Requires) =>
            {
                return Err("Invalid version floor or presence-only condition marked required");
            }
            _ => {}
        }
        if input.relationships[..index].contains(record) {
            return Err("Duplicate exact relationship assertion");
        }
        match row.evidence.basis {
            Basis::Inferred => result.unverified_candidates.push(row),
            Basis::AuthorStatement | Basis::ProviderStructured => {
                result.unverified_attributed_source_statements.push(row);
            }
        }
    }
    Ok(result)
}

fn validate_stdin(expected: &File, reader: impl Read) -> Result<Vec<u8>, &'static str> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_INPUT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read stdin")?;
    if bytes.len() > MAX_INPUT {
        return Err("Input exceeds 64 KiB");
    }
    // Decode typed structs directly: a Value intermediary would discard duplicate keys.
    let Object(input): Object<Input> =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid exact-file review JSON")?;
    let mut output = serde_json::to_vec(&review(expected, &input)?)
        .map_err(|_| "Could not serialize offline review")?;
    output.push(b'\n');
    Ok(output)
}

fn main() -> Result<(), &'static str> {
    let mut args = std::env::args_os().skip(1).take(5);
    let mut next = || {
        args.next()
            .and_then(|s| s.into_string().ok())
            .ok_or("Usage: review_file_relationships PROVIDER GAME PROJECT FILE < JSON")
    };
    let expected = File {
        project: Project {
            provider: next()?,
            game: next()?,
            id: next()?,
        },
        id: next()?,
    };
    if args.next().is_some() {
        return Err("Usage: review_file_relationships PROVIDER GAME PROJECT FILE < JSON");
    }
    let output = validate_stdin(&expected, io::stdin().lock())?;
    io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|_| "Could not write review to stdout")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURE: &str = include_str!("research/file_relationships/synthetic.json");

    fn expected() -> File {
        File {
            project: Project {
                provider: "fixture-host".into(),
                game: "fixture-game".into(),
                id: "8".into(),
            },
            id: "42".into(),
        }
    }

    fn fixture() -> Value {
        serde_json::from_str(FIXTURE).unwrap()
    }

    fn validate(value: Value) -> Result<Value, &'static str> {
        let bytes = serde_json::to_vec(&value).unwrap();
        validate_stdin(&expected(), bytes.as_slice())
            .map(|output| serde_json::from_slice(&output).unwrap())
    }

    #[test]
    fn review_keeps_exact_scope_and_claims_separate_without_comparing_version_labels() {
        let input = fixture();
        let output = validate(input.clone()).unwrap();
        assert_eq!(
            output,
            json!({
                "status": "offline_shape_review_only",
                "subject": input["subject"],
                "compatibility": {"status":"unavailable"},
                "unverified_attributed_source_statements": [input["relationships"][0]],
                "unverified_candidates": [input["relationships"][1]],
            })
        );
        assert_ne!(
            output["unverified_attributed_source_statements"][0]["target"],
            output["subject"]["project"]
        );
        let escaped = FIXTURE.replace("2.9-preview", "2.9-\\u0065xact");
        let output: Value =
            serde_json::from_slice(&validate_stdin(&expected(), escaped.as_bytes()).unwrap())
                .unwrap();
        assert_eq!(
            output["unverified_attributed_source_statements"][0]["constraint"]["label"],
            "2.9-exact"
        );
    }

    #[test]
    fn wrong_file_and_id_reuse_do_not_match() {
        let input = fixture();
        let mut wrong = expected();
        wrong.id = "43".into();
        assert!(validate_stdin(&wrong, FIXTURE.as_bytes()).is_err());

        let mut file = input["subject"].clone();
        let mut variant = input.clone();
        variant["relationships"][0]["constraint"] =
            json!({"type":"exact_target_file","file": file});
        assert!(validate(variant.clone()).is_err());
        file["project"] = input["relationships"][0]["target"].clone();
        variant["relationships"][0]["constraint"]["file"] = file.clone();
        assert!(validate(variant).is_ok());
        for (name, value) in [("id", json!("43")), ("project", file["project"].clone())] {
            let mut variant = input.clone();
            variant["relationships"][0]["evidence"]["subject"][name] = value;
            assert!(validate(variant).is_err());
        }
    }

    #[test]
    fn builds_are_report_scope_and_unavailable_is_not_unknown_or_empty() {
        let mut input = fixture();
        input["compatibility"] = json!({"status":"unknown"});
        assert_eq!(
            validate(input.clone()).unwrap()["compatibility"]["status"],
            "unknown"
        );
        let mut evidence = input["relationships"][0]["evidence"].clone();
        evidence["basis"] = json!("provider_structured");
        input["compatibility"] = json!({"status":"provider_reported","evidence":evidence,"builds":["1.42","01.42-preview"]});
        assert_eq!(
            validate(input.clone()).unwrap()["compatibility"],
            input["compatibility"]
        );
        for builds in [json!([]), json!(["1.42", "1.42"])] {
            let mut variant = input.clone();
            variant["compatibility"]["builds"] = builds;
            assert!(validate(variant).is_err());
        }
        input["compatibility"]["evidence"]["basis"] = json!("inferred");
        assert!(validate(input).is_err());
    }

    #[test]
    fn forbids_unknown_fields_and_duplicate_keys_at_multiple_levels() {
        for (path, key) in [
            (vec![], "permission_granted"),
            (vec!["subject"], "allow_publish"),
            (vec!["compatibility"], "verified"),
        ] {
            let mut variant = fixture();
            let mut object = &mut variant;
            for part in path {
                object = &mut object[part];
            }
            object[key] = json!(true);
            assert_eq!(validate(variant), Err("Invalid exact-file review JSON"));
        }
        let original = fixture();
        for replaced in [
            FIXTURE.replacen("\"subject\":", "\"subject\":{},\"subject\":", 1),
            FIXTURE.replacen(
                "\"provider\":",
                "\"provider\":\"fake\",\"pro\\u0076ider\":",
                1,
            ),
            FIXTURE.replacen("\"status\":", "\"status\":\"unknown\",\"status\":", 1),
            FIXTURE.replacen("\"type\":", "\"type\":\"unspecified\",\"type\":", 1),
            FIXTURE.replacen("\"role\":", "\"role\":\"conflicts\",\"role\":", 1),
        ] {
            assert_ne!(replaced, FIXTURE);
            assert_eq!(
                validate_stdin(&expected(), replaced.as_bytes()),
                Err("Invalid exact-file review JSON")
            );
        }
        let mut unexpected = original;
        unexpected["relationships"][1]["constraint"]["otherwise"] = json!("fake");
        assert!(validate(unexpected).is_err());
    }

    #[test]
    fn enforces_shape_bounds_and_safe_citation_locators() {
        let mut prefix = FIXTURE.as_bytes().to_vec();
        prefix.resize(MAX_INPUT, b' ');
        assert!(validate_stdin(&expected(), prefix.as_slice()).is_ok());
        prefix.push(b' ');
        assert_eq!(
            validate_stdin(&expected(), prefix.as_slice()),
            Err("Input exceeds 64 KiB")
        );
        assert!(validate_stdin(&expected(), b"{}{} ".as_slice()).is_err());
        let input = fixture();
        for url in [
            "http://example.test/file",
            "https:example.test/file",
            "https://example.test/../file",
            "https://person@example.test/file",
            "https://example.test/file?key=synthetic",
        ] {
            let mut variant = input.clone();
            variant["relationships"][0]["evidence"]["source_url"] = json!(url);
            assert!(validate(variant).is_err());
        }
        let mut duplicate = input.clone();
        duplicate["relationships"][1] = input["relationships"][0].clone();
        assert!(validate(duplicate).is_err());
        let mut overflow = input.clone();
        overflow["relationships"] = json!(vec![input["relationships"][0].clone(); 129]);
        assert!(validate(overflow).is_err());
        let mut promoted = input;
        promoted["relationships"][0]["role"] = json!("requires");
        assert!(validate(promoted).is_err());
    }

    #[test]
    fn positional_arrays_cannot_replace_named_records() {
        let original = fixture();
        assert!(
            validate(json!([
                original["subject"],
                original["compatibility"],
                original["relationships"]
            ]))
            .is_err()
        );
        let mut subject = original.clone();
        subject["subject"] = json!([original["subject"]["project"], "42"]);
        assert!(validate(subject).is_err());
        let mut project = original.clone();
        project["subject"]["project"] = json!(["fixture-host", "fixture-game", "8"]);
        assert!(validate(project).is_err());
        let row = &original["relationships"][0];
        let mut entry = original.clone();
        entry["relationships"][0] = json!([
            row["target"],
            row["role"],
            row["constraint"],
            row["evidence"]
        ]);
        assert!(validate(entry).is_err());
        let evidence = &row["evidence"];
        let mut nested = original.clone();
        nested["relationships"][0]["evidence"] = json!([
            evidence["subject"],
            evidence["source_id"],
            evidence["source_url"],
            evidence["locator"],
            evidence["basis"]
        ]);
        assert!(validate(nested).is_err());
    }
}
