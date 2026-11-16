//! Native read-only catalog manager. JSON reports or standalone visual HTML output.
use athena_mod_knowledge::{Result, Store};
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "Usage: knowledge-manager DATABASE status|check|duplicates|browse|visual [QUERY [AFTER_ID [LIMIT]]]";
    if args.len() < 2 {
        return Err(usage.into());
    }
    let valid = match args[1].as_str() {
        "browse" | "visual" => args.len() <= 5,
        "status" | "check" | "duplicates" => args.len() == 2,
        _ => false,
    };
    if !valid {
        return Err(usage.into());
    }
    let store = Store::open_read_only(&args[0])?;
    if args[1] == "visual" {
        println!(
            "{}",
            store.visual_catalog(
                args.get(2).map(String::as_str).unwrap_or(""),
                args.get(3).map(String::as_str).unwrap_or(""),
                args.get(4)
                    .map(|s| s.parse::<u32>())
                    .transpose()?
                    .unwrap_or(50)
            )?
        );
        return Ok(());
    }
    let report = match args[1].as_str() {
        "status" => store.catalog_status()?,
        "check" => store.catalog_integrity()?,
        "duplicates" => store.duplicate_candidates()?,
        "browse" => store.browse_catalog(
            args.get(2).map(String::as_str).unwrap_or(""),
            args.get(3).map(String::as_str).unwrap_or(""),
            args.get(4)
                .map(|s| s.parse::<u32>())
                .transpose()?
                .unwrap_or(50),
        )?,
        _ => unreachable!(),
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    if args[1] == "check" && report["ok"] != true {
        return Err("Catalog integrity check failed".into());
    }
    Ok(())
}
