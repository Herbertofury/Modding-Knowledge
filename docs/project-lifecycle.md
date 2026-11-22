# Project lifecycle evidence (Rust consumer contract)

User goal: every relevant mod/project manager can display, filter and sort mod support, creator activity and source availability, without making those apps depend on Athena. `src/lifecycle.rs` is a read-only typed projection, not a running scraper, premium activation or shipped app UI.

## Three independent dimensions

- **Availability**, scoped to an exact canonical provider project URL: available, missing, access blocked, rate limited, retrieval failed, explicitly removed or deleted. A 404 is missing, not deletion. A 403 is access blocked, not a ban. Explicit deletion/removal needs a provider or attributed creator statement. Losing one source does not mean a mod is deleted everywhere.
- **Project support**, scoped to a stable project ID: supported, paused, discontinued. Only an identity-verified creator statement can establish these. An old release, bug count, or silence cannot establish discontinued support.
- **Creator activity**, scoped to a stable creator ID: dated verified public activity or explicit creator-declared inactivity. Public activity is not proof of maintenance for every mod. Retain original activity time separately from retrieval time. UI label: “Activity observed [event date]”, never an unqualified “currently active”. Absence of activity is unknown, not inactive.

Adapters must verify identity, statement authorship, exact project scope and meaning before creating records; enum validation is not source verification. No real creator allegation is seeded. Original event date may be unknown for a statement, but is required for observed activity. Fetching an old statement again must not change its event date or turn it into a new event.

Each observation has a stable unique ID, subject, state, authority, evidence URL, original event timestamp when known, observed timestamp, review deadline and lifecycle (active/retracted/superseded). Use canonical HTTPS references without credentials or token query strings. Archive manifests can link these observation IDs; archived content or failed capture is not automatically evidence of deletion or discontinuation.

## Consumer rules

`project(subject, dimension, evidence, now)` returns scoped retained history, current evidence, optional current state, conflict/has_stale_evidence flags and last-checked timestamp. It is deterministic and does not mutate input. Stale-only evidence produces no current state. Conflicting active fresh states produce no current state until explicitly resolved; later timestamps alone do not erase contradictions. Superseded/retracted evidence remains visible in history but cannot drive current badges. The has_stale_evidence flag means some retained active evidence is stale; inspect current evidence separately.

Display source and authority alongside every statement, exact time and freshness; searchable/sortable columns for status, source, last checked, event date and conflicts. Preserve unknown separately from unsupported, deleted and inactive. Retrieval warnings must preserve the last-good release, media snapshots, installed files and other sources. Warnings remain informative; no forced update, replacement, download or uninstall action is exposed by this module.

## Scope remaining

Persist history and corrections in the native SQLite observation store, implement independently tested provider adapters, integrate cards/columns/filtering in MO2R/ModHunter/Noxviola/Athena, wire user notification preferences and premium entitlement, and verify real native flows. Do not collide with ModHunter's active release branch. Existing author-source migration evidence remains separate: this module neither asserts bans nor redirects update sources.
