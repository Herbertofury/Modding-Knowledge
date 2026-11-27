# Modhold detail: one local, read-only file identity review

The [official detail endpoint](https://modholdmods.com/developers) provides public metadata separately from authenticated downloads. This helper accepts a **previously saved, public** detail response from stdin. It does not fetch, request credentials, add a source, choose a file, change a load order, or publish a record.

```sh
python tools/review_modhold_detail.py --id 6 --game skyrim-se < warband-public-detail.json
```

The selected ID, game and canonical page must agree. Main archives with a matching version are all retained and never automatically ranked; optional, unknown and old roles are not silently promoted. Timestamps are original provider strings, not guessed UTC times. Archive size and SHA-256 are provider claims, not verified downloads. Keep the original if later review needs omitted fields.

For Relationships, a provider required flag and its note remain separate. In particular, the conditional ESL backport flag supplies neither a runtime comparison nor proof of current applicability. An empty conflict list is a count for this response, not proof of compatibility.

The exact [Oskar response](https://modholdmods.com/api/v1/mods/13) returned four file rows (main 1157) and nine requirements. The exact [Warband response](https://modholdmods.com/api/v1/mods/6) returned three file rows (main 2937 and optional Bridge 560) and ten requirements. Both pass the offline helper. Synthetic tests additionally exercise multiple same-version main files; none are selected automatically. Neither game nor mod archives were run, fetched or verified here.
