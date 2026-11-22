#[path = "../examples/support/new_sqlite.rs"]
mod new_sqlite;

use athena_mod_knowledge::Store;
use std::path::{Path, PathBuf};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let base = std::env::temp_dir();
        // Existing directory names are not ours. Never delete or reuse them.
        for n in 0..100 {
            let path = base.join(format!("mk-new-output-{}-{n}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("Could not create fixture dir: {error}"),
            }
        }
        panic!("No unused fixture dir");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn refuses_sqlite_special_names_without_reporting_a_portable_file() {
    for value in ["", ":memory:"] {
        let error = new_sqlite::open_new(Path::new(value)).err().unwrap();
        assert_eq!(error.to_string(), "Supply a new on-disk SQLite output path");
    }
}

#[test]
fn creates_persistent_database_and_cannot_open_its_path_twice_for_import() {
    let scratch = Scratch::new();
    let dest = scratch.0.join("my portable snapshot.sqlite");

    let store = new_sqlite::open_new(&dest).unwrap();
    store.register_game("fixture", "fixture").unwrap();
    store
        .register_entity("fixture:project", "fixture", "mod", "Fixture mod")
        .unwrap();
    let before = std::fs::read(&dest).unwrap();
    assert_eq!(
        new_sqlite::open_new(&dest).err().unwrap().to_string(),
        "Output exists; refusing overwrite"
    );
    assert_eq!(std::fs::read(&dest).unwrap(), before);

    drop(store);
    assert!(Store::open_read_only(&dest).is_ok());
    let db =
        rusqlite::Connection::open_with_flags(&dest, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let persisted: String = db
        .query_row(
            "SELECT name FROM entities WHERE id = 'fixture:project'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(persisted, "Fixture mod");
}

#[test]
fn concurrent_initializers_have_only_one_winner() {
    let scratch = Scratch::new();
    let dest = scratch.0.join("one.sqlite");
    let go = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        let start = || {
            go.wait();
            new_sqlite::open_new(&dest)
                .map(drop)
                .map_err(|e| e.to_string())
        };
        let first = scope.spawn(start);
        let second = start();
        let results = [first.join().unwrap(), second];
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .find_map(|r| r.as_ref().err())
                .unwrap()
                .to_string(),
            "Output exists; refusing overwrite"
        );
    });
    assert!(Store::open_read_only(&dest).is_ok());
}
