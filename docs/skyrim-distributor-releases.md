# Skyrim distributor release-file index

This separate pack adds 16 publisher release/asset identities across
[SPID](https://github.com/powerof3/Spell-Perk-Item-Distributor/releases),
[KID](https://github.com/powerof3/Keyword-Item-Distributor/releases),
[Base Object Swapper](https://github.com/powerof3/BaseObjectSwapper/releases),
and [Seasons of Skyrim](https://github.com/powerof3/SeasonsOfSkyrim/releases).
Some publisher archives reuse a filename. Store the repository, release ID,
and asset ID rather than keying by any display label. Thirteen files have
provider SHA-256 metadata; three older files do not. No archives were acquired
or independently hashed.

Each project has tagged source evidence and separate desktop/VR Address
Library relationships. These are alternative requirements depending on which
variant is installed, not two mandatory dependencies for the same build.
The pinned CommonLib sources refer to the 1.7.99.0 AE constant. KID, BOS, and
Seasons have a conditional guard in their tagged source for earlier binaries
on newer address-library runtimes. Neither fact identifies which build is
inside an archive or verifies exact installed compatibility with Steam 1.7.104,
GOG, or VR. No unsupported tuple is promoted to an exact match.

```sh
cargo run --locked --example seed_reviewed_bundle -- --skyrim-distributor-releases skyrim-distributor-releases.sqlite
cargo run --locked --example query -- skyrim-distributor-releases.sqlite github:powerof3/Keyword-Item-Distributor
```

The sidecar uses the current native Store, and verification compares every
row after unchanged replay and read-only reopen. It does not overwrite the
main catalogue. No new executable services, performance assumptions, archive
redistribution rights, or mod-manager rollout are implied. The data pack adds
records only; no external optimizer is needed to rewrite them.
