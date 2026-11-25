# Sims 4: Frankk and Simsonian Library

This optional portable catalog covers 14 public Frankk mods, seven
Frankk/S4TK tool records, and 21 entries from the Simsonian Library's public
catalog. Four linked projects are included to resolve explicit relationship
or lineage references. A catalog listing does not establish download access,
release currency, game compatibility, or permission to redistribute.

- [Frankk's pinned site data](https://github.com/frankkulak/mods-web-data/tree/958853a50847ca18d353c222a6d8dc4af9f9503f/build)
  contains an exact February 2026 bulletin for Windows and macOS. That dated
  author report is not current patch acceptance. Only the 14 public-index
  mods are included, not the unlisted Simway work in progress.
- [Simsonian Library's newer catalog](https://simsonianlibrary.com/) takes
  precedence over its older Wix mirror. For example its Archaeology Kit
  changelog adds 2.0.13. Patreon landing pages require a manual selection;
  package bytes and public availability have not been checked.
- [Classic RPG Spells](https://simsonianlibrary.com/classic-rpg-spells/index.html)
  requires Spellbook Injector V2. [Archaeology Kit](https://simsonianlibrary.com/archaeology-kit/index.html)
  only requires it for the included custom spells. The kit's documented
  conflict with Small Archaeology Overhaul causes its dig piles to stop
  yielding artifacts. S4TK's translation tooling is a developer credit,
  not a player dependency.
- [Simsco Candy Factory](https://simsonianlibrary.com/candy-factory/index.html)
  integrates with, but does not require, Simsco Canning Factory.
  [Pirate Treasure](https://simsonianlibrary.com/pirate-treasure/index.html)
  explicitly requires XML Injector and Island Living.
- Provider names and public project pages are preserved. Direct Google Drive
  sharing URLs are deliberately omitted; use the author page.
- No catalog file is overwritten or combined by this command. An unknown
  issue count or a missing conflict report does not mean zero or compatible.
  No mod archive, account, install, current game test, or automatic
  acquisition is part of this sidecar.

```sh
cargo run --locked --example seed_reviewed_bundle -- --sims4-creator-relationships Sims4-Creator-Relationships.sqlite
cargo run --locked --example query -- Sims4-Creator-Relationships.sqlite simsonian:project:archaeology-kit
```
