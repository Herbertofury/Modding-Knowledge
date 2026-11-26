# Minecraft Java: Sinytra's NeoForge compatibility lanes

Load **11 exact GitHub JAR identities** into a new local snapshot (no download):

```sh
cargo run --locked --example seed_reviewed_bundle -- --minecraft-sinytra sinytra.sqlite
python tests/test_sinytra_reviewed.py sinytra.sqlite
```

| Reviewed lane | Connector | Declared dependencies | Optional |
|---|---|---|---|
| 26.2 | [3.1.0-beta.4](https://github.com/Sinytra/Connector/releases/tag/3.1.0-beta.4%2B26.2) | Forgified Fabric API, Launchpad | Connector Extras |
| 26.1.2 | [3.0.0-beta.7](https://github.com/Sinytra/Connector/releases/tag/3.0.0-beta.7%2B26.1.2) | Forgified Fabric API, Launchpad | Connector Extras |
| 1.21.1 | [2.0.0-beta.17](https://github.com/Sinytra/Connector/releases/tag/2.0.0-beta.17%2B1.21.1) | Forgified Fabric API. The tagged guide does not list Launchpad. | Connector Extras |

Keep [26.2 install instructions](https://github.com/Sinytra/Connector/blob/3.1.0-beta.4%2B26.2/README.md) separate from the [1.21.1 install instructions](https://github.com/Sinytra/Connector/blob/2.0.0-beta.17%2B1.21.1/README.md), which explicitly warn against installing the original Fabric API alongside its replacement. The recent upstream policy marks 1.21.1 LTS and 1.20.1 EOL; older tagged docs differ. Do not replace one Minecraft lane with another or derive exact compatible dependency versions from these source relationships.

The [snapshot](../data/minecraft-sinytra-reviewed.json) records release tag, publisher asset ID, byte count and provider-supplied digest where present. One older Extras asset has no GitHub SHA-256 in the observed metadata. No JARs were downloaded or run. An official URL or checksum is not proof of installation compatibility or permission to mirror the file. Reporting counts remain unknown, not zero.
