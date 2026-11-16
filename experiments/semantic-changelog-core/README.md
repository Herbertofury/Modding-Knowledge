# Read-only semantic changelog foundation

Experimental, independent Rust parser/comparison slice for MO2R, ModHunter,
Noxviola and shared Modding-Knowledge. Not yet integrated into any app or premium
entitlement. Not a replacement for author release notes, xEdit or Sims4 Studio.

## What actually works

- Skyrim SE plugin bytes: bounded TES4/GRUP/record/subrecord traversal, canonical
  master-resolved FormIDs, EDID, and WEAP DATA Value/Weight/Damage. An equal-length
  damage change is a numeric field delta. TES4 flags are compared, header and
  record metadata are retained as opaque evidence. Unknown payload differences
  are labeled opaque, never interpreted as a bug fix or compatibility change.
- Sims4 DBPF2/index3: resource TGI enumeration with shared type/group/high-instance
  index fields, bounded uncompressed and recognized zlib resource decoding,
  added/removed/payload-changed resource classification. Repacking or compression
  changes with identical decoded payload produce no resource delta. Unsupported
  compression yields NotCompared, even when stored bytes match.
- Immutable byte-slice APIs and a read-only CLI; CLI labels reports generated,
  prints SHA256 of both compared snapshots and never installs or edits inputs.
- Malformed lengths, duplicate identities, ambiguous masters, bounded expansion,
  parser-version mismatch and incomplete-side absence are handled explicitly.

## Run

```
cargo run --locked -- skyrim old.esp new.esp CanonicalName.esp
cargo run --locked -- sims4 old.package new.package
cargo test --locked
cargo test --locked --release
cargo clippy --locked --all-targets -- -D warnings
```

## Important limits

Skyrim comparison is a **narrow field projection**, always marked incomplete at
whole-plugin semantic scope. It does not interpret compressed record bodies,
localized strings, all record fields, group ownership/context, reference graphs,
Papyrus or effective load-order/runtime behavior. Uninterpreted header/body bytes
are not author intent. Tests use synthetic fixtures, not qualified game files.
HEDR shape is checked; no complete plugin validity or game compatibility claim.

DBPF comparison is **resource-level payload evidence**, not field-level CAS,
tuning, mesh, animation or texture interpretation. RefPack and nonrecognized
compression remain unknown. Certain nonstandard/extended index representations
are rejected. DBPF2 is not intrinsically a Sims4-only file identification, so
caller must supply the game context. Changes to compression/package layout are
not gameplay changes. `.ts4script` bytecode analysis is not implemented.

Inputs are limited to 64 MiB; decoded resource budget is 64 MiB/16 MiB each; 100k records,
100k total Bethesda subrecords and 32 group nesting. Oversized inputs refuse rather
than silently truncating. This is a configurable-budget design follow-up, not a
claim every real mod fits. API cancellation is cooperative; CLI uses normal OS
termination, not a product job UI. Binding to installed/downloaded release lineage,
source versions, parser cache, signed entitlement and UI remains application work.

## Research and reuse

- [TES5Edit WEAP schema](https://github.com/TES5Edit/TES5Edit/blob/dev-4.1.5/Core/wbDefinitionsTES5.pas): WEAP DATA binary fields verified from the primary implementation. No xEdit code vendored.
- [Sims4Tools DBPF specification](https://github.com/Kuree/Sims4Tools/wiki/DBPF-Format) and [compression](https://github.com/Kuree/Sims4Tools/wiki/Compression): index/compression format facts. Decoder sniffs recognized zlib framing; compression marker alone does not prove algorithm.
- [houseCARL v2.1.2 FieldsDiff](https://github.com/Avick3110/houseCARL/blob/v2.1.2/src/housecarl-core/FieldsDiff.cs) and RecordPoles provide a strong broad adapter candidate: deep content diff, list order, unavailable/absent/null, and winner/previous/named poles. Those are load-order comparisons, not automatically two release-file versions. GPL3/Mutagen license and two-version isolation require review before integration. No houseCARL code copied.
- Existing ModHunter plugin_analysis scans header/record facts, not full field diff.
  MO2R sims4.rs/sims4_install.rs handle archive/signature/routing, not DBPF resources.

## Next acceptance

Integrate exact release identities and hashes, parser/versioned evidence contracts,
full xEdit-level record schema adapters, localized strings and compressed Bethesda
records; resource-specific Sims4 schemas; structured serialization and rich visual
before/after UI. Never turn an unread/unsupported field into unchanged or claim a
runtime effect from a changed numeric value. Preserve author notes separately and
present missing-changelog comparison as generated evidence.

## Local edit provenance foundation (not a deployed monitor)

`history` accepts explicitly supplied stable-read artifact bytes only after opt-in.
It detects equal-size content changes with SHA256, ignores metadata-only touches,
retains before/after observation times and content hashes, and filters by detection
time/tool. `ObservedWrite`, `Inferred`, `UserReported` and `Unknown` remain distinct.
An observed-write claim requires a trusted host event binding process and artifact;
this library does not create or independently authenticate that event. A launched
xEdit/DynDOLOD/Sims4Studio process or a timestamp match is only correlation.

Pause/resume rebases explicitly rather than assigning unobserved edits to a tool.
Bounded history rejects overflow instead of silently discarding edits. No automatic path/process-argument collection, file-content retention, network
upload or background watcher occurs here. Host-supplied labels/evidence IDs can
contain private strings; the host must sanitize them or use opaque identifiers.
Persistent continuity status stays incomplete after pause, overflow or an explicit
host-reported gap and must accompany query/export results.
The core is in-memory: durable local history, scoped directory watch UI, stable
read/race protection, replacement/rename identity, snapshot store, consent and
retention controls remain integration work. A missing observation, restart or
watcher overflow must invalidate continuity. SHA hashes are not file backups.

Semantic reports must be computed from the exact bytes matching each revision,
with parser/version and existing partial/unknown limitations retained. The history module alone does not attach reports or prove tool identity.
`edit_analysis` now verifies both revision hashes and lengths against supplied
immutable bytes before returning the supported Skyrim-field/DBPF-resource report
alongside unchanged attribution. Wrong/reversed snapshots fail before parsing.
Hash verification does not authenticate host provenance or make partial decoding
complete. Artifact identity and known-gap status must still accompany UI/export separately;
`AnalyzedEdit` alone does not carry either. Sims4 swatch-specific decoding,
DynDOLOD generated-output ownership and complete xEdit schema coverage remain open.
MO2 MCP `run_executable` (inspected supplied Python source, lines1642–1676) reports
launch PID/exit status; it provides no per-file writer telemetry. Never promote
that response to directly observed edit attribution.

### Real watcher integration acceptance (still unimplemented)

- Windows: treat successful completed notifications with zero transferred bytes
  and `ERROR_NOTIFY_ENUM_DIR` as loss, call `mark_gap` on every affected history
  before reconciliation, then rescan the selected scope. For asynchronous calls,
  use completion byte counts, not the undefined immediate `lpBytesReturned`.
- Linux: `IN_Q_OVERFLOW` invalidates the whole watch instance's affected scope.
  Rebuild watches/reconcile directory state, including new nested directories;
  inotify is nonrecursive. Invalidated/unmounted watches and read failures need
  explicit gap handling. Neither event counts nor writer identity can be derived
  reliably from coalesced notifications; inotify supplies no writer PID.
- A rescan recovers current state, not lost intermediate changes. Even an identical
  hash after rescan cannot prove nothing changed. Keep known-gap status in every
  UI/export. `continuity_preserved` only means no known gap among supplied
  observations, never a guarantee of complete edit capture between snapshots.
- Scope opt-in, pause/resume, rename/replacement identities, race-safe stable reads,
  bounded recursive registration/queues, dropped-event recovery, cancellation and
  synthetic overflow plus real-platform tests are required before app integration.
  Notification-derived observations default to Unknown attribution. Process launch
  correlation is only inferred; direct writer proof requires separate evidence.

Primary contracts: [Microsoft ReadDirectoryChangesW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-readdirectorychangesw)
and [Linux inotify manual](https://man7.org/linux/man-pages/man7/inotify.7.html).
The recovery tests here exercise the ledger contract, not a working OS watcher.


`edit_analysis::skyrim_edit` and `sims4_edit` connect the ledger to real parsers
without retaining snapshots or writing files. Hashing checks cooperative
cancellation per64KiB; malformed matching bytes still fail parser validation.
Read-only byte binding is implemented; durable snapshot storage and user-facing
report attachment remain application work.

### Explicit local metadata checkpoint storage

`checkpoint::save_new(path, &history)` creates a new JSON checkpoint, syncs its
file contents, and refuses any existing destination. `checkpoint::load(path)`
checks bounded input, version, schema, capacities and timeline consistency, then
restores history **disabled with a persistent known gap**. A restart cannot prove
no changes occurred while the app was away. Original file contents and semantic
payloads are not stored, only hashes, observations and caller-supplied labels.

This is local checkpoint persistence, not an automatic watcher/database, encrypted
vault, crash-proof directory transaction or authenticated audit log. Caller must
select a private, non-synced directory not writable by untrusted processes and
sanitize IDs/labels. Unix new files request0600; Windows inherits directory ACLs,
which the application must establish before use. Final symlink/nonregular entries are refused; Unix open additionally uses
no-follow/nonblocking flags, and Windows opens the final reparse point rather
than following it. Ancestor-directory trust is still a host precondition. A write failure can
leave an incomplete new file. Never retry by truncating/replacing it: report the
failure and use an explicit new checkpoint path. No automatic deletion occurs.
Large histories fail rather than truncate; durable snapshot bytes, atomic catalog
rotation, retention/export UI, recovery indexing and real-app integration remain
open. Load validates structure, not origin or the truth of writer evidence.
