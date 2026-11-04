<!-- uorc:non-authoritative -->

# Language 1.2 source readiness

This work starts from public integration commit
`5a01915fb467652b28a9fa11339bbea614f8b280` (tree
`3d4440d7dcd5b2b7e0deacab207a356bc29dedc2`, 215 modules). It prepares a
separate source candidate and checks source equivalence. It changes no
authoritative module, project configuration, integration lock, template policy,
acceptance command, executable annotation, or axiom policy.

## Supported source operation

`prepare.py` delegates module parsing, writing, and notation conversion to the
existing official PrismPM helper, which calls `lexlean::sdk::read_module`,
`write_module`, and `to_notation`. It does not implement a LexLean parser.
For legacy modules, only the root schema moves from `/1` to `/2`; every
declaration and all other module fields must read back identically. Builtin
glossary header references move from 1.1.0 to 1.2.0 and the separate candidate
configuration selects language 1.2. Existing notation is rendered by the same
official writer and checked again. Absent annotation fields remain absent.

At the frozen base this preserves 6,042 declarations, including 2,384 explicit
axiom arrays, 1,260 explicit executable flags (942 true and 318 false), 47
production annotations, and all 308 native-core declaration policies. The
complete native Varint module, including imports, term nodes and proof nodes,
is compared, with only its root schema changing. One existing notation module,
`Core/RuntimeMachineTests.lex.tex`, needs the official writer's multiline
formatting; its parsed IR remains identical. `Specification.lex.tex` has
noncanonical legacy JSON, which the official writer canonicalizes without
changing its declarations. Neither correction removes a source obligation.

In inspected PrismPM source `33953384ea84a6196a907982dc30f6a3c0d1a955`, this
helper belongs to internal conformance tools and is not installed by that
revision's SDK recipe. This does not determine the current candidate SDK's
capabilities. Its existence in an official source checkout does not make it an
installed SDK command. Reuse an already available official helper for
diagnostics; do not copy its implementation into this project or replace the
release pipeline with a host tool. The selected installed SDK's public source
read/write or export interface must be identified and qualified before this
becomes the accepted project migration.

With an existing official helper and a separate output directory:

```sh
python3 tests/migration/prepare.py \
  --source /absolute/frozen-compression \
  --output /absolute/new-migration-output \
  --reader /absolute/official/prismpm-semantic-source
python3 tests/migration/audit.py \
  --source /absolute/frozen-compression \
  --candidate /absolute/new-migration-output/candidate \
  --reader /absolute/official/prismpm-semantic-source \
  --report /absolute/new-migration-output/readback-audit.json
python3 tests/migration/test_annotation_preservation.py \
  --reader /absolute/official/prismpm-semantic-source \
  --source /absolute/new-migration-output/candidate/src/Uorc/Core/RuntimePeakTests.lex.tex
```

The last command uses the actual official writer and reader to plant losses
of executable, exact-axiom, and production annotations. The unchanged source
passes and each planted change is rejected by the same full-IR comparison.
Output must be outside the source checkout and previously absent. Path/Git
lexicons are explicitly refused until their own migration is supplied.

## Compiler boundary

The current official compiler's `lexlean migrate --to 1.2 --check` refuses
this mixed project without changing any input: 43 notation modules are not
language-1.1 data inputs, the native-core Varint module is outside that
command's semantic-only migration, and Specification's legacy JSON is not
canonical. This is why changing only the language row is insufficient.

The separate candidate can generate its own diagnostic lock with the existing
compiler, then run the actual formatter:

```sh
lexlean --project /absolute/new-migration-output/candidate/lexlean.toml lock
lexlean --project /absolute/new-migration-output/candidate/lexlean.toml fmt --check --all
```

On the frozen base, whole-graph formatting reaches the real source error in
`Core/PlanningTests.lex.tex`: executable `checkTrace` calls non-executable
`Core.StepMachine.step` (`LLT4001`). Its owner must repair the source; migration
does not remove the executable flag or filter the module. A selected formatter
check of Specification, Registry, Claims, VarintContract, Integrity.Contract,
and native Varint succeeds. That selected result is not whole-graph acceptance.

## Authority and validation dependencies

`crates/model/src/authority.rs` currently extracts legacy semanticdata from
Specification, Registry, Claims, VarintContract, and Integrity.Contract;
`varint.rs` separately extracts native coredata. Canonical notation cannot be
activated while those readers still depend on the old surface. The validators
and generators already consume module JSON; a supported official SDK reader
can supply the same values without another grammar or a second authority.

The readiness API is `AuthorityGraph::load_with_reader(root, reader)`, with
matching ClaimPolicy and VarintContract entry points. `ModuleReader` accepts
complete source text and returns the complete official decoded module value,
including all proof, axiom, executable and production metadata, or an error.
The importer validates the expected module and semantic/core schema, then runs
the existing validators and generators. The unchanged default entry points
remain legacy-schema1 readers; they do not silently accept notation or try a
host fallback. The explicit example requires the reader path as a positional
argument and fails when it is absent or unsuccessful.

With an already installed project Rust toolchain, the diagnostic harness can
exercise the actual notation authority and existing code generator:

```sh
cargo run --offline --locked -p repo-model --example authority_sdk_projection -- \
  /absolute/new-migration-output/candidate \
  /absolute/official/prismpm-semantic-source /absolute/projections --write
cargo run --offline --locked -p repo-model --example authority_sdk_projection -- \
  /absolute/new-migration-output/candidate \
  /absolute/official/prismpm-semantic-source /absolute/projections --check
```

On the frozen base, direct baseline and notation imports produce all 29
projections byte-identically. Actual validators reject planted charter/body
and claim-axiom errors, malformed data and notation, a missing explicit
reader, and a stale projection after changing source. These are source/model
diagnostics; the example is not wired in as an alternative acceptance gate.

Other affected consumers are the claim mutation scripts, integrity binding,
vector and stream scripts, product projection/control scripts, varint
validation, and `xtask/src/product_receipts.rs`. Their read and mutation/write
operations need the same supported SDK interface, with their current defect
owners, source inventories, and rejection requirements preserved. An exported
JSON cache is diagnostic evidence, not a replacement source for these gates.

The remaining accepted path is concrete: expose and qualify the official
installed source interface; connect each consumer to it; fix the demonstrated
source and proof failures; render the immutable SDK/template bindings through
their supported update path; regenerate through the actual authority graph;
and run both complete original native-platform and offline gates in that
selected SDK. Neither this branch nor a newer host compiler result asserts
that those stages have passed. Builtin Option elimination and BinaryProgram
installed qualification retain their separate upstream ownership.
