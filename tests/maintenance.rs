use athena_mod_knowledge::{Observation, Store, maintenance::MaintenanceAction};
use serde_json::{Value, json};

const ENTITY: &str = "fixture:project";
fn data() -> Store {
    let s = Store::open(":memory:").unwrap();
    s.register_game("fixture", "Synthetic game").unwrap();
    s.register_entity(ENTITY, "fixture", "mod", "Synthetic project")
        .unwrap();
    s.set_source_policy("fixture", "https://example.org/policy", true, true, "v1")
        .unwrap();
    s
}
fn row(id: &str, when: i64, version: &str) -> Observation {
    Observation {
        id: id.into(),
        entity_id: ENTITY.into(),
        source_id: "fixture".into(),
        source_url: "https://example.org/durable-advisory".into(),
        predicate: "maintenance_kitchen_presets_v1".into(),
        value: json!({
            "version":1,"kind":"generated_file","trigger":"replace_suite_life",
            "risk":"Synthetic separate kitchen preset warning",
            "preserve":[{"scope":"mod_generated_state","label":"Keep the generated presets", "location":"preset.dat"}],
            "autofix":false,"tested_in_game":false,"install_status":"unverified"
        }).to_string(),
        observed_unix: when,
        applicable_build: "unverified".into(),
        mod_version: version.into(),
        completeness: "partial".into(),
        policy_revision: "v1".into(),
    }
}
fn report(s: &Store, now: i64, age: i64) -> Value {
    s.maintenance_report(ENTITY, "1.11", MaintenanceAction::ReplaceMod, now, age)
        .unwrap()
}

#[test]
fn exact_version_action_window_and_withdrawal_do_not_claim_local_safety() {
    let mut s = data();
    let mut duplicate = row("second", 12, "1.11");
    duplicate.source_url = "https://example.org/durable-advisory".into();
    s.import(&[
        row("first", 10, "1.11"),
        duplicate,
        row("future", 300, "1.11"),
        row("other-version", 11, "1.12"),
    ])
    .unwrap();
    assert_eq!(report(&s, 9, 100)["coverage"], "unknown");
    let earliest = report(&s, 10, 0);
    assert_eq!(earliest["advisories"][0]["id"], "first");
    let later = report(&s, 20, 50);
    assert_eq!(later["advisories"].as_array().unwrap().len(), 1);
    assert_eq!(later["advisories"][0]["id"], "second");
    assert_eq!(
        later["advisories"][0]["preserve"][0]["scope"],
        "mod_generated_state"
    );
    assert_eq!(
        later["advisories"][0]["preserve"][0]["basename_hint_only"],
        "preset.dat"
    );
    assert_eq!(later["safe_to_apply"], Value::Null);
    assert_eq!(later["local_installation_verified"], false);
    assert_eq!(report(&s, 20, 1)["stale_rows"], 2);
    assert_eq!(report(&s, 20, 1)["coverage"], "unknown");
    let other = s
        .maintenance_report(ENTITY, "1.11", MaintenanceAction::SaveWithoutMod, 20, 50)
        .unwrap();
    assert_eq!(other["coverage"], "unknown");
    assert!(
        s.maintenance_report(ENTITY, "unknown", MaintenanceAction::ReplaceMod, 20, 50)
            .is_err()
    );
    s.set_source_policy("fixture", "https://example.org/policy", true, false, "v2")
        .unwrap();
    assert_eq!(report(&s, 20, 50)["coverage"], "unknown");
}

#[test]
fn refuses_duplicate_fields_paths_misclassified_scope_and_unreviewed_evidence() {
    let mut s = data();
    let mut path = row("path", 10, "1.11");
    path.value = path.value.replace("preset.dat", "../slot.save");
    let mut wrong = row("scope", 11, "1.11");
    wrong.value = wrong.value.replace("mod_generated_state", "game_save");
    let mut duplicate = row("duplicate", 12, "1.11");
    duplicate.value = duplicate
        .value
        .replace("\"location\":", "\"location\":null,\"location\":");
    let mut bad = row("overclaim", 13, "1.11");
    bad.value = bad.value.replace("\"autofix\":false", "\"autofix\":true");
    let mut unreviewed = row("unreviewed", 14, "1.11");
    unreviewed.completeness = "unknown".into();
    s.import(&[path, wrong, duplicate, bad, unreviewed])
        .unwrap();
    let r = report(&s, 20, 50);
    assert_eq!(r["invalid_rows"], 4);
    assert_eq!(r["unreviewed_rows"], 1);
    assert_eq!(r["coverage"], "unknown");
    assert_eq!(r["advisories"], json!([]));
}

#[test]
fn out_of_scope_rows_do_not_consume_bound_and_in_scope_overflow_fails_closed() {
    let mut s = data();
    let future = (0..525)
        .map(|i| row(&format!("future-{i}"), 1000, "1.11"))
        .collect::<Vec<_>>();
    s.import(&future).unwrap();
    s.import(&[row("in-scope", 10, "1.11")]).unwrap();
    assert_eq!(report(&s, 20, 100)["advisories"][0]["id"], "in-scope");
    assert!(
        s.maintenance_report(ENTITY, "1.11", MaintenanceAction::ReplaceMod, 1000, 100)
            .unwrap_err()
            .to_string()
            .contains("bound")
    );
}
