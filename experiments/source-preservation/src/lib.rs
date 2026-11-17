//! Private, offline evidence vault. No fetching, execution, deletion or installer replay.
#![forbid(unsafe_code)]
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
const CHUNK: usize = 1024 * 1024;
const SCHEMA: i64 = 1;
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn reference(s: &str) -> Result<()> {
    let u = url::Url::parse(s)?;
    if !matches!(u.scheme(), "https" | "http")
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
    {
        return Err("Use a canonical HTTP(S) evidence URL without credentials, query or fragment; never persist signed transport URLs".into());
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Page,
    Image,
    Video,
    Audio,
    Stylesheet,
    Font,
    ReplayArchive,
    Installer,
    Other,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputAsset {
    pub id: String,
    pub source_url: String,
    pub kind: Kind,
    pub media_type: String,
    /// An authorized local capture/download. None retains an explicit capture gap.
    pub path: Option<PathBuf>,
    pub missing_reason: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallerChoices {
    pub installer_asset_id: String,
    pub installer_sha256: String,
    pub tool: String,
    pub tool_version: String,
    pub recorded_unix: i64,
    /// Raw choice log is another retained asset; no inferred selections or replay.
    pub choice_log_asset_id: String,
    pub evidence_method: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capture {
    pub id: String,
    pub project_id: String,
    pub version: String,
    pub page_url: String,
    pub captured_unix: i64,
    pub adapter: String,
    pub policy_reference: String,
    pub authorized_to_retain: bool,
    pub private_only: bool,
    /// Adapter assertion, not independent proof that every page/media was discovered.
    pub discovery_finished: bool,
    pub discovery_scope: String,
    pub pinned: bool,
    pub retain_until_unix: Option<i64>,
    pub lifecycle_observation_ids: Vec<String>,
    pub installer_choices: Option<InstallerChoices>,
    pub assets: Vec<InputAsset>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub source_url: String,
    pub kind: Kind,
    pub media_type: String,
    pub bytes: u64,
    pub sha256: Option<String>,
    pub chunks: Vec<String>,
    pub missing_reason: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    /// Paths are removed before persistence. Authorization is not redistribution permission.
    pub capture: Capture,
    pub assets: Vec<Asset>,
    pub inventory_complete: bool,
    pub retained_bytes: u64,
}
#[derive(Clone, Copy)]
pub struct Limits {
    pub max_asset_bytes: u64,
    pub max_capture_bytes: u64,
    pub max_unique_bytes: u64,
}
pub struct Vault {
    db: Connection,
}
impl Vault {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        // Reserve the path without replacing another vault or arbitrary user file.
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let reservation = options.open(path.as_ref())?;
        drop(reservation);
        let db = Connection::open(path)?;
        db.execute_batch(
            "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;
        CREATE TABLE chunks(hash TEXT PRIMARY KEY, bytes BLOB NOT NULL);
        CREATE TABLE captures(id TEXT PRIMARY KEY, manifest TEXT NOT NULL);
        PRAGMA user_version=1;",
        )?;
        Ok(Self { db })
    }
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        if db.pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))? != SCHEMA {
            return Err("Unsupported vault schema".into());
        }
        db.execute_batch("PRAGMA synchronous=FULL; PRAGMA busy_timeout=5000;")?;
        Ok(Self { db })
    }
    /// All-or-nothing capture; cancellation/error/quota failure preserves every old capture.
    /// Streaming fixed-size chunks deduplicate across versions without loading video into RAM.
    pub fn ingest(
        &mut self,
        mut capture: Capture,
        limits: Limits,
        cancelled: impl Fn() -> bool,
    ) -> Result<Manifest> {
        if !capture.authorized_to_retain
            || !capture.private_only
            || capture.id.is_empty()
            || capture.project_id.is_empty()
            || capture.adapter.is_empty()
            || capture.policy_reference.is_empty()
            || capture.discovery_scope.is_empty()
            || capture.captured_unix <= 0
            || capture.assets.is_empty()
        {
            return Err(
                "Missing retention authority, provenance, inventory or private-only boundary"
                    .into(),
            );
        }
        reference(&capture.page_url)?;
        if capture
            .retain_until_unix
            .is_some_and(|v| v < capture.captured_unix)
        {
            return Err("Retention expiry predates capture".into());
        }
        let mut ids = HashSet::new();
        for a in &capture.assets {
            reference(&a.source_url)?;
            if a.id.is_empty()
                || !ids.insert(a.id.clone())
                || a.media_type.is_empty()
                || (a.path.is_some() == a.missing_reason.is_some())
                || a.missing_reason
                    .as_ref()
                    .is_some_and(|x| x.trim().is_empty())
            {
                return Err(
                    "Each unique asset needs a file or explicit missing reason, not both".into(),
                );
            }
        }
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if tx
            .query_row("SELECT 1 FROM captures WHERE id=?", [&capture.id], |_| {
                Ok(())
            })
            .optional()?
            .is_some()
        {
            return Err("Capture ID exists; snapshots are immutable".into());
        }
        let mut unique: u64 = tx
            .query_row::<i64, _, _>(
                "SELECT coalesce(sum(length(bytes)),0) FROM chunks",
                [],
                |r| r.get(0),
            )?
            .try_into()?;
        let mut total = 0u64;
        let mut assets = Vec::new();
        for input in &capture.assets {
            if cancelled() {
                return Err("Capture cancelled".into());
            }
            let mut asset = Asset {
                id: input.id.clone(),
                source_url: input.source_url.clone(),
                kind: input.kind.clone(),
                media_type: input.media_type.clone(),
                bytes: 0,
                sha256: None,
                chunks: vec![],
                missing_reason: input.missing_reason.clone(),
            };
            if let Some(path) = &input.path {
                let md = std::fs::symlink_metadata(path)?;
                if !md.is_file() || md.file_type().is_symlink() {
                    return Err("Capture input must be a regular file, not a symlink/device".into());
                }
                let mut f = File::open(path)?;
                let before = f.metadata()?;
                if !before.is_file()
                    || before.len() > limits.max_asset_bytes
                    || total
                        .checked_add(before.len())
                        .is_none_or(|n| n > limits.max_capture_bytes)
                {
                    return Err("Capture exceeds declared byte quota".into());
                }
                let mut full = Sha256::new();
                let mut buf = vec![0; CHUNK];
                loop {
                    if cancelled() {
                        return Err("Capture cancelled".into());
                    }
                    let mut n = 0;
                    while n < CHUNK {
                        let got = f.read(&mut buf[n..])?;
                        if got == 0 {
                            break;
                        }
                        n += got;
                    }
                    if n == 0 {
                        break;
                    }
                    asset.bytes = asset
                        .bytes
                        .checked_add(n as u64)
                        .ok_or("Byte count overflow")?;
                    if asset.bytes > limits.max_asset_bytes
                        || total
                            .checked_add(asset.bytes)
                            .is_none_or(|v| v > limits.max_capture_bytes)
                    {
                        return Err("Capture exceeds byte quota".into());
                    }
                    full.update(&buf[..n]);
                    let digest = hash(&buf[..n]);
                    let existing: Option<Vec<u8>> = tx
                        .query_row(
                            "SELECT bytes FROM chunks WHERE hash=? AND length(bytes)<=1048576",
                            [&digest],
                            |r| r.get(0),
                        )
                        .optional()?;
                    match existing {
                        Some(b) if b != buf[..n] => {
                            return Err("Existing chunk corruption or digest collision".into());
                        }
                        Some(_) => {}
                        None => {
                            unique = unique.checked_add(n as u64).ok_or("Storage overflow")?;
                            if unique > limits.max_unique_bytes {
                                return Err(
                                    "Vault unique-byte quota reached; no prior capture was evicted"
                                        .into(),
                                );
                            }
                            tx.execute(
                                "INSERT INTO chunks VALUES (?,?)",
                                params![digest, &buf[..n]],
                            )?;
                        }
                    }
                    asset.chunks.push(digest);
                }
                let after = f.metadata()?;
                if before.len() != asset.bytes
                    || before.len() != after.len()
                    || before.modified()? != after.modified()?
                {
                    return Err(
                        "Source changed during capture; retry after download finishes".into(),
                    );
                }
                asset.sha256 = Some(format!("{:x}", full.finalize()));
                total += asset.bytes;
            }
            assets.push(asset);
        }
        if let Some(c) = &capture.installer_choices {
            let a = assets
                .iter()
                .find(|a| a.id == c.installer_asset_id)
                .ok_or("Installer asset absent")?;
            if c.choice_log_asset_id == c.installer_asset_id
                || a.kind != Kind::Installer
                || a.sha256.as_deref() != Some(&c.installer_sha256)
                || c.tool.is_empty()
                || c.tool_version.is_empty()
                || c.evidence_method.is_empty()
                || c.recorded_unix <= 0
                || !assets.iter().any(|a| {
                    a.id == c.choice_log_asset_id
                        && a.kind == Kind::Other
                        && matches!(a.media_type.as_str(), "application/json" | "text/plain")
                        && a.sha256.is_some()
                })
            {
                return Err(
                    "Choice record must bind exact retained installer hash and choice evidence"
                        .into(),
                );
            }
        }
        // Local paths are private adapter implementation details, never persisted.
        for a in &mut capture.assets {
            a.path = None;
        }
        capture.assets.clear();
        let complete = capture.discovery_finished && assets.iter().all(|a| a.sha256.is_some());
        let manifest = Manifest {
            schema: 1,
            capture,
            assets,
            inventory_complete: complete,
            retained_bytes: total,
        };
        if cancelled() {
            return Err("Capture cancelled".into());
        }
        tx.execute(
            "INSERT INTO captures VALUES (?,?)",
            params![manifest.capture.id, serde_json::to_string(&manifest)?],
        )?;
        tx.commit()?;
        Ok(manifest)
    }
    pub fn manifest(&self, id: &str) -> Result<Manifest> {
        let json: String =
            self.db
                .query_row("SELECT manifest FROM captures WHERE id=?", [id], |r| {
                    r.get(0)
                })?;
        Ok(serde_json::from_str(&json)?)
    }
    /// Writes bytes, never executes or interprets them. Caller must use a temporary output
    /// and publish it only after success because integrity failure can follow partial writes.
    pub fn copy_asset(&self, capture: &str, asset: &str, mut output: impl Write) -> Result<u64> {
        let m = self.manifest(capture)?;
        let a = m
            .assets
            .iter()
            .find(|a| a.id == asset)
            .ok_or("Unknown asset")?;
        let expected = a.sha256.as_ref().ok_or("Asset was not captured")?;
        let mut full = Sha256::new();
        let mut count = 0;
        for h in &a.chunks {
            let b: Vec<u8> = self.db.query_row(
                "SELECT bytes FROM chunks WHERE hash=? AND length(bytes)<=1048576",
                [h],
                |r| r.get(0),
            )?;
            if b.len() > CHUNK || hash(&b) != *h {
                return Err("Corrupt archive chunk".into());
            }
            full.update(&b);
            output.write_all(&b)?;
            count += b.len() as u64;
        }
        if count != a.bytes || format!("{:x}", full.finalize()) != *expected {
            return Err("Archive asset integrity mismatch".into());
        }
        Ok(count)
    }
    pub fn unique_bytes(&self) -> Result<u64> {
        Ok(self
            .db
            .query_row::<i64, _, _>(
                "SELECT coalesce(sum(length(bytes)),0) FROM chunks",
                [],
                |r| r.get(0),
            )?
            .try_into()?)
    }
    /// Retention review only: this library never deletes snapshots or source files.
    pub fn retention_candidates(&self, now: i64) -> Result<Vec<String>> {
        let mut s = self
            .db
            .prepare("SELECT manifest FROM captures ORDER BY id")?;
        let mut out = vec![];
        for r in s.query_map([], |r| r.get::<_, String>(0))? {
            let m: Manifest = serde_json::from_str(&r?)?;
            if !m.capture.pinned && m.capture.retain_until_unix.is_some_and(|t| t <= now) {
                out.push(m.capture.id)
            }
        }
        Ok(out)
    }
}
