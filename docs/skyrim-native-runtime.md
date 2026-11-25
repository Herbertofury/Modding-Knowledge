# Skyrim native runtime and file evidence

11 core projects across desktop and VR, 20 identifiable GitHub assets,
and exact reviewed runtime tuples. Provider digests were recorded where
offered. Archive bytes were not downloaded or verified. This does not
change the live catalog, a game profile or a mod manager.

- [SKSE’s official chart](https://skse.silverlock.org/) pairs Steam game
  1.7.104 with SKSE 2.3.1, GOG game 1.6.1179 with SKSE 2.2.6, legacy
  1.5.97 with SKSE 2.0.20, and VR 1.4.15 with SKSEVR 2.0.12.
  [The older official Steam 2.2.6 release](https://github.com/ianpatt/skse64/releases/tag/v2.2.6)
  explicitly targets 1.6.1170. The identical mod version does not mean
  it is the GOG file. The SKSE author chart excludes Game Pass and Epic.
- [Desktop Address Library](https://www.nexusmods.com/skyrimspecialedition/mods/32444)
  and [VR Address Library](https://www.nexusmods.com/skyrimspecialedition/mods/58101)
  are distinct. The indexed desktop 1.7.104.0 v13 label is preserved
  without guessing an exact Nexus file ID. The exact
  [VR v0.310.0](https://github.com/alandtse/skyrim_vr_address_library/releases/tag/v0.310.0)
  provider asset is recorded separately.
- [Engine Fixes beta5](https://github.com/aers/EngineFixesSkyrim64/releases/tag/7.0.21-beta5)
  remains labeled beta even though the hosting metadata does not set its
  prerelease flag. The FOMOD name is not exact runtime selection evidence.
- [Underwater NG's author](https://www.nexusmods.com/skyrimspecialedition/articles/12345)
  reports testing hotfix 1.0.1.4 on 1.7.104.0, SKSE 2.3.1 and Address
  Library v13 with the native renderer. SKSE Menu Framework is needed
  for its in-game menus, not the core. Its ESL backport condition applies
  only to games older than 1.6.1130. Other renderers and older builds
  were not retested for that hotfix.

```sh
cargo run --locked --example seed_reviewed_bundle -- --skyrim-native-runtime skyrim-native-runtime.sqlite
cargo run --locked --example query -- skyrim-native-runtime.sqlite silverlock:skse64
```

Unknown compatible file IDs, issue counts, licenses and test scope stay
unknown. There are no rehosted binaries, automatic downloads or game tests.
