# Arena and Daggerfall engine tracks

This optional pack has five projects, five publisher credits and an
original DOS data note. Keep original Daggerfall data, Daggerfall Unity
mods, and Arena's OpenTESArena engine separate. All project navigation
points to official publisher pages. No game, binary, archive, executable
offsets, or user data are included. No load order or installation is performed.

- [Daggerfall Unity's refreshed 1.1.1](https://github.com/Interkarma/daggerfall-unity/releases/tag/v1.1.1-cve-2025)
  uses the same four filenames as [the original 1.1.1](https://github.com/Interkarma/daggerfall-unity/releases/tag/v1.1.1)
  but they have different asset IDs. Some byte sizes changed. Upstream says
  Linux was republished unchanged, but the old published assets lack digests.
  The catalog does not claim independently identical bytes.
- [OpenTESArena 0.18.0](https://github.com/afritz1/OpenTESArena/releases/tag/opentesarena-0.18.0)
  distinguishes Raspberry Pi 5 from the older Pi 4 package, and tracks
  Windows x64, Linux x64 and macOS ARM64. The publisher reports additions to
  combat and services but still lists missing shop actions. This is not
  full-game parity and [original game data remain required](https://github.com/afritz1/OpenTESArena/wiki).
- [Quest Pack 1](https://www.nexusmods.com/daggerfallunity/mods/2?tab=files)
  has a desktop offer that names DFU 1.1.1 and requires [Quest Actions Extension](https://www.nexusmods.com/daggerfallunity/mods/677?tab=files).
  Its older Android courtesy port is labeled unsupported by the desktop
  author. The dependency also has separate desktop and Android offers.
  None of the Nexus offers is treated as a known exact file ID or SHA-256.
- [World of Daggerfall](https://www.nexusmods.com/daggerfallunity/mods/249)
  names a separate terrain component with exclusions according to the
  publisher's indexed excerpt. The full description/files returned a
  403, so scope and file identity remain limited. No workaround attempted.
- Cached Nexus bug-tab counts record exactly which source and scope was
  observed. A visible zero is not current or cross-provider proof of zero
  issues. No statuses, bug health ratings or runtime results inferred.

Sixteen exact GitHub file identities across four releases retain original
IDs, upload timestamps, byte sizes and digest metadata where supplied.
No game-file replacement or CVE reproduction is part of this pack.

```sh
cargo run --locked --example seed_reviewed_bundle -- --early-tes-engine-tracks early-tes-engine-tracks.sqlite
cargo run --locked --example query -- early-tes-engine-tracks.sqlite daggerfall-unity:project:dfu
```

The optional seed creates a new database then checks replay and read-only
reopening before publishing it. It does not overwrite the regular catalog.
