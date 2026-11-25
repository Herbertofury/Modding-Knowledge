use athena_mod_knowledge::Store;
use std::{fs, process::Command};
#[test]
fn cli_is_read_only_and_rejects_missing_future_and_invalid_arguments() {
    let dir = std::env::temp_dir().join(format!("knowledge-manager-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let db = dir.join("catalog.sqlite");
    let exe = env!("CARGO_BIN_EXE_knowledge-manager");
    assert!(
        !Command::new(exe)
            .arg(&db)
            .arg("status")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!db.exists());
    drop(Store::open(&db).unwrap());
    let before = fs::read(&db).unwrap();
    for command in ["status", "check", "duplicates", "browse"] {
        let out = Command::new(exe).arg(&db).arg(command).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap();
        assert_eq!(before, fs::read(&db).unwrap());
    }
    assert!(
        !Command::new(exe)
            .arg(&db)
            .args(["status", "unexpected"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.pragma_update(None, "user_version", 99).unwrap();
    drop(conn);
    let future = fs::read(&db).unwrap();
    assert!(
        !Command::new(exe)
            .arg(&db)
            .arg("check")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(future, fs::read(&db).unwrap());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn real_cli_filters_game_and_kind_before_pagination_and_leaves_foreign_rows_out() {
    use athena_mod_knowledge::Observation;
    let dir = std::env::temp_dir().join(format!("knowledge-manager-scoped-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let db = dir.join("catalog.sqlite");
    let mut store = Store::open(&db).unwrap();
    for id in ["oblivion-family", "minecraft-java"] {
        store.register_game(id, id).unwrap();
    }
    store
        .set_source_policy("p", "https://example.org/terms", true, true, "v1")
        .unwrap();
    for (id, game, kind) in [
        ("a:foreign", "oblivion-family", "guide"),
        ("b:profile", "minecraft-java", "creator"),
        ("c:match", "minecraft-java", "mod"),
        ("d:next", "minecraft-java", "mod"),
    ] {
        store
            .register_entity(id, game, kind, "Iris fixture")
            .unwrap();
        store
            .import(&[Observation {
                id: id.into(),
                entity_id: id.into(),
                source_id: "p".into(),
                source_url: "https://example.org/item".into(),
                predicate: "catalog.name".into(),
                value: "Iris".into(),
                observed_unix: 10,
                applicable_build: "unknown".into(),
                mod_version: "unknown".into(),
                completeness: "unknown".into(),
                policy_revision: "v1".into(),
            }])
            .unwrap();
    }
    drop(store);
    let before = fs::read(&db).unwrap();
    let exe = env!("CARGO_BIN_EXE_knowledge-manager");
    let run = |args: &[&str]| Command::new(exe).arg(&db).args(args).output().unwrap();
    let result = run(&["browse-in", "minecraft-java", "mod", "Iris", "", "1"]);
    assert!(result.status.success(), "{:?}", result.stderr);
    let first: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(first["entities"][0]["id"], "c:match");
    assert_eq!(first["next_cursor"], "c:match");
    assert_eq!(first["filters"]["game_id"], "minecraft-java");
    let second = run(&["browse-in", "minecraft-java", "mod", "Iris", "c:match", "1"]);
    let second: serde_json::Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(second["entities"][0]["id"], "d:next");
    assert!(second["next_cursor"].is_null());
    let out = run(&["visual-in", "minecraft-java", "mod", "Iris", "", "1"]);
    assert!(out.status.success());
    let html = String::from_utf8(out.stdout).unwrap();
    assert!(html.contains("Catalog filters: game <code>minecraft-java</code>"));
    assert!(html.contains("c:match"));
    assert!(!html.contains("a:foreign"));
    assert!(!html.contains("b:profile"));
    assert!(!run(&["browse-in", "not valid", "mod"]).status.success());
    assert_eq!(before, fs::read(&db).unwrap());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn evidence_command_uses_the_public_policy_and_preserves_the_original_file() {
    use athena_mod_knowledge::Observation;
    let dir =
        std::env::temp_dir().join(format!("knowledge-manager-evidence-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let db = dir.join("existing catalog.sqlite");
    let mut store = Store::open(&db).unwrap();
    store.register_game("game", "Game").unwrap();
    store
        .register_entity("fixture:item", "game", "mod", "Item")
        .unwrap();
    store
        .set_source_policy("public", "https://example.org/legal", true, true, "1")
        .unwrap();
    store
        .set_source_policy("withdrawn", "https://example.org/legal", true, true, "1")
        .unwrap();
    for source_id in ["public", "withdrawn"] {
        store
            .import(&[Observation {
                id: source_id.into(),
                entity_id: "fixture:item".into(),
                source_id: source_id.into(),
                predicate: "catalog.name".into(),
                value: source_id.into(),
                source_url: "https://example.org/item".into(),
                observed_unix: 5,
                applicable_build: "unknown".into(),
                mod_version: "unknown".into(),
                completeness: "partial".into(),
                policy_revision: "1".into(),
            }])
            .unwrap();
    }
    store
        .set_source_policy("withdrawn", "https://example.org/legal", true, false, "1")
        .unwrap();
    drop(store);
    let before = fs::read(&db).unwrap();
    let execute = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_knowledge-manager"))
            .arg(&db)
            .arg("evidence")
            .args(args)
            .output()
            .unwrap()
    };
    let result = execute(&["fixture:item", "--predicate", "catalog.name"]);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["count"], 1);
    assert_eq!(report["observations"][0]["id"], "public");
    let result = execute(&["fixture:item", "--mod-relations-v2"]);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["count"], 0);
    assert_eq!(report["mod_relationships"]["coverage"], "unknown");
    assert!(!execute(&[]).status.success());
    assert_eq!(before, fs::read(&db).unwrap());
    fs::remove_dir_all(dir).unwrap();
}
