use athena_mod_knowledge::{Observation, Store};
use serde_json::{Value, json};

const CONTEXT: &str = "fixture runtime; loader fixture";
fn store() -> Store {
    let s = Store::open(":memory:").unwrap();
    s.register_game("fixture", "Synthetic game").unwrap();
    s.register_entity("fixture:mod", "fixture", "mod", "Synthetic mod")
        .unwrap();
    s.set_source_policy("fixture", "https://example.org/policy", true, true, "v1")
        .unwrap();
    s
}
fn row(id: &str, observed: i64, reported: i64, assertion: &str) -> Observation {
    Observation {
        id: id.into(),
        entity_id: "fixture:mod".into(),
        source_id: "fixture".into(),
        source_url: format!("https://example.org/fixture/{id}"),
        predicate: "health_claim_v1".into(),
        value: json!({
            "assertion": assertion,
            "authority": "community_report",
            "lineage": id,
            "context": CONTEXT,
            "status": "active",
            "reported_unix": reported,
        })
        .to_string(),
        observed_unix: observed,
        applicable_build: "fixture-build-1".into(),
        mod_version: "1.0".into(),
        completeness: "partial".into(),
        policy_revision: "v1".into(),
    }
}
fn report(s: &Store, now: i64, max_age: i64) -> Value {
    s.health_report("fixture:mod", "fixture-build-1", "1.0", CONTEXT, now, max_age)
        .unwrap()
}

#[test]
fn receipts_seen_after_cutoff_do_not_change_historical_claims_or_diagnostics() {
    let mut s = store();
    let old = report(&s, 10, 20);
    let mut future_bad = row("later-malformed", 30, 8, "broken");
    future_bad.value = "{}".into();
    s.import(&[row("only-seen-later", 12, 8, "warning"), future_bad])
        .unwrap();
    assert_eq!(report(&s, 10, 20), old);
    let boundary = report(&s, 12, 20);
    assert_eq!(boundary["state"], "partial_evidence");
    assert_eq!(boundary["invalid_claims"], 0);
    assert_eq!(boundary["claims"].as_array().unwrap().len(), 1);
    assert_eq!(report(&s, 30, 20)["state"], "stale");
    assert_eq!(report(&s, 30, 20)["invalid_claims"], 1);
    assert_eq!(s.observations("fixture:mod").unwrap().len(), 2);
}

#[test]
fn later_receipt_only_introduces_conflict_once_seen_and_exact_version_stays_isolated() {
    let mut s = store();
    let mut other = row("broader-version", 2, 1, "broken");
    other.mod_version = "unknown".into();
    s.import_health_claims(&[
        row("works", 3, 2, "works"),
        row("breakage", 20, 1, "broken"),
        other,
    ])
    .unwrap();
    let early = report(&s, 10, 100);
    assert_eq!(early["state"], "partial_evidence");
    assert_eq!(early["claims"].as_array().unwrap().len(), 1);
    let later = report(&s, 20, 100);
    assert_eq!(later["state"], "conflicting");
    assert_eq!(later["distinct_report_lineages"], 2);
    assert_eq!(later["claims"][0]["id"], "breakage");
    assert_eq!(later["claims"][1]["id"], "works");
    s.set_source_policy("fixture", "https://example.org/policy", true, false, "v2")
        .unwrap();
    let withdrawn = report(&s, 20, 100);
    assert_eq!(withdrawn["state"], "unknown");
    assert!(withdrawn["claims"].as_array().unwrap().is_empty());
}

#[test]
fn future_and_wrong_release_receipts_cannot_exhaust_historical_query_bound() {
    let mut s = store();
    s.import_health_claims(&[row("earlier", 10, 1, "warning")])
        .unwrap();
    for page in 0..10 {
        let high = (0..1000)
            .map(|i| row(&format!("future-{page}-{i}"), 200, 1, "broken"))
            .collect::<Vec<_>>();
        s.import_health_claims(&high).unwrap();
    }
    let mut different = row("wrong-release", 3, 1, "broken");
    different.mod_version = "2.0".into();
    s.import_health_claims(&[different]).unwrap();
    let before = report(&s, 100, 100);
    assert_eq!(before["claims"].as_array().unwrap().len(), 1);
    assert_eq!(before["claims"][0]["id"], "earlier");
    assert!(
        s.health_report("fixture:mod", "fixture-build-1", "1.0", CONTEXT, 200, 100)
            .unwrap_err()
            .to_string()
            .contains("10000")
    );
}
