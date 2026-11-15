# Bounded official Modrinth creator/mod refresh

Scope: Sodium AANobbMI, Lithium gvQqBUqZ, Iris YL57xq9U, FerriteCore uXXizFIs, ModernFix nmDcB62a, Entity Culling NNAgCjsB and ImmediatelyFast 5ZwdcRci. Only official anonymous project and team-member endpoints. No OAuth, API token, cookie, private endpoint or new grant is used. The first real refresh observed five public account IDs and seven accepted project-team memberships. It does not enumerate every creator or creator's complete output.

## Pipeline

1. Read reviewed `config/modrinth-source-policy.json`; enabled/public/retain/publish must all permit this scope.
2. Fetch fixed validated project IDs, derive validated eight-character team IDs; at most fourteen requests, timeout 20 seconds each, one MiB maximum response, no redirects or automatic retry.
3. Whitelist public metadata. Exclude display/legal name, email, bio, account permissions, payouts, private fields, project bodies, images and mod files even if a response supplies them.
4. Validate project/team identity and accepted memberships. Malformed/incomplete batches leave the prior dataset byte-for-byte unchanged. HTTP 429 records a backoff; no quota tests or aggressive retry.
5. Atomically save projected metadata and sanitized receipt. Failure receipts contain exception types only, never bodies, headers or credentials.
6. Merge reviewed data, build a new database with native Rust, verify exact replay/reopen/integrity, then create the latest compressed artifact and digest. Only a successful build is committed. Prior dated artifact and Git history remain preserved.

The workflow commits through the repository's ephemeral GitHub Actions token with contents-write permission. No long-lived credential is created. Pushes are non-force; concurrent changes cause a safe failure rather than overwriting another writer. Only the scoped data/artifact paths are staged. The hourly schedule is configured, not a guarantee of uninterrupted execution. Inspect last-success time and failure receipt for freshness.

## Semantics

Public team member does not mean sole author. Roles are raw provider claims. Creator counts refer only to accounts observed across the seven selected projects. Loader lists are project metadata, not validated per-release compatibility. Null metadata stays unknown. Original license claims are retained without converting restricted licenses into permissive badges.

Policy withdrawal disables new reads/import builds. Previously published data requiring removal needs an explicit retention/publication response; the current scheduler does not silently purge history. Do not broaden provider routes or include new fields without reviewing permission and private-data boundaries.

## Tests and real evidence

Seven offline tests cover denied policy/no request, whitelist redaction, partial/mismatched batch rejection, exact last-good preservation and rate-limit backoff. Real anonymous fetching and native creator querying were executed separately; mocked tests alone are not claimed as live acceptance. Current integration has six Rust tests plus these seven Python tests.
