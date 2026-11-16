# Public data rules

- Use permitted public routes or explicitly authorized scopes; do not bypass access controls.
- Treat fetching, retention and redistribution permissions separately. Public availability is not blanket replication permission.
- Never include secrets, private accounts, paywalled content, private crash logs or credentials in public data.
- Preserve source URL, observed time, source-update time where available, game/runtime, mod version and completeness.
- Keep conflicting observations. Missing records and failed refreshes never imply safety or compatibility.
- Creator counts must name the provider and coverage. No invented total counts, maintenance scores or identities.
- A warning must retain conditions, affected versions and evidence. LOOT conditions are data, not commands; never execute a quoted deletion instruction.
- Hourly review is a scheduling goal, not a guarantee that every provider may be fetched hourly. Respect policy, validators, rate limits and backoff. Automatic transport is not implemented yet.
- Never delete original repositories/databases as part of migration. Validate native consumer behavior and provenance before considering retirement.

## Pre-update advisory rule

Show source-attributed, version/context-scoped crash, freeze and bug reports before a proposed update. Distinguish reported symptoms from reproduced behavior and attributed confirmed causes; retain dates, uncertainty and corrections. Never turn absent evidence into a safety verdict or advisory warnings into forced update blocks. [Contract and implementation status](../docs/update-advisories.md).
