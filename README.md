# Modding Knowledge

**The central, public, app-consumable home for multi-game modding knowledge.** Athena and other tools can consume the same versioned records instead of keeping divergent private catalogs.

## Actual data: initial reviewed snapshot

| Dataset | Real entities | Attributed observations | Meaning |
|---|---:|---:|---|
| Bethesda tools | 11 | 21 | Provider-declared runtime/source/troubleshooting facts |
| LOOT bounded sample | 101 | 101 | 100 conditional plugin metadata guides plus complete retained top-level context |
| Modrinth projects | 3 | 42 | Sodium, Lithium, Iris public project metadata |
| **Combined** | **115** | **164** | Partial coverage, not 115 unique mods or completed game coverage |

**[Download original populated SQLite snapshot](artifacts/modding-knowledge-2026-11-14.sqlite.gz)** · [Structured records](data/reviewed-bundle.json) · [Verification](validation.json) · [Knowledge wiki](wiki/Home.md) · [Source inventory](SOURCES.md)

## Live public creator/mod refresh

[Latest populated database](artifacts/modding-knowledge-latest.sqlite.gz) · [Latest validation/counts](artifacts/latest-validation.json) · [Refresh success/failure receipt](data/live/modrinth-refresh.json) · [Live reviewed metadata](data/live/modrinth-current.json)

The first real refresh added **5 public creator accounts and 7 project-team membership observations**, yielding **120 entities / 171 observations** on 2026-11-15. Future counts are authoritative in the linked validation receipt, not this historical count. Creator roles are provider-declared team memberships, not claims of sole authorship or full mod catalogs.

The bounded official Modrinth refresh is configured hourly at minute 23 UTC for three explicitly selected project IDs. It fetches at most six anonymous API responses, excludes private/account fields, and preserves last-good data on failure. Source receipt distinguishes attempts from successful observation time. Scheduled execution can be delayed or disabled by GitHub; freshness must be checked, not assumed. Other providers are not automatically refreshed yet.

The original snapshot has eight game namespaces and eight source scopes. It remains a dated reviewed snapshot; the separate latest artifact receives the bounded Modrinth refresh, not comprehensive multi-provider coverage. Missing records never prove compatibility or safety. LOOT conditions remain unevaluated; do not present conditional warnings as unconditional bad-mod verdicts.

## Use it

```sh
cargo test --locked --all-targets
cargo run --locked --example seed_reviewed_bundle -- new-snapshot.sqlite
python tools/validate_snapshot.py new-snapshot.sqlite
cargo run --locked --example query -- new-snapshot.sqlite modrinth:project:AANobbMI
```

Or decompress the supplied `.sqlite.gz`, verify its SHA256 against `validation.json`, and query SQLite. Rust consumers can use the library API described in [Storage/API](docs/storage-api.md). The Rust package retains its original `athena-mod-knowledge` name for continuity. There is no hosted HTTP API yet.

```sql
SELECT e.name,o.predicate,o.value,o.source_url,o.observed_unix,
       o.applicable_build,o.mod_version,o.completeness
FROM entities e JOIN observations o ON o.entity_id=e.id
JOIN sources s ON s.id=o.source_id
WHERE s.can_retain=1 AND s.can_publish=1
  AND s.policy_revision=o.policy_revision
ORDER BY e.name,o.observed_unix DESC,o.id;
```

## Organized scope

- Creator identities, linked official accounts/sites and attribution
- Mod/project/release metadata, versions and scoped compatibility
- Tools, practices, how-tos, crash diagnosis and maintenance
- Source-attributed broken/outdated/incompatible warnings with conditions
- Source policies, provenance, freshness, uncertainty and extraction failures

[Coverage and priorities](wiki/Coverage.md) · [Public rules](wiki/Rules.md) · [Migration and refresh](docs/migration-refresh.md)

## Verification and limits

Actual Rust imports, exact replay/reopen equality, independent SQLite integrity/FK checks and six tests passed locally. Existing-output refusal preserved the original database hash. Windows/Linux CI is included; inspect its exact commit result before claiming hosted acceptance. No native game tests, automatic refresh, full creator inventory, or complete mod corpus is claimed.

## Provenance and licensing

The clean-room foundation originated in [Athena PR #174](https://github.com/Herbertofury/Athena/pull/174), exact reviewed head `8fdb9e45f469035a00722192281f0eb675b9852b`. That source remains preserved. No private GameSync source/data, credentials, memberships or paid content was copied.

LOOT's included metadata retains [CC0 license text](data/loot-CC0-LICENSE). Other included records are small attributed factual projections, not copied repository source, artwork or complete provider databases. Provider license claims refer to their projects, not this entire repository. A repository-wide code license has not yet been assigned; public visibility alone is not a blanket reuse license.
