# Keyword Item Distributor staging review

This read-only preflight complements the release records and FLM staging review. Point it at a disposable, explicit Data staging directory, not an open game or a profile with active writers.

```text
python -B tools/review_skyrim_kid_layout.py /path/to/staged/Data
python -B tools/review_skyrim_kid_layout.py /path/to/staged/Data --counts-only
```

The [Nexus instructions](https://www.nexusmods.com/skyrimspecialedition/mods/55728) recommend the `_KID.ini` suffix. Pinned [KID 4.1.0 source](https://github.com/powerof3/Keyword-Item-Distributor/blob/99adaaa1d8cefe320524802b57e91723f66b1c7c/src/LookupConfigs.cpp) delegates discovery to a pinned [CLibUtil helper](https://github.com/powerof3/CLibUtil/blob/aca80148853a8c0cb3e3cab3eaa20f1886cc99e7/include/CLIBUtil/distribution.hpp). The helper walks one directory level, tests the exact lowercase extension and checks whether the case-sensitive marker occurs anywhere in the path. The engine calls it on relative `Data\`; a filename like `A_KID-review.ini` is a candidate at these pins. `A_kid.ini`, `A_KID.INI`, and nested files are not candidates. This is not a recommendation to break the documented naming convention, and does not prove any downloaded DLL matches the source.

Only the unnamed INI section is considered by the pinned caller. The report samples basic single-line UTF-8 assignments there, preserves repeated declarations and distinguishes ordinary keyword forms, ANY/ALL filter contexts, exclusions and [exclusive groups](https://github.com/powerof3/Keyword-Item-Distributor/blob/99adaaa1d8cefe320524802b57e91723f66b1c7c/src/ExclusiveGroups.cpp). Unsupported syntax is not guessed. [Filter resolution](https://github.com/powerof3/Keyword-Item-Distributor/blob/99adaaa1d8cefe320524802b57e91723f66b1c7c/src/Data/BaseFilters.cpp) is more complex than text slicing: the report never turns these candidates into unconditional dependencies. Staging name presence doesn't prove game load, an exact form exists, compatibility, or that anything is safe to delete. The inspector does not evaluate virtual filesystems, FormIDs, merge maps, installed games, precedence, or save state.

The inspector refuses nonordinary matching files and bounds reads and output. It shares the Windows/Linux identity check with the FLM inspector, whose documented behavior and outputs are unchanged. Counts-only omits local filenames, plugin names and line numbers. No downloads or filesystem writes.
