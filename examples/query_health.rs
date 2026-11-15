//! No network; reads an existing reviewed SQLite database without creating it.
use athena_mod_knowledge::{Result, Store};
fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 8 {
        return Err("Usage: query_health DB ENTITY GAME_BUILD MOD_VERSION RUNTIME_CONTEXT NOW_UNIX MAX_AGE_SECONDS".into());
    }
    let store = Store::open_read_only(&a[1])?;
    let report = store.health_report(&a[2], &a[3], &a[4], &a[5], a[6].parse()?, a[7].parse()?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
