<!-- uorc:non-authoritative -->
# Profile executable eligibility diagnostic

The authoritative change is `Integrity.Profile.select`'s executable declaration.
The actual selector body and all public types, errors and theorems are unchanged.
The diagnostic caller admits no effects and checks both `rust-core` and
`rust-std`. Removing only that executable declaration rejects the real call with
`LLT4001`; restoring it reproduces the same passing source and semantic identities.

These are source and target eligibility checks with the saved official language
1.2 compiler. They do not claim preservation verification, production export,
Wasm execution, or original pinned product SDK acceptance. The caller is a
diagnostic fixture and adds no production root to the product model. Exact
project, lock, source and compiler identities are recorded in `evidence.json`.
