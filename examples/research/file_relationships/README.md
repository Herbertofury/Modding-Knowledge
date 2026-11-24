# Exact-file relationship review (offline Rust example)

This is a local, offline review command. It reads one JSON object from stdin
(maximum 64 KiB) and only emits JSON to stdout on success. There is no default
packet, input path option, network access, database operation or consumer integration.

```sh
cargo run --locked --offline --example review_file_relationships -- fixture-host fixture-game 8 42 < examples/research/file_relationships/synthetic.json
cargo test --locked --offline --example review_file_relationships
```

The example uses invented IDs and a reserved `example.test` source. It does not
open any links. Pass the independent expected **provider, game, project ID and
file ID** as four separate command-line arguments; the packet and every piece of
evidence must match them exactly. IDs are opaque, case-sensitive strings.

Use `synthetic.json` for the required names and nesting: top-level `subject`,
`compatibility`, and `relationships`; no additional fields, including permission,
are accepted. Records must be named JSON objects, not positional arrays; duplicate
keys at any level, trailing JSON, excessive size, and unsupported roles or shapes
are rejected with a nonzero exit and empty stdout. Errors do not echo supplied
evidence. There are at most 128 relationships and at most 128 reported build
labels. Do not send private URLs, credentials or personal data.

A review is anchored to an exact provider/game/project/file tuple. Every evidence
record must identify that same subject. A target file must belong to the qualified
target project even if its unqualified ID matches another file; cross-provider
targets are possible, but names and URL substrings do not join identities. Source
IDs and exact HTTPS references are retained as attributed inputs, not verified
ownership. Project-level declarations cannot be upgraded into exact-file evidence
without an actual file-scoped reference.

Requirements, optional dependencies, works-with, conflicts and conditional
load-after remain literal roles. `embedded` is separately attributed bundled
content, not an install requirement; provider examples include [Modrinth
embedded dependencies](https://docs.modrinth.com/api/operations/getversionfromidornumber/)
and [CurseForge EmbeddedLibrary](https://docs.curseforge.com/rest-api/#tocS_FileRelationType).
`patch_for` labels a patch target without asserting that the target must be
installed. Either role still requires exact scope and evidence; a provider
version-level or project-level fact alone cannot establish a file-scoped claim. Author statements and structured provider fields
retain distinct provenance in `unverified_attributed_source_statements`; inferred
candidates go into `unverified_candidates` and
must not be presented as declared dependencies. Version constraints remain
unknown/unspecified, a qualified exact file, or a raw stated minimum label with
its applicability. A floor that applies only when a target is already installed
cannot become a required install. This does not compare labels as versions or
evaluate arbitrary conditions, alternate groups or DLC logic.

Compatibility status is `unknown`, `unavailable`, or `provider_reported` with
nonempty raw build labels and source evidence tied to that exact file. Zero relationship rows are not a confirmed
none; the example does not assert complete coverage. No output is a compatibility,
availability, download, safety, rights or admission decision.

This fills a different layer than the [Nexus response shape decoder](../nexus_relationships/)
or the [pinned Oblivion LOOT projection](../../../docs/loot-import.md).
Neither provider logic is copied. Existing [source admission](../../../SOURCES.md)
and [storage policies](../../../docs/storage-api.md) remain unchanged.
