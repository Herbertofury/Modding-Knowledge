# Sims 4 CAS custom content: reviewed files and appearance notes

Reviewed November 26, 2026. The sidecar keeps 7 original Patreon posts and 2 source-qualified creator identities separate. It records 13 attachment names as observed labels, not stable binary IDs, verified bytes, or approved download URLs.

| Project | What the page actually establishes |
| --- | --- |
| [Jamie Skin & Sim](https://www.patreon.com/thisisthem/posts/jamie-skin-sim-93816776) | Distinct skin and overlay packages plus the Sim ZIP. The author says Magic Bot eyebrows, nose, mouth and chin sliders are included in the Sim folder for the pictured look and warns about other facial slider overrides. |
| [Jacob & Barry Skins & Sims](https://www.patreon.com/thisisthem/posts/jacob-barry-sims-99261674) | Two separate skin/overlay pairs and one shared Sim archive. Do not reuse one actor's overlay for the other in the inventory or treat the shared archive as one skin. |
| [Salma Skin & Sim](https://www.patreon.com/thisisthem/posts/salma-skin-sim-94841540) | Skin, overlay, contact-lens package and Sim archive are separate listings. No required dependency on the lens package is inferred. |
| [Eyebrow](https://www.patreon.com/magicbot/posts/default-eyebrow-53944028), [nose](https://www.patreon.com/magicbot/posts/default-nose-55675247), [mouth](https://www.patreon.com/magicbot/posts/default-mouth-56990147), [chin](https://www.patreon.com/magicbot/posts/default-chin-54872952) | Four distinct creator default-slider posts. The eyebrow package is explicitly named in the reviewed attachment list. The other three post identities do not identify files or confirm the copies bundled with Jamie. |

## What an app should do with this

Show the skin, overlay, lenses and Sim archive as separate choices. Interpret the four Magic Bot links as candidate appearance aids for Jamie’s imported Sim, not hard dependencies for the standalone skin or overlay. Check for bundled or already installed copies first. This catalog has no exact filename/resource evidence for a load-order winner or compatibility with every custom slider. A project may report a conflict without identifying a specific conflicting package. A description of the author’s screenshots or compatibility with an HQ mod does not establish that HQ is required.

Respect the creator’s Tray vs. Mods instruction: the Sim archive must first be reviewed and extracted; Tray records and CC packages take different paths. The sidecar does not inspect archive contents, infer current patch compatibility, or use a provider attachment URL. Download starts at the author’s page and obeys supported access. No automatic installation, file redistribution, current game testing or facial identity inference is claimed.

## Verify using an independent SQLite file

```sh
cargo run --locked --example seed_reviewed_bundle -- --sims4-cas-cc sims4-cas-review.sqlite
python tests/test_sims4_cas_reviewed.py sims4-cas-review.sqlite
```

The importer refuses overwrite and checks that an identical second import does not duplicate records. This is a sidecar, not a modification of the existing bundled catalog, and no external CAS consumer is wired yet.
