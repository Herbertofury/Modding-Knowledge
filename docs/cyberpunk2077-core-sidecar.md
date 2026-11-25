# Cyberpunk 2077 core release and relationship evidence

Optional native sidecar, reviewed November 25, 2026: 13 original-project
listings, eight provider publisher identities, 16 releases and 42 asset
records. GitHub tags/IDs, channel, publication time, size and provider
digest are kept together. No archive was downloaded, matched against a
local file, or installed. Release channel and compatibility are independent.

- [The publisher's 2.5 game announcement](https://www.cyberpunk.net/en/news/52125/cyberpunk-2077s-edgerunners-2-update-2-5-is-out-now)
  establishes the game's update, not the compatibility of any mod.
- [Let There Be Flight v0.4.7](https://github.com/jackhumbert/let_there_be_flight/releases/tag/v0.4.7)
  distinguishes four required libraries, an optional ArchiveXL dependency
  on game 2.5+, and optional Mod Settings for menu/key rebinding. Its
  reported save-lock symptom was not reproduced by the maintainer and is not
  a proved save corruption case. It is not promoted to exact-runtime health.
- [WolvenKit 9.1.1](https://github.com/WolvenKit/WolvenKit/releases/tag/9.1.1)
  directs 2.5 users to 10.x. The latest stable release label alone must not
  drive a 2.5 recommendation. The 10.x author beta and separate nightly
  channel are retained as such, even if their mirror flags differ.
- [ArchiveXL v1.28.11](https://github.com/psiberx/cp2077-archive-xl/releases/tag/v1.28.11)
  documents a garment-load crash fix. This does not prove runtime behavior
  for a particular load order. PDB bundles are symbols, not the mod archive.

Run without modifying your current catalog or installing any mod:

```sh
cargo run --locked --example seed_reviewed_bundle -- --cyberpunk2077-core cyberpunk2077-core.sqlite
cargo run --locked --example query -- cyberpunk2077-core.sqlite cyberpunk:project:let-there-be-flight
```

Unlisted dependencies and report counts remain unknown. This sidecar does
not alter the aggregate reviewed bundle or app integrations.
