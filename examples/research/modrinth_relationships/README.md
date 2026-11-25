# Review one exact Modrinth version
An opt-in, offline evidence reader. Supply a locally obtained, permitted
response for one immutable version ID and the expected project ID. This
tool does not make requests or follow links from input.

```sh
cargo run --locked --example review_modrinth_version -- Sample01 Version1 \
  < examples/research/modrinth_relationships/synthetic-response.json
```

The [official version endpoint](https://docs.modrinth.com/api/operations/getversionfromidornumber/)
documents project and version IDs, optional dependency/target information,
and the roles required, optional, incompatible, and embedded. A human version
number can be ambiguous; supply the actual version ID, not a display name.
The reviewed JSON response must match both expected IDs.

- No list (or null), an explicitly empty list, and reported rows are distinct.
  Only the exact version's reported coverage is stated.
- Project and version IDs are kept separately. Both names in a row are a
  provider assertion, not an independently verified target identity join.
  A filename by itself is unresolved literal evidence, never a path or download.
- All reported rows survive; optional and embedded rows never become required.
  Unknown future role strings are carried as unknown. Source loader/game lists
  describe the source version and establish no target compatibility.
- Malformed or oversized selected fields are rejected rather than partially
  projected. Raw files, author details, and changelogs are not copied.

For example, [Iris zsoi0dso](https://modrinth.com/version/zsoi0dso) identifies the
exact Iris release and publicly displays required-content information. That
listing is not an API response, cannot fill missing target IDs, and was not
passed through this parser. The bundled fixture is synthetic; its five rows
exercise the full field contract. This is not public catalog ingestion, source
aggregation, provider authentication, or an installer.
