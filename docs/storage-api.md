# Athena mod knowledge: native storage foundation

A clean-room Rust library for public, attributed multi-game knowledge. This is an isolated initial storage component, not a complete database, scraper, live provider integration, UI or recommendation engine. No private source code or real creator data is included.

## Contract

- Stable explicit game/entity/source IDs; entities distinguish creator/mod/release/tool/guide/warning. Provider-qualified IDs are recommended; never merge matching names automatically.
- Observations retain source URL, predicate/value, game build, mod version, observation time, completeness and policy revision. Different IDs preserve conflicting claims. Unknown and missing remain unknown; a missing broken-mod report is not evidence of compatibility.
- Import accepts only a current policy revision explicitly permitting BOTH retention and public publication. Policy registration is reviewed trusted input, not a conclusion inferred from public visibility. Fetch permission is separate and not implemented here.
- Changing/revoking source policy immediately hides historical observations from public reads. Historical rows are retained internally; a policy requiring erasure needs a separately authorized retention workflow before deployment. Do not distribute the raw database when it contains withdrawn or non-public rows; an export projection is NOT implemented yet.
- Observation IDs are immutable. Replays must match all fields; an ID collision rolls back the entire bounded import. Failed refresh attempts never erase last-good observations.
- Read pages contain up to 1000 rows, deterministic timestamp/ID order. Offset pagination is for stable snapshots; concurrent live ingestion requires a future cursor/snapshot API.
- HTTPS source references reject userinfo, query and fragments to reduce accidental secret ingestion. This deliberately excludes some legitimate provider URLs; source adapters must supply reviewed canonical references. This validation is not an SSRF transport guard, since no fetching occurs.
- Caller must review public text: arbitrary observation values are not automatically anonymized or licensed by this library.

## API

`Store::open`, `register_game`, `set_source_policy`, `register_entity`, `import`, `observations`, `observations_page`, `observations_page_for_predicate`, and `record_refresh`.

No automatic schema upgrades, network requests, credentials, subscription, download, scraped data, or installer. Existing unrecognized databases and future schema versions are rejected without replacing data. Use a separate file from Foundry workspace storage.

## Verification

`cargo test --locked --offline` and `cargo clippy --locked --offline --all-targets -- -D warnings`.

Five tests cover policy denial/withdrawal, idempotent replay and atomic rollback, conflicting observations and failed refresh retention, invalid relationships/credential-bearing references, on-disk reopen/integrity, pagination and future-schema preservation. Fixtures are invented and marked `fixture`; no real compatibility assertion is established.

## Next integration gates

Add source-policy provenance/permission scope, creator accounts and reversible identity matching, provider-scoped count semantics, public export/hash manifest and schema contract, knowledge search/FTS, refresh status queries, bounded authorized adapters, native Athena routing and consumer API adapters. Import existing Minecraft-Mod-Vault/GameSync data only after public/private/license review, without copying unknown private content. Scarlet coverage is submitted mods only; WCIF counts are WCIF-observed counts, never total creator output. Game patch and mod version must be explicit on compatibility claims.

## Initial real catalog seed

`cargo run --locked --example seed_public_catalog -- new-catalog.sqlite`

This executable creates three actual source-attributed records (Sodium, Lithium, SKSE64) from reviewed official repository pages. It refuses an existing output path. These are manually reviewed factual identity/category records, not copied source code or provider databases. No latest-version, compatibility, maintenance-score, mod-count or safety assertion is inferred. The generated file was reopened independently and returned three records with `integrity_check=ok`. This is a deliberately small starting corpus, not the requested final multi-game knowledge coverage.

## Real Modrinth metadata snapshot

`cargo run --locked --example seed_modrinth -- new-modrinth.sqlite`

The checked-in, reviewed projection `data/modrinth-public-2026-11-14.json` contains real official API observations for Sodium, Lithium and Iris, fetched without authentication with an identifying User-Agent. The importer produced **3 project entities and 42 source-attributed observations**, independently reopened with integrity `ok`. It retains provider IDs, team/organization IDs when supplied, repository/issue/wiki links, license claims, publication/update timestamps and loader/client/server claims. Null fields remain absent; project-level loader claims are not per-release compatibility verification. Provider `approved` status is not an Athena safety verdict.

Sodium's observed license identifier is `LicenseRef-Polyform-Shield-1.0.0`; do not label it unrestricted/permissively open source. The other two reported `LGPL-3.0-only`. These are provider claims requiring source-license review before code reuse. No descriptions, images, mod files, credentials or account details are copied. The importer does not poll and the frozen snapshot has no automatic freshness guarantee. Automatic permitted refresh and full consumer API remain follow-up work.


## Consumer query and bounded live refresh

`Store::open_read_only` cannot create/migrate databases; `cargo run --example query -- FILE ENTITY_ID` returns JSON with provenance and scope. Public team memberships now join the generated bundle as creator entities. Refresh metadata is explicitly partial, API-provided, and not complete authorship or compatibility. See `docs/refresh-modrinth.md`.

### Read a single evidence type without filtering an already-capped page

```sh
cargo run --locked --example query -- FILE EXACT_ENTITY_ID --predicate conditional_plugin_metadata
cargo run --locked --example query -- FILE EXACT_ENTITY_ID 1000 --predicate conditional_plugin_metadata
```

The optional predicate is exact and case-sensitive. The current publication and
retention policy, revision and predicate are all applied before the 1,000-row
page. Within the selected entity and evidence type, newer observations come
first, then ID breaks ties. The JSON preserves each source URL, revision, build
and version scope, completeness and raw value. The old unfiltered command and
its output fields are unchanged.

An empty page does not mean that no relationship exists. This query does not
parse descriptions, join names or IDs, evaluate LOOT conditions, infer reverse
links or turn author evidence into tested compatibility. The
`conditional_plugin_metadata` predicate is the existing LOOT evidence type; its
payload and game-family scope stay unchanged.

### Offline provider relationship research

The [Nexus relationships research example](../examples/research/nexus_relationships/README.md)
links official API documentation and tests synthetic responses for distinguishing exact
file-version requirements, DLC groups, and materialized candidates. It is not a
supported provider client, an admission decision, or a live collector. It does not
change the catalog, fetch anything, or recommend or install mods.
