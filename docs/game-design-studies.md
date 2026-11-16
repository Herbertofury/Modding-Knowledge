# Shared RTS and roguelite design studies

[Structured study layer](../data/game-design-studies.json) · [Canonical Games collection](https://github.com/Herbertofury/Games) · [RTS](https://github.com/Herbertofury/Games/wiki/Real-Time-Strategy) · [Roguelites](https://github.com/Herbertofury/Games/wiki/Roguelites)

Games already owns a broad canonical collection. This shared database adds **10 stable game references and 3 foundation candidates**, not a duplicate catalog or a claim to list the objectively best games. It is a separate JSON dataset, not yet imported into the mod-compatibility SQLite artifact. Athena, GameSync and other apps can consume it without rebuilding discovery metadata. No app has been wired to it in this change.

## Read-only consumption

```sh
python tools/query_game_studies.py --kind inspiration
python tools/query_game_studies.py --kind foundation_candidate
python tools/query_game_studies.py --catalog /path/to/Games/data/catalog.json
python -m unittest discover -s tests -p test_game_studies.py -v
```

Stable IDs join against Games `items[].id`, never fuzzy names. The manifest records the reviewed Games commit. Supply that revision for reproducible resolution; newer catalog revisions may remove entries, reported as `missing`. Resolution proves ID membership only: the tool does not cryptographically verify the supplied catalog revision. It reads a trusted user-selected local JSON file, not a bounded untrusted-input sandbox. The tool does not download, merge or silently replace records. Records are alphabetically sorted, filterable by purpose and explicitly `not_tested`.

## Evidence and reuse boundaries

Game identities and source URLs were checked against the existing pinned Games snapshot. Individual gameplay study questions are **editorial hypotheses**, not claims of measured mechanics, hours, quality, or hands-on tests. Follow their official store links before deeper comparison. Existing ratings do not prove design suitability. No ratings, trailers, screenshots or copyrighted descriptions are copied here.

The three foundation overview/license sources were read on 2026-11-16. Godot's engine is MIT; OpenRA provides GPL v3 text; 0 A.D. states GPL code and CC BY-SA art/sound. These are screening facts, **not blanket clearance**. Pin a candidate source revision, check file-level licenses/dependencies and separately audit assets before incorporating anything. OpenRA's original commercial-game assets are not licensed by its engine license. 0 A.D.'s old GitHub mirror is archived; current development moved to Wildfire Games Gitea. No code, assets, engines or games were installed or benchmarked. The shared repository's own unassigned overall code license remains unchanged.

## Athena / GameSync design target, not delivered gameplay

- Athena and recruited Greek gods fight hordes in the action mode. A full RTS separately supports economies, construction, armies, god abilities, research, scouting and counterplay.
- Shared roster/lore and optional rewards connect modes; neither should become a mandatory grind or stripped-down minigame for the other. Each mode needs an independently satisfying progression path.
- Course and subject mastery can unlock themed boss challenges, perks and cosmetics. A simulation badge is not a professional certification. Completion must be idempotent; imports and repeated rewards need versioned proof, revocation/error handling and abuse tests.
- A 100+ hour campaign and extensive endgame are **content aspirations**, not measured playtime, implemented scope or a deadline promise. Count distinct encounters, builds, maps, factions and meaningful decisions rather than inflated repetition.
- Start with one god, one arena, one RTS map, one economy loop and one optional cross-mode reward. Measure combat readability, frame times, input latency, save integrity and whether either mode pressures players into unwanted chores before scaling content.
- Endgame evaluation should test boss modifiers, build diversity, strategic map changes, repeatability and accessibility. Avoid copying named franchises' characters, music, assets, maps or distinctive presentation.

## Expansion and acceptance

Keep broad discovery in Games. Add shared study rows by exact identity, source/review date, purpose and explicit uncertainty. New engine candidates need code/asset license separation and target-platform checks. No engine wins from stars, popularity or microbenchmarks alone. Retain raw runs, hardware/settings, frame-time distributions, memory, entity counts, pathfinding/AI load, save/reload and soak results from a representative combined prototype before recommending a foundation. Future empirical records need an evidence schema rather than changing `not_tested` to a marketing label.
