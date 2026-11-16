# Rust database management app

The native `knowledge-manager` executable is the first working management slice,
not a finished graphical app. It shares the existing Rust store, without an
Athena dependency. It browses the actual SQLite catalog, reports storage and
policy visibility, finds review-only duplicate candidates, and checks integrity.
It performs no network requests or writes, never creates a missing file, and
rejects unsupported schema versions.

```sh
cargo build --locked --release --bin knowledge-manager
# Decompress an existing reviewed snapshot first, or use the existing reviewed seed.
cargo run --locked --example seed_reviewed_bundle -- catalog.sqlite
cargo run --locked --bin knowledge-manager -- catalog.sqlite status
cargo run --locked --bin knowledge-manager -- catalog.sqlite browse Sodium
cargo run --locked --bin knowledge-manager -- catalog.sqlite browse '' '' 25
cargo run --locked --bin knowledge-manager -- catalog.sqlite duplicates
cargo run --locked --bin knowledge-manager -- catalog.sqlite check
```

Every successful command emits JSON. `check` returns nonzero for structural or
foreign-key failures. All commands return nonzero on invalid arguments, an
unreadable file or a query failure. Browse is bounded to 1–200 entities per page;
pass its `next_cursor` as `AFTER_ID` with the same query. A null cursor ends the
scan. Search is a literal ASCII-case-insensitive substring of ID or name, not
SQL wildcard syntax. Cursor ordering is deterministic on an unchanged snapshot;
concurrent catalog changes are not a cross-call snapshot guarantee.

Browse and duplicate hints recheck source publication policy. Same-name groups
are restricted to the same game and kind, capped at 100 with an explicit
truncation flag, and **never automatically merged**. Name matches cannot prove
identity. Open the existing observation query to inspect exact provenance:
`cargo run --locked --example query -- catalog.sqlite ENTITY_ID`.

Status includes withheld row counts but not their content. Allocated/reclaimable
page-byte metrics are not compressed file size, WAL size, measured query speed
or guaranteed compaction savings. Structural integrity does not validate factual
claims or game compatibility. Raw databases may contain withdrawn observations;
these commands do not authorize distributing those files.

## Backlog and acceptance gates

- [x] Rust native executable: real catalog browsing, bounded pagination,
  policy-aware duplicate hints, local storage metrics, integrity and JSON output.
- [ ] Graphical power-user management app over these same APIs, keyboard-first
  browsing/filtering, detailed source inspection and accessible controls.
- [ ] Import preview and transactional apply through reviewed source policy;
  immutable evidence, idempotency, clear row conflicts and recoverable rollback.
- [ ] Reversible identity decisions with evidence and aliases, not name merging.
- [ ] Full-text/index strategy and compression benchmarks against representative
  massive catalogs; measure latency, disk, memory and correctness before adoption.
- [ ] Explicit safe export projection excluding withdrawn/private records.
- [ ] Evidence-led improvement suggestions with baselines, review and rollback;
  never silently rewrite claims, discard provenance or apply destructive tuning.
- [ ] Cross-project compatibility tests for Athena, Noxviola, MO2R and Mod Hunter.

These are unfinished gates, not claims of delivery. Shared update-advisory and
creator-move contracts remain independent workstreams. This app must consume
those contracts rather than fork their definitions.
