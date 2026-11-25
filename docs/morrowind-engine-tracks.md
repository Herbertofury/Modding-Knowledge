# Morrowind engine tracks: reviewed portable sidecar

The edition is **The Elder Scrolls III: Morrowind**. Keep the original game
executable's extensions separate from replacement-engine mods. A working
Morrowind ESP/ESM on one engine is not evidence that its Lua, shader or
executable hooks work on the other. No Morrowind assets, executables, mod
archives, account information or inferred file permissions are included.

- [OpenMW downloads](https://openmw.org/downloads/) and official GitHub release
  metadata distinguish [0.51.0 stable](https://github.com/OpenMW/openmw/releases/tag/openmw-0.51.0)
  from [0.52.0 RC1](https://github.com/OpenMW/openmw/releases/tag/openmw-52-rc1).
  Windows installer, Linux archive, macOS architecture-specific disk images,
  and debug assets are distinct records. The [OpenMW manual](https://openmw.readthedocs.io/en/stable/reference/modding/extended.html)
  says that its Lua mods must target OpenMW, and that the MWSE format is not interchangeable.
- [MWSE](https://github.com/MWSE/MWSE) documents the MGE XE and nightly updater
  routes. The [rolling GitHub tag](https://github.com/MWSE/MWSE/releases/tag/build-automatic)
  retains its old release time and stable-looking provider flag while
  individual files carry current asset IDs, dates and digests. An unrelated
  historical 2.0.0 archive has no publisher digest in the reviewed endpoint.
- The indexed publisher [Tamriel Rebuilt release page](https://www.tamriel-rebuilt.org/downloads/main-release)
  says 26.08.23 requires [Tamriel_Data](https://www.tamriel-rebuilt.org/downloads/resources)
  26.08 or later and offers a choice of original executable plus Code Patch
  or OpenMW. These are **alternative engine paths**, not a request to
  install both. Direct Nexus/publisher text retrieval returned 403; indexed
  release/dependency excerpts are marked candidate/partial with no exact
  assets promoted. Same rule for [MGE XE](https://www.nexusmods.com/morrowind/mods/41102),
  [Morrowind Code Patch](https://www.nexusmods.com/morrowind/mods/19510) and
  [Patch for Purists](https://www.nexusmods.com/morrowind/mods/45096).
- GitHub asset SHA-256 values are publisher metadata, **not matched bytes**.
  Exact engine/platform compatibility, archives, current per-project issue
  counts, and licenses/redistribution terms for individual assets were not
  independently verified. No automated installation or user data changes.

The native store stages, imports and replays the reviewed embedded pack,
then closes and reopens it read-only to compare every observation before
publishing a new output file. It refuses to overwrite existing databases.

```sh
cargo run --locked --example seed_reviewed_bundle -- --morrowind-engine-tracks morrowind-engine-tracks.sqlite
cargo run --locked --example query -- morrowind-engine-tracks.sqlite morrowind:project:openmw
```
