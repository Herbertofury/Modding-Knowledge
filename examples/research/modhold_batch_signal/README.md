# ModHold batch version signal (offline research example)

[ModHold's public API documentation](https://modholdmods.com/developers)
describes a read-only stats endpoint for up to 100 exact mod IDs. This example
accepts caller-supplied JSON on stdin and does not contact any provider, store
state, download files, edit the database, or add automatic update checks.

Pass an object with `previous` and `stats`. The first is an array of previously
reviewed detail rows containing `id`, `version`, `updated`, and `files`. Every
file needs its exact positive `id`, raw `kind` and raw `version`. The second
is the public batch envelope, `{"mods":{"12":{...}}}`. Its keys must all be
among the explicitly selected baseline IDs. Remove unrelated detail fields if
the source records would exceed the 2 MiB input limit. Output reports only
project IDs and signal statuses.

```sh
cd examples/research/modhold_batch_signal
python3 -m unittest test_classify
python3 classify.py < local-reviewed-batch.json
```

A missing or invalid row is not a removal, ban, or confirmation of unchanged
state. A valid row matches only the version label, update text and exact map of
file IDs, kinds and labels. Matching signals cannot establish unchanged title,
author, access terms, game identity, relationships, hashes or file contents.
Use the full details route for a definitive update decision and keep existing
last-good state on a failed or incomplete check. Do not auto-download or
silently change refresh semantics based on this research example.

On November 25, 2026, one tokenless request for the documented example IDs
returned only ID 12, with no known reason for omitted 40 and 77. A separate
public batch for IDs 12, 1699 and 1107 returned all three; the projected
version/file signals matched one subsequent public detail response for each.
This is a three-project observation, not a general completeness or latency
benchmark. No response bodies or project assets are included here. This is
separate from the existing detailed-source contract, admission and retention
policy in [config/modhold-source-contract.json](../../../config/modhold-source-contract.json).
