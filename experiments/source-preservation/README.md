# Source preservation core (experimental)

Shared Rust **private local vault** for MO2R, ModHunter, Athena and other mod managers. This is a working archive intake, storage, verification, restore and safe media-preview slice. It is **not yet the complete premium feature**, a browser crawler or an integrated native manager screen.

## Goal and acceptance backlog

Preserve the source page and all authorized media (including playable videos), description, images, source metadata, original installer archive/FOMOD and observed installation choices. Keep accessible after a source disappears. Preserve original mod/download files read-only. Never report complete when something was not discovered, fetched or retained. Source unavailability, deletion, discontinued support and creator inactivity are separate evidence states.

- [x] Immutable provenance-bearing captures; prior captures survive failures.
- [x] Transactional fixed-size chunk ingestion, SHA256 integrity, cross-version deduplication, local byte quotas, cancellation callback; no network or execution.
- [x] Arbitrary original assets, HTML/WACZ and installers retained byte-for-byte without extraction or execution. Choice logs bind to the exact original installer SHA256, tool/version/time/evidence method, not only a filename/version label.
- [x] Explicit resource gaps and discovery scope. No inaccessible resource disappears from the manifest. `inventory_complete` means only the adapter's declared inventory is finished/retained, not proof of full website fidelity.
- [x] Black offline gallery with local image/video playback controls and lazy/preload-metadata loading. Original HTML/scripts never execute in this viewer. Unsupported types remain preserved but are not rendered.
- [x] Pinning and expiry review; **no automatic eviction**. Full vault refuses a new capture, rather than destroying an old one. Quotas are caller-configurable, never silent sampling.
- [ ] Authorized browser/page capture adapters for MO2R/ModHunter/Noxviola, including full lazy-loading discovery, pagination, CSS/background resources and all frames.
- [ ] Media acquisition adapters for direct files and permitted HLS/DASH with every segment, track, caption and poster accounted for. DRM/access denied remains an explicit gap, not a promise of recovery.
- [ ] WACZ capture/replay integration and fidelity testing. Safe gallery is not full visual page replay.
- [ ] Installer/FOMOD choice-event recording in the real native installer and version-specific choice schema; original archive is retained now, but this does not reconstruct choices that were never observed. Reconfigure through the supported native installer after explicit user choice; never silently replay selections against a changed archive or install over user edits.
- [ ] Native archive tab, version timeline, image carousels, video player, source/lifecycle evidence links, storage controls, active progress/cancel/status and background job scheduling without UI/MO2 stalls.
- [ ] Multi-process/adversarial-input hardening, storage benchmarks, Windows native acceptance, accessibility and real denied/deleted-source replay tests.

## Local use

```sh
cargo test --locked
cargo run --locked -- init private-vault.sqlite
cargo run --locked -- ingest private-vault.sqlite capture.json 10737418240 53687091200 536870912000
cargo run --locked -- manifest private-vault.sqlite capture-1
cargo run --locked -- preview private-vault.sqlite capture-1 new-preview-directory
cargo run --locked -- extract private-vault.sqlite capture-1 installer new-restored.zip
cargo run --locked -- retention-review private-vault.sqlite 1900000000
```

All byte quotas are explicit arguments: per asset, per capture, unique stored content. Media is streamed in 1 MiB chunks, including assets above SQLite's single-BLOB limit. Archive creation, extraction and preview refuse to overwrite existing targets. Input JSON shape is illustrated in `examples/capture.json` (paths must refer to actual authorized local captures). Missing assets use `path:null` plus a nonempty `missing_reason`. Existing files use a path and `missing_reason:null`.

New Unix vault files are created mode 0600 and preview staging/output directories mode 0700; restored files are mode 0600. Windows ACL enforcement is still a host integration requirement. The vault holds private bytes, not a public metadata dataset. Keep it **outside repositories and mod folders** in an OS-private directory; secure filesystem ownership, disk encryption and safe backups are the host application's responsibility. This crate is not an encryption product. Never commit snapshots, account content, cookies, headers, signed transport URLs or proprietary mod media. Source URL fields accept canonical queryless HTTP(S) references; adapters may use ephemeral authorized transport URLs without saving them. Authorization, attribution, retention and redistribution are separate questions; `private_only` is mandatory. The library cannot inspect opaque files to remove secrets: sanitize capture headers, sensitive URLs and unrelated account contents before intake.

Original HTML/WACZ is intentionally opaque; parsing and safe replay are separate adapter work. The generated preview uses local allowlisted media types with signature checks, escaped strings and a restrictive CSP, never source scripts/forms. Media decoders still belong in an up-to-date sandboxed webview. Generated previews are private and duplicate media bytes for portability; do not use preview export as the scalable library UI. A native asset protocol should stream verified vault chunks instead.

`copy_asset` validates every chunk and final SHA/length but may write partial output before detecting later corruption; the CLI stages privately and atomically publishes a verified output using a no-clobber hard link. Filesystems without hard links fail without publishing the final file. Other consumers must stage and commit only successful outputs. Preview generation also stages privately and only renames into place on success; trusted parent directories/no hostile concurrent destination creation are required. Use a trusted private vault directory: this first slice does not defend against an attacker racing filesystem symlinks or mutating the SQLite database while it is in use. Inputs reject existing symlinks/devices and changed size/mtime; a cooperating capture adapter must finish downloads before intake. A concurrent same-size/same-time rewrite is outside this check, so SHA identifies bytes actually observed, not an original publisher guarantee.

CLI signal cancellation, resumable jobs and large/untrusted metadata limits remain pending; the cancellation callback is available to Rust library consumers. Logical content quotas exclude SQLite page/index overhead, metadata, transaction journals and exported previews. Provision disk headroom; disk-full errors roll back the transaction. `retention-review` only lists expired, unpinned captures for review; it does not delete anything. No general garbage collector or compactor exists in this slice. Reopening a corrupt or untrusted database is not a supported import route. Manifest data is not a lifecycle verdict; store observation IDs from the independent creator/project status contract.

Dependencies are pinned to versions available and tested in the execution environment, not claimed to be the newest or fastest. Root Modding-Knowledge has its own versioned metadata API. This independent experiment has no mandatory dependency on Athena or any game mode and does not change the public dataset schema.

## Actual code inspected and implementation choices

- MO2 pinned upstream `4ba22f0d8b6f9e08e44008b8395cd774551b4ebe`, [`src/modinfodialognexus.cpp`](https://github.com/ModOrganizer2/modorganizer/blob/4ba22f0d8b6f9e08e44008b8395cd774551b4ebe/src/modinfodialognexus.cpp): `NexusTab::onModChanged` reads `getNexusDescription`, converts BBCode to HTML and calls QWebEngine `setHtml`. `NexusTabWebpage` forwards clicked links externally. This tab path does not implement a complete archival asset manifest or verified offline video retention. No claim that every MO2 codepath was audited.
- User screenshot was inspected at pixel level: Nexus Info tab, description, large preview image, Refresh/Open in Browser/Endorse/Untracked controls, custom URL field. The screenshot alone cannot establish whether the image survives offline.
- Existing MO2R `experiments/rust-release-core/src/lib.rs` preserves raw modlist bytes; ModHunter `native/crates/modhunter-core/src/importer.rs` has source/custom URL resolution and Watchtower owns source metadata. Do not add crawling or decoding to those import hot paths; enqueue a separate preservation job after authoritative source identity is resolved. Keep the current native release work untouched.
- Existing Modding-Knowledge `src/lib.rs` policy-aware public observations and `docs/database-manager.md` are public catalog infrastructure. Private archival payloads must not be mixed into those published SQLite artifacts.
- [SingleFile known issues](https://github.com/gildas-lormeau/SingleFile/blob/main/known-issues.md) and [FAQ](https://github.com/gildas-lormeau/SingleFile/blob/main/faq.md) make page saving useful but not proof of all playable media. Do not call a poster a saved video.
- [Browsertrix archived items](https://docs.browsertrix.com/user-guide/archived-items/) and [concepts](https://docs.browsertrix.com/user-guide/concepts/) provide an established WACZ/replay route to evaluate before inventing a full crawler. This core retains WACZ bytes but has not implemented/interoperability-tested replay.

## Verification

Fixture regression tests cover streamed multi-chunk bytes/reopen equality, source preservation, dedup, quota rollback, cancellation rollback, explicit missing media, discovery uncertainty, canonical URL validation, immutable capture IDs, retention pins, original installer hash-bound choice evidence, corruption detection and overwrite refusal. CLI offline-preview/source-deletion fixtures are additional checks, not real browser/native acceptance. No download or installation of a user's mod is performed by these tests.

Local validation: 14 Rust regression tests passed; synthetic CLI smoke passed original FOMOD archive byte round-trip, preserved playable MP4 decoding after all source input files were removed, no-clobber restore, private Unix modes and inert source-script handling. FFmpeg decoded the saved video, but no actual browser/native visual playback test was run. `cargo clippy` could not run because the toolchain component is absent.
