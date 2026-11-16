# Author source continuity (shared contract v1)

Requested 2026-11-16 for Noxviola, MO2R and Mod Hunter premium features. Modding-Knowledge owns common evidence and interpretation; apps own entitlement, consent, scheduling and provider adapters. This is a tested contract and read-only decision helper, not deployed monitoring or a paid feature activation.

## Rule

Warn users about source disruption and retain update continuity through verified additional sources. Separate **site account status**, **creator identity linkage**, and **each mod's source mapping**. Never infer a ban from HTTP 403/404, removal, a timeout or an absent listing. A creator statement remains creator-reported, not a provider-confirmed verdict. Do not speculate on motives or repeat unsupported accusations. Preserve contradictory/retracted observations rather than replacing history.

`tools/author_transitions.py` validates schema v1 records and returns a non-mutating decision. Canonical HTTPS evidence URLs, observation and review deadlines, source accounts, stable creator ID, evidence authority, lifecycle, game and explicit per-project mappings are required. See `tests/test_author_transitions.py` for synthetic fixtures only. No real author enforcement claim is seeded in this change. Validation checks structure, not the truth of a remote page: reviewed adapters must establish content, authorship and identity before supplying evidence.

- Status vocabulary: unknown, unavailable, removed, suspended, banned, active.
- Lifecycle: active, retracted, superseded. Retain each observation under a unique ID; consumers must resolve conflicting active observations conservatively before execution.
- Enforcement requires provider notice or creator statement; the UI must display its authority and link, never just an unqualified badge.
- Destination identity requires a reviewed creator cross-link or provider-verified link. Similar names alone are insufficient.
- Mapping requires exact source and destination mod pages and game namespace. No automatic whole-catalog claim from one verified mod.
- Fresh active records, identity evidence, verified per-project mapping, user tracking consent and an available provider adapter are all required for `track_additional_source`. This is a suggestion, not a network operation.
- Retain the old source, installed version, provenance and last-good release records. Cross-site version labels are not automatically comparable; require adapter-specific release identity and compatibility validation.
- Never download or install from a relocation record. Downloads, credentials and persistent grants follow their own controls. Do not export private libraries, tokens or account details.
- A failed new-source refresh remains a failure, not proof that the creator or mod disappeared. Show freshness, partial coverage and access limits. Retractions and corrections must update warnings and suspend new tracking decisions without erasing history.

## Implementation and acceptance backlog

- [x] Shared rule, validator and deterministic consumer decision helper.
- [x] Tests: missing page versus ban, attributed statement, stale/retracted data, identity and project gates, no implicit downloads, credential-shaped URLs, ambiguity and nonmutation.
- [ ] Integrate this contract with native SQLite observations and source-policy gates; no current snapshot counts changed.
- [ ] Verified real-author records, including the user's SethSaidThat/ModHold lead, only after source/identity review. The user's example is a research lead, not a seeded factual allegation.
- [ ] Noxviola, MO2R and Mod Hunter: consume shared records, show source-linked warnings, consent/preference controls, follow/unfollow and tested adapter refresh.
- [ ] Multi-observation conflict resolution and retraction UI with retained receipts.
- [ ] Native lifecycle, accessibility and live end-to-end tests. Premium entitlement wiring is not implemented here.

Run `python -m unittest discover -s tests -p 'test_*.py'`. Existing Rust database verification remains `cargo test --locked --all-targets`.
