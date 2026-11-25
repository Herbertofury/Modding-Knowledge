# Sims 4 memorial archive: reviewed sidecar

This independent sidecar covers the 49 listings visible in the
[Scumbumbo memorial archive](https://scumbumbomods.com/), plus two public
creator identities and the [official MC Command Center](https://deaderpool-mccc.com/)
project used as an inclusion target. Project URLs are the archive's
canonical pages. Original archive credit is not later maintainer identity.
The [patch-status page](https://scumbumbomods.com/status) was reviewed on
November 25, 2026, at its displayed 1.129 patch label.

- Four historical broken notices and one obsolete entry are stored as dated,
  source-scoped warnings. An old project-page download is not recommended
  merely because it remains available.
- Maintainer compatibility, update, uncertain, and conditional entries
  remain separately attributed. The patch label is not an exact Windows or
  macOS build, file identity, compatibility verdict, or game test.
- Nine listed MCCC inclusions are archive reports, never dependencies or
  an instruction to replace files. They are not independent evidence of
  module-level parity or complete MCCC coverage.
- The [original Tuning Error Notifier page](https://scumbumbomods.com/tuning-error-notifier/)
  describes a specific load-order warning, not an unconditional conflict.
  The alarm fix is an optional workaround, not a requirement.
- [XML Injector 4.2](https://scumbumbomods.com/xml-injector/) preserves main
  script versus the separate mailbox installation-test component. Its listed
  archive version does not prove game compatibility for 1.129.
- Unknown relationship coverage is explicit. No downloads or binary
  redistribution, support/community scraping, or automated acquisition
  is authorized. Data consists of linked public facts and short excerpts.

Build a separate new SQLite file; do not replace a live catalog:

```sh
cargo run --locked --example seed_reviewed_bundle -- --scumbumbo-sims4 scumbumbo-sims4.sqlite
cargo run --locked --example query -- scumbumbo-sims4.sqlite scumbumbo:project:xml-injector
```

The ordinary seed and refresh process is unchanged. The sidecar is a
bounded research input, not an automatic merge or an installation tool.
