# Project support and availability backlog

Request 2026-11-17: premium mod managers must show whether a creator remains active, whether a specific mod is supported/discontinued, and whether it is deleted or unavailable. Apply shared evidence across MO2R, ModHunter, Noxviola and Athena without making any depend on another app.

- [x] Read existing author-source-continuity contract/PR #6 and current shared rules before implementation.
- [x] Rust evidence validation and conservative deterministic projection with tests.
- [ ] Reviewed provider adapters, persisted observation history and migration.
- [ ] App columns/cards, filters, source links, timestamps and stale/conflict badges; no implied deployment from this contract.
- [ ] Premium entitlement, notification controls and real native acceptance.

Preserve separate source availability, project support and creator activity. Old releases, failed fetches, HTTP errors and activity elsewhere never prove abandonment. Do not touch the active ModHunter release branch.
