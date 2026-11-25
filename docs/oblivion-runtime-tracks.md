# Oblivion and Oblivion Remastered runtime tracks

Keep the classic 2006 release and Remastered as separate game IDs even where
their master filename matches. The six game-scoped project records (five
distinct projects, because LOOT has two contexts), five credited publisher
labels, and one platform update note preserve 31 facts and evidence entries.
No game, mods, account sessions, libraries, script extender binaries,
patch files, or offsets are included.

- [xOBSE](https://github.com/llde/xOBSE) is the classic extender. Its English
  original 1.2.0.416 runtime evidence is separate from the extender version,
  other storefronts/localizations and the remaster.
- [OBSE64](https://github.com/ianpatt/obse64) is the Remastered extender.
  The [publisher listing](https://www.nexusmods.com/oblivionremastered/mods/282)
  labels 0.2.2 as alpha and claims Steam executable build 1.512.105.0.
  The publisher excludes Game Pass, Microsoft Store and Epic for that
  release. An empty GitHub releases list is not no upstream distribution.
  Public project routes are retained, no exact Nexus file IDs promoted.
- [Address Library](https://www.nexusmods.com/oblivionremastered/mods/4475?tab=files)
  distinguishes the v2 player package from the v2 developer offsets at the
  same claimed Steam target. The project header version is not a file key.
- [UORP](https://www.nexusmods.com/oblivionremastered/mods/477?tab=files)
  distinguishes the base patch 1.0.9 from the Deluxe companion 1.0.2, which
  requires the base. The [author's version history](https://www.afkmods.com/Unofficial%20Oblivion%20Remastered%20Patch%20Version%20History.html)
  gives build label 235906. It is not equated to the [Bethesda update label 1.2](https://bethesda.net/en-US/news/the-elder-scrolls-iv-oblivion-remastered-update-1-2)
  or to an executable's Steam version. Exact Nexus file IDs/digests are unknown.
- [LOOT 0.29.2](https://github.com/loot/loot/releases/tag/0.29.2) detects the
  Remastered PC game separately; the source contains Steam and Microsoft
  Store branches. That is not evidence that ANY extender or mod works on
  both. The similarly named [masterlist repository](https://github.com/loot/oblivion-remastered)
  describes itself as unused; this pack does not name a default rules file
  or apply any rules.

Six exact GitHub asset IDs across four releases preserve the publisher
dates, software channels, sizes and digests where present. This checks
metadata, not the contents or the runtime behavior of an archive.
Reported issue count is unknown, not zero. No installation, update,
Nexus restrictions bypass or game compatibility testing was performed.

```sh
cargo run --locked --example seed_reviewed_bundle -- --oblivion-runtime-tracks oblivion-runtime-tracks.sqlite
cargo run --locked --example query -- oblivion-runtime-tracks.sqlite oblivion-remastered:project:obse64
```

The optional mode builds atomically beside the chosen new output. It
checks replay and the reopened database without altering the current
catalog or attempting to read a game directory.
