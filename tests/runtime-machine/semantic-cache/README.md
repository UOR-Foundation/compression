# Authentic machine semantic cache

The authoritative source is the exact committed `.lex.tex`, never this cache.
The existing official PrismPM reader uses `lexlean::sdk::read_module` and
`write_module`; export-semantic-cache.py only orchestrates that reader. It
checks the existing reader's source/binary/SDK identities, exact source commit,
repeat reads and byte-exact notation round trips. It builds no parser/compiler.

Use the existing authenticated helper from the Heap cache producer:

```
python tests/runtime-machine/export-semantic-cache.py \
  --helper /absolute/existing/prismpm-semantic-source \
  --prismpm-root /absolute/official/PrismPM \
  --source-ref <exact-public-machine-commit> \
  --output tests/runtime-machine/semantic-cache/runtime-machine.json
```

The default selection is RuntimeMachine, RuntimeMachineEngine and
RuntimeMachineRefinement. Repeat --module to select additional exact modules.
The producer fails on an unavailable/mismatched reader or noncanonical source;
a missing cache must not be replaced with an approximated AST or generated Lean.

The unchanged RuntimeMachine helper module now has an authenticated cache in
public commit 0ff59f5187f5a9ca78118e141c72031dc540e5a1 at
`tests/heap/semantic-cache/machine-entry-ad6aac3.json`. Exact cache SHA256 is
`5676b8d3a9a1a1fa82508fd63187f9f9356fe4c86ceac03dc3f749ed187b0fb0`;
its49,284-byte source is SHA256
`7dc5661651faeac4d267157303971d77601be59d3ab2ae6ce1c6ab2f8ef20e81`.
This executor fetched and verified those exact bytes and all31 definition entries.
It is the frozen ad6aac30 helper input, not a parse of later Engine, Semantic or
Refinement increments. The already-built reader has not been transferred here;
those newer modules still require fresh exports through the same retained reader.

Official LexLean check/build artifacts here retain generated modules, lexicon
closures and production eligibility, but no serialized semantic module AST.
The export recipe remains ready for the existing reader. Diagnostic caches do not
qualify runtime behavior, proofs, production export, physical memory, SDK locks
or release acceptance.
