# Skyrim FormList Manipulator: separate source, edition and alternative evidence

This reviewed pack ties the [original creator's source](https://github.com/MaskedRPGFan/FormList-Manipulator/tree/82a694e5d33194ae223f23846b1f180ee394cdee) to the [original Nexus listing](https://www.nexusmods.com/skyrimspecialedition/mods/74037) without equating the artifacts. CMake identifies the pinned source as 1.7.0.0; the original Nexus page shows 1.8.1 and says the linked source is outdated. The source MIT license does not settle the archive or installed DLL bytes.

The pinned README describes alphabetical reads under Data before Data/FLM, `_FLM.ini` naming and cross-file alias/filter/group visibility. Desktop and VR Address Libraries are separate. The original project's later description labels MergeMapper optional and describes ordering with KID when present, not an unconditional requirement to install KID. The Tweaks "Load Editor IDs" setting and the VR dependency minimum are publisher statements for the project, not exact archive acceptance.

The [drLemis patch](https://www.nexusmods.com/skyrimspecialedition/mods/189947) and [abcd99 NG](https://www.nexusmods.com/skyrimspecialedition/mods/193349) are distinct alternatives with distinct authors, even if their publisher describes replacement. Neither has permission to overwrite the original from this data, and no files, hashes, save compatibility or runtime were verified. Nexus descriptions were available indexed; file-tab direct requests returned 403. Skip guesses and do not synthesize download URLs.

Seed a new offline file only: `cargo run --locked --example seed_reviewed_bundle -- --skyrim-flm NEW.sqlite`. All observations are partial and the normal snapshot is unchanged.
