# One offline catalog for the reviewed early Elder Scrolls packs

The optional composite opens the three separately reviewed packs as one
SQLite file, covering Arena, original Daggerfall data, Daggerfall Unity,
Morrowind, classic Oblivion and Oblivion Remastered. It has 35 entities and
92 reviewed observations. The underlying original datasets stay separate
and unchanged. This is not the default aggregate catalog.

The new build refuses repeated game/source/provider identifiers or
invalid schemas rather than silently overriding them. Every source
policy, evidence row, exact file ID, issue-count scope and edition
boundary is kept. Nothing gets installed, fetched, launched or tested in
a game. Bundled data remains metadata-only.

```sh
cargo run --locked --example seed_reviewed_bundle -- --classic-tes-reviewed classic-tes-reviewed.sqlite
cargo run --locked --example query -- classic-tes-reviewed.sqlite daggerfall-unity:project:dfu
```

- [Morrowind tracks](morrowind-engine-tracks.md)
- [Oblivion tracks](oblivion-runtime-tracks.md)
- [Arena and Daggerfall tracks](early-tes-engine-tracks.md)
