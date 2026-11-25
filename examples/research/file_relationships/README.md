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
load-after remain literal roles. Author statements and structured provider fields
retain distinct provenance in `unverified_attributed_source_statements`; inferred
candidates go into `unverified_candidates` and
must not be presented as declared dependencies. Version constraints remain
unknown/unspecified, a qualified exact file, or a raw stated minimum label with
its applicability. A floor that applies only when a target is already installed
cannot become a required install. This does not compare labels as versions or
evaluate arbitrary conditions, alternate groups or DLC logic. A single optional
`"applicability": {"type":"option_selected","option":"SMP shawl"}` records a
reviewed, nonempty option label separately from the existing target constraint.
Rows with this applicability appear only in
`unverified_conditional_source_statements` or
`unverified_conditional_candidates`, never the unconditional arrays. Unconditional
records and packets retain their old output shape. The tool does not determine
which options are installed or selected; it rejects compound conditions and
does not use labels to join unrelated options.

Compatibility status is `unknown`, `unavailable`, or `provider_reported` with
nonempty raw build labels and source evidence tied to that exact file. Zero relationship rows are not a confirmed
none; the example does not assert complete coverage. No output is a compatibility,
availability, download, safety, rights or admission decision.

This fills a different layer than the [Nexus response shape decoder](../nexus_relationships/)
or the [pinned Oblivion LOOT projection](../../../docs/loot-import.md).
Neither provider logic is copied. Existing [source admission](../../../SOURCES.md)
and [storage policies](../../../docs/storage-api.md) remain unchanged.

## Public Nexus file citation fixture

`mages-daily-outfit-smp.json` is based on [Nexus's indexed view for the
optional shawl file](https://www.nexusmods.com/skyrimspecialedition/mods/155656?show_file=667812&tab=files)
and the [parent outfit listing](https://www.nexusmods.com/skyrimspecialedition/mods/155656).
The [patch author's post](https://nasunosakuragawa.blogspot.com/2025/09/mages-daily-outfit-smp-patch.html)
credits Nasuno alongside Suna203 and gives additional author-specific setup/version
notes. The [FSMP target project](https://www.nexusmods.com/skyrimspecialedition/mods/57339)
is recorded as the required dependency of this optional **file**, not a global
requirement on the base outfit. Keep the patch's XML available after a BodySlide
build, per the file description. The parent project version is not the version
of the patch file. Runtime compatibility remains unknown.

```sh
cargo run --locked --offline --example review_file_relationships -- nexus skyrimspecialedition 155656 667812 < examples/research/file_relationships/mages-daily-outfit-smp.json
```

The only accepted query-bearing citation is an exact public Nexus link of the
form `https://www.nexusmods.com/GAME/mods/PROJECT?show_file=FILE&tab=files`,
matching the independently supplied Nexus subject. It accepts no tracking keys,
credentials, fragments, alternate parameters, or different game/project/file.
This URL check opens nothing and does not attest the contents of a supplied
citation or map the URL's public file number to the global v3 ID.

The separate [BHUNP refit](https://www.nexusmods.com/skyrimspecialedition/mods/156597)
reports the optional shawl file requirement only if the SMP shawl is chosen.
The documented choice is an example of a conditional requirement, not an
unconditional one. The exact BHUNP subject file number was not proven by the
indexed sources used here, so no exact-file BHUNP row is included. The
conditional code path uses a separately synthetic subject. Do not infer
requirements for other conversions from the outfit name. This fixture admits no source to
the database, checks no mod bytes, and changes no manager/install workflow.
