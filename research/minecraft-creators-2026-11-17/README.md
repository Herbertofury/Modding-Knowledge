# Minecraft creator research: reviewed intake

Four source-reviewed project records, not four complete creator catalogs. This deliberately small intake supports wider creator discovery without promoting generated text into trusted runtime data. Observed 2026-11-17.

## Review result

A broad generated discovery roster was useful for leads, but had an incorrect Regions Unexplored attribution and uneven per-creator release/media evidence. The reviewed JSON retains only primary-source-checked statements; it does not reproduce that roster. The official [Modrinth source link](https://modrinth.com/mod/regions-unexplored) resolves to [Apollounknowndev/RegionsUnexplored](https://github.com/Apollounknowndev/RegionsUnexplored), whose README distinguishes current owner Apollo, former owner UHQ_Games and contributor KirboSoftware. No cross-site personal identity is inferred from names.

## Coverage and next gates

- Project/source and publishing links: four reviewed records.
- Release/date: one historical Create release, not a latest-version guarantee.
- Complete catalogs, Bedrock, author numeric IDs, avatars and media inventory: not completed.
- No media files copied; public visibility does not grant reuse rights.
- No ban, relocation, compatibility, safety or maintenance verdict.
- Native schema reviewed in `docs/storage-api.md`: runtime admission needs provider-qualified entities and immutable source-attributed observations, current retention/publication policy, freshness and scope. The schema-native projection is `data/minecraft-creators-reviewed.json` (4 entities / 10 observations); its dedicated import example is `seed_minecraft_creators`. It remains outside the live bundle until shared-loader integration.
- Existing Sodium/Iris runtime entries must be reconciled by provider IDs, not duplicated by names.

This branch adds research, native-schema data, a scoped source policy and an import example. It does not expand the scheduled fetch scope, alter provider permissions, or regenerate the SQLite distribution.
