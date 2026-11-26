//! Read-only JSON query for consumers; does not create/migrate databases or fetch.
use athena_mod_knowledge::{Result, evidence_report};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let report = evidence_report::report(&args)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
