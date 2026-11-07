<!-- uorc:non-authoritative -->
# Exact diagnostic project configurations

These are unchanged configuration, lock and Lean-workspace snapshots from the
named component investigations. Every configuration declares language1.2.
manifest.json binds every supplied byte; runtime-image config and lock were
also checked against the recorded4ac0a3e source-build manifest.

The public product still declares locked language1.1. These snapshots do not
change its language, locks or dependencies, and component success does not
establish product SDK acceptance. A configuration is data for reproducing its
named diagnostic project; materialize its exact source closure into a fresh
directory before using it. The historical lock belongs to the recorded SDK71
compiler semantics; another official compiler needs its own diagnostic lock.

RuntimeImage's direct roots declare rust-std exports. Such annotations do not
restrict the targets of a caller transitively: eligibility checks the selected
caller's target/effect closure. NativeLibrary/Core-Wasm still require their own
closure, signature, IR and ABI checks. Only the recorded Std getters were
rendered/compiled/probed in this evidence set.

Parser-only caches use the official read_module API without a linked project;
they record that fact and their reader/package identities. No project-level
acceptance is inferred from them.
