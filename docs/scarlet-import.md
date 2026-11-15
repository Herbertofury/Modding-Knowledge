# Scarlet live-source bounded ingestion checkpoint

Observed 2026-11-15 through the public SFW table, no login/cookies/credentials/API grants. User-reported direct permission from Scarlet is recorded, not independently verified. This is a clean-room factual-metadata sample, not copied GameSync private implementation.

## Verified

- Visible source total 7,907; captured first 10 rows only, alphabetic source order.
- Source patch 1.129; source update label November 13, 3:28 PM PST.
- CSV export control means **Export Your Mods**, not all source records. Do not claim bulk export completed.
- Python projector passes six tests: real snapshot, duplicate rejection, >1,000 rows without clipping, shared distribution URL identities, unsafe URLs, unexpected status.
- Native rusqlite Store imported 10 entities/10 observations, replayed without duplicates and verified exact equality after close/reopen.
- `artifacts/scarlet-sfw-sample.sqlite.gz` is the actual database artifact. `data/scarlet-reviewed.json` is the app-ready observation projection.

## Limits and next step

No automatic network refresh, full coverage, creator identity resolution or live-game compatibility test is claimed. No Notes column/full article text/artwork copied. **Consult the linked source notes**: e.g. the second record's Broken verdict applies to optional CommonTraitsMenu.package, not every file. Status is Scarlet's attributed claim, scoped to the exact distribution link and patch; absence/Unknown is never safe. Flagged and Removed describe list policy, not security verdicts. Observation timestamp has day-level precision (UTC midnight representation), not exact capture second.

WCIF public creator directory exists but automated reuse policy/API authorization remains unresolved. No WCIF rows ingested. Existing private GameSync A1:Z1000 adapter is incomplete and not reused or published.

This source sample is separate from the combined latest snapshot. Integrating it must preserve other refreshes and source policies. Hosted CI and publication status are reported separately, not inferred from this document. The owner explicitly authorized public factual-metadata ingestion and reports direct Scarlet permission for this purpose. Attribution is preserved; no broader content license is asserted. Next valuable pass: a bounded public-table pagination capture with row-count reconciliation, explicit status-note qualifiers, stable record identities across renamed rows, and strict no-last-good-overwrite on partial/error results. Do not treat the 7,907 SFW count as all Scarlet editions or all Sims 4 mods.
