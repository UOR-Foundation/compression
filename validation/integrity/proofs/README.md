<!-- uorc:non-authoritative -->
# Saved component proof receipts

These are actual scoped diagnostic compiler and replay receipts for the source
proof modules. They are not product pinned-SDK or full archive acceptance.
`scoped-project.toml` and `scoped-lexlean.lock` preserve exact diagnostic bytes;
reproduction places them under their original `lexlean.toml` and `lexlean.lock`
names beside the same source snapshot after official authoring conversion.
Original repository locks are unchanged.

`combined-verify.json` and `combined-attestation.json` cover six modules;
`replay-*.json` are the actual normalized successful leanchecker process records.
The original collision and state scopes and their restored controls are preserved
separately. `isolated-soundness-mutant.json` is a negative-attribution diagnostic
with non-target proof declarations omitted; it is never a substitute for the
complete scoped or product verification. Other mutant records retain full scopes.
`evidence.json` states exact source/tool/lock identity and remaining boundaries.
