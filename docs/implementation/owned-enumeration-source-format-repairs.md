# Exact enumeration source-format repairs: unqualified

This checkpoint responds only to actual source-format feedback on the frozen
`9ca824709b0c6ffe4dbb647cb11a050b04247c19` proof roots. It does not expand the
mathematical work or claim a compiler/kernel pass.

The retained compiler's ordinary diagnostic check, build and verify attempts
all failed before kernel checking:

- Model: LLT4001, `phase link: semantic-module JSON member
  $.declarations[8].proof.definitions is outside the closed schema`.
- Owned proof import closure: LLT4001, `phase link: binder ticket in
  ticketMatches is spelled like the declaration ticket and would capture it
  in generated Lean`.

These are retained compiler diagnostic results, not pinned-SDK acceptance.
The original qualification receipt has SHA256
`d7c6eddfc56234e95927a3199b7d70048302d78f5e2e20a52f74823a283a03b2`.
The exact compiler messages contain backticks around the identified names;
only that punctuation is omitted in the readable excerpts above.

## Only the identified repair classes

1. Omit two empty `linear_arithmetic.definitions` fields in Model, on
   `model_adjacent_ranks_have_no_intermediate_rank` and
   `model_decoder_quotient_strictly_decreases`. Every nonempty definitions list
   and every other proof node/strategy stays unchanged.
2. Alpha-rename `ticketMatches`'s parameter `ticket` to fresh `presentedTicket`,
   with exactly its three lexically bound variable projections.
3. Apply the same capture-avoiding rename to `acknowledge`, with its one bound
   condition argument. This additional collision was found by source audit
   under the documented rule, not a second observed compiler result.

Argument order/types, function/member references, fields, state transitions,
ownership and all 63 theorem statements/domains/policies are preserved. The
owned proof module stays byte-identical. No source import or declaration is
pruned, no language/compiler/lock/policy changes, and no acceptance is weakened.

`tests/enumeration/source_format_equivalence.py` compares exact baseline source,
normalizes only the two permitted empty proof fields, and reverses only the two
scope-aware binder renames. It requires whole-module AST and byte restoration,
separate records for both production definitions, unchanged global function
references and exact old/new canonical contracts for every theorem. Its
negative controls plant unrelated proof/domain/reference/production changes
and require rejection, then restore the passing baseline. It rejects optimized
Python and filters. This structural diagnostic is not an elaborator or proof.

## Fresh evidence, old receipts retained

`owned-enumeration-source-format-repair-evidence.json` records the exact old/new
source, statement/domain and proof identities, alpha-equivalence records, audit
negative controls and fresh runtime command outputs.
`owned-enumeration-proof-repair-finite-evidence.json` is a new actual finite
statement-evaluation receipt bound to the repaired source/import closures.

The original runtime diagnostics and four intended source-IR mutations are
rerun against these exact repaired files. All 63 finite statement checks and
three intended model IR mutations are rerun as well. False antecedents and
arbitrary empty-Arena machine fixtures remain explicitly separated from general
proof or reachable-state evidence.

All prior runtime, frozen-current-Core, proof and Stream receipts remain
unchanged and tied to their original checkpoints. They are not relabeled as
new same-source results. No fresh whole-current-Core or Stream qualification is
claimed by this repair; production alpha-equivalence is separately checked.

## Same complete official obligations

Send both exact source roots back to the same retained executor:

```
lexlean check src/Uorc/Search/OwnedEnumerationModel.lex.tex
lexlean build src/Uorc/Search/OwnedEnumerationModel.lex.tex
lexlean verify src/Uorc/Search/OwnedEnumerationModel.lex.tex
lexlean check src/Uorc/Search/OwnedEnumerationProofs.lex.tex
lexlean build src/Uorc/Search/OwnedEnumerationProofs.lex.tex
lexlean verify src/Uorc/Search/OwnedEnumerationProofs.lex.tex
```

Bind every source/configuration/import identity, preserve all 63 declarations
and report actual diagnostics, kernel replay and observed axiom results. These
commands have not been re-executed for the repair at publication. Expected
standard axiom policies remain declarations, not observed audits. Complete
project verification and exact-SDK `just vv` remain mandatory.

Forward shortlex ordering, actual producer/arena/builder refinement, direct
candidate-sequence handoff to the Wire-owned closed trial, canonical membership,
fair independent discovery, native execution and full SDK acceptance remain
open. The reusable Stream adapter is not inserted as a redundant reader into
the closed trial. Ordinary best-known compression must not wait for exhaustive
certificate search.
