# App-facing scoped Mod Health evidence

`Store::import_health_claims` and `Store::health_report` provide an additive, schema-v1-compatible Rust interface. This is not a port of private GameSync source and does not publish private account data. Existing reviewed observations and snapshots are unchanged.

## Contract

Each `health_claim_v1` observation retains entity, exact game build/edition, exact mod version, canonical source URL, observed time, public policy revision and completeness. Its JSON value requires:

- `assertion`: works / broken / warning / unknown.
- `authority`: official_release / maintainer_report / community_report / independent_test. Classification must be reviewed against the source, never inferred from popularity or username.
- `lineage`: stable original report lineage, shared across mirrors/reposts; not a vote counter.
- `context`: exact runtime/loader/configuration context supplied by the adapter. Unknown context is not a wildcard.
- `reported_unix`: original report time, not a scraper refresh date.
- `status`: active / retracted / superseded. Historical rows remain visible; these statuses are explicitly supplied, not inferred. Automatic successor resolution is not implemented.

Extra JSON fields are rejected by the structured import entry point: raw comments, usernames, private identities and arbitrary payloads do not belong in this public contract. Source policy and byte-identical replay remain enforced. General observation imports retain their existing behavior; malformed health values from that route are counted as invalid rather than promoted to claims.

## Interpretation

Queries require exact build, version and context; no cross-edition compatibility inheritance. No evidence is `unknown`, never green. Active expired evidence is `stale`; source reporting dates control freshness, so repeated scraping cannot make old claims current. Matching current assertions remain `partial_evidence`, not a compatibility verdict. Works versus broken/warning is `conflicting`; all attributable reports remain returned. Authority labels remain distinct and do not override opposing evidence. Independent lineage counts are not independently verified corroboration or reputation scores.

Reads recheck current public policy, hiding withdrawn observations. Query reads every observation page with a 10,000-record hard bound; exceeding it fails explicitly. This bounded query is not a complete scan of the Internet, a crash diagnosis or game acceptance test. Existing LOOT conditional metadata and Modrinth metadata are **not** automatically converted into compatibility assertions.

## Consumer

```
cargo run --locked --example query_health -- existing.sqlite ENTITY GAME_BUILD MOD_VERSION RUNTIME_CONTEXT NOW_UNIX MAX_AGE_SECONDS
```

The example opens read-only and emits attributable JSON. Current real snapshots have no validated `health_claim_v1` assertions yet; their result is correctly unknown. Tests are synthetic and explicitly named as such. Actual permitted community adapters, trusted authority classification, retraction links, scan-job coverage, and tested app integration remain subsequent work.
