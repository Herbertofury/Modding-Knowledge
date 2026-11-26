# MC Command Center: exact release pairs and author conflict evidence

This optional offline snapshot records [Deaderpool's public release table](https://deaderpool-mccc.com/downloads) as observed November 26, 2026 UTC. It does not install mods, update the default snapshot, or establish compatibility with any later patch.

| Paired releases | Windows build | macOS build | Lane |
| --- | --- | --- | --- |
| 2026.6.0 | 1.129.39.1030 | 1.129.39.1230 | Standard |
| 2026.5.0 | 1.128.90.1030 | 1.128.90.1230 | Standard |
| 2026.4.0 | 1.127.41.1030 | 1.127.41.1230 | Standard |
| 6.6.0 | 1.56.52.1020 | 1.56.52.1220 | Legacy Edition |

The release dates of the mods weren't observed; the dates in the data refer to game patches. The author tables identify eight distinct Google Drive file IDs, not available verified ZIP bytes. Keep the author links as manual download references. No file was acquired, so hashes, sizes, filenames, redirect accessibility and byte equivalence to alternate hosts are not asserted. Patron early builds are not in this pack.

[Module documentation](https://deaderpool-mccc.com/documentation) calls for installed modules to match the core version. [MC Woohoo](https://deaderpool-mccc.com/downloads) is a separate optional project when considering the core; installing the optional project requires the exact same core version. Never apply a Legacy package to a standard release lane based on name alone.

[Deaderpool's conflict notes](https://deaderpool-mccc.com/troubleshooting) provide scoped publisher reports, not independently verified conflicts on particular game builds. The [AutoSave listing](https://modthesims.info/d/562569/autosave-for-sims4.html), [TwistedMexi catalog](https://www.twistedmexi.com/Mods/) and [Candyd listing](https://modthesims.info/download.php?t=591183) corroborate target identity, not current breakage. The Candyd conflict is attached only to MC Woohoo. Three core target identities remain unresolved; Cowplant Tweaks and InTeen are retained separately as qualified warnings, not converted into additional confirmed conflicts. Never remove or disable installed mods solely on this evidence.

```sh
cargo run --locked --example seed_reviewed_bundle -- --sims4-mccc sims4-mccc-reviewed.sqlite
python tools/verify_mccc_snapshot.py sims4-mccc-reviewed.sqlite
```
