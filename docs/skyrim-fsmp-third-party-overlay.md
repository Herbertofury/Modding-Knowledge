# Faster HDT-SMP: the separate v4.2 DLL overlay

[Third-party project 179975](https://www.nexusmods.com/skyrimspecialedition/mods/179975?tab=description)
is not the [official Faster HDT-SMP project 57339](https://www.nexusmods.com/skyrimspecialedition/mods/57339).
The requirements table still says V3.04; the separately dated v4.2 notes and the
[author's pinned patch guide](https://github.com/Tonycubed2/Fast-HDT-SMP-Variant-Patch/blob/0daf53ac6653e8f1fb1dd815b74dcff75e621be7/patches/README.md)
identify the [official 4.1.1 code commit](https://github.com/DaymareOn/hdtSMP64/commit/2215ec3ba09adf0843a3291ed0437a6ca6d385e2).
All 16 source hunks were independently checked on exact pinned blobs with no
fuzz or offset. This does not prove that downloaded DLLs use those source bytes.

The v4.2 author notes describe an override of `hdtsmp64.dll` and two mutually
exclusive CPU options, NonAVX and AVX2. The old GitHub release list is not
evidence of the newer Nexus files. The [pinned sample](https://github.com/Tonycubed2/Fast-HDT-SMP-Variant-Patch/blob/0daf53ac6653e8f1fb1dd815b74dcff75e621be7/patches/fsmp-4.1.1/smpExemptions.ini)
has two active NPC name substrings; those are not harmless blank examples,
and we do not know whether an exact distributed archive activates that file.
Effects on the separate legacy MCM are unknown. No files were downloaded,
compiled, or run, and no archives are mirrored.

Review explicit input without inspecting or modifying an installed mod:

```sh
python tools/review_fsmp_overlay.py --base 4.1.1 --variant avx2 --avx2 unknown
```

This reports missing evidence, the stated file overlap, and the exact required
base. Even a clean result is review only, not a game, CPU, or install verdict.
