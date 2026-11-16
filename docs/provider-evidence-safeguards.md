# Provider identity and issue-count safeguards

Reviewed 2026-11-16. These are tested reference projections, not a live scraper or a deployed SQLite import.

## Hash lookup is not author/project identity

Nexus's official [`getFileByMD5` documentation](https://github.com/Nexus-Mods/node-nexus-api/blob/master/docs/classes/_nexus_.nexus.md#getfilebymd5) returns multiple possible matches, including uploads at different locations and possible collisions. `tools/provider_evidence.py::md5_candidates` retains normalized file/project/creator candidates, size, source and observation time. It never selects or merges an identity, even for one result. Equal hash and size do not prove author identity or authorize project merging. Strong content checksums can identify retained bytes but remain separate from ownership, release lineage and licensing evidence.

Adapters must record actual lookup coverage, provider/game context and reviewed identity links. No automatic duplicate deletion or catalog migration is implemented. Candidate records are copied without mutating caller state; normalized fields exclude arbitrary private payloads and credential-shaped URLs.

## GitHub issue pages include pull requests

[GitHub's official issue API documentation](https://docs.github.com/en/rest/issues/issues) identifies pull requests using the `pull_request` key. The tested projection excludes every row containing that key, including empty/null values. It counts only supplied issue rows by open/closed state, separately counts repository-mapped bug labels, retains issue/source URLs and observation time, and rejects duplicate issue numbers or mixed-repository issue URLs.

The output explicitly leaves repository-wide totals and confirmed defect counts unknown. A fetched page, empty page, or filtered query is not a complete project count; an upstream fetch failure must not be passed as a successful empty page. Labels are evidence of classification, not verified defects. Adapters must supply a reviewed repository-specific label mapping rather than assume every project uses `bug`.

## Acceptance backlog

- [x] Synthetic regression fixtures: identical MD5 across projects/creators, single/empty ambiguity, PR-marker exclusion, issue-vs-bug classification, open/closed state, page-vs-total distinction, duplicate/mixed-source rejection and nonmutation.
- [ ] Native ingestion integration with public-source policy checks, timestamps, pagination receipts and explicit query/state/label scope.
- [ ] Full count reconciliation across pages with stable snapshot/error semantics, stale last-good fallback and unavailable states; never replace a failed request with zero.
- [ ] Reviewed real provider fixtures and native app end-to-end tests before claiming live coverage.
