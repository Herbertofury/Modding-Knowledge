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

## Visual-first catalog slice

`knowledge-manager catalog.sqlite visual [QUERY [AFTER_ID [LIMIT]]] > catalog.html`
creates a self-contained interactive HTML catalog page from real public records.
Open that local file in a browser. Its compact table has image/gallery,
project/author, content and category columns, with live text filtering across the
page and explicit evidence-status labels. It is an export view, not yet a desktop
installation, live database editor or full visual-management application.

Media is source-linked and click-to-load: images and native playable video use
separate explicit source display permission. No media is downloaded or cached by
the Rust command. Unsupported codecs and unavailable media remain possible;
opening a source is not a guarantee its media can play. Retention permission is
shown separately and does not itself implement retention. Imported data remains
subject to the existing policy review and publication gates.

### Shared presentation observation contract

The existing immutable/provenanced observations store carries JSON values:

- `presentation.content`: `{"label":"ESPFE","evidence_status":"verified"}`
- `presentation.category`: `{"label":"Shoes","evidence_status":"provider_declared"}`
- `presentation.media`: `{"kind":"image","url":"https://example.org/art.png","alt":"Example only","evidence_status":"provider_declared","display_allowed":true,"retention_allowed":false}`

These are schema examples, **not real catalog claims**. Status is one of
`verified`, `provider_declared`, `inferred`. It must reflect retained source
proof; the renderer does not verify a claim by displaying its label. Source URL,
time and immutable observation ID come from the parent observation. Conflicts
remain separate. Missing/invalid claims display unknown, never guessed badges.
Media requires canonical HTTPS without embedded credentials, query or fragment;
that intentionally excludes some signed/source URLs. Unknown permissions mean
no display. Policy withdrawal hides the entire claim/media on regeneration.
Previously exported HTML remains a dated snapshot, not a revocation-aware app.

Content describes observed file structure or verified plugin flags; category
describes purpose. Skyrim ESP/ESM/ESL/ESPFE and Sims 4 package/script/DLL are not
interchangeable with shoes, tattoos or clothing. An `.esp` extension alone does
not establish the ESL flag or ESPFE; `.package` does not establish shoes/tattoos.
Adapters must retain evidence and explicitly distinguish inferred classifications.
No filename classifier or header parser is shipped by this slice.

Next visual gates: retained-media content-addressed cache with rights/freshness
and storage budget, visual quality/ownership checks, thumbnails, fast native
hover gallery, keyboard gallery navigation and verified native browser playback
QA. Existing records have no presentation observations yet, so they correctly
show no reviewed media/unknown categories rather than invented artwork.

Organization controls now include instant field-specific filtering, ascending /
descending name/content/category sorting, and game or entity-type grouping with
visible separators. All operate on the currently exported bounded page, not an
unbounded full-catalog scan. Changing filters preserves existing gallery elements
and media state instead of reconstructing or refetching them. These controls do
not yet provide saved user-defined groups or separators in the persistent catalog.

Validation: Rust tests cover malformed/untrusted markup, evidence statuses,
rights exclusion, policy withdrawal, playable-video markup and no automatic
media loads. DOM checks against 50 real catalog records cover no-match filtering,
restoration, group separators and sort direction. Native rendered-browser/playback
QA is blocked in this executor: Chromium exits with `socket() failed: Operation
not permitted`. DOM validation is not a screenshot or successful media playback.

Default visual design is OLED black with neutral panels, readable high-contrast
text and blue links/feedback, not dominant purple or green. A theme selector
provides neutral slate and light alternatives; changing it does not reload data
or media. Theme selection is local to the open export and is not persisted yet.
