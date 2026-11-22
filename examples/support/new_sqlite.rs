//! New portable output for the small single-purpose catalog examples.
use athena_mod_knowledge::{Result, Store};
use std::{fs::OpenOptions, path::Path};

pub fn open_new(path: &Path) -> Result<Store> {
    if path.as_os_str().is_empty() || path == Path::new(":memory:") {
        return Err("Supply a new on-disk SQLite output path".into());
    }
    // Always pass a filesystem path, not a SQLite in-memory name or file: URI.
    let path = std::path::absolute(path)?;
    // Reserve the output exclusively. Unlike exists() followed by SQLite open,
    // competing runs cannot both open an absent path as a new catalog.
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => drop(file),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err("Output exists; refusing overwrite".into());
        }
        Err(error) => return Err(error.into()),
    }
    Store::open(&path)
}
