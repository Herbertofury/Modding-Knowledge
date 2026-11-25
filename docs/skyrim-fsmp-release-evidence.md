# Faster HDT-SMP: exact archive and installer evidence

This separate read-only sidecar is not a mod installer. It records five
maintainer GitHub files, including a separately marked development build,
and the latest-pointer match on November 25, 2026. Names, archive IDs, byte
counts and hashes are **provider metadata**, not independently verified ZIP
bytes. Nexus is the listing linked by the publisher; this catalog does not
equate its file IDs or binaries with the GitHub assets.

- [Maintainer and identity](https://github.com/DaymareOn/hdtSMP64/blob/2215ec3ba09adf0843a3291ed0437a6ca6d385e2/README.md)
- [Publisher v4.1.1 archive](https://github.com/DaymareOn/hdtSMP64/releases/tag/v4.1.1)
- [Exact installer XML source](https://github.com/DaymareOn/hdtSMP64/blob/2215ec3ba09adf0843a3291ed0437a6ca6d385e2/fomod%20tree/fomod/ModuleConfig.xml)
- [Exact build packaging source](https://github.com/DaymareOn/hdtSMP64/blob/2215ec3ba09adf0843a3291ed0437a6ca6d385e2/.github/workflows/build.yml)

## What a manager can show

1. Four CPU paths are an installer choice, not four independent updates.
   The presence of a path is not a speed ranking, a local CPU capability
   check, or permission to auto-select an option.
2. Before a 3.x-to-4.x change where the old MCM plugin is present, warn
   about the original maintainer's save dependency guidance. The installer
   has a conditional same-name stub choice. Do not remove that plugin or
   replay preferences without checking the user's exact profile and archive.
3. The readme associates SKSE Menu Framework with the configuration UI
   and the validator with authoring. Neither establishes a mandatory
   physics runtime dependency. Other requirements remain unknown.
4. Keep desktop SE/AE, GOG, and VR unconfirmed at the exact binary level;
   do not interpret a project-level platform claim as a verified DLL match.

Build into a new path:
`cargo run --locked --example seed_reviewed_bundle -- --skyrim-fsmp skyrim-fsmp-reviewed.sqlite`
