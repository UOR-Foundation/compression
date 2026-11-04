# Official compiler-parsed diagnostic cache

`runtime-owners.json` is one read-only diagnostic bundle for matching source call signatures in another executor. It includes the complete official parsed modules for the seven runtime/audit modules and their seven scenario modules. The `.lex.tex` sources remain authoritative; no production build, compiler or runtime loads this cache as a semantic source.

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

Provenance includes the official PrismPM commit, helper source and binary hashes, Cargo lock hash, exact vendored LexLean crate hash and reproduction commands. This diagnostic use of the official SDK changes no compressor production dependency or lock. The earlier four-file cache tied to source commit `314c9b5` is superseded by this bundle.

The known production preservation regression remains owned by LexLean #310 / PR #393. This cache does not change that gate's result, establish pinned SDK acceptance, or measure physical memory.
