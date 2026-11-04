# Official compiler-parsed diagnostic cache

`runtime-owners.json` is one read-only diagnostic bundle for matching source call signatures in another executor. It includes all 23 owned modules: runtime/audit modules, four proof modules, the reference for moved allocation and scenario modules. The `.lex.tex` sources remain authoritative; no production build, compiler or runtime loads this cache as a semantic source. Each module has an explicit role, and every declaration retains its actual executable/axiom metadata.

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

Provenance includes the official PrismPM commit, helper source and binary hashes, Cargo lock hash, exact vendored LexLean crate hash and reproduction commands. This diagnostic use of the official SDK changes no compressor production dependency or lock. The earlier four-file cache tied to source commit `314c9b5`, and the fourteen-module e87 cache (SHA-256 `7bf8c8898aac8776919c0c10646a19b65cb81a8b23f461c181762705b78386e9`), remain available in Git history. The current cache adds proof/cost/ticket modules and the guard-preserving fused moved-child update; existing API signatures are preserved.

The retained pre-fix preservation failure and candidate #393 consumer qualification remain separate evidence. This cache does not accept export, establish pinned SDK acceptance, or measure physical memory.
