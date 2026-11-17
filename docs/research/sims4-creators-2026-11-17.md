# Sims 4 creator discovery: bounded official-source tranche

This reviewed tranche contains four source-scoped creator identities, three projects and twelve factual observations. It is not a complete Sims 4 creator catalog. Broad discovery leads must be checked before ingestion.

## Source checks

- [TwistedMexi's official directory](https://www.twistedmexi.com/Mods/) explicitly links Patreon and Better Exceptions. [The linked release post](https://www.patreon.com/TwistedMexi/posts/better-v3-17-mod-130049745) displays version 3.17 and `June 19`, but no year in the retrieved text. Release date stays null. The directory's release-candidate label and the post's public-release text are retained as a channel caveat, not silently reconciled. The Patreon profile destination could not be retrieved; the outgoing official link is verified, not authenticated ownership.
- [MCCC official downloads](https://deaderpool-mccc.com/downloads.html) lists 2026.6.0 and a publisher-declared test against PC 1.129.39.1030 / Mac 1.129.39.1230. October 27 is the stated game-patch date, **not** the mod release date. MCCC 6.6.0 for Legacy Edition remains distinct from 2026.6.0. No compatibility with later patches is independently established.
- [Teleport Any Sim's official maintained page](https://scumbumbomods.com/teleport-any-sim/) attributes the original to Scumbumbo, the update to Andrew, and displays March 1, 2024. It explicitly points to [Sims4Studio](https://sims4studio.com/post/72734); that destination timed out. Andrew is a source-scoped maintainer identity, not an inferred merge with every same-name account. The date is a listed update, not proof of the newest release.

Media records point to public project pages with images. Direct asset URLs and reuse rights remain unknown; no images, avatars, articles, attachments or archives are copied. Dates, release channels, identifiers and compatibility remain separate fields. No private account data, email or paywall bypass was used.

## Integration and tests

Canonical input: `data/sims4-creators-reviewed.json`; explicit factual-source scope: `config/sims4-reviewed-source-policy.json`. A shared integration owner must merge this named input into the reviewed-bundle builder after checking the policy. Do not add wildcard ingestion or permit external JSON to grant its own publication rights.

An isolated integration rehearsal against main `46182a50477ab01706b6a038e51978f3bce5e555` passed Rust tests (10), Python tests (35 including four new regressions), native Store import, identical replay, close/reopen exact equality and independent SQLite integrity/foreign-key validation. The rehearsal produced 4,781 entities / 4,894 observations (base 4,774 / 4,882), preserving existing entities. These are not live game tests. Shared builder/generated bundle changes are left to the integration owner to avoid competing tranches overwriting one another.

## Remaining coverage

Expand independently into CAS, build/buy sets, animation/poses, accessibility and lesser-known creators; resolve actual profile IDs only from provider evidence. Record version-specific release dates, direct public media references and rights review without copying assets. Never infer bans, relocation, abandonment, aliases or compatibility from missing links or matching names. The broader Pro discovery response is a source of leads, not blanket verified database content.
