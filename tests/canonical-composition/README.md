<!-- uorc:non-authoritative -->
# Composition diagnostics

parsed-modules.json contains the complete unchanged official parsed_module
objects for the12 new source modules at29bd9d362f0e45a1e3d32c392d25da20c18925f4.
The source files remain authoritative. All12 have two byte-identical official
reads and byte-exact SDK writer round trips. The reader/package identity,
source hashes, imports and actual declaration policies are retained.

Cache SHA256: ddd73a21290a3bdad7df31f4f4b2c8d41508695069a9daa169f780dcc0fa56cc.
This is a signature/diagnosis cache, never a production dependency. Source
format/check/build success does not establish kernel or scenario acceptance.

diagnostic-project preserves the exact language1.2 project/lock/Lean workspace
for the recorded checks. Existing dependencies were privately migrated with
the official CLI and exact declaration equality, as recorded in
docs/implementation/canonical-composition-evidence.json. The public product
remains language1.1. Reproduction must preserve that migration boundary and
the recorded existing reference-proof failure; do not prune that theorem.
