# Genre-aware game knowledge and endgame pillar

[Reviewed evidence](../data/game-progression-evidence.json) complements [existing design studies](game-design-studies.md). Canonical discovery stays in [Games](https://github.com/Herbertofury/Games); stable Steam identities can join without fuzzy title matching. This tranche has eight source-reviewed references, not a comprehensive best-games ranking, code audit, media archive or live Athena integration. No gameplay was installed or tested. A publisher description is attributed evidence, not an independent quality verdict.

## Durable product goal

Athena/databases must retain and keep developing knowledge of released, early-access and upcoming survivors-like roguelites, RTS strategy games and top-down action RPG/adventures. Each genre deserves a genuinely deep game, not a reskin of one underlying minigame. Discover promising work, study actual implementations when legitimately available, preserve evidence and turn lessons into tested features. Maintain launch/version changes, developer identities, official links, media provenance, strengths, limitations, design questions and review dates. Do not infer that a missing item is unreleased or a failed fetch means abandoned.

The query supports genre/status/text filters and due-for-review extraction. It is local, read-only and deterministic with `--as-of`; it does not itself schedule or perform network refreshes. Every record remains visible after becoming stale. There are no verified upcoming entries in this initial tranche; the coverage queue is explicitly pending, not silently fulfilled by old games. Expand discovery in Games before adding deeper study records here. Retain prior evidence when new patch mechanics differ.

```sh
python tools/query_game_progression.py --genre survivors_like
python tools/query_game_progression.py --status historical_reference
python tools/query_game_progression.py --search extraction
python tools/query_game_progression.py --as-of 2026-12-01 --due-only
python -m unittest discover -s tests -p 'test_game_progression.py' -v
```

## Source-derived lessons, not automatic adoption

- Vampire Survivors: readable carry-over upgrades and reversible build experimentation.
- Halls of Torment: 30-minute runs, extracted equipment, class synergies and quest metaprogression provide several progression layers.
- Deep Rock Galactic: Survivor: mining, route decisions, objectives and extraction create pressure beyond enemy-count growth. Its observed Steam release is September 17, 2025, not an assumption of ongoing early access.
- Zero Hour: EA's actual source release makes deeper code study possible. This tranche reviewed the announcement, not the source; source availability is not permission to reuse all assets or proof of portability.
- Diablo III 2.0.1 explicitly removed its Paragon cap; Diablo IV 2.0 explicitly set a maximum of 300. These are historical version-scoped observations. The user's uncapped progression goal remains valid as an original requirement; do not mislabel both games as universally uncapped.
- PoE's historical Siege of the Atlas and PoE2's observed store description demonstrate specialization, endgame crafting, boss paths and multiple encounter types. They inspire depth, not a claim that we have shipped equivalent scope.

All factual sources, observations, proposed adaptations and acceptance questions are separate in the JSON. Original prose only; media remains source-linked with no retained files. Assets need independent rights/provenance before retention or redistribution.

## Genre-specific acceptance targets

**Survivors-like:** configurable 10/20/30-minute and endless experiences; adapt pacing, loot, boss cadence and build completion to each length rather than time-stretching. ModHunter's more specific request wins: main runs start at 20 minutes, with 20/30/60/120/endless choices; 10-minute portal side sessions are separate. Endless means no hidden forced victory timer, but still needs bounded entity/memory budgets, numerical overflow tests, checkpoints and explicit extraction. Deep post-cap mastery must add choices and goals, not solely infinitely inflated health bars. Test build diversity, viable off-meta routes, readable enemy patterns, movement, pickups, audio and configurable accessibility.

**RTS:** economy, construction, production queues, technology, faction asymmetry, scouting/fog, unit counters, army control and pathfinding. Endgame includes strategic objectives, campaign consequences, challenge maps and strong AI/opponent variety. Do not impose auto-shooting or endless stat advantages onto competitive matches. Test command responsiveness, replay/simulation determinism where applicable, large armies, recovery from economic losses and map balance.

**Action RPG/adventure:** exploration, combat/skill mastery, loot/crafting, quests, bosses, build respecialization and persistent world/progression. Endgame requires player-directed encounter routes, target farming, risk/reward, pinnacle challenges and multiple meaningful build archetypes. Hours played, number of affixes and a perpetual XP meter do not establish depth. Use progression milestones and verified encounter coverage instead of asserting PoE-scale completion.

## Cross-mode rewards without coercion or broken economies

The prior Athena design already connects Greek-god roster/lore and optional rewards while requiring an independently satisfying progression path in each mode. Preserve that, and give every mode substantial endgame. Cosmetic, collection and shared mastery rewards can connect modes without forcing an RTS player to grind action runs. Any transferable power needs explicit normalization and economy tests. Keep competitive parity separate from personal unlimited mastery.

Acceptance before shipping: durable event ID and reward-ledger version; at-most-once award under duplicate completion/retry/import; atomic save; resume after interrupted grant; no currency multiplication through conversion loops; rule-version migration and rollback; clear preview of what transfers and what stays mode-local. These are requirements, not an implemented reward service. Do not transfer learning achievements as professional certifications.

## Next work and coverage limits

- Source audit: pinned Zero Hour simulation/command/replay systems and appropriate open foundations; no claim of completed source-level parity.
- Discovery: expand survivor, RTS and ARPG release coverage and upcoming verification from official developer/news sources. Track expansions separately from base games.
- Consumer integration: expose these records and review queue in Athena's visual library; measure actual usage before claiming ingestion.
- Experiments: benchmark one meaningful endgame slice per genre, then increase content depth. Preserve raw performance, encounter and reward-integrity results.
- Scheduled research should update only observed changes, preserve last-good records, back off failures and report meaningful releases/design findings. This CLI is not that scheduler.
