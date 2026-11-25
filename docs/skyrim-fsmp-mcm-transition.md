# Legacy MCM transition evidence (source only)

The same-named legacy FSMP MCM ESP follows three different patterns in four
reviewed FOMOD source revisions:

- [v3.5.0](https://github.com/DaymareOn/hdtSMP64/blob/3b0b80e30c2ff4766d01e888ff4c1896247d8163/fomod%20tree/fomod/ModuleConfig.xml): the final group names a copy of the old ESP.
- [v4.0.0](https://github.com/DaymareOn/hdtSMP64/blob/a7de94f26656be67a39533d54482bd7a00b80d31/fomod%20tree/fomod/ModuleConfig.xml) and [v4.0.1](https://github.com/DaymareOn/hdtSMP64/blob/1b4c7415b7e362a69a975483959dcb3d5946c6fc/fomod%20tree/fomod/ModuleConfig.xml): the pinned XML bytes are identical and contain no named legacy ESP copy rule. The release/archive identities remain distinct.
- [v4.1.1](https://github.com/DaymareOn/hdtSMP64/blob/2215ec3ba09adf0843a3291ed0437a6ca6d385e2/fomod%20tree/fomod/ModuleConfig.xml): when the old ESP is present (active or inactive), a conditional same-name stub is available if the user chooses to keep it.

These are pinned source rules, not an appraisal of any local archive, installed
variant, existing save or provider equivalence. Never auto-apply a choice or
remove a plugin. If the exact revision or existing save dependents are unknown,
show the source evidence and leave the decision open.

`cargo run --locked --example seed_reviewed_bundle -- --skyrim-fsmp-mcm-history NEW.sqlite` builds a separate catalog.
`python tools/build_skyrim_fsmp_mcm_history.py NEW.html` builds the offline comparison.
