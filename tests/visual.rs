use athena_mod_knowledge::{Observation, Store};
use serde_json::json;
#[test]
fn evidence_media_rights_xss_and_withdrawal() {
    let mut s = Store::open(":memory:").unwrap();
    s.register_game("fixture", "Fixture").unwrap();
    s.register_entity(
        "fixture:1",
        "fixture",
        "mod",
        "<script>alert('name')</script>",
    )
    .unwrap();
    s.set_source_policy("fixture", "https://example.org/terms", true, true, "1")
        .unwrap();
    let mut rows = vec![];
    for (i,(predicate,value)) in [
        ("presentation.content",json!({"label":"ESPFE","evidence_status":"verified"})),
        ("presentation.category",json!({"label":"<img src=x onerror=alert(1)>","evidence_status":"inferred"})),
        ("presentation.media",json!({"kind":"image","url":"https://example.org/image.png","alt":"Fixture art","evidence_status":"provider_declared","display_allowed":true,"retention_allowed":false})),
        ("presentation.media",json!({"kind":"video","url":"https://example.org/video.mp4","evidence_status":"verified","display_allowed":true,"retention_allowed":true})),
        ("presentation.media",json!({"kind":"image","url":"javascript:alert(1)","evidence_status":"verified","display_allowed":true})),
        ("presentation.media",json!({"kind":"image","url":"https://example.org/withheld.png","evidence_status":"verified","display_allowed":false})),
        ("presentation.content",json!({"label":"DLL","evidence_status":"trust_me"})),
    ].into_iter().enumerate() {
        rows.push(Observation{id:i.to_string(),entity_id:"fixture:1".into(),source_id:"fixture".into(),source_url:"https://example.org/source".into(),predicate:predicate.into(),value:value.to_string(),observed_unix:1,applicable_build:"unknown".into(),mod_version:"unknown".into(),completeness:"partial".into(),policy_revision:"1".into()});
    }
    s.import(&rows).unwrap();
    let p = s.presentation("fixture:1").unwrap();
    assert_eq!(p["media"].as_array().unwrap().len(), 2);
    assert_eq!(p["claims"].as_array().unwrap().len(), 2);
    let h = s.visual_catalog("", "", 20).unwrap();
    assert!(h.contains("data-kind=\"video\""));
    assert!(h.contains("&lt;script&gt;"));
    assert!(h.contains("&lt;img src=x"));
    assert!(!h.contains("<img src="));
    assert!(!h.contains("javascript:alert"));
    assert!(!h.contains("withheld.png"));
    assert!(h.contains("inferred"));
    s.set_source_policy("fixture", "https://example.org/terms", true, false, "1")
        .unwrap();
    assert_eq!(s.presentation("fixture:1").unwrap()["media"], json!([]));
    assert!(
        !s.visual_catalog("", "", 20)
            .unwrap()
            .contains("Fixture art")
    );
}

#[test]
fn qualified_titles_link_to_exact_project_or_profile_without_guessing_ambiguous_rows() {
    let mut s = Store::open(":memory:").unwrap();
    s.register_game("game", "Game").unwrap();
    s.set_source_policy("test", "https://example.org/terms", true, true, "1")
        .unwrap();
    let records = [
        (
            "modrinth:project:YL57xq9U",
            "mod",
            "Iris",
            "provider_claim.source_url",
            "https://api.modrinth.com/v2/project/YL57xq9U",
            json!("https://github.com/IrisShaders/Iris"),
        ),
        (
            "modrinth:project:12345678",
            "mod",
            "Wrong ID",
            "provider_claim.source_url",
            "https://api.modrinth.com/v2/project/XL57xq9U",
            json!("https://github.com/IrisShaders/Iris"),
        ),
        (
            "github:creator:Tester",
            "creator",
            "Profile",
            "reviewed.creator_identity",
            "https://example.org/record",
            json!({"profiles":[{"url":"https://github.com/Tester","status":"provider-qualified account URL"},{"url":"https://example.org/maybe","status":"unverified"}]}),
        ),
        (
            "github:creator:Two",
            "creator",
            "Ambiguous",
            "reviewed.creator_identity",
            "https://example.org/record",
            json!({"profiles":[{"url":"https://github.com/Two","status":"provider-qualified account URL"},{"url":"https://example.org/Two","status":"provider-qualified account URL"}]}),
        ),
    ];
    for (id, kind, label, predicate, url, value) in records {
        s.register_entity(id, "game", kind, label).unwrap();
        s.import(&[Observation {
            id: id.into(),
            entity_id: id.into(),
            source_id: "test".into(),
            source_url: url.into(),
            predicate: predicate.into(),
            value: value.to_string(),
            observed_unix: 1,
            applicable_build: "unknown".into(),
            mod_version: "unknown".into(),
            completeness: "partial".into(),
            policy_revision: "1".into(),
        }])
        .unwrap();
    }
    let html = s.visual_catalog("", "", 20).unwrap();
    assert!(html.contains(r#"<strong><a href="https://modrinth.com/project/YL57xq9U" rel="noreferrer noopener" target="_blank">Iris</a></strong>"#));
    assert!(html.contains(r#"<strong><a href="https://github.com/Tester" rel="noreferrer noopener" target="_blank">Profile</a></strong>"#));
    assert!(
        html.contains("<strong>Wrong ID</strong>") && html.contains("<strong>Ambiguous</strong>")
    );
    assert!(!html.contains("https://modrinth.com/project/12345678"));
    assert!(!html.contains("https://example.org/maybe"));
    assert!(
        html.contains("Recorded project / profile pages")
            && html.contains("Reviewed evidence links")
    );
    assert!(!html.contains("<img src=") && !html.contains("<iframe"));
    s.set_source_policy("test", "https://example.org/terms", true, false, "1")
        .unwrap();
    let withdrawn = s.visual_catalog("", "", 20).unwrap();
    assert!(!withdrawn.contains("https://github.com/Tester"));
}
