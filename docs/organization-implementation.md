# Applying organization research to the shared catalog

This implements a bounded saved-view contract and turns the [source-linked research](organization-research.md) into acceptance work. It does not modify the separately developed native database app or claim runtime parity with Windows, Finder, MO2, WizTree or the other reference tools.

## Applied implementation

`tools/catalog_views.py` validates and copies versioned saved views. Initial supported scopes are catalog and collection; profile-specific scope binding remains native-manager work. Fields come from the consumer's privacy-filtered allowlist. Visible columns are independent of all-field search; hiding Author must not silently exclude author search. Multi-field sort requires explicit null placement and a final stable-ID key. Explicit all/any filter groups avoid ambiguous AND/OR precedence. Unknown filters are distinct from measured zero. Filter depth/node counts, grouping and sort keys are bounded. Load order, install priority, raw SQL and unknown fields are rejected.

The default emphasizes Image, Name, Content and Category columns, deep-black styling and all-allowed-field search. High-contrast/custom theme references are supported; a view stores presentation only, not global catalog mutations or executable ordering. No SQL generation, search execution, theme rendering or UI is claimed by validation. Field typing beyond scalar exact matches, richer operators, full native field registry and schema migrations remain backlog.

## Reference-to-work mapping

- Windows AQS: stable field IDs and explicit Boolean groups, rather than localized labels or surprising precedence. Add typed query compiler and visible scope/parse errors; test identical expert and visual query results.
- Finder and Zotero: multiple projections over stable selection, dynamic saved queries and many-to-many collections. Implement reversible membership operations separately from deleting records or bytes. Finder research was limited to official indexed excerpts after full-page retrieval was rate-limited.
- MO2 pinned code: filter inversion, persistent controls and ordered separators are concrete findings. Separators and computed groups are different models; never infer arbitrary subgroup support from the current MO2 tree builder. Test collapsed/empty separator drag boundaries and unchanged install/load order after display sorting.
- WizTree: link size overview to precise rows; distinguish logical, allocated, unique-retained and reclaimable estimates. Matching names/sizes produce candidates, not safe-to-delete duplicates.
- Everything: indexed narrowing before expensive content search, with visible scope and cancellable work. Implement query revision guards and bounded page fetches.
- digiKam: independent taxonomy axes and image-backed labels. Game, file format, content, style, author and maintenance state are not one overloaded category tree.

Exact official sources and MO2 commit/file paths are preserved in the research document. Proposed application features and performance targets there are not attributed as already existing in each reference tool.

## Highest-priority visual and organizational acceptance

- [ ] Gorgeous deep-black default across the database-manager surface, not dominant purple/green; customizable themes, accessible contrast and colored separators/status accents with text/icon equivalents.
- [ ] Creator/project image columns, retained attributed galleries and playable media previews. Respect source access/retention policy, preserve originals/provenance and distinguish missing, unavailable, corrupt and unsupported media. No invented replacement artwork. Keyboard-accessible preview and reduced motion; no hover-only essentials.
- [ ] Compact Content icon strip plus Category labels/icons with screen-reader names. Skyrim ESP and inspected ESL-flagged ESP (ESPFE) remain distinct from uninspected files. Sims4 package/script/DLL formats do not prove clothing/shoes/tattoos content; classify from evidence with independent format/content facets and unknown states.
- [ ] Premium organization: persistent layouts, separators, nested computed groups, rapid multi-field filtering/sorting, selectable columns, density, keyboard/range/bulk selection and clear all-matching versus page scope. Display sorting must leave installed overwrite priority and plugin load order unchanged.
- [ ] Exact unique-project versus membership counts, unknown facets, privacy-filtered facet counts, stable pagination, canceled-query revision guards and preserved selection across table/gallery/inspector views.
- [ ] Real-source review, instrumented representative datasets and evidence receipts before claiming parity or performance. Use the research acceptance suite; proposed latency/scale targets are not achieved measurements.

## Truthful operation progress

- [ ] Every long-running user operation exposes its named phase, observable completed/total work and throughput where measurable. Use indeterminate progress for unknown work; never synthesize percentages or mark queued work complete.
- [ ] Distinguish queued, running, waiting/blocked, paused, cancel-requested, canceled, completed, partially failed and failed states. “Cancel requested” is not “canceled” until workers acknowledge; preserve partial results and exact retry scope.
- [ ] Expose last progress time, source coverage, errors and resource/speed tradeoffs. Pause/cancel controls appear only when implemented; do not promise interruption of noncancelable work. Progress labels must remain accessible without color or motion.
- [ ] Validate progress event ordering, stale-event suppression, unknown totals, phase transitions, reconnection and partial failures against real operations before claiming live behavior.

## Verification and remaining scope

- [x] Saved-view structure, privacy allowlist, no mutation, visual default, hidden-column search independence, stable-sort requirement, explicit unknowns and bounded filter tests.
- [ ] Native query/domain implementation, typed field operators, collections/separators, progress protocol, real media retention/rendering and accessible app workflows.
- [ ] Desktop benchmarks, exact-source runtime comparisons and end-to-end acceptance. Shared Python validators are reference contracts, not an optimized native renderer or database query engine.

## Evidence-based health visuals and on-card media

- [ ] Mod/project health bars and damaged visual treatments communicate explicit underlying states, not invented numeric quality scores. Distinguish update available, stale evidence, reported issue, reproduced failure and unknown; outdated alone is not broken. Offer source/details and accessible text equivalents. Warnings remain informative and never block the user's update decision.
- [ ] On-card multi-image navigation and video controls preserve user preferences, focus and selected item. Respect media/source access, reduced motion and performance budgets; lazy-load and cancel stale requests. No autoplay/audio surprises or bypassed access requirements. Unsupported/missing media states remain explicit.
