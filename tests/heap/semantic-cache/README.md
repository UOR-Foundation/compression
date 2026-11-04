<!-- uorc:non-authoritative -->

# Official compiler-parsed diagnostic cache

`runtime-owners.json` is one read-only diagnostic bundle for matching source call signatures in another executor. It includes all 26 owned modules: runtime/audit modules, five proof modules, the reference for moved allocation and scenario modules. The `.lex.tex` sources remain authoritative; no production build, compiler or runtime loads this cache as a semantic source. Each module has an explicit role, and every declaration retains its actual executable/axiom metadata.

The supported PrismPM helper delegates parsing and notation writing to `lexlean::sdk::read_module` and `lexlean::sdk::write_module`. It does not reconstruct declarations or use a second semantic parser. With a clean official PrismPM checkout and its locked dependencies:

```sh
cd /absolute/official-PrismPM
cargo build --offline --locked -p repo-conformance --bin prismpm-semantic-source
target/debug/prismpm-semantic-source read < /absolute/compression/src/Uorc/Core/Heap.lex.tex
```

To reproduce the full bundle from a committed compressor source snapshot:

```sh
cd /absolute/compression
node tests/heap/export-cache.mjs \
  /absolute/official-PrismPM/target/debug/prismpm-semantic-source \
  /absolute/official-PrismPM \
  /absolute/output/runtime-owners.json
```

The script refuses source bytes that differ from the named commit. It requires two byte-identical official reads and a byte-exact official notation round trip for every source. It records source paths, hashes, imports and unchanged `parsed_module` objects with schema `lexlean/semantic-module/2`. Definition summaries are projections of those parsed objects, including every executable flag, axiom list and any production annotation. Atomic table writes and old index/table diagnostic controls retain their explicit non-executable markings.

Six proof-only helpers omit an explicit executable field. Their summaries preserve this as `executable: null` and `executable_explicit: false`, while `parsed_module` is untouched. The official linker treats the omitted field as non-executable; production-closure inspection at the earlier 23-module checkpoint excluded all four then-present proof modules. RuntimePeakProofs adds the sixth omitted helper and has no production roots. Runtime/scenario definitions and every axiom list still require explicit metadata. The cache does not invent a shadow semantic declaration to fill the omission.

Provenance includes the official PrismPM commit, helper source and binary hashes, Cargo lock hash, exact vendored LexLean crate hash and reproduction commands. This diagnostic use of the official SDK changes no compressor production dependency or lock. The earlier four-file cache tied to source commit `314c9b5`, and the fourteen-module e87 cache (SHA-256 `7bf8c8898aac8776919c0c10646a19b65cb81a8b23f461c181762705b78386e9`), remain available in Git history. The current cache adds proof/cost/ticket modules, the guard-preserving fused moved-child update, and the endpoint peak helpers with their conditional arithmetic proofs and source-owned scenarios. Existing API signatures and runtime layouts are preserved. The separate peak receipts do not replace or extend the nine-suite/577-policy qualification of the previous 23 modules.

The retained pre-fix preservation failure and candidate #393 consumer qualification remain separate evidence. This cache does not accept export, establish pinned SDK acceptance, or measure physical memory.
