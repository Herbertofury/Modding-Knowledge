# Informative pre-update advisories (v1)

Requested 2026-11-16: every mod/project manager should warn about reports of crashes, freezes and bugs affecting a proposed update. This shared contract applies to Noxviola, MO2R, Mod Hunter, Athena and other project-manager consumers. Users retain authority: warnings never disable, cancel, defer or replace their selected update. This contract does not add a premium gate or require another app.

## Evidence and interpretation

`tools/update_advisories.py` is a tested read-only Python contract/projection, not a deployed feed or installer. It complements existing native `health_claim_v1` evidence described in [Mod Health](mod-health-query.md). It does not change that Rust/SQLite schema or silently translate generic health claims into update-specific claims.

Every observation has a stable ID, provider-qualified project ID, explicit affected release IDs, game build and runtime context, symptom (crash/freeze/bug), evidence level, canonical public source URL, original report lineage, report time, observation time, review deadline and active/retracted/superseded lifecycle. An empty affected-version list or null context means unknown, not universally affected. Exact-string version matching is intentional; reviewed provider adapters must resolve release aliases or ranges before supplying exact release IDs.

Evidence levels distinguish community reports, maintainer reports, reproduced symptoms and maintainer-confirmed cause. Reproduction is not proof of cause. A maintainer-confirmed cause label is an attributed claim, not a verdict of this database. Validation verifies structure only: source adapters/reviewers must verify source content, provenance, evidence classification and publication policy. Private logs, usernames, credentials and arbitrary comments are excluded; do not publish private project identities through this public contract.

Candidate queries match the project, target version, build and runtime. Mismatched known context is excluded; missing context remains visibly uncertain. Original reporting time and review deadlines govern freshness: refreshing an old report cannot make it fresh. Historical/retracted/superseded observations remain visible but must not appear as current warnings. Preserve conflicting observations; resolve correction relationships explicitly before native projection. Report lineage deduplicates mirrors, not independent users or votes. No reports means unknown coverage, never safe/compatible.

The result always returns `update_allowed: true`, `blocks_update: false` and `automatic_action: null`. These are the advisory subsystem's outputs, not authority to override unrelated security or installer failures. Consumers must show attributable reports before update confirmation, with exact/uncertain scope, evidence label, age and coverage. Keep Proceed available; optional dismiss/details/backup controls must not silently alter the plan. A failed or offline evidence fetch shows unavailable/last-checked state and leaves the user's update choice intact.

## Acceptance backlog

- [x] Shared strict v1 contract and non-mutating candidate-specific projection.
- [x] Tests for exact version/context, all symptoms, unknown scope, stale original reports, lifecycle, lineage, no cause promotion, public-field validation and never-block behavior.
- [ ] Reviewed live update-specific evidence intake; no factual mod failure records seeded by this change.
- [ ] Native SQLite projection/source-policy checks and adapters to existing health evidence; retain stable observation/correction relationships.
- [ ] All manager consumers: pre-update warning UI, user-controlled Proceed, accessibility, dismiss/preferences, offline/retraction/conflict states and actual end-to-end update tests.
- [ ] Source coverage/fetch receipts, bounded incremental refresh and operational freshness monitoring.

Tests: `python -m unittest discover -s tests -p 'test_*.py'`. This is shared foundation work, not proof that any app already displays the warning.
