# CurseForge exact-file relationship evidence

This offline review example uses the [official exact-file response](https://docs.curseforge.com/rest-api/#get-mod-file) and [FileDependency contract](https://docs.curseforge.com/rest-api/#filedependency). It accepts bytes from a response the operator is already allowed to retain and review; it does not contact CurseForge, handle keys, copy download URLs, publish records, or make an install decision.

```sh
cargo run --locked --example review_curseforge_file -- 99 111 222 < examples/research/curseforge_relationships/synthetic-response.json
cargo test --locked --example review_curseforge_file
```

The three command arguments are **expected numeric game, mod, and exact file IDs**. All must match the returned envelope. The example does not interpret project-level aggregates or HTML lists as exact-file evidence. The JSON projection keeps embedded library, optional, required, tool, incompatible and include distinct and retains the provider code even for an unknown role. More than one role for a target stays visible without inventing a resolution. The documented dependency row carries a target **mod ID**, not a target file. Loader/runtime/version resolution, access, hash verification and whether a file is distributable are independent decisions. A valid recorded relation is never itself installation permission.

- Missing and null dependencies give `not_reported`; an actual empty array gives `reported_empty`, a statement only about that one response.
- Invalid scope, duplicate selected fields, malformed payloads and out-of-budget records fail without a partial projection. The limit is 512 KiB and 1,024 relations. Provider-supplied download URLs or unrelated fields are not copied.
- When a dependency replacement is proposed, verify the exact target's loader/game and reviewed constraints rather than selecting by similar display name. Keep last-good evidence after an authorization or transient error; never treat error as a row removal.
- [CurseForge's API requirements](https://docs.curseforge.com/rest-api/) and [third-party API terms](https://support.curseforge.com/support/solutions/articles/9000207405-curse-forge-3rd-party-api-terms-and-conditions) are separate from this local review. No authenticated lookup was attempted.

## Real public example, not a captured API response

[SPMovesets on CurseForge](https://www.curseforge.com/minecraft/mc-mods/spmovesets) is project **1761617**. The author-listed [Survival Update file **9634311**](https://www.curseforge.com/minecraft/mc-mods/spmovesets/files/9634311) is for Forge Minecraft Java **1.20.1**, published November 15, 2026. The [project-level relations page](https://www.curseforge.com/minecraft/mc-mods/spmovesets/relations/dependencies) displayed GeckoLib and playerAnimator as required. The [exact file's public related-projects view](https://www.curseforge.com/minecraft/mc-mods/spmovesets/files/9634311/dependencies) did not expose usable rows in the reviewed rendering. That is **unknown exact-file coverage**, not a confirmed empty authorized API array. No dependency file IDs, automatic installation, game verification, redistribution or successor substitution are asserted. The project lists All Rights Reserved; nothing from its binary, art, or protected pages is copied.

The current Enderloom launcher parser maps declared embedded/optional/required/incompatible roles. This offline research retains tool/include/future codes as evidence, without changing the launcher's install semantics or introducing a dependency on any consumer.
