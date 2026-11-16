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
