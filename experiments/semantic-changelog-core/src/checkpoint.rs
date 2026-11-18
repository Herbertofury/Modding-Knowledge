//! Explicit local metadata checkpoints, never original artifact bytes.
//! Caller must choose a private non-synced directory and sanitize identifiers.
use crate::history::{Edit, History, Revision};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};
const LIMIT: usize = 64 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    history: StoredHistory,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredHistory {
    artifact: String,
    current: Option<Revision>,
    edits: Vec<Edit>,
    capacity: usize,
    enabled: bool,
    gap: bool,
    continuity_lost: bool,
}
impl StoredHistory {
    fn into_disabled(self) -> History {
        History {
            artifact: self.artifact,
            current: self.current,
            edits: self.edits,
            capacity: self.capacity,
            enabled: false,
            gap: true,
            continuity_lost: true,
        }
    }
}
#[derive(Serialize)]
struct Borrowed<'a> {
    version: u32,
    history: &'a History,
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
}
fn validate(h: &History) -> Result<(), String> {
    if !label(&h.artifact) || !(1..=100_000).contains(&h.capacity) || h.edits.len() > h.capacity {
        return Err("invalid history bounds".into());
    }
    if !h.edits.is_empty() && h.current.is_none() {
        return Err("missing current revision".into());
    }
    let mut last_time = 0;
    for e in &h.edits {
        if e.before.observed_at_ms > e.after.observed_at_ms
            || e.before.observed_at_ms < last_time
            || e.before.sha256 == e.after.sha256
            || e.before.length > LIMIT as u64
            || e.after.length > LIMIT as u64
            || !e.attribution.valid()
            || e.workflow.as_ref().is_some_and(|w| !label(w))
        {
            return Err("invalid edit evidence".into());
        }
        last_time = e.after.observed_at_ms;
    }
    if h.current
        .as_ref()
        .is_some_and(|r| r.length > LIMIT as u64 || r.observed_at_ms < last_time)
    {
        return Err("invalid current revision".into());
    }
    Ok(())
}
/// New checkpoint only: existing file/symlink is never replaced. A failed write
/// can leave an incomplete new file; caller must treat errors as failure, never
/// retry by truncating. No automatic cleanup or overwrite occurs.
/// Unix creation requests0600. Windows inherits destination ACLs: host must
/// establish private storage; this API does not claim encryption or ACL setup.
pub fn save_new(path: &Path, history: &History) -> Result<(), String> {
    validate(history)?;
    let data = serde_json::to_vec(&Borrowed {
        version: 1,
        history,
    })
    .map_err(|e| e.to_string())?;
    if data.len() > LIMIT {
        return Err("checkpoint exceeds bound".into());
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    file.write_all(&data)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
/// Reload always disables observation and marks continuity incomplete: downtime
/// can hide edits even if a later rescan finds the same bytes. Metadata integrity
/// is validated, not authenticated; local tampering cannot prove writer identity.
pub fn load(path: &Path) -> Result<History, String> {
    let entry = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !entry.file_type().is_file() {
        return Err("checkpoint must be a regular non-symlink file".into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let file = options.open(path).map_err(|e| e.to_string())?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > LIMIT as u64 {
        return Err("invalid checkpoint file or size".into());
    }
    let mut data = Vec::new();
    file.take((LIMIT + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > LIMIT {
        return Err("checkpoint exceeds bound".into());
    }
    let envelope: Envelope = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    if envelope.version != 1 {
        return Err("unsupported checkpoint version".into());
    }
    let history = envelope.history.into_disabled();
    validate(&history)?;
    Ok(history)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{Attribution, Observation};
    fn path() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "history-checkpoint-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ))
    }
    fn history() -> History {
        let mut h = History::new("opaque-1".into(), 5).unwrap();
        h.set_enabled(true);
        h.observe(b"a", 1, Attribution::Unknown, None).unwrap();
        h.observe(
            b"b",
            2,
            Attribution::UserReported {
                tool: "xEdit".into(),
            },
            Some("workflow-1".into()),
        )
        .unwrap();
        h
    }
    #[test]
    fn roundtrip_is_disabled_and_gapped() {
        let p = path();
        save_new(&p, &history()).unwrap();
        let mut h = load(&p).unwrap();
        assert_eq!(h.artifact(), "opaque-1");
        assert_eq!(h.query(Some("xEdit"), 0, 9).count(), 1);
        assert!(!h.continuity_preserved());
        assert!(h.observe(b"b", 3, Attribution::Unknown, None).is_err());
        h.set_enabled(true);
        assert_eq!(
            h.observe(b"b", 3, Attribution::Unknown, None),
            Ok(Observation::RebasedAfterGap)
        );
        std::fs::remove_file(p).unwrap();
    }
    #[test]
    fn existing_file_not_overwritten() {
        let p = path();
        std::fs::write(&p, b"keep").unwrap();
        assert!(save_new(&p, &history()).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"keep");
        std::fs::remove_file(p).unwrap();
    }
    #[test]
    fn truncated_and_future_versions_refused() {
        let p = path();
        std::fs::write(&p, b"{").unwrap();
        assert!(load(&p).is_err());
        let mut v = serde_json::to_value(Borrowed {
            version: 2,
            history: &history(),
        })
        .unwrap();
        std::fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(load(&p).is_err());
        v["version"] = 1.into();
        v["history"]["capacity"] = 0.into();
        std::fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(load(&p).is_err());
        std::fs::remove_file(p).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn new_file_private_and_symlink_refused() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let p = path();
        let link = path();
        save_new(&p, &history()).unwrap();
        assert_eq!(
            std::fs::metadata(&p).unwrap().permissions().mode() & 0o077,
            0
        );
        symlink(&p, &link).unwrap();
        assert!(load(&link).is_err());
        assert!(save_new(&link, &history()).is_err());
        std::fs::remove_file(link).unwrap();
        std::fs::remove_file(p).unwrap();
    }
}

#[cfg(all(test, unix))]
mod special_file_tests {
    use super::*;
    #[test]
    fn fifo_is_refused_without_opening_or_waiting() {
        let p = std::env::temp_dir().join(format!("history-fifo-{}", std::process::id()));
        let c = std::ffi::CString::new(p.as_os_str().as_encoded_bytes()).unwrap();
        // SAFETY: owned, NUL-terminated path remains alive for syscall duration.
        assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
        assert!(load(&p).is_err());
        std::fs::remove_file(p).unwrap();
    }
}
