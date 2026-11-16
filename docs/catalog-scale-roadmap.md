# Catalog scale and continuous improvement

Requested 2026-11-16: Modding-Knowledge should serve massive creator/project catalogs while continually improving compact size, query speed, convenience, organization and easy access. It should intelligently find measurable improvements, not merely accumulate undifferentiated data. Preserve source evidence and historical provenance throughout.

## Backlog and acceptance

- [ ] Establish representative small/large benchmark corpora, distributions and repeatable baseline receipts: source/version, row counts, disk bytes, import throughput, query latency percentiles, peak memory and integrity results. Synthetic scale data must remain explicitly synthetic.
- [ ] Benchmark indexed lookups, pagination, incremental ingestion, deduplicated content storage and compression. Adopt only measured wins without lost observations, identities, evidence or query correctness. Compact projections must retain links to complete source provenance.
- [ ] Stable creator/project/release IDs, aliases and reviewed cross-provider mappings; indexed taxonomy/search and documented lightweight consumer APIs. Never merge people or projects from similar names alone.
- [ ] Bound incremental refresh and resource use; prioritize changed sources and expose freshness, coverage, failures and last-good state.
- [ ] Self-improvement loop: identify bottleneck, propose candidate, measure against baseline, run integrity/replay/query regressions, preserve receipts, review and roll back if worse. Do not silently rewrite production data, change policy, delete history or promote unverified facts.
- [ ] Benchmark advisory selection against large scoped datasets before native integration. The current Python advisory helper is an in-memory reference implementation, not a demonstrated high-volume database query engine.

No catalog-size, speed or compression target is claimed achieved by this roadmap. Existing snapshot receipts remain authoritative. Planned optimizations must be validated on realistic data before reporting progress as performance improvement.
