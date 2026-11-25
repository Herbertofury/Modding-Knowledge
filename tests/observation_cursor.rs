use athena_mod_knowledge::{Observation, ObservationAnchor, Store};

fn observation(i: usize, timestamp: i64, source: &str) -> Observation {
    Observation {
        id: format!("fixture-{i:05}"),
        entity_id: "fixture:mod".into(),
        source_id: source.into(),
        source_url: "https://fixture.example/mod".into(),
        predicate: if i.is_multiple_of(5) {
            "relationships".into()
        } else {
            "general".into()
        },
        value: format!("fixture:{i}"),
        observed_unix: timestamp,
        applicable_build: "fixture".into(),
        mod_version: "fixture".into(),
        completeness: "partial".into(),
        policy_revision: "r1".into(),
    }
}

fn anchor(row: &Observation) -> ObservationAnchor<'_> {
    ObservationAnchor {
        observed_unix: row.observed_unix,
        id: &row.id,
    }
}

#[test]
fn cursor_keeps_all_equal_timestamp_ties_and_crosses_older_range() {
    let mut store = Store::open(":memory:").unwrap();
    store.register_game("fixture", "Fixture").unwrap();
    store
        .register_entity("fixture:mod", "fixture", "mod", "Fixture mod")
        .unwrap();
    store
        .set_source_policy("a", "https://fixture.example/a", true, true, "r1")
        .unwrap();
    store
        .set_source_policy("b", "https://fixture.example/b", true, true, "r1")
        .unwrap();
    let rows: Vec<_> = (0..6200)
        .map(|i| {
            observation(
                i,
                if i < 4550 { 20 } else { 10 },
                if i.is_multiple_of(3) { "a" } else { "b" },
            )
        })
        .collect();
    for chunk in rows.chunks(1000) {
        store.import(chunk).unwrap();
    }
    // Compare stable traversal, including same-time pages and predicate applied
    // before page limits, with the existing byte-preserving offset reader.
    for predicate in [None, Some("relationships")] {
        let mut all = Vec::new();
        let mut pivot: Option<Observation> = None;
        let mut offset = 0;
        loop {
            let page = store
                .observations_after("fixture:mod", predicate, pivot.as_ref().map(anchor))
                .unwrap();
            let baseline = match predicate {
                None => store.observations_page("fixture:mod", offset).unwrap(),
                Some(p) => store
                    .observations_page_for_predicate("fixture:mod", p, offset)
                    .unwrap(),
            };
            assert_eq!(page, baseline);
            let count = page.len();
            all.extend(page.iter().map(|r| r.id.clone()));
            pivot = page.into_iter().last();
            offset += count as u32;
            if count < 1000 {
                break;
            }
        }
        assert_eq!(all.len(), if predicate.is_some() { 1240 } else { 6200 });
        assert!(all.windows(2).all(|ids| ids[0] < ids[1]));
    }

    let first = store.observations_after("fixture:mod", None, None).unwrap();
    assert_eq!(first.last().unwrap().source_id, "a");
    let last = anchor(first.last().unwrap());
    store
        .import(&[observation(8001, 30, "b"), observation(8002, 20, "b")])
        .unwrap();
    let offset_next = store.observations_page("fixture:mod", 1000).unwrap();
    assert_eq!(
        offset_next[0],
        *first.last().unwrap(),
        "an offset duplicates a row after a newer insert"
    );
    store
        .set_source_policy("a", "https://fixture.example/a", true, false, "r2")
        .unwrap();
    let mut pivot = last;
    let mut previous;
    let mut continued = Vec::new();
    loop {
        let page = store
            .observations_after("fixture:mod", None, Some(pivot))
            .unwrap();
        let count = page.len();
        continued.extend(page.iter().map(|r| r.id.clone()));
        assert!(page.iter().all(|r| r.source_id == "b"));
        if count < 1000 {
            break;
        }
        previous = page.into_iter().last().unwrap();
        pivot = anchor(&previous);
    }
    let mut expected: Vec<_> = rows
        .iter()
        .filter(|r| r.source_id == "b" && r.id.as_str() > last.id)
        .map(|r| r.id.clone())
        .collect();
    // A later ID at the same timestamp lands between the two timestamp groups.
    let split = expected.partition_point(|id| id.as_str() < "fixture-04550");
    expected.insert(split, "fixture-08002".into());
    assert_eq!(continued, expected);
    assert!(
        store
            .observations_after("fixture:mod", Some(""), None)
            .is_err()
    );
    assert!(
        store
            .observations_after(
                "fixture:mod",
                None,
                Some(ObservationAnchor {
                    observed_unix: -1,
                    id: "x"
                })
            )
            .is_err()
    );
}
