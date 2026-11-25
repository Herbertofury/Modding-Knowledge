//! Inspect a reviewed database only; never creates, installs or restores.
use athena_mod_knowledge::{Result, Store, maintenance::MaintenanceAction};

fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 7 {
        return Err(
            "Usage: query_maintenance DB ENTITY MOD_VERSION ACTION AS_OF_UNIX MAX_AGE_SECONDS"
                .into(),
        );
    }
    let store = Store::open_read_only(&a[1])?;
    let action = MaintenanceAction::parse(&a[4])?;
    let report = store.maintenance_report(&a[2], &a[3], action, a[5].parse()?, a[6].parse()?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
