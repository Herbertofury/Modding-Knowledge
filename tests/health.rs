use athena_mod_knowledge::{Observation, Store};
fn fixture() -> Store {
    let s = Store::open(":memory:").unwrap();
    s.register_game("oblivion", "Oblivion").unwrap();
    s.register_entity("mod", "oblivion", "mod", "Synthetic test only")
        .unwrap();
    s.set_source_policy("fixture", "https://example.org/policy", true, true, "v1")
        .unwrap();
    s
}
fn row(id: &str, assertion: &str) -> Observation {
    Observation { id:id.into(),entity_id:"mod".into(),source_id:"fixture".into(),source_url:"https://example.org/issue/1".into(),predicate:"health_claim_v1".into(),value:serde_json::json!({"assertion":assertion,"authority":"community_report","lineage":"issue:1","context":"runtime:test;loader:test","status":"active","reported_unix":1}).to_string(),observed_unix:2,applicable_build:"1.2".into(),mod_version:"1.0".into(),completeness:"partial".into(),policy_revision:"v1".into() }
}
#[test]
fn unknown_conflict_scope_and_lineage() {
    let mut s = fixture();
    assert_eq!(
        s.health_report("mod", "1.2", "1.0", "runtime:test;loader:test", 3, 5)
            .unwrap()["state"],
        "unknown"
    );
    s.import_health_claims(&[row("a", "works"), row("b", "broken")])
        .unwrap();
    let r = s
        .health_report("mod", "1.2", "1.0", "runtime:test;loader:test", 3, 5)
        .unwrap();
    assert_eq!(r["state"], "conflicting");
    assert_eq!(r["distinct_report_lineages"], 1);
    assert_eq!(r["claims"].as_array().unwrap().len(), 2);
    assert_eq!(
        s.health_report("mod", "remastered", "1.0", "runtime:test;loader:test", 3, 5)
            .unwrap()["state"],
        "unknown"
    );
    assert_eq!(
        s.health_report("mod", "1.2", "1.0", "runtime:other", 3, 5)
            .unwrap()["state"],
        "unknown"
    );
    assert_eq!(
        s.health_report("mod", "1.2", "1.0", "runtime:test;loader:test", 100, 5)
            .unwrap()["state"],
        "stale"
    );
}
#[test]
fn rejected_batch_is_atomic_and_policy_withdrawal_applies() {
    let mut s = fixture();
    let mut invalid = row("b", "works");
    invalid.value = "{}".into();
    assert!(
        s.import_health_claims(&[row("a", "works"), invalid])
            .is_err()
    );
    assert!(s.observations("mod").unwrap().is_empty());
    s.import_health_claims(&[row("a", "works")]).unwrap();
    s.set_source_policy("fixture", "https://example.org/policy", true, false, "v2")
        .unwrap();
    assert_eq!(
        s.health_report("mod", "1.2", "1.0", "runtime:test;loader:test", 3, 5)
            .unwrap()["state"],
        "unknown"
    );
}
#[test]
fn retract_and_unrelated_metadata_never_mean_good() {
    let mut s = fixture();
    let mut a = row("a", "works");
    let mut v: serde_json::Value = serde_json::from_str(&a.value).unwrap();
    v["status"] = "retracted".into();
    a.value = v.to_string();
    s.import_health_claims(&[a]).unwrap();
    let mut m = row("m", "works");
    m.predicate = "description".into();
    s.import(&[m]).unwrap();
    let r = s
        .health_report("mod", "1.2", "1.0", "runtime:test;loader:test", 3, 5)
        .unwrap();
    assert_eq!(r["state"], "unknown");
    assert_eq!(r["claims"].as_array().unwrap().len(), 1);
}
#[test]
fn refreshing_old_report_does_not_renew_claim() {
    let mut s = fixture();
    let mut r = row("old", "works");
    r.observed_unix = 100;
    s.import_health_claims(&[r]).unwrap();
    assert_eq!(
        s.health_report("mod", "1.2", "1.0", "runtime:test;loader:test", 101, 5)
            .unwrap()["state"],
        "stale"
    );
}

#[test]
fn generic_scope_placeholders_cannot_promote_retained_reports() {
    let mut s = fixture();
    let mut unscoped = row("source-with-unverified-build", "warning");
    unscoped.applicable_build = "unverified".into();
    s.import_health_claims(std::slice::from_ref(&unscoped))
        .unwrap();

    // Legacy observations stay in the source ledger; the exact consumer must
    // not convert identical "unverified" strings into a user-facing warning.
    assert_eq!(s.observations("mod").unwrap()[0].id, unscoped.id);
    for (build, version, context, missing) in [
        ("unverified", "1.0", "runtime:test;loader:test", "game build"),
        ("1.2", " UnKnOwN ", "runtime:test;loader:test", "mod version"),
        ("1.2", "1.0", "unspecified", "runtime context"),
        ("1.2", "1.0", "*", "runtime context"),
        ("1.2", "N/A", "runtime:test;loader:test", "mod version"),
    ] {
        let why = s
            .health_report("mod", build, version, context, 3, 5)
            .unwrap_err()
            .to_string();
        assert!(why.contains(missing), "{why}");
    }
    let unrelated = s
        .health_report("mod", "1.2", "1.0", "runtime:test;loader:test", 3, 5)
        .unwrap();
    assert_eq!(unrelated["state"], "unknown");
    assert!(unrelated["claims"].as_array().unwrap().is_empty());
}

#[test]
fn valid_qualified_scope_matching_stays_exact_without_normalizing_versions() {
    let mut s = fixture();
    let mut qualifying = row("qualified-report", "warning");
    qualifying.mod_version = "1.0-beta+build.5".into();
    let context = "{\"loader\":\"TestLoader; version unspecified\",\"runtime\":\"1.2\"}";
    let mut value: serde_json::Value = serde_json::from_str(&qualifying.value).unwrap();
    value["context"] = context.into();
    qualifying.value = value.to_string();
    s.import_health_claims(&[qualifying]).unwrap();
    let exact = s
        .health_report("mod", "1.2", "1.0-beta+build.5", context, 3, 5)
        .unwrap();
    assert_eq!(exact["state"], "partial_evidence");
    assert_eq!(exact["compatibility_verdict"], "not_determined");
    for version in ["1.0", "1.0-beta+build.6", "1.0-BETA+build.5"] {
        let r = s.health_report("mod", "1.2", version, context, 3, 5).unwrap();
        assert_eq!(r["state"], "unknown");
        assert_eq!(r["claims"].as_array().unwrap().len(), 0);
    }
}
