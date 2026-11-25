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
