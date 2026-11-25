# Lot 51: exact file listings and conditional requirements

This optional offline SQLite pack keeps six [Lot 51](https://lot51.cc/)
projects and three related dependency/integration records separate from the
default catalog. The same format can be queried by the existing read-only manager.

| Project | Exact publisher file | Primary game tag displayed by CurseForge |
| --- | --- | --- |
| [Core Library](https://lot51.cc/mods/core-library) | [1.43.1](https://www.curseforge.com/sims4/mods/lot-51-core-library/files/9258134) | 1.128.90 |
| [Fashion Authority](https://lot51.cc/mods/fashion-authority) | [2.5.2](https://www.curseforge.com/sims4/mods/fashion-authority/files/9256147) | 1.128.90 |
| [Suite Life](https://lot51.cc/mods/suite-life) | [1.11](https://www.curseforge.com/sims4/mods/suite-life/files/7587564) | 1.121.342 |
| [Simzlink](https://lot51.cc/mods/simzlink) | [1.10.5](https://www.curseforge.com/sims4/mods/simzlink/files/8482974) | 1.126.58 |
| [PlumbBros](https://lot51.cc/mods/plumbbros) | [2.4](https://www.curseforge.com/sims4/mods/plumbbros/files/7698717) | 1.121.372 |
| [Sunrise](https://lot51.cc/mods/sunrise) | [1.4](https://www.curseforge.com/sims4/mods/sunrise/files/8815113) | 1.127.41 |

The [creator’s 1.129.39 status](https://lot51.cc/game-release/1.129.39)
is a separately attributed compatibility declaration. These tags are not
max-version requirements, byte equivalence across hosts, or an independent game test.

[Suite Life](https://lot51.cc/mods/suite-life) needs Core Library at least
1.38 and **exactly one** supported venue list. For ordinary worlds the author
allows [Universal Venue List](https://basementalcc.com/mods/universal-venue-list/)
or [Venue Changes](https://zerbu.tumblr.com/post/160347810775/the-sims-4-mod-venue-changes);
use Venue Changes only for a Vacation World. The underlying
`mod.requirement_choice.v1` row retains that rule. The manager’s v2 Relationships view lists these
in **Alternative**, not **Required**. The legacy view leaves new roles
unclassified. Nothing is installed. Sunrise’s optional Home Hub integration and Simzlink’s integration
with [SNB – Bills](https://www.simrealist.com/snb-bills) are not hard dependencies.

No file bytes, hashes, current bug counts, unattended downloads, install
decisions, or blanket redistribution authorization were inferred. Query the
latest author and publisher evidence before selecting a file.

```sh
cargo run --locked --example seed_reviewed_bundle -- --sims4-lot51 sims4-lot51-reviewed.sqlite
python tools/verify_lot51_snapshot.py sims4-lot51-reviewed.sqlite --html sims4-lot51-reviewed.html
cargo run --locked --example query -- sims4-lot51-reviewed.sqlite curseforge:project:1146500
```

## Before removing or replacing files

These are first-party maintenance advisories, not an installation scan or
a game-tested repair. The database and expandable offline notes distinguish:

- **Game save:** [Suite Life’s hotel guide](https://lot51.cc/node/534) warns
  that Hotel lots can be cleared if you save without Suite Life. Preserve a
  separate copy of the affected game save and do not overwrite it during a
  test without Suite Life.
- **Generated presets:** [Lot 51’s data-file note](https://lot51.cc/node/710)
  identifies `lot51_hotels.dat` as shared Kitchen Menu Presets. Keep it
  across updates; it does not substitute for the affected save backup.
- **Thread data and referenced CC:** [Fashion Authority’s usage notes](https://lot51.cc/node/571)
  say Threads are saved per game slot in `lot51_fashion_authority_data`,
  beside the script. Protect that directory separately from actual outfit
  CC. Removing referenced outfit CC can damage a Thread even if the mod
  itself is still installed.

An advisory describes the conditions in the cited guide, not proof that a
given user installed the mod, experienced the issue, or has a complete
backup. The pack never runs, modifies, or restores a save.
