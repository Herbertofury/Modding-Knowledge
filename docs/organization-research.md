# Premium organization: research and implementation brief

Research date: 2026-11-16. Scope: Athena, Modding-Knowledge database manager, MO2R, ModHunter, Noxviola and other applicable project managers. Evidence levels below distinguish official documentation, inspected source, and proposed design. This is not a claim of runtime parity: no desktop benchmark or end-to-end interaction with these reference applications was performed in this pass.

## Outcome, not a checkbox list

Users should recognize a project visually, find it through any known property, rearrange their working view without corrupting installation order, and discover problems or missing information without opening hundreds of records. Organization must be an integrated primary workspace, not a settings page appended to a pretty gallery.

The best architecture combines a relational catalog, personal collections, hierarchical facets, saved queries, spatial/size overviews, and a fast reversible workbench. No single folder tree can represent every relationship correctly.

## Reference findings and what to adopt

### Windows: familiar navigation and explicit query scope

Official Explorer guidance documents Quick access pinning and separate file movement actions. Windows Search's canonical AQS supports property restrictions, Boolean composition, ranges and null values. Microsoft explicitly warns that mixed AND/OR binds left-to-right and recommends parentheses; canonical property identifiers avoid UI-language-dependent queries. These are different layers: Explorer presentation is not the search grammar.

Adopt pinned places, breadcrumbs, tabs, visible scope and keyboard-first commands. Store language-independent field IDs rather than translated labels. Do not reproduce surprising operator precedence; display a parsed condition tree and require explicit grouping for mixed logic. Search scope must visibly distinguish all records, this collection, installed profile, and currently filtered results.

Sources: [Explorer](https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows), [canonical AQS](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-advancedquerysyntax), [tabs release evidence](https://support.microsoft.com/en-gb/servicing/os/windows-11/2022/10/october-18-2022-kb5019509-os-build-22621-675-out-of-band).

### Finder: multiple projections, non-exclusive labels, persistent smart views

Apple's indexed official guide describes Icon/List/Column/Gallery views, grouping independent of sorting, tags spanning storage locations, and dynamically updating Smart Folders. Multiple tags can apply to one object. Full-page retrieval returned HTTP429; these findings are limited to official indexed excerpts, not an exhaustive Finder audit.

Adopt table/gallery/tree/inspector modes over the same selected IDs; a user should not lose selection, search or scroll anchor when changing mode. A saved smart collection is a query, not a frozen copied list. A gallery can reveal visual identity while a table exposes precise metadata; neither should replace the other.

Sources: [Finder organization](https://support.apple.com/guide/mac-help/organize-your-files-in-the-finder-mchle9f0a1b2/mac), [tags](https://support.apple.com/en-ie/guide/mac-help/mchlp15236/27/mac/27).

### Mod Organizer 2: actual source-level findings

Cloned official repository, inspected commit `4ba22f0d8b6f9e08e44008b8395cd774551b4ebe`. No application run or copied implementation.

- `src/modlistsortproxy.cpp`: typed category/content criteria, per-criterion inversion, AND/OR modes, separator show/hide/normal filtering. Text terms form AND segments joined by OR. Search checks enabled columns including author, uploader, notes, categories and mod ID. Conflict detection recognizes mixed, overwriting, overwritten, redundant and archive/loose variants. Sorting handles special backup/overwrite cases.
- `src/filterlist.cpp`: inactive/active/inverted criterion state, restored filter controls.
- `src/modlistbypriorityproxy.cpp`: separator children derive from profile priority sequence; drag/drop handles collapsed and expanded boundary cases. Therefore a separator is not simply a categorical label.
- `src/qtgroupingproxy.cpp`: groups map back to source rows; inspected tree builder has a subgroups TODO. Do not claim it already supplies arbitrary nested grouping.

Adopt filter inversion, persistent controls, strong conflict cues and ordered collapsible separators. Improve on hidden-column-dependent search: hiding a visual column must not silently change default search scope. Keep custom view order distinct from game load order and installed overwrite priority.

Source: [pinned repository tree](https://github.com/ModOrganizer2/modorganizer/tree/4ba22f0d8b6f9e08e44008b8395cd774551b4ebe/src). Relevant source files can be reached under that immutable tree.

### WizTree: overview linked to precise rows

Official documentation describes a size-weighted treemap, size-ordered flat file view, CSV import/export for comparing snapshots, and duplicate candidates identified through name/size/optionally date. That is not cryptographic content identity. Its guide distinguishes logical and allocated size filters, inclusion/exclusion, name versus full path, date ranges and Boolean terms; exclusions can reduce scan work. It warns displaying all results can be slow.

Adopt synchronized treemap/tree/table selection, useful for retained media and cached artifacts. Display logical size, allocated size, unique retained bytes and estimated reclaimable bytes separately. Offer snapshot changes and cost-by-project/creator/source. Never label equal-size/name candidates safe to delete; require content identity and reference checks. A treemap is excellent for space distribution but poor at finding tiny important objects, so retain searchable rows and a zero-size/missing-data lane.

Sources: [WizTree features](https://diskanalyzer.com/about), [advanced search and export guide](https://diskanalyzer.com/guide).

### Everything: composable fast search with visible state

Official search docs describe typed property searches, saved filters independent of the current search text, bookmarks/macros, visible active search options and a visual advanced-search builder. Content searches are not indexed there and may be expensive; narrowing by cheap metadata first reduces files read.

Adopt cheap indexed narrowing before expensive content/embedding/media analysis. Let users name reusable filters and combine them with a fresh query. Never conceal whether content search is enabled, incomplete or still running. Offer cancellable expensive search and clear scanned/remaining counts rather than freezing the interface.

Source: [Everything searching](https://www.voidtools.com/support/everything/searching/).

### Zotero: collection membership is not file duplication

Official collection docs establish many-to-many membership: one record may belong to several collections without copying the record. Removing a collection membership differs from moving the item to trash. Search documentation distinguishes metadata-only versus full-text modes, explicit parent/child matching and dynamic saved queries. Current search docs and a newer release article differ in complex-query detail, so do not use them as proof of an exact installed-version UI.

Adopt non-destructive playlists/collections, an unfiled inbox, and explicit Remove from collection versus Delete record versus Remove downloaded media. A project can belong to several game setups, research boards and creator collections while sharing canonical metadata. Parent/child search must state whether a release file match returns the file, its project, or both.

Sources: [collections](https://www.zotero.org/support/collections_and_tags), [searching](https://www.zotero.org/support/searching).

### digiKam: visual taxonomy needs independent dimensions

Its official Tags View describes hierarchical tags, shortcuts/icons, thumbnails as tag icons and moving/reparenting tags. The manual specifically demonstrates that mixing classification axes into one hierarchy creates ambiguous repeated labels; an independent contextual tag can avoid repeated subtrees. Removing a tag does not delete media.

Adopt image-backed category icons, tag shortcuts and taxonomy editing, but separate game, file format, content, visual style, author, workflow status and compatibility. A Sims clothing package is not one inseparable path: game=Sims4, format=package, content=clothing/shoes, and style tags are separate facets. Use stable taxonomy IDs and aliases to preserve saved queries when labels change.

Source: [digiKam Tags View](https://docs.digikam.org/en/left_sidebar/tags_view.html).

## Shared domain model: implementation recommendations, not claims about references

1. **Entity identity:** immutable creator/project/release/file/media IDs. URLs and names are mutable aliases. Maintain provenance and observation timestamps.
2. **Physical location:** zero or more local/remote locations with existence/check state. Changing a logical collection never moves bytes.
3. **Collection membership:** many-to-many, personal/shared scope, optional manual position. Separate collection deletion from entity deletion.
4. **Separator:** owned by a particular ordered workspace or profile, stable ID, title/color/icon, collapsed state. Empty separators remain editable. Sorting by a field may suspend manual reorder, with a visible explanation.
5. **Computed group:** query projection such as Game > Category > Creator. Distinct from manual separators; group counts specify unique projects versus memberships. Multi-category entities must not silently inflate totals.
6. **Facet:** typed values with source/confidence/review state; hierarchical taxonomy with synonyms. Unknown, unavailable, not-applicable and measured zero remain distinct.
7. **View:** versioned query AST + scope + visible columns/width/pinning + sort array + grouping array + density + thumbnail size + expansion state + manual-order reference. Personal presentation is not a global catalog mutation.
8. **Mutation journal:** action ID, target IDs, before/after values, actor, time, undo capability, partial failure results. Bulk operations must show their exact scope before changing anything.
9. **Artifact relationships:** dependencies, conflicts, supersedes, translation, fork, variant and source migration. Preserve directional types instead of a generic related-items bag.
10. **Refresh state:** each source's last attempted and successful check, errors, next eligible check, retry budget. An author absent from one site is not proof of deletion or inactivity everywhere.

## Premium workbench specification

- Main table: pinned image + identity, Content icon strip with accessible text, category chips, source/creator, compatibility, version, advisories, dates, metrics. Reorder/resize/hide columns; field chooser search; named column sets; optional compact density.
- Identity inspector: pinned gallery, video controls, source attribution, all creator destinations, releases, dependency tree, issue counts, changelog and provenance. Keyboard-triggered preview; avoid hover-only access.
- Navigation: pinned collections, nested categories, smart views, recent/favorites, unfiled/unreviewed, stale-source and missing-media queues. Back/forward restores query and selection.
- Search: visible chips with include/exclude/any/all controls, grouped Boolean builder and expert text syntax backed by the same AST. Field aliases autocomplete; explain parse errors at their location. Query examples are generated from actual supported fields, never pretend unsupported filters work.
- Filters: per-facet counts under a defined counting model; ranges with unit/timezone labels; missing-value facets; quick reset per facet and clear-all. Zero-result view explains restrictive filters and offers reversible relaxation.
- Sort: stable multi-key ascending/descending, explicit null placement and final ID tie-breaker. Numeric versions require a chosen version comparator, not naive lexical order. Date type remains visible.
- Group: nested collapsible groups with counts, totals and optional representative thumbnails. Collapse/expand current/all; preserve selected hidden rows with visible selection count.
- Separate manual collection ordering, install priority and plugin load order. Drag handles and target previews make the operation's meaning explicit. Never change executable order just because the user alphabetized a view.
- Batch: shift/range selection, keyboard multi-selection, select page versus all matching, tag/category assignment, collection movement, export and refresh queue. Scope must state whether filtered-out/hidden members are included. Undo for reversible local metadata changes.
- Comparison: side-by-side project/release diffs, before/after screenshots and file manifests. A queue of candidate updates has user-controlled accept/hold/skip, not forced installs.
- Storage: linked treemap and rows, per-creator/project/media-kind breakdown, duplicates review, pinned originals and cache policy. Distinguish regenerated thumbnails from irreplaceable retained originals.
- Accessibility: color is supplementary to text/icon, screen-reader names, full keyboard operation, focus restoration, scalable text, high contrast, reduced motion. Expanding a gallery must not shift the selected record unpredictably.
- Automation: rule preview/dry-run, per-rule provenance, undoable proposals and exclusions. Self-organization can suggest merges/categories; it must not silently overwrite user labels or merge creators on names alone.

## Performance and correctness architecture

Rust shared query/domain core; platform/UI adapters consume the same serialized contracts. SQL queries are parameterized from a validated AST with allowed fields/operators. Push indexed predicates and bounded pagination into the database, not a whole-database browser download. Composite keyset pagination must include every sort key plus stable ID, with a snapshot/revision strategy so updates do not skip or repeat records.

Virtualize rows and thumbnails, retain selection by ID, cancel stale queries and image requests, decode media off the UI thread, cap thumbnail memory with an LRU and isolate corrupt media. Facet counts can be asynchronous but must carry the query revision; old counts cannot overwrite newer filters. Surface approximate or stale counts honestly. Retained original files need reference tracking, checksums and restore checks before cache cleanup.

Proposed measured targets, not achieved claims: cached indexed filter-to-first-row p95 <=150 ms on a stated 100k-record fixture; interactive control response <=100 ms; no UI-thread image decode; bounded resident thumbnail memory; large-catalog query benchmark at 1m entities; cancellation under 250 ms after expensive content search. Publish machine/dataset/version and cold versus warm runs rather than asserting Rust alone guarantees speed.

## Acceptance suite

1. A project belongs to three collections; remove one membership, other memberships and media remain.
2. Same title for two authors; sort/filter/select never confuses identities.
3. Collapse a separator, filter to one child, drag another item across its boundary; preview and resulting manual order agree. Repeat ascending/descending and with empty separators.
4. Sort by downloads or group by category; installed overwrite priority and plugin load order byte-for-byte unchanged.
5. Hide Author column; all-field search still finds author, unless user explicitly selected visible-fields scope.
6. Include clothing AND game Sims4, exclude shoes, retain unknown category separately; visual query and expert query produce identical IDs.
7. Unknown issue count does not pass zero-bugs filter. A stale zero displays its timestamp, not a safe-update badge.
8. One project in two categories contributes once to unique-project total; membership total is separately labeled.
9. Equal dates/metrics spanning several pages produce no duplicates/skips; concurrent refresh follows documented snapshot rules.
10. Rename taxonomy label and merge an alias; saved view using stable IDs remains valid and user overrides remain intact.
11. Rapidly arrow through rows while images load; late image result never appears on another project's row/inspector.
12. Broken image, missing video, offline source and unsupported codec have different truthful states; no invented replacement artwork.
13. Keyboard-only user filters, groups, previews, multi-selects and undoes; screen reader announces content/category and collapsed state.
14. Select all matching across pages, change query, then batch edit; stale selection is explicitly retained or cleared, never silently retargeted.
15. Same file name/size but different bytes remains a duplicate candidate, never automatically removed.
16. Skyrim `.esp` with verified ESL flag is represented differently from uninspected `.esp`; `.package` alone never proves shoes.
17. Query excludes a private/withheld record; facets, thumbnails and counts do not disclose its hidden metadata.
18. New source data changes a creator URL; collection membership and manual order survive canonical identity migration.
19. Export/import view preserves field IDs, scope and timezone independent of UI language; unsupported version fails visibly.
20. Bulk edit interrupted mid-batch reports exact completed/failed IDs and permits safe retry without duplicate effects.

## Project mapping and delivery order

**Shared Modding-Knowledge:** field registry, typed query AST, saved-view schema, taxonomy/provenance, collection and separator model, reference datasets/tests. Keep application-specific profile/load-order mechanics in the owning manager.

**Rust database app:** first deliver visual table + Content/Category + facets + stable sort + preview + saved views; then collections/separators/bulk reversible edits; then storage analysis and rule review. Read-only HTML export is useful proof but not a substitute for a tested native editing workflow.

**Athena:** project/creator/artifact collections, research inbox, saved idea and pending-feature views, source tracking and visually rich design library. Surface forgotten/unimplemented user requests explicitly.

**MO2R/ModHunter/Sims4Foundry/Enderloom:** domain-specific parsers, profile scope, conflicts/dependencies and media. Shared field semantics; no false equivalence between filesystem format and gameplay content.

**Noxviola:** independently usable followed-creator/project collections and sidebars, saved filters and alerts. Share contracts/methods without requiring Athena to run.

No affected repository is claimed updated by this research document. Changes should be integrated by the assigned implementation agents after checking current backlog and avoiding parallel edits.
