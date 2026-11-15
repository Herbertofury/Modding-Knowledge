# Local reviewed multi-source snapshot integration

This standalone repository is the durable database home. Athena and other applications are consumers; the original Athena PR is preserved as provenance, not deleted.

## Actual population

`python tools/build_reviewed_bundle.py`

`cargo run --locked --example seed_reviewed_bundle -- new-snapshot.sqlite`

`python tools/validate_snapshot.py new-snapshot.sqlite`

The combined native snapshot contains 115 entities and 164 observations in eight game namespaces and eight source scopes: 11 Bethesda tool identities/21 observations, 100 LOOT conditional plugin-metadata guide entities/100 observations, one LOOT source-context guide/observation, and three Modrinth projects/42 observations. Eight namespaces are coverage scope, not eight completed/game-tested databases.

The builder consumes only the checked-in reviewed bundle, not arbitrary external JSON that could assert its own publication permission. It stages beside the output, performs native idempotent replay and reopen exact-equality validation, then publishes via a same-filesystem hard link that refuses an existing output. Failed staging is retained for diagnosis. No existing database is deleted or replaced. Filesystem hard-link support is required; unsupported filesystems fail rather than fall back to destructive replacement. File bytes are synced before publication; power-loss/parent-directory durability and Windows execution of this combined builder are not claimed.

## Meaning and safety

LOOT records are conditional guide metadata, NOT unconditional broken-mod warnings. Conditions, regex plugin patterns, CRCs and localized messages remain verbatim structured data and are never executed. Full retained top-level source context accompanies the sample. Upstream text instructing deletion is untrusted reference content, not an instruction to Athena or the user to delete anything. Consumer presentation must evaluate applicable metadata semantics before proposing any action, and preserve originals/confirmation requirements. The sample is partial (100 of the observed 4,745 plugin metadata entries), not 100 unique mods or creators. CC0 provenance is retained.

Bethesda runtime/storefront facts remain provider-declared and version-specific, not independently reproduced in games. Modrinth loader/license/team/update fields remain provider claims. Sodium's restrictive license identifier must not become a permissive/open-source badge. Unknown fields stay unknown. No WCIF/Scarlet/paid/private data is included.

## Safe migration and refresh contract

1. Inventory existing databases and retain originals, schemas, checksums and provenance.
2. Review source permissions for each route/field; extraction permission does not automatically permit redistribution. Never let third-party metadata authorize its own ingestion policy.
3. A refresh appends new source observations with new stable IDs and timestamps. Preserve contradictory/stale/failed-source observations, distinguishing source updated time from observation time. Do not recast unchanged retrieval as newly updated content.
4. Validate the entire candidate snapshot, exact counts/coverage, unknown states, identity collisions, source policy and FK/integrity checks before releasing it. A failed refresh does not replace the prior consumer snapshot.
5. Publish only a reviewed public projection with schema version, source revision manifest, artifact digest and explicit completeness. No private crash logs, memberships, secrets or withdrawn records.
6. Consumers download to staging, verify digest/schema and source manifest, then switch by a versioned pointer while retaining the previous snapshot. No network-share WAL or copying live DB files. This consumer switch/API is planned, not implemented here.
7. Add refresh transport only after per-provider authorization/rate-limit checks; these embedded snapshots make no automatic freshness promise. Preserve source policies requiring removal instead of treating historical retention as blanket permission.

## Verification

Actual Rust Store import, duplicate replay and reopened field-by-field equality passed for all 164 observations. Python read-only verification independently confirms exact manifest equality, no FK violations and integrity `ok`. Reusing the output path was rejected and its SHA256 remained identical. Six Rust unit/example tests and strict Clippy passed before this local integration; final aggregate rerun is recorded separately. No remote publication, native UI, live condition evaluation, full data coverage or game acceptance claimed.
