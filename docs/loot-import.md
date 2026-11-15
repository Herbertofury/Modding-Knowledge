# Pinned CC0 LOOT ingestion

The latest projection contains all 4,745 plugin rows from `loot/oblivion` commit `b3ffc84f6d730f6efb5083448d78611df16e8d78`, masterlist Git blob `8c613acedf019890b950e4c4b2450151cead1025`. This is a complete document import, not a complete game catalog. Original 100-row artifact and Git history remain preserved.

Source: https://github.com/loot/oblivion/blob/b3ffc84f6d730f6efb5083448d78611df16e8d78/masterlist.yaml
License: included `data/loot-CC0-LICENSE`. Upstream license was reviewed separately from the application code license.

`tools/import_loot.py` uses PyYAML 6.0.3 safe loading. It refuses any unreviewed Git blob, oversized source, duplicate plugin identity or oversized observation. It preserves every parsed plugin field, including conditions, CRCs and translated messages; YAML anchors are expanded semantically. All top-level context is retained. YAML comments/style are not database fields; original pinned source remains available upstream.

```sh
python -m pip install PyYAML==6.0.3
python tools/import_loot.py reviewed-masterlist.yaml OBSERVED_UNIX new-output-directory
```

Review the resulting files before placing them in `data/`. The command accepts only the pinned reviewed source, not arbitrary downloaded documents or policy assertions. Updating the pin requires a new content/license review. No network requests or source instructions are executed by this importer. Output must be a new directory, so failed validation cannot replace last-good data. The receipt records observed time, source commit/blob/SHA256 and observed versus imported row counts. Observation IDs include source identity and observation time; replay at the same time is deterministic.

The native snapshot importer now uses 500-observation transactions, keeping the existing 1,000-row Store bound intact. The whole staging database is published only after identical replay, close/reopen equality and validation succeed. Failed staging is retained, never substituted for the existing snapshot. No live database migration or game tests are implied.

Classic Oblivion and Oblivion Remastered are distinct applicability contexts. Current LOOT application source `03f22e99f037c08778731319a6bae0276e7464a0` maps Remastered to the shared `loot/oblivion/v0.29/masterlist.yaml` (game_settings.cpp and its expected-URL test). That routing does not make every row applicable to both editions. Consumers must evaluate game/file/version/CRC conditions and top-level context before showing actionable warnings. Imported expressions remain **not evaluated**; absence is unknown, not safe.

This tranche is pinned and not automatically refreshed. The existing hourly Modrinth workflow rebuilds combined snapshots while retaining this reviewed LOOT tranche. No private author/contact data, mod files or assets were imported.
