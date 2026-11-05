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

This executor's official LexLean check/build artifacts retain generated modules,
lexicon closures and production eligibility, but no serialized semantic module
AST. The already-built reader has not been transferred to this executor, so no
machine parsed cache is claimed here yet. The export recipe is ready for reuse
with that reader. Diagnostic caches do not qualify runtime behavior, proofs,
production export, physical memory, SDK locks or release acceptance.
