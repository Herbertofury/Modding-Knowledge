# Private source and original-installer preservation

Premium cross-manager goal: preserve full source previews, all authorized media including video, original mod installer/FOMOD archives, exact observed user choices, provenance and source lifecycle evidence so deletion does not erase the user's ability to view or reconfigure a retained mod. Never mutate the original mods or silently replay options against a changed archive.

The [Rust source-preservation experiment](../experiments/source-preservation/README.md) provides independently usable transactional chunked storage, deduplication, SHA verification, explicit capture gaps, private offline gallery, pinned retention review, original-byte restore and installer-hash-bound choice evidence. It is intentionally separate from the public Modding-Knowledge dataset and from the managers' import/UI hot paths.

Status: experimental intake/storage/viewer slice, not integrated premium release. Browser capture adapters, comprehensive media discovery, WACZ replay, native FOMOD option-event recording/reconfiguration UI, cross-manager source-history pages and Windows/native acceptance remain unresolved-active. The linked backlog preserves all those requirements. Unknown availability never implies deleted/discontinued/inactive. Lifecycle observation IDs reference an independent evidence reducer.

No user screenshots, source pages, mod archives or private metadata are included in this repository. Fixture tests use synthetic bytes only. Respect retention and redistribution separately; the private vault never publishes original assets.
