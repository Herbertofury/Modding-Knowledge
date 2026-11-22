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

Queries require exact build, version and context; no cross-edition compatibility inheritance. Blank values and whole-field placeholders (`unverified`, `unknown`, `unspecified`, `n/a` and `*`) fail with an error naming the missing scope field. A source row with an unverified build remains in the general observation ledger, but matching the word “unverified” can no longer make it an exact-version health warning. Qualified context strings are still opaque, and valid versions and contexts are matched byte-for-byte. No evidence is `unknown`, never green. Active expired evidence is `stale`; source reporting dates control freshness, so repeated scraping cannot make old claims current. Matching current assertions remain `partial_evidence`, not a compatibility verdict. Works versus broken/warning is `conflicting`; all attributable reports remain returned. Authority labels remain distinct and do not override opposing evidence. Distinct report-lineage counts are not independent reporters, independently verified corroboration or reputation scores.

Reads apply current public policy, even for historical time cutoffs. A receipt observed after the requested as-of time cannot change the counts, freshness, or claims returned for that time. Freshness still follows the original report date once its receipt is in view. The reader returns at most 10,000 valid claims for the exact entity/build/version and opaque runtime context. Different runtime contexts do not count against that limit. A separate 20,000-row scan limit covers the entity/build/version and as-of time across contexts, including malformed claims. Malformed claims count as invalid at that broader scope because they may lack a trustworthy context. Either limit fails explicitly without returning a partial report. This bounded query is not a complete scan of the Internet, a crash diagnosis or game acceptance test. Existing LOOT conditional metadata and Modrinth metadata are **not** automatically converted into compatibility assertions.

## Consumer

```
cargo run --locked --example query_health -- existing.sqlite ENTITY GAME_BUILD MOD_VERSION RUNTIME_CONTEXT NOW_UNIX MAX_AGE_SECONDS
```

The example opens read-only and emits attributable JSON. The current combined snapshot includes four community warning assertions from six reviewed GitHub source documents (four issues and two release records). Original factual summaries, times, issue states and scope context accompany those claims. Release facts remain separate and never become compatibility verdicts. Lithium #804 has an unverified game scope, not inferred certainty. Tests are synthetic and explicitly named as such; the read-only CLI is also exercised on the real populated snapshot. Automatic community refresh, corroboration/authority verification beyond this reviewed tranche, retraction links, scan-job coverage and tested app integration remain subsequent work.
