//! Explicit single-artifact sampled INI monitoring. Can miss intermediate edits.
use crate::{
    edit_analysis::{self, AnalyzedEdit},
    history::{Attribution, History, Observation},
    ini,
};
use std::{fs::OpenOptions, io::Read, path::Path};
pub enum Sample {
    Baseline,
    Unchanged,
    RebasedAfterGap,
    Changed(Box<AnalyzedEdit>),
}
pub struct IniMonitor {
    history: History,
    previous: Option<Vec<u8>>,
}
impl IniMonitor {
    pub fn new(artifact: String, capacity: usize) -> Result<Self, String> {
        let mut history = History::new(artifact, capacity)?;
        history.set_enabled(true);
        Ok(Self {
            history,
            previous: None,
        })
    }
    pub fn history(&self) -> &History {
        &self.history
    }
    pub fn mark_gap(&mut self) {
        self.history.mark_gap();
        self.previous = None;
    }
    pub fn sample(&mut self, bytes: &[u8], time_ms: u64) -> Result<Sample, String> {
        if let Err(e) = ini::parse(bytes, || false) {
            self.mark_gap();
            return Err(e);
        }
        let observation = match self
            .history
            .observe(bytes, time_ms, Attribution::Unknown, None)
        {
            Ok(x) => x,
            Err(e) => {
                self.mark_gap();
                return Err(e.into());
            }
        };
        let result = match observation {
            Observation::Baseline => Sample::Baseline,
            Observation::Unchanged => Sample::Unchanged,
            Observation::RebasedAfterGap => Sample::RebasedAfterGap,
            Observation::Changed => {
                let old = self
                    .previous
                    .as_ref()
                    .ok_or("previous snapshot unavailable")?;
                let edit = self
                    .history
                    .query(None, 0, u64::MAX)
                    .last()
                    .ok_or("edit receipt unavailable")?;
                match edit_analysis::ini_edit(edit, old, bytes, || false) {
                    Ok(r) => Sample::Changed(Box::new(r)),
                    Err(e) => {
                        self.mark_gap();
                        return Err(e);
                    }
                }
            }
        };
        self.previous = Some(bytes.to_vec());
        Ok(result)
    }
    pub fn poll(&mut self, path: &Path, time_ms: u64) -> Result<Sample, String> {
        match read_snapshot(path) {
            Ok(b) => self.sample(&b, time_ms),
            Err(e) => {
                self.mark_gap();
                Err(e)
            }
        }
    }
}
/// Defensive double read of the same opened regular file. Detects observed
/// instability, not a transactional filesystem snapshot or all concurrent writes.
/// Private trusted parent-directory required; no original-file modification.
pub fn read_snapshot(path: &Path) -> Result<Vec<u8>, String> {
    let entry = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !entry.file_type().is_file() {
        return Err("settings source must be regular non-symlink file".into());
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
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    let before = file.metadata().map_err(|e| e.to_string())?;
    if !before.is_file() || before.file_type().is_symlink() || before.len() > 4 * 1024 * 1024 {
        return Err("invalid settings source size/type".into());
    }
    let mut first = Vec::new();
    file.by_ref()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut first)
        .map_err(|e| e.to_string())?;
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut second = Vec::new();
    file.by_ref()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut second)
        .map_err(|e| e.to_string())?;
    let after = file.metadata().map_err(|e| e.to_string())?;
    if first != second
        || first.len() as u64 != before.len()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err("settings source unstable during sample".into());
    }
    Ok(first)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changes_are_unknown_writer_and_private() {
        let mut m = IniMonitor::new("opaque".into(), 5).unwrap();
        assert!(matches!(
            m.sample(b"x=secretA", 1).unwrap(),
            Sample::Baseline
        ));
        let Sample::Changed(r) = m.sample(b"x=secretB", 2).unwrap() else {
            panic!()
        };
        assert_eq!(r.edit.attribution, Attribution::Unknown);
        assert!(!format!("{r:?}").contains("secret"));
    }
    #[test]
    fn malformed_sample_creates_gap_not_stale_delta() {
        let mut m = IniMonitor::new("opaque".into(), 5).unwrap();
        m.sample(b"x=1", 1).unwrap();
        assert!(m.sample(b"broken", 2).is_err());
        assert!(matches!(
            m.sample(b"x=3", 3).unwrap(),
            Sample::RebasedAfterGap
        ));
        assert_eq!(m.history().query(None, 0, 9).count(), 0);
        assert!(!m.history().continuity_preserved());
    }
    #[test]
    fn sampled_reversion_does_not_prove_no_edit() {
        let mut m = IniMonitor::new("opaque".into(), 5).unwrap();
        m.sample(b"x=1", 1).unwrap();
        assert!(matches!(m.sample(b"x=1", 2).unwrap(), Sample::Unchanged)); /* unchanged refers to sampled bytes only */
    }
    #[test]
    fn actual_file_edit_detected_and_original_preserved() {
        let p = std::env::temp_dir().join(format!("ini-poll-smoke-{}", std::process::id()));
        std::fs::write(&p, b"[S]\nx=1").unwrap();
        let mut m = IniMonitor::new("opaque".into(), 5).unwrap();
        m.poll(&p, 1).unwrap();
        std::fs::write(&p, b"[S]\nx=2").unwrap();
        assert!(matches!(m.poll(&p, 2).unwrap(), Sample::Changed(_)));
        assert_eq!(std::fs::read(&p).unwrap(), b"[S]\nx=2");
        std::fs::remove_file(p).unwrap();
    }
    #[test]
    fn missing_file_marks_gap() {
        let p = std::env::temp_dir().join(format!("absent-ini-poll-{}", std::process::id()));
        let mut m = IniMonitor::new("opaque".into(), 5).unwrap();
        m.sample(b"x=1", 1).unwrap();
        assert!(m.poll(&p, 2).is_err());
        assert!(!m.history().continuity_preserved());
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn capacity_failure_preserves_receipt_and_rebases() {
        let mut m = IniMonitor::new("opaque".into(), 1).unwrap();
        m.sample(b"x=1", 1).unwrap();
        m.sample(b"x=2", 2).unwrap();
        assert!(m.sample(b"x=3", 3).is_err());
        assert_eq!(m.history().query(None, 0, 9).count(), 1);
        assert!(!m.history().continuity_preserved());
        assert!(matches!(
            m.sample(b"x=4", 4).unwrap(),
            Sample::RebasedAfterGap
        ));
    }
    #[test]
    fn backwards_clock_is_gap_not_fabricated_timestamp() {
        let mut m = IniMonitor::new("opaque".into(), 4).unwrap();
        m.sample(b"x=1", 5).unwrap();
        assert!(m.sample(b"x=2", 4).is_err());
        assert!(matches!(
            m.sample(b"x=3", 6).unwrap(),
            Sample::RebasedAfterGap
        ));
        assert_eq!(m.history().query(None, 0, 9).count(), 0);
    }
}
