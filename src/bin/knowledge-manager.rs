//! Native read-only catalog manager. JSON reports or standalone visual HTML output.
use athena_mod_knowledge::{Result, Store};

const USAGE: &str = "Usage: knowledge-manager DATABASE status|check|duplicates|browse|visual [QUERY [AFTER_ID [LIMIT]]]\n       knowledge-manager DATABASE browse-in|visual-in GAME KIND [QUERY [AFTER_ID [LIMIT]]]\nGAME and KIND are exact catalog IDs, or * for either dimension. Quote '*' in a shell.";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--help"] || args == ["-h"] {
        println!("{USAGE}");
        return Ok(());
    }
    let Some(command) = args.get(1).map(String::as_str) else {
        return Err(USAGE.into());
    };
    let scoped = matches!(command, "browse-in" | "visual-in");
    let browsing = matches!(command, "browse" | "browse-in" | "visual" | "visual-in");
    if !(match command {
        "browse" | "visual" => args.len() <= 5,
        "browse-in" | "visual-in" => (4..=7).contains(&args.len()),
        "status" | "check" | "duplicates" => args.len() == 2,
        _ => false,
    }) {
        return Err(USAGE.into());
    }

    let store = Store::open_read_only(&args[0])?;
    if browsing {
        fn any(id: &str) -> Option<&str> {
            (id != "*").then_some(id)
        }
        let (game, kind, start) = if scoped {
            (any(&args[2]), any(&args[3]), 4)
        } else {
            (None, None, 2)
        };
        let query = args.get(start).map(String::as_str).unwrap_or("");
        let cursor = args.get(start + 1).map(String::as_str).unwrap_or("");
        let limit = args
            .get(start + 2)
            .map(|s| s.parse::<u32>())
            .transpose()?
            .unwrap_or(50);
        if matches!(command, "visual" | "visual-in") {
            println!(
                "{}",
                store.visual_catalog_scoped(query, cursor, limit, game, kind)?
            );
        } else {
            let report = store.browse_catalog_scoped(query, cursor, limit, game, kind)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        return Ok(());
    }
    let report = match command {
        "status" => store.catalog_status()?,
        "check" => store.catalog_integrity()?,
        "duplicates" => store.duplicate_candidates()?,
        _ => unreachable!(),
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    if command == "check" && report["ok"] != true {
        return Err("Catalog integrity check failed".into());
    }
    Ok(())
}
