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
    if args.len() < 4 {
        return Err(
            "usage: semantic-changelog-core <skyrim|sims4> OLD NEW [canonical-plugin-basename]"
                .into(),
        );
    }
    let old = input(Path::new(&args[2]))?;
    let new = input(Path::new(&args[3]))?;
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
