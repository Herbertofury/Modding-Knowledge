# Hytale exact file and game-channel evidence

The optional SQLite sidecar retains 12 project listings, 11 credited uploader
accounts, and 17 file variants. Fifteen are [CurseForge](https://www.curseforge.com/hytale)
file identities; two are exact [HyperPerms GitHub assets](https://github.com/HyperSystems-Development/HyperPerms/releases/tag/2.10.0).
No archives were downloaded. License labels and public download pages do not
grant permission to rehost any asset.

- **Separate mod file status from game channel.** CurseForge marks both
  [Server Essentials 1.1.1](https://www.curseforge.com/hytale/mods/server-essentials/files/9433095)
  and [Server Info Commands 1.0.2](https://www.curseforge.com/hytale/mods/server-info-commands/files/9433130)
  as R, but the author explicitly says those files were built against
  0.8.0-pre.3. Neither is automatically a stable-game recommendation. Both
  projects' exact 0.7 files are listed separately.
- **Keep same-named archives distinct.** HyStash version 1.4.0 uses the
  same filename for [0.7.2](https://www.curseforge.com/hytale/mods/wonderous-wardrobes/files/9261993)
  and [0.7.6](https://www.curseforge.com/hytale/mods/wonderous-wardrobes/files/9486348).
  The author labels those separately and reports their own 0.7.6 test, not
  an independent compatibility test. HyStash was renamed within the same
  project; it is not the separate Wardrobe library.
- **Version labels are claims, not install evidence.** Portal World's author
  names Hytale Server 0.7.4, while the provider shows the wider 0.7 tag.
  This is not a statement about the newest server or the client.
- **Empty platform lists are scoped.** Four exact file pages list no related
  projects; that does not prove zero runtime prerequisites or apply to
  other releases. Missing issue counts and file hashes remain unknown.

The [stable game notes](https://hytale.com/news/2026/10/2026-10-13-hotfixes-update-7/)
and [Update 8 pre-release notes](https://hytale.com/news/2026/10/2026-10-22-pre-release-patch-notes-update-8/)
are separate first-party records, not tests of any mod.

```sh
cargo run --locked --example seed_reviewed_bundle -- --hytale-file-channels hytale-file-channels.sqlite
cargo run --locked --example query -- hytale-file-channels.sqlite curseforge:hytale:project:1782560
```

Build into a new output filename. The ordinary seed, live aggregate and
existing catalogs remain unchanged. No version was installed or game-tested.
