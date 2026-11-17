use sha2::{Digest, Sha256};
use source_preservation::*;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    dir: PathBuf,
    v: Vault,
}
impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "preservation-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let v = Vault::create(dir.join("vault.sqlite")).unwrap();
        Self { dir, v }
    }
    fn input(&self, name: &str, data: &[u8]) -> PathBuf {
        let p = self.dir.join(name);
        fs::write(&p, data).unwrap();
        p
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}
fn capture(path: PathBuf) -> Capture {
    Capture {
        id: "first".into(),
        project_id: "fixture:mod:1".into(),
        version: "1".into(),
        page_url: "https://example.org/mod/1".into(),
        captured_unix: 100,
        adapter: "fixture/1".into(),
        policy_reference: "user-authorized-personal-retention".into(),
        authorized_to_retain: true,
        private_only: true,
        discovery_finished: true,
        discovery_scope: "fixture supplied inventory".into(),
        pinned: true,
        retain_until_unix: Some(200),
        lifecycle_observation_ids: vec![],
        installer_choices: None,
        assets: vec![InputAsset {
            id: "page".into(),
            source_url: "https://example.org/mod/1".into(),
            kind: Kind::Page,
            media_type: "text/html".into(),
            path: Some(path),
            missing_reason: None,
        }],
    }
}
fn limits() -> Limits {
    Limits {
        max_asset_bytes: 8 * 1024 * 1024,
        max_capture_bytes: 16 * 1024 * 1024,
        max_unique_bytes: 16 * 1024 * 1024,
    }
}
#[test]
fn multi_chunk_roundtrip_reopen_and_dedup() {
    let mut f = Fixture::new();
    let data = vec![19; 3 * 1024 * 1024 + 67];
    let p = f.input("video", &data);
    let mut c = capture(p.clone());
    c.assets[0].kind = Kind::Video;
    let m = f.v.ingest(c.clone(), limits(), || false).unwrap();
    assert!(m.inventory_complete);
    assert!(m.capture.assets.is_empty());
    assert_eq!(m.assets[0].chunks.len(), 4);
    let u = f.v.unique_bytes().unwrap();
    assert_eq!(u, 1024 * 1024 + 67);
    c.id = "second".into();
    f.v.ingest(c, limits(), || false).unwrap();
    assert_eq!(f.v.unique_bytes().unwrap(), u);
    let v = Vault::open(f.dir.join("vault.sqlite")).unwrap();
    let mut out = vec![];
    v.copy_asset("second", "page", &mut out).unwrap();
    assert_eq!(out, data);
    assert_eq!(fs::read(p).unwrap(), data);
}
#[test]
fn quota_rollback_preserves_previous() {
    let mut f = Fixture::new();
    let c = capture(f.input("one", b"one"));
    f.v.ingest(c, limits(), || false).unwrap();
    let mut c = capture(f.input("two", b"other"));
    c.id = "two".into();
    let l = Limits {
        max_unique_bytes: 4,
        ..limits()
    };
    assert!(f.v.ingest(c, l, || false).is_err());
    assert_eq!(f.v.unique_bytes().unwrap(), 3);
    assert!(f.v.manifest("first").is_ok());
    assert!(f.v.manifest("two").is_err());
}
#[test]
fn missing_media_not_complete_and_not_deletion() {
    let mut f = Fixture::new();
    let mut c = capture(f.input("page", b"page"));
    c.assets.push(InputAsset {
        id: "video".into(),
        source_url: "https://example.org/video".into(),
        kind: Kind::Video,
        media_type: "video/mp4".into(),
        path: None,
        missing_reason: Some("403 access denied; lifecycle unknown".into()),
    });
    let m = f.v.ingest(c, limits(), || false).unwrap();
    assert!(!m.inventory_complete);
    assert!(m.capture.lifecycle_observation_ids.is_empty());
    assert!(f.v.copy_asset("first", "video", vec![]).is_err());
}
#[test]
fn cancelled_transaction_removes_new_chunks() {
    let mut f = Fixture::new();
    let c = capture(f.input("big", &vec![7; 2 * 1024 * 1024]));
    let checks = AtomicU64::new(0);
    assert!(
        f.v.ingest(c, limits(), || checks.fetch_add(1, Ordering::Relaxed) >= 2)
            .is_err()
    );
    assert_eq!(f.v.unique_bytes().unwrap(), 0);
    assert!(f.v.manifest("first").is_err());
}
#[test]
fn immutable_and_signed_urls_rejected() {
    let mut f = Fixture::new();
    let c = capture(f.input("page", b"hello"));
    f.v.ingest(c.clone(), limits(), || false).unwrap();
    assert!(f.v.ingest(c.clone(), limits(), || false).is_err());
    let mut c = c;
    c.id = "other".into();
    c.page_url.push_str("?token=secret");
    assert!(f.v.ingest(c, limits(), || false).is_err());
    assert_eq!(f.v.unique_bytes().unwrap(), 5);
}
#[test]
fn authority_and_retention() {
    let mut f = Fixture::new();
    let mut c = capture(f.input("page", b"hello"));
    c.authorized_to_retain = false;
    assert!(f.v.ingest(c.clone(), limits(), || false).is_err());
    c.authorized_to_retain = true;
    f.v.ingest(c.clone(), limits(), || false).unwrap();
    assert!(f.v.retention_candidates(300).unwrap().is_empty());
    c.id = "unpinned".into();
    c.pinned = false;
    f.v.ingest(c, limits(), || false).unwrap();
    assert_eq!(f.v.retention_candidates(300).unwrap(), vec!["unpinned"]);
    assert!(f.v.manifest("unpinned").is_ok());
}
#[test]
fn choices_bind_original_installer_and_log() {
    let mut f = Fixture::new();
    let data = b"original archive bytes incl FOMOD";
    let mut c = capture(f.input("installer.zip", data));
    c.assets[0].kind = Kind::Installer;
    c.assets.push(InputAsset {
        id: "choices".into(),
        source_url: c.page_url.clone(),
        kind: Kind::Other,
        media_type: "application/json".into(),
        path: Some(f.input("choices", br#"{"skin":"A"}"#)),
        missing_reason: None,
    });
    c.installer_choices = Some(InstallerChoices {
        installer_asset_id: "page".into(),
        installer_sha256: format!("{:x}", Sha256::digest(data)),
        tool: "MO2".into(),
        tool_version: "fixture".into(),
        recorded_unix: 110,
        choice_log_asset_id: "choices".into(),
        evidence_method: "installer-observed option IDs".into(),
    });
    f.v.ingest(c.clone(), limits(), || false).unwrap();
    c.id = "bad".into();
    c.installer_choices.as_mut().unwrap().installer_sha256 = "wrong".into();
    assert!(f.v.ingest(c, limits(), || false).is_err());
    let mut restored = vec![];
    f.v.copy_asset("first", "page", &mut restored).unwrap();
    assert_eq!(restored, data);
}
#[test]
fn corrupt_blob_refuses_export() {
    let mut f = Fixture::new();
    let c = capture(f.input("page", b"original"));
    f.v.ingest(c, limits(), || false).unwrap();
    let db = rusqlite::Connection::open(f.dir.join("vault.sqlite")).unwrap();
    db.execute("UPDATE chunks SET bytes=x'626164'", []).unwrap();
    assert!(f.v.copy_asset("first", "page", vec![]).is_err());
}
#[test]
fn init_never_overwrites_existing() {
    let f = Fixture::new();
    let p = f.input("important", b"precious");
    assert!(Vault::create(&p).is_err());
    assert_eq!(fs::read(p).unwrap(), b"precious");
    assert!(Vault::open(f.dir.join("absent")).is_err());
}
#[test]
fn unknown_discovery_stays_partial() {
    let mut f = Fixture::new();
    let mut c = capture(f.input("page", b"page"));
    c.discovery_finished = false;
    assert!(
        !f.v.ingest(c, limits(), || false)
            .unwrap()
            .inventory_complete
    );
}
#[cfg(unix)]
#[test]
fn symlink_input_rejected() {
    let mut f = Fixture::new();
    let p = f.input("page", b"page");
    let l = f.dir.join("link");
    std::os::unix::fs::symlink(p, &l).unwrap();
    assert!(f.v.ingest(capture(l), limits(), || false).is_err());
}
#[cfg(unix)]
#[test]
fn private_unix_vault_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    assert_eq!(
        fs::metadata(f.dir.join("vault.sqlite"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}
#[test]
fn installer_cannot_be_its_own_choice_evidence() {
    let mut f = Fixture::new();
    let data = b"archive";
    let mut c = capture(f.input("original", data));
    c.assets[0].kind = Kind::Installer;
    c.installer_choices = Some(InstallerChoices {
        installer_asset_id: "page".into(),
        installer_sha256: format!("{:x}", Sha256::digest(data)),
        tool: "MO2".into(),
        tool_version: "fixture".into(),
        recorded_unix: 100,
        choice_log_asset_id: "page".into(),
        evidence_method: "fake choice log".into(),
    });
    assert!(f.v.ingest(c, limits(), || false).is_err());
    assert_eq!(f.v.unique_bytes().unwrap(), 0);
}
#[test]
fn corrupt_preview_leaves_no_final_or_staging() {
    let mut f = Fixture::new();
    let mut bytes = vec![7; 1024 * 1024 + 5];
    bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut c = capture(f.input("image", &bytes));
    c.assets[0].kind = Kind::Image;
    c.assets[0].media_type = "image/png".into();
    let m = f.v.ingest(c, limits(), || false).unwrap();
    let db = rusqlite::Connection::open(f.dir.join("vault.sqlite")).unwrap();
    db.execute(
        "UPDATE chunks SET bytes=x'00' WHERE hash=?",
        [&m.assets[0].chunks[1]],
    )
    .unwrap();
    let out = f.dir.join("preview");
    let r = std::process::Command::new(env!("CARGO_BIN_EXE_source-preservation"))
        .args([
            "preview",
            f.dir.join("vault.sqlite").to_str().unwrap(),
            "first",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(!out.exists());
    assert!(
        !fs::read_dir(&f.dir).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("pending"))
    );
}
