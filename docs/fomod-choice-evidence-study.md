# FOMOD choice preservation and review

Study date: 2026-11-17. Status: pinned source research plus a tested **review-only contract**, not a FOMOD installer integration or completed preservation feature.

## Actual sources inspected

- [salma](https://github.com/lextpf/salma/tree/874874f12fc35661a0a74b8c1a6258d99c99faf8), pinned `874874f12fc35661a0a74b8c1a6258d99c99faf8`: README, Cargo manifest, LICENSE.md, inference service, diagnostics, CSP solver/options, atom assembly and replay selection validation. This is a targeted source study, not a claim every source file was audited or the binary was executed.
- [Vortex report #21864](https://github.com/Nexus-Mods/Vortex/issues/21864), including all seven returned comments, and [merged fix #22250](https://github.com/Nexus-Mods/Vortex/pull/22250), source head `2528bba124780d382686396ffdf71bd1df499e42`, merge `e9281a4af55322f26e745a0b276121ab95c35158`. Reviewed the actual three-file patch and pinned LICENSE.md. The report's external GIF was not inspected; no visual reproduction claim.
- Existing [preservation PR12](https://github.com/Herbertofury/Modding-Knowledge/pull/12) at `c24497b2c5282b1aec6e22ad71a234c5803637cf`: archive retention, hash binding and raw log evidence, not semantic verification of choices. This follow-up is additive and does not replace that vault or touch product releases.

## Findings and useful distinctions

salma is already Rust-based: a cdylib with a C ABI and Python MO2/C++ HTTP hosts. Its pipeline parses FOMOD XML, expands file operations, evaluates candidate outputs and searches constraints. Cached fomod-plus selections are candidates checked against available installed-file evidence, not automatically authoritative historical clicks. The inference service explicitly permits approximate results when its search budget is exhausted and partial evidence following scan/hash errors. It uses contested-file FNV-1a comparisons and limits installed-file hashing to 256 MiB. These are useful inference/performance mechanisms, not cryptographic archive identity or proof of complete historical selection recovery.

The CSP code applies budgets and capped SelectAny enumeration, with exact-mode exceptions. Output-tree and mismatch-detail serialization cap lists at 5,000 entries. An adapter must retain truncation, unknown external dependency conditions, search phase, mismatch categories and ambiguity. Its weighted confidence score is a heuristic, not an empirically calibrated probability. Identical installed output can have multiple selection histories; later edits, conflicts, generated files and omissions make reconstruction weaker. Never relabel inference as recorded user intent, even on an output match.

Replay validation in `fomod_service.rs` uses name-based sets in a validation pass, treats a missing steps array as valid, and logs some unknown selections rather than rejecting them. Repeated names and absent groups therefore require additional host-side identity/completeness checks. Do not infer that a successful validation means all desired choices were reconstructed.

Vortex's report separates two failures: reinstall losing visible choices and updates silently applying old choices. Its merged fix separates attended UI presets from engine presets: manual reinstall displays prior choices for review; unattended operation remains distinct. The patch guards repeated callback application per step. Its option matching is index-first with name fallback, so blindly transferring positions/names to changed XML would risk the wrong option. The issue discussion explicitly warns that updates can radically change installers. This motivates exact artifact binding, visible review and no silent cross-version replay, not a claim current Vortex is still unfixed.

## New original Rust contract

`experiments/source-preservation/src/choice_review.rs` adds typed recorded-interaction, inferred-files and unknown-origin evidence; archive/config/context SHA256 binding; raw-evidence digest/tool/version/time; explicit completeness/ambiguity; document-local occurrence coordinates and review reasons. Both selected and deselected values can be retained. Duplicate coordinates or malformed provenance are rejected.

Only declared recorded evidence with identical archive, ModuleConfig and known external context can be offered as **attended preselection**. Every result requires review and always forbids automatic replay. Changes, unknown context, inference and partial/ambiguous evidence remain visible reasons. No install operations, extraction, XML interpretation, file deletion, source access or replay occur. This API does not prove adapter truth, validate option cardinality/visibility, or grant installation authority.

The contract intentionally does not reinterpret old PR12 arbitrary evidence-method strings as proven recorded clicks. Existing raw logs remain unchanged. A future trusted adapter must verify retained raw hashes, parse/validate the exact XML and complete option schema, and establish an actual observed interaction before assigning the recorded tier. An imported inferred plan stays inferred after user review; a newly observed selection creates a new evidence record rather than overwriting history.

## License and reuse

Both inspected LICENSE.md files contain GPL v3. Cargo metadata in salma has no license declaration, and its unrar dependency explicitly compiles proprietary unRAR sources; derivative distribution needs complete upstream/dependency review. No upstream source or assets were copied into the MIT experimental vault. The new review contract is original implementation from requirements and identified failure cases, not a port of either replay or CSP solver. Linking/importing upstream implementations is a separate decision.

## Acceptance and next steps

Nine new Rust tests and fourteen existing vault regressions pass locally: exact recorded attended-only behavior, inferred/unknown tiers, same-config repacks, changed XML, unknown/changed dependency context, partial/ambiguous states, duplicate coordinates and invalid digest/origin. These are contract/fixture tests, not native MO2/Vortex or real FOMOD replay acceptance.

Next implement opt-in per-install observation with immutable installer/config hashes, retained original logs and complete selected/deselected options. Present prior recorded choices and separately labeled inference candidates in a reviewable wizard with current dependency/cardinality evaluation. Reconfiguration should stage a new output and preserve original/conflict state; test repeated names, reordered/renamed/removed options, changed dependencies, interrupted installs and version changes. Any salma process must be isolated, version-pinned, preserve partial/timeout evidence, and have resource/cancel bounds. A successful inference result never installs automatically.

PR12 remains draft and was observed merge-conflicted with main during this study. This follow-up targets its feature branch; resolving base conflicts and native integration remain separate work. No merge/release claimed.
