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

Search a particular game and record kind in the **whole catalog** before
pagination, not just within the already rendered page:

```sh
knowledge-manager catalog.sqlite browse-in minecraft-java mod Iris
knowledge-manager catalog.sqlite visual-in minecraft-java mod Iris > iris-catalog.html
```

Both commands also accept `[AFTER_ID [LIMIT]]` after the query. To allow any game
or kind, use a quoted `"*"` for that dimension. Pass the same game, kind and
query when continuing a cursor. IDs are exact. Unknown IDs yield no results;
unknown evidence is not a negative compatibility verdict. The legacy
`browse` and `visual` commands still search across all games and kinds.

Browse and duplicate hints recheck source publication policy. Same-name groups
are restricted to the same game and kind, capped at 100 with an explicit
truncation flag, and **never automatically merged**. Name matches cannot prove
identity. Inspect source evidence directly in the portable manager by passing the exact
ID returned by a browse command:

```sh
knowledge-manager catalog.sqlite evidence adeepindigo:project:dental-care --mod-relations-v2
knowledge-manager catalog.sqlite evidence modrinth:project:YL57xq9U --predicate provider_claim.source_url
```

The same report still powers `cargo run --locked --example query -- catalog.sqlite ENTITY_ID`.
The manager can use the same offset and flags, including the pinned Oblivion and
Fallout 4 LOOT modes. The v2 relationships report keeps requirements, optional
integrations, compatible connections, conflicts, alternatives and unknowns in
separate columns. Linked claims retain source, scope and exact conditions; an
empty column on a page does not establish absence or evaluate compatibility.

Status includes withheld row counts but not their content. Allocated/reclaimable
page-byte metrics are not compressed file size, WAL size, measured query speed
or guaranteed compaction savings. Structural integrity does not validate factual
claims or game compatibility. Raw databases may contain withdrawn observations;
these commands do not authorize distributing those files.

## Portable evaluation ZIP

Same-repository pull requests can produce an unsigned evaluation artifact for
Windows and Linux. It contains one locally built binary, its exact reviewed
SQLite catalog, an offline scoped HTML example and a short start guide. The CI
packager verifies that the entire catalog matches the reviewed snapshot,
extracts and executes the native CLI outside the checkout, rechecks integrity,
confirms the scoped query/output and refuses writes to the database. It retains
the exact executable and catalog bytes with a source-commit and hash receipt.
This is a preview, not a production release, installer or compatibility verdict.

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

Project and creator names link directly when this page has one unambiguous
provider-qualified destination. Modrinth project routes require a matching
public API observation for the exact ID; explicitly reviewed project URLs and
account profiles come from their evidence rows. When there are several
recorded pages, show them separately rather than picking one. Other rows
keep the original name and offer labeled reviewed source links without
implying a canonical mod page or cross-platform identity. These links do
not fetch or load before a user chooses them.

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

### Open a separately reviewed data pack

Pass an extracted sidecar SQLite file as the database argument; do not
replace the main catalog. The visual report uses the same read-only manager:

```sh
knowledge-manager hytale-file-channels.sqlite visual-in hytale mod "Portal World" > hytale-view.html
knowledge-manager cyberpunk2077-core.sqlite visual-in cyberpunk2077 mod "Let There Be Flight" > cyberpunk-view.html
```

The project heading links directly only when the reviewed source policy and
project identity support a canonical route. Hytale and Cyberpunk file or
release pages remain evidence links, not project-name destinations. The
Sims 4 policies support the vetted Frankk, Simsonian Library, and Scumbumbo
project pages; Frankk's site route is scoped to the exact source record.
Unknown or ambiguous identities keep their evidence links without guessing
an author profile. No mod acquisition, live catalogue write, or install is
performed when creating the report.

The separately reviewed Skyrim runtime pack can also be queried directly:

```sh
knowledge-manager skyrim-native-runtime.sqlite runtime-in skyrim-se Steam 1.7.104.0
knowledge-manager skyrim-native-runtime.sqlite runtime-in skyrim-se GOG 1.6.1179
```

Supply the storefront and exact game executable version. A missing match is
unknown; it is not a reason to fall back to another store or install a nearby
version. The command returns reviewed source links and does not detect,
download or install files.

### Read exact Skyrim publisher files

Point the manager at a separately reviewed sidecar, not the bundled general
catalogue if it has no Skyrim release rows:

```sh
knowledge-manager skyrim-distributor-releases.sqlite release-files github:powerof3/Keyword-Item-Distributor
knowledge-manager skyrim-distributor-releases.sqlite release-files github:powerof3/Keyword-Item-Distributor --html > reviewed-kid.html
```

Repeated FOMOD names stay separate by release and provider asset ID. Links
are included only when the cited source and exact recorded archive path agree.
A source release page is retained when an asset link was not recorded; no
URL is guessed. Provider digests are not locally verified archive bytes.
The output does not check which installer option matches an installed runtime,
contact providers, download mods, change SQLite, or overwrite catalogues.

Use two exact tags for a version-to-version file metadata comparison:

```sh
knowledge-manager skyrim-distributor-releases.sqlite compare-release-files github:powerof3/Keyword-Item-Distributor v4.0.6 v4.1.0
```

Pairs are matched only by observed provider asset ID, exact filename, or an
equal reported SHA-256; the three observations remain separate. The reviewed
KID example has the same FOMOD filename, different provider asset IDs and
different reported digests. None of these signals implies a replacement or
runtime match. Both source observation records remain in the output, and
unpaired files are listed without assuming removal. Missing/ambiguous tags
and invalid retained rows are refused. No download or SQLite write occurs.
