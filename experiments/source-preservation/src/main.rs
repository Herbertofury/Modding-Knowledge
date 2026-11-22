use source_preservation::{Capture, Kind, Limits, Result, Vault};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
struct Stage(std::path::PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn private_dir(path: &Path) -> Result<()> {
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b.create(path)?;
    Ok(())
}
fn preview(v: &Vault, id: &str, dir: &Path) -> Result<()> {
    if dir.try_exists()? || fs::symlink_metadata(dir).is_ok() {
        return Err("Preview destination already exists".into());
    }
    let parent = dir
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = dir
        .file_name()
        .ok_or("Preview needs a directory name")?
        .to_string_lossy();
    let stage_path = parent.join(format!(".{name}.pending-{}", std::process::id()));
    private_dir(&stage_path)?;
    let stage = Stage(stage_path);
    preview_into(v, id, &stage.0)?;
    if fs::symlink_metadata(dir).is_ok() {
        return Err("Preview destination appeared; refusing overwrite".into());
    }
    fs::rename(&stage.0, dir)?;
    Ok(())
}
fn preview_into(v: &Vault, id: &str, dir: &Path) -> Result<()> {
    // Original HTML/JS is never opened by this viewer.
    let m = v.manifest(id)?;
    let mut h = format!(
        "<!doctype html><html><head><meta charset=utf-8><meta http-equiv=Content-Security-Policy content=\"default-src 'none'; img-src 'self'; media-src 'self'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'\"><title>Preserved source</title><style>body{{background:#050505;color:#eee;font:16px system-ui;margin:2rem}}article{{border:1px solid #444;border-radius:12px;padding:1rem;margin:1rem 0}}img,video{{max-width:100%;max-height:65vh}}.gallery{{display:flex;overflow:auto;gap:1rem}}.gallery article{{min-width:320px;max-width:75vw}}a{{color:#ddd}}code{{overflow-wrap:anywhere}}</style></head><body><h1>{}</h1><p>Version {} · capture time {} (Unix seconds)</p><p>Inventory complete: {}. Discovery scope: {}. This does not prove every resource on the source site was discovered.</p><p>Private local archive. Lifecycle status is separate from capture availability.</p><p><a href=\"{}\" rel=\"noreferrer noopener\">Original source (requires internet)</a></p><div class=gallery>",
        esc(&m.capture.project_id),
        esc(&m.capture.version),
        m.capture.captured_unix,
        m.inventory_complete,
        esc(&m.capture.discovery_scope),
        esc(&m.capture.page_url)
    );
    for (n, a) in m.assets.iter().enumerate() {
        h.push_str(&format!(
            "<article><h2>{}</h2><p>{:?} · {} bytes</p>",
            esc(&a.id),
            a.kind,
            a.bytes
        ));
        let ext = match (&a.kind, a.media_type.as_str()) {
            (Kind::Image, "image/png") => Some("png"),
            (Kind::Image, "image/jpeg") => Some("jpg"),
            (Kind::Image, "image/gif") => Some("gif"),
            (Kind::Image, "image/webp") => Some("webp"),
            (Kind::Video, "video/mp4") => Some("mp4"),
            (Kind::Video, "video/webm") => Some("webm"),
            _ => None,
        };
        if let (Some(ext), Some(_)) = (ext, &a.sha256) {
            let name = format!("asset-{n}.{ext}");
            let path = dir.join(&name);
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)?;
            v.copy_asset(id, &a.id, &mut f)?;
            f.sync_all()?;
            drop(f);
            let mut sig = [0u8; 12];
            let nread = fs::File::open(&path)?.read(&mut sig)?;
            let ok = match ext {
                "png" => sig.starts_with(b"\x89PNG\r\n\x1a\n"),
                "jpg" => sig.starts_with(b"\xff\xd8\xff"),
                "gif" => sig.starts_with(b"GIF87a") || sig.starts_with(b"GIF89a"),
                "webp" => nread >= 12 && &sig[..4] == b"RIFF" && &sig[8..12] == b"WEBP",
                "mp4" => nread >= 12 && &sig[4..8] == b"ftyp",
                "webm" => sig.starts_with(b"\x1a\x45\xdf\xa3"),
                _ => false,
            };
            if ok {
                h.push_str(&match a.kind {
                    Kind::Video => {
                        format!("<video controls preload=metadata src=\"{name}\"></video>")
                    }
                    _ => format!("<img loading=lazy src=\"{name}\" alt=\"{}\">", esc(&a.id)),
                });
            } else {
                fs::remove_file(&path)?;
                h.push_str("<p>Preview withheld: media signature does not match. Original bytes remain preserved.</p>");
            }
        } else if let Some(reason) = &a.missing_reason {
            h.push_str(&format!("<p>Not captured: {}</p>", esc(reason)));
        } else {
            h.push_str("<p>Retained as opaque original; not executed by safe viewer. Use extract for original bytes.</p>");
        }
        h.push_str(&format!(
            "<p>SHA256 <code>{}</code></p></article>",
            esc(a.sha256.as_deref().unwrap_or("not retained"))
        ));
    }
    h.push_str("</div></body></html>");
    fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&m)?)?;
    fs::write(dir.join("index.html"), h)?;
    Ok(())
}
fn run() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("init") if a.len()==3=>{Vault::create(&a[2])?;},
        Some("ingest") if a.len()==7=>{let mut v=Vault::open(&a[2])?;let c:Capture=serde_json::from_slice(&fs::read(&a[3])?)?;let m=v.ingest(c,Limits{max_asset_bytes:a[4].parse()?,max_capture_bytes:a[5].parse()?,max_unique_bytes:a[6].parse()?},||false)?;println!("{}",serde_json::to_string_pretty(&m)?);},
        Some("manifest") if a.len()==4=>println!("{}",serde_json::to_string_pretty(&Vault::open(&a[2])?.manifest(&a[3])?)?),
        Some("preview") if a.len()==5=>preview(&Vault::open(&a[2])?,&a[3],Path::new(&a[4]))?,
        Some("extract") if a.len() == 6 => {
            let v = Vault::open(&a[2])?;
            let dest = Path::new(&a[5]);
            let parent = dest.parent().filter(|p|!p.as_os_str().is_empty()).unwrap_or(Path::new("."));
            let name = dest.file_name().ok_or("Extraction requires filename")?.to_string_lossy();
            let stage_path = parent.join(format!(".{name}.pending-{}",std::process::id()));
            private_dir(&stage_path)?;
            let stage = Stage(stage_path);
            let temp = stage.0.join("verified-asset");
            let mut options = OpenOptions::new(); options.create_new(true).write(true);
            #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
            let mut f = options.open(&temp)?;
            v.copy_asset(&a[3], &a[4], &mut f)?;
            f.flush()?; f.sync_all()?; drop(f);
            fs::hard_link(&temp,dest)?;
        },
        Some("retention-review") if a.len()==4=>println!("{}",serde_json::to_string(&Vault::open(&a[2])?.retention_candidates(a[3].parse()?)?)?),
        _=>return Err("Usage: source-preservation init DB | ingest DB INPUT_JSON MAX_ASSET_BYTES MAX_CAPTURE_BYTES MAX_UNIQUE_BYTES | manifest DB CAPTURE | preview DB CAPTURE NEW_DIRECTORY | extract DB CAPTURE ASSET NEW_FILE | retention-review DB UNIX_TIME".into())
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
