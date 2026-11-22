use athena_mod_knowledge::{
    Observation, Store,
    loot::{self, SourceFileSet},
};
use serde_json::{Value, json};

const ENTITY: &str = "fixture:plugin:F.esp";
const SOURCE: &str = "https://github.com/loot/oblivion/blob/b3ffc84f6d730f6efb5083448d78611df16e8d78/masterlist.yaml";

fn evidence(plugin: Value) -> Observation {
    Observation {
        id: "fixture-evidence".into(),
        entity_id: ENTITY.into(),
        source_id: "loot-oblivion".into(),
        source_url: SOURCE.into(),
        predicate: loot::OBSERVATION_PREDICATE.into(),
        value: json!({
            "source_commit_sha": "b3ffc84f6d730f6efb5083448d78611df16e8d78",
            "source_blob_sha": "8c613acedf019890b950e4c4b2450151cead1025",
            "condition_evaluation": "not_evaluated",
            "plugin_metadata": plugin,
        })
        .to_string(),
        observed_unix: 2,
        applicable_build: "conditional".into(),
        mod_version: "unverified".into(),
        completeness: "partial".into(),
        policy_revision: "fixture-v1".into(),
    }
}

#[test]
fn consumer_gets_separate_roles_and_exact_provenance_using_public_api() {
    let mut store = Store::open(":memory:").unwrap();
    store.register_game("fixture", "Fixture").unwrap();
    store
        .register_entity(ENTITY, "fixture", "guide", "Fixture")
        .unwrap();
    store
        .set_source_policy("loot-oblivion", SOURCE, true, true, "fixture-v1")
        .unwrap();
    let constraint = json!({"name":"Other.dll","condition":"file(\"A.esp\")",
        "constraint":"version(\"Other.dll\", ==, \"1\")","display":"Do not interpret"});
    let import = evidence(
        json!({"name":"Fixture(.*)\\.esp","req":["A.esp",constraint],
        "after":["A.esp"],"inc":[]}),
    );
    let mut unrelated = evidence(Value::Null);
    unrelated.id = "another-predicate".into();
    unrelated.predicate = "compatibility".into();
    unrelated.observed_unix = 3;
    store.import(&[import, unrelated]).unwrap();

    let rows = store
        .observations_page_for_predicate(ENTITY, loot::OBSERVATION_PREDICATE, 0)
        .unwrap();
    let [row] = rows.as_slice() else {
        panic!("expected only typed source evidence");
    };
    let e = loot::project(row);
    assert!(std::ptr::eq(row, e.observation()));
    assert_eq!(e.observation().source_url, SOURCE);
    assert_eq!(e.observation().observed_unix, 2);
    assert_eq!(e.observation().policy_revision, "fixture-v1");
    let f = e.fields().unwrap();
    assert_eq!(f.plugin_source_label, "Fixture(.*)\\.esp");
    assert_eq!(
        f.requirements,
        SourceFileSet::RecordedUnevaluated(vec![json!("A.esp"), constraint])
    );
    assert_eq!(f.incompatibilities, SourceFileSet::RecordedEmpty);
    assert_eq!(
        f.load_after_if_present,
        SourceFileSet::RecordedUnevaluated(vec![json!("A.esp")])
    );
    assert_eq!(e.to_json()["conditions_and_constraints"], "not_evaluated");
    assert_eq!(e.to_json()["observation_id"], row.id);
    assert!(
        store
            .observations_page_for_predicate(ENTITY, loot::OBSERVATION_PREDICATE, 1)
            .unwrap()
            .is_empty()
    );

    store
        .set_source_policy("loot-oblivion", SOURCE, true, true, "fixture-v2")
        .unwrap();
    assert!(
        store
            .observations_page_for_predicate(ENTITY, loot::OBSERVATION_PREDICATE, 0)
            .unwrap()
            .is_empty()
    );
    store
        .set_source_policy("loot-oblivion", SOURCE, true, false, "fixture-v1")
        .unwrap();
    assert!(
        store
            .observations_page_for_predicate(ENTITY, loot::OBSERVATION_PREDICATE, 0)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn unknown_empty_and_malformed_remain_distinct_with_exact_raw_values() {
    let row = evidence(json!({"name":"A.esp","after":null,"inc":[]}));
    let e = loot::project(&row);
    let f = e.fields().unwrap();
    assert_eq!(f.requirements, SourceFileSet::NotRecorded);
    assert_eq!(f.incompatibilities, SourceFileSet::RecordedEmpty);
    assert_eq!(
        f.load_after_if_present,
        SourceFileSet::MalformedOrUnsupported(Value::Null)
    );
    assert_eq!(
        e.to_json()["source_declared_requirements"],
        json!({"status":"not_recorded","raw":null})
    );
    assert_eq!(
        e.to_json()["source_declared_incompatibilities"],
        json!({"status":"recorded_empty","raw":[]})
    );
    for raw in [
        json!(null),
        json!(["file.esm", 5]),
        json!([{"name":"F.esm","condition":false}]),
        json!([{"name":"F.esm","future":true}]),
        json!([{"name":"F.esm","detail":[{"lang":"en","text":"unprojected"}]}]),
    ] {
        let row = evidence(json!({"name":"F.esp","after":raw}));
        let e = loot::project(&row);
        assert_eq!(
            e.fields().unwrap().load_after_if_present,
            SourceFileSet::MalformedOrUnsupported(raw.clone())
        );
        assert_eq!(
            e.to_json()["source_load_after_if_present"],
            json!({"status":"malformed_or_unsupported","raw":raw})
        );
    }
}

#[test]
fn rejects_other_sources_pins_and_out_of_scope_shapes_without_hidden_joins() {
    let valid_row = || evidence(json!({"name":"F.esp","after":["A.esp"]}));
    for (field, raw) in [
        ("source_commit_sha", json!("unreviewed")),
        ("source_blob_sha", json!("unreviewed")),
        ("condition_evaluation", json!("evaluated")),
        ("plugin_metadata", json!({"name":" "})),
        ("plugin_metadata", json!(false)),
    ] {
        let mut row = valid_row();
        let mut parsed: Value = serde_json::from_str(&row.value).unwrap();
        parsed[field] = raw;
        row.value = parsed.to_string();
        assert!(loot::project(&row).fields().is_none(), "{field}");
    }
    let mut row = valid_row();
    row.source_id = "other-loot".into();
    assert!(loot::project(&row).fields().is_none());
    let mut row = valid_row();
    row.source_url = "https://example.org/not-the-pinned-author".into();
    assert!(loot::project(&row).fields().is_none());
    let mut row = valid_row();
    row.predicate = "compatibility".into();
    assert!(loot::project(&row).fields().is_none());
    let mut row = valid_row();
    row.value = "[".into();
    let e = loot::project(&row);
    assert!(e.fields().is_none());
    assert_eq!(
        e.to_json(),
        json!({"observation_id": "fixture-evidence", "status": "malformed_or_unsupported"})
    );
}

#[test]
fn browse_applies_role_and_literal_before_match_page_across_a_large_snapshot() {
    use loot::{BROWSE_PAGE_LIMIT, BROWSE_SCAN_LIMIT, SourceRole};
    const LABEL: &str = "Regex(.*)\\.esp";
    let mut store = Store::open(":memory:").unwrap();
    store
        .register_game("oblivion-family", "Oblivion family")
        .unwrap();
    store
        .set_source_policy("loot-oblivion", SOURCE, true, true, "fixture-v1")
        .unwrap();
    // Mixed identity, timestamps tied within groups, newer false matches, and recoverable
    // sparse work slices that cross the old 1,000-row entity page ceiling.
    let entities = ["fixture-a", "fixture-b"];
    for id in entities {
        store
            .register_entity(id, "oblivion-family", "guide", id)
            .unwrap();
    }
    let mut expected = Vec::new();
    let mut records = Vec::new();
    for i in 0..1900 {
        let positive = i >= 1550 && i % 9 != 0;
        let metadata = if positive {
            let entry = if i % 2 == 0 {
                json!(LABEL)
            } else {
                json!({"name":LABEL,"condition":"fixture condition not evaluated","constraint":"raw fixture constraint"})
            };
            json!({"name":"Observed fixture source","req":[entry]})
        } else {
            match i % 7 {
                0 => json!({"name":"Alternate role","inc":[LABEL]}),
                1 => json!({"name":"Alternate role","after":[LABEL]}),
                2 => json!({"name":"Unknown requirements"}),
                3 => json!({"name":"Explicit empty requirements","req":[]}),
                4 => json!({"name":"Case-sensitive label","req":["regex(.*)\\.esp"]}),
                5 => json!({"name":"Incorrect shape","req":[LABEL,null]}),
                _ => json!({"name":"Not a regex search","req":["RegexFilm.esp"]}),
            }
        };
        let mut row = evidence(metadata);
        row.id = format!("fixture-{i:04}");
        row.entity_id = entities[i % entities.len()].into();
        row.observed_unix = (1900 - i as i64) / 3;
        if positive {
            expected.push(row.id.clone());
        }
        records.push(row);
    }
    for batch in records.chunks(1000) {
        store.import(batch).unwrap();
    }
    let mut seen = Vec::new();
    let mut later_rows = 0;
    let mut empty_work_pages = 0;
    let mut cursor = None;
    for _ in 0..12 {
        let page = store
            .browse_loot(SourceRole::Requires, Some(LABEL), cursor.as_ref())
            .unwrap();
        assert!(page.scanned <= BROWSE_SCAN_LIMIT);
        assert!(page.matches.len() <= BROWSE_PAGE_LIMIT);
        if page.matches.is_empty() && page.next.is_some() {
            empty_work_pages += 1;
        }
        later_rows += page.scanned;
        for m in page.matches {
            assert_eq!(m.observation.source_url, SOURCE);
            assert_eq!(m.fields.plugin_source_label, "Observed fixture source");
            assert_eq!(
                m.relationships_json(),
                loot::project(&m.observation).to_json()
            );
            seen.push(m.observation.id);
        }
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert!(cursor.is_none());
    assert_eq!(empty_work_pages, 6);
    assert_eq!(later_rows, 1900);
    assert_eq!(seen, expected);
    assert_eq!(seen.len(), 311);
    for role in [SourceRole::Incompatible, SourceRole::LoadAfterIfPresent] {
        let page = store.browse_loot(role, Some(LABEL), None).unwrap();
        assert!(!page.matches.is_empty());
        assert!(
            page.matches
                .iter()
                .all(|m| m.fields.plugin_source_label == "Alternate role")
        );
    }
}

#[test]
fn browse_does_not_scan_hidden_or_unrelated_sources_and_rechecks_resume_policy() {
    use loot::{BROWSE_PAGE_LIMIT, BrowseCursor, SourceRole};
    let mut store = Store::open(":memory:").unwrap();
    store
        .register_game("oblivion-family", "Oblivion family")
        .unwrap();
    store
        .register_entity(ENTITY, "oblivion-family", "guide", ENTITY)
        .unwrap();
    for id in ["fixture", "loot-oblivion"] {
        store
            .set_source_policy(id, SOURCE, true, true, "fixture-v1")
            .unwrap();
    }
    let row = || evidence(json!({"name":"Source.esp","req":["Target.esp"]}));
    let mut batch = Vec::new();
    for i in 0..101 {
        let mut r = row();
        r.id = format!("match-{i:04}");
        batch.push(r);
    }
    let mut wrong_source = row();
    wrong_source.id = "a-wrong-source".into();
    wrong_source.source_id = "fixture".into();
    let mut wrong_url = row();
    wrong_url.id = "a-wrong-url".into();
    wrong_url.source_url = "https://example.org/other-pin".into();
    let mut wrong_predicate = row();
    wrong_predicate.id = "a-wrong-predicate".into();
    wrong_predicate.predicate = "unrelated".into();
    let mut wrong_blob = row();
    wrong_blob.id = "a-wrong-blob".into();
    wrong_blob.value = wrong_blob
        .value
        .replace("8c613acedf019890b950e4c4b2450151cead1025", "unreviewed");
    batch.extend([wrong_source, wrong_url, wrong_predicate, wrong_blob]);
    store.import(&batch).unwrap();
    let page = store.browse_loot(SourceRole::Requires, None, None).unwrap();
    assert_eq!(page.scanned, BROWSE_PAGE_LIMIT + 1); // only the wrong blob reaches parsing
    assert_eq!(page.matches.len(), BROWSE_PAGE_LIMIT);
    let cursor = page.next.unwrap();
    assert_eq!(cursor.observed_unix, 2);
    assert_eq!(cursor.observation_id, "match-0099");
    let end = store
        .browse_loot(SourceRole::Requires, None, Some(&cursor))
        .unwrap();
    assert_eq!(end.scanned, 1);
    assert_eq!(end.matches[0].observation.id, "match-0100");
    assert!(end.next.is_none());
    for bad in ["", "   ", &"x".repeat(1025)] {
        assert!(
            store
                .browse_loot(SourceRole::Requires, Some(bad), None)
                .is_err()
        );
    }
    for bad in [
        BrowseCursor {
            observed_unix: -1,
            observation_id: "x".into(),
        },
        BrowseCursor {
            observed_unix: 1,
            observation_id: "".into(),
        },
    ] {
        assert!(
            store
                .browse_loot(SourceRole::Requires, None, Some(&bad))
                .is_err()
        );
    }
    for (retain, publish, revision) in [
        (false, true, "fixture-v1"),
        (true, false, "fixture-v1"),
        (true, true, "fixture-v2"),
    ] {
        store
            .set_source_policy("loot-oblivion", SOURCE, retain, publish, revision)
            .unwrap();
        let page = store
            .browse_loot(SourceRole::Requires, None, Some(&cursor))
            .unwrap();
        assert!(page.matches.is_empty());
        assert_eq!(page.scanned, 0);
        assert!(page.next.is_none());
    }
}

const FO4_ENTITY: &str = "loot:fallout4:plugin-metadata:00123456789abcdef012";
const FO4_SOURCE: &str = "https://github.com/loot/fallout4/blob/4c7858a84bc65b34904b53c5e6252d3f6f9b1247/masterlist.yaml";

fn fo4_observation(plugin: Value) -> Observation {
    Observation {
        id: "synthetic-fo4-observation".into(),
        entity_id: FO4_ENTITY.into(),
        source_id: "loot-fallout4".into(),
        source_url: FO4_SOURCE.into(),
        predicate: loot::OBSERVATION_PREDICATE.into(),
        value: json!({
            "source_commit_sha":"4c7858a84bc65b34904b53c5e6252d3f6f9b1247",
            "source_blob_sha":"eed7994e786c896463887d03bcb3f14426b37753",
            "condition_evaluation":"not_evaluated",
            "plugin_metadata":plugin,
        })
        .to_string(),
        observed_unix: 2,
        applicable_build: "conditional".into(),
        mod_version: "unverified".into(),
        completeness: "partial".into(),
        policy_revision: "synthetic-fo4-v1".into(),
    }
}

#[test]
fn fallout4_uses_separate_pin_preserves_roles_and_never_evaluates_conditions() {
    let raw = json!({"name":"Fixture(.*)\\\\.esp",
                "req":[{"name":"Fixture.esm","constraint":"version(Fixture, 1)"}],
                "after":[{"name":"Conditional.esp","condition":"unknown(Fixture)"}]});
    let row = fo4_observation(raw.clone());
    let result = loot::project_fallout4(&row);
    assert!(std::ptr::eq(result.observation(), &row));
    let fields = result.fields().unwrap();
    assert_eq!(fields.plugin_source_label, raw["name"]);
    assert_eq!(
        result.to_json()["source_declared_requirements"]["raw"],
        raw["req"]
    );
    assert_eq!(
        result.to_json()["source_load_after_if_present"]["raw"],
        raw["after"]
    );
    assert_eq!(fields.incompatibilities, SourceFileSet::NotRecorded);
    assert_eq!(
        result.to_json()["conditions_and_constraints"],
        "not_evaluated"
    );
    assert!(loot::project(&row).fields().is_none());
    assert!(loot::project_fallout4(&evidence(raw)).fields().is_none());

    let unsupported = fo4_observation(json!({"name":"Fixture.esp","req":[],"inc":null}));
    let unsupported_view = loot::project_fallout4(&unsupported);
    let fields = unsupported_view.fields().unwrap();
    assert_eq!(fields.requirements, SourceFileSet::RecordedEmpty);
    assert_eq!(
        fields.incompatibilities,
        SourceFileSet::MalformedOrUnsupported(Value::Null)
    );
    assert_eq!(fields.load_after_if_present, SourceFileSet::NotRecorded);
}

#[test]
fn fallout4_refuses_wrong_document_blob_namespace_predicate_or_row_scope() {
    let make = || fo4_observation(json!({"name":"Fixture.esp","req":["Test.esm"]}));
    for (field, value) in [
        ("source_commit_sha", "different"),
        (
            "source_blob_sha",
            "8c613acedf019890b950e4c4b2450151cead1025",
        ),
        ("condition_evaluation", "evaluated"),
    ] {
        let mut row = make();
        let mut payload: Value = serde_json::from_str(&row.value).unwrap();
        payload[field] = json!(value);
        row.value = payload.to_string();
        assert!(loot::project_fallout4(&row).fields().is_none());
    }
    for entity in [
        "loot:fallout4-vr:plugin-metadata:00123456789abcdef012",
        "loot:fallout4:source-context",
        "loot:oblivion:plugin-metadata:00123456789abcdef012",
        "loot:fallout4:plugin-metadata:",
    ] {
        let row = Observation {
            entity_id: entity.into(),
            ..make()
        };
        assert!(loot::project_fallout4(&row).fields().is_none());
    }
    for row in [
        Observation {
            source_id: "loot-oblivion".into(),
            ..make()
        },
        Observation {
            source_url: SOURCE.into(),
            ..make()
        },
        Observation {
            predicate: "compatibility".into(),
            ..make()
        },
    ] {
        assert!(loot::project_fallout4(&row).fields().is_none());
    }
}

#[test]
fn fallout4_projection_does_not_admit_source_and_oblivion_browse_does_not_cross_pins() {
    let mut store = Store::open(":memory:").unwrap();
    store
        .register_game("fallout4", "Synthetic Fallout 4")
        .unwrap();
    store
        .register_entity(FO4_ENTITY, "fallout4", "guide", "Synthetic")
        .unwrap();
    let make = || fo4_observation(json!({"name":"Fixture.esp","req":["Fixture.esm"]}));
    assert!(store.import(&[make()]).is_err());
    assert!(store.observations(FO4_ENTITY).unwrap().is_empty());

    // Only this in-memory fixture is authorized for both retention and visibility.
    store
        .set_source_policy("loot-fallout4", FO4_SOURCE, true, true, "synthetic-fo4-v1")
        .unwrap();
    store.import(&[make()]).unwrap();
    let public = store.observations(FO4_ENTITY).unwrap();
    assert!(loot::project_fallout4(&public[0]).fields().is_some());
    assert!(
        store
            .browse_loot(loot::SourceRole::Requires, None, None)
            .unwrap()
            .matches
            .is_empty()
    );
    store
        .set_source_policy("loot-fallout4", FO4_SOURCE, true, false, "synthetic-fo4-v1")
        .unwrap();
    assert!(store.observations(FO4_ENTITY).unwrap().is_empty());
}

#[test]
fn browse_rejects_wrong_game_namespace_before_the_work_limit() {
    use loot::{BROWSE_SCAN_LIMIT, SourceRole};
    const TARGET: &str = "DLCShiveringIsles.esp";
    const WRONG_ENTITY: &str = "loot:oblivion:plugin-metadata:11111111111111111111";
    let mut store = Store::open(":memory:").unwrap();
    store
        .register_game("oblivion-family", "Oblivion family")
        .unwrap();
    store.register_game("skyrim-se", "Skyrim SE").unwrap();
    store
        .register_entity(ENTITY, "oblivion-family", "guide", "Fixture")
        .unwrap();
    store
        .register_entity(WRONG_ENTITY, "skyrim-se", "guide", "Misbound fixture")
        .unwrap();
    store
        .set_source_policy("loot-oblivion", SOURCE, true, true, "fixture-v1")
        .unwrap();
    let value = json!({"name": "Synthetic source.esp", "req": [TARGET]});
    let allowed = evidence(value.clone());
    let wrong: Vec<_> = (0..BROWSE_SCAN_LIMIT)
        .map(|i| {
            let mut row = evidence(value.clone());
            row.id = format!("wrong-namespace-{i:04}");
            row.entity_id = WRONG_ENTITY.into();
            row.observed_unix = allowed.observed_unix + 1;
            row
        })
        .collect();
    store.import(&wrong).unwrap();
    store.import(std::slice::from_ref(&allowed)).unwrap();

    let page = store
        .browse_loot(SourceRole::Requires, Some(TARGET), None)
        .unwrap();
    assert_eq!(page.scanned, 1);
    assert!(page.next.is_none());
    assert_eq!(page.matches.len(), 1);
    assert_eq!(page.matches[0].observation, allowed);
    // Do not change the source ledger, which still exposes raw, policy-permitted rows.
    assert_eq!(
        store
            .observations_page_for_predicate(WRONG_ENTITY, loot::OBSERVATION_PREDICATE, 0)
            .unwrap()
            .len(),
        wrong.len(),
    );
}
