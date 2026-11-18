use sha2::{Digest, Sha256};
use std::{env, fs::File, io::Read, path::Path};
fn input(p: &Path) -> Result<Vec<u8>, String> {
    let mut f = File::open(p).map_err(|e| e.to_string())?;
    let before = f.metadata().map_err(|e| e.to_string())?;
    if !before.is_file() || before.len() > 64 * 1024 * 1024 {
        return Err("not a bounded regular input file".into());
    }
    let mut b = Vec::new();
    f.by_ref()
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    let after = f.metadata().map_err(|e| e.to_string())?;
    if b.len() as u64 != before.len()
        || after.len() != before.len()
        || after.modified().ok() != before.modified().ok()
    {
        return Err("input changed during read".into());
    }
    Ok(b)
}
fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    if args.get(1).is_some_and(|s| s == "poll-ini") {
        return poll_ini(&args);
    }
    if args.len() < 4 {
        return Err(
            "usage: semantic-changelog-core <skyrim|sims4|ini> OLD NEW [canonical-plugin-basename]"
                .into(),
        );
    }
    let read = if args[1] == "ini" {
        semantic_changelog_core::monitor::read_snapshot
    } else {
        input
    };
    let old = read(Path::new(&args[2]))?;
    let new = read(Path::new(&args[3]))?;
    println!(
        "Generated comparison, not author changelog or compatibility verdict.\nold SHA256: {:x}\nnew SHA256: {:x}",
        Sha256::digest(&old),
        Sha256::digest(&new)
    );
    match args[1].as_str() {
        "skyrim" => {
            let name = args.get(4).ok_or(
                "Skyrim comparison requires canonical plugin basename shared by both versions",
            )?;
            let a = semantic_changelog_core::skyrim::parse(&old, name, || false)?;
            let b = semantic_changelog_core::skyrim::parse(&new, name, || false)?;
            println!("{:#?}", semantic_changelog_core::compare(&a, &b)?);
        }
        "ini" => {
            let a = semantic_changelog_core::ini::parse(&old, || false)?;
            let b = semantic_changelog_core::ini::parse(&new, || false)?;
            println!("{:?}", semantic_changelog_core::ini::compare(&a, &b));
        }
        "sims4" => {
            let a = semantic_changelog_core::dbpf::parse(&old, || false)?;
            let b = semantic_changelog_core::dbpf::parse(&new, || false)?;
            println!(
                "Resource-level payload comparison only; CAS/mesh/tuning fields not interpreted.\n{:#?}",
                semantic_changelog_core::dbpf::compare(&a, &b)
            );
        }
        _ => return Err("unsupported format".into()),
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Comparison unavailable: {e}");
        std::process::exit(1);
    }
}

fn poll_ini(args: &[String]) -> Result<(), String> {
    use semantic_changelog_core::{
        edit_analysis::Evidence,
        monitor::{IniMonitor, Sample},
    };
    if args.len() != 5 {
        return Err("usage: semantic-changelog-core poll-ini FILE SAMPLE_COUNT INTERVAL_MS (explicit foreground monitoring)".into());
    }
    let count: usize = args[3].parse().map_err(|_| "invalid sample count")?;
    let interval: u64 = args[4].parse().map_err(|_| "invalid interval")?;
    if !(1..=1000).contains(&count)
        || !(250..=60000).contains(&interval)
        || (count as u64) * interval > 3600000
    {
        return Err("bounded poll requires1..1000 samples,250..60000ms interval,max1hour scheduled intervals".into());
    }
    let mut monitor = IniMonitor::new("selected-settings-file".into(), count)?;
    println!(
        "Foreground INI polling. Ctrl+C stops. Values/keys/paths omitted. Sampled changes only, may miss intermediate edits; writer always unknown. No files written."
    );
    let mut failures = 0usize;
    for i in 0..count {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_millis(interval));
        }
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "system clock before epoch")?
            .as_millis()
            .try_into()
            .map_err(|_| "timestamp overflow")?;
        match monitor.poll(Path::new(&args[2]), time) {
            Ok(Sample::Baseline) => println!("sample{}/{}: baseline", i + 1, count),
            Ok(Sample::Unchanged) => println!("sample{}/{}: sampled bytes unchanged", i + 1, count),
            Ok(Sample::RebasedAfterGap) => println!(
                "sample{}/{}: rebaseline; missed history remains unknown",
                i + 1,
                count
            ),
            Ok(Sample::Changed(r)) => {
                let Evidence::IniSettings(report) = r.evidence else {
                    return Err("unexpected evidence kind".into());
                };
                println!(
                    "sample{}/{}: {} setting deltas; section-order={} key-order={}; writer=unknown",
                    i + 1,
                    count,
                    report.changes.len(),
                    report.section_order_changed,
                    report.key_order_changed
                );
            }
            Err(_) => {
                failures += 1;
                println!(
                    "sample{}/{}: unavailable; history gap recorded (private diagnostics omitted)",
                    i + 1,
                    count
                );
            }
        }
    }
    println!(
        "Finished requested samples. Known gaps: {}. No complete edit-capture guarantee.",
        !monitor.history().continuity_preserved()
    );
    if failures > 0 {
        return Err("one or more settings samples unavailable; gap retained".into());
    }
    Ok(())
}
