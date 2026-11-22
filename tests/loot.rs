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
