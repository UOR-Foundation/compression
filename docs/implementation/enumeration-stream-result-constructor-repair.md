# Exact Stream Result constructor annotations

This separate annotation-only follow-up starts at the published producer repair
`a78af7c622cd787d8c1838fb7a369f04248fe9bf`, tree
`54858e784819b10c92a3f50c297898a272d6cfad`. Its Stream source is the same
Phase-repaired source published in `48b9436b2443c1fd3df725572cdea10863f11f39`.
The producer repair is preserved intact. No capability or claim is added.

## Exact source scope

Only `src/Uorc/Search/EnumerationStream.lex.tex` changes among source files.
Exactly four Result construction terms gain `type_arguments`:

1. `start`, `/declarations/11/body/then_value`: `Result.ok [Cursor, Error]`.
2. `start`, `/declarations/11/body/else_value`: `Result.error [Cursor, Error]`.
3. `acknowledge`, `/declarations/23/body/then_value`: `Result.ok [Cursor, Error]`.
4. `acknowledge`, `/declarations/23/body/else_value`: `Result.error [Cursor, Error]`.

Each type is a JSON `named` node with local member `Cursor` or `Error` and an
empty `arguments` array. Both constructors use success/error order, including
`Result.error`. Each enclosing function already declares `Result Cursor Error`.
The gate checks this exact declared return type before and after the edit at
each of the four sites. These are the complete Result construction-term
inventory in Stream. Match patterns are not construction terms and are unchanged.

All four sites are static same-class source findings, not four independent
compiler stops. The previously retained producer `LLT4001` is not a Stream
compiler receipt and is not relabelled here.

Stream source SHA-256:

- Before: `6c0c3275c0670e5ecb61d51457cee9da46a3f3e455df61f3d1d634cfb5cffd02`.
- After: `f45eb69da4831cecaaf282310b899c83b1b35098c4b0b97ea7d6e32172b95d6b`.

Erasing only those four specific arrays recovers every byte of the original
Stream source, including its imports and surrounding surface. No blanket
annotation erasure is used. Bodies, payloads, branch order, predicates, binder
names, argument order, record field order, declared types, policies, charges,
prepayment and custody transitions are unchanged. The nine Phase matches from
the prior repair remain intact.

## Compiler schema and existing examples

The separately published producer receipt records read-only inspection of
pinned LexLean `71defb4c663aaec5b9a3aab04cbc6a03e7106227`, file
`crates/lexlean/src/ir/semantic.rs`, SHA-256
`c5309614b5693bed5e5f4c75aa28bcb5c98e88443b0e8095d00605e5a7edef21`.
Its lines 463–468 define constructor `type_arguments`; lines 7669–7688 require
exactly `[ok, error]` for both Result constructors and select the corresponding
payload type. This gate binds that published audit unchanged; it does not
repeat the compiler source inspection or execute a compiler.

As independent repository schema examples, `Uorc.Core.ByteBuilder.start`,
declaration 7, uses `[Builder, Error]` on both `Result.error` and `Result.ok`,
matching its declared `Result Builder Error`. The gate checks both exact JSON
nodes and the return signature and records their hashes. Their file remains
byte-identical to the baseline. These checks are structural, not elaboration.

## Executed focused gate

Run from the repository root without filters or optimization:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B tests/enumeration/stream_result_constructor_equivalence.py
```

The final gate SHA-256 is
`352a85ed6dc87da6e536cf92355f87d1f7f1fdbfc22b205068a379b78db00556`.
Before changing the Stream source, this exact gate exited 1 with no stdout and:

```text
ValueError: wrong, swapped, missing or extra Result type pair: /declarations/11/body/then_value
```

After the four additions, the command exited 0 with empty stderr. Its complete
stdout is preserved in
[`enumeration-stream-result-constructor-evidence.json`](enumeration-stream-result-constructor-evidence.json).
The result is `pass-exact-source-annotation-audit-only`.

The gate parses duplicate-key-free canonical semantic JSON, checks the complete
38-declaration Stream module and its surrounding bytes, verifies the entire
Result construction inventory and four declared-type contexts, and compares the
candidate to the exact baseline plus only the authorized additions. Exact
four-site erasure must recover the complete original bytes.

All 522 other tracked baseline files are compared byte-for-byte and included in
the frozen SHA-256 manifest. This covers the published annotated producer,
conversion helper, all other Search/Core/Wire sources, model and policy records,
locks, existing gates and historical receipts. The manifest SHA-256 is
`a9cd97a9cb9da2dbcc5778a38fc2e3e859a687b339b0e2cc946e56be81346289`.
Both complete proof roots are also parsed: all 45 Model theorem declarations and
18 OwnedProofs declarations, including statements, bodies and axiom policies,
retain their full declaration hashes. Nothing is regenerated as a new claim.

### Planted defects and restoration

The same checker rejects 47 mutations to complete candidate Stream bytes:

- At each of the four sites, eight annotation defects: missing, swapped, wrong
  success type, wrong error type, one-element pair, extra element, incorrectly
  module-qualified success type, and a parameterized error type. That is 32
  independent annotation controls.
- Four changed success/error payload or custody bodies, both changed declared
  result types, reordered condition arguments, a changed parameter binder, a
  changed Phase predicate, a changed payment charge, a changed match binder,
  an unrequested match-pattern annotation, a changed axiom policy, and reordered
  success record fields. That is 14 unrelated-AST controls.
- One changed import in the non-JSON source surface.

Seven further frozen-file controls reject changed complete theorem policies in
both proof roots and changed bytes in the producer, conversion helper, model
claim policy, LexLean configuration and historical Stream runtime receipt.
The original candidate or file is rechecked after every mutation. Mutants are
in-memory candidates; none replace the tracked source.

Eleven real subprocess controls reject Python `-O`, a filter argument, a gate
with the source check omitted, omitted frozen-file checking, omitted complete
proof inventories, omitted schema examples, omitted source mutants, omitted
frozen-file mutants, wrong source-file selection, omitted final source reread,
and execution of an unchanged gate under the wrong filename. Each must fail
with the intended diagnostic and produce no success stdout. A normalized AST
seal covers the full gate implementation; there are no removable assertions.
This detects omissions and drift, not an attacker rewriting both implementation
and expected seal. Final rereads bind all frozen files and the Stream bytes
actually inspected, followed by one final exact-delta check.

These negative controls validate structural preservation only. They are not
executions of the mutated custody, payment or compiler behavior.

## Preserved historical evidence and remaining qualification

The earlier finite Stream Phase/runtime receipt remains byte-identical, with
SHA-256 `caae96cfac86f18d5ff86366faf89742b8a6bdfa87c236b5c08a5f9911cb9939`.
It still names source
`6c0c3275c0670e5ecb61d51457cee9da46a3f3e455df61f3d1d634cfb5cffd02`,
not this annotation-only source. No finite runtime receipt is rerun, upgraded,
or relabelled. The earlier producer receipt likewise remains unchanged and
still binds the published producer source
`a736436cc5c54a4740d1660a0d06f8b5b548ed5051f356d29de22c1e4117cdd0`.

This follow-up does not run official source checking, SDK/bootstrap tools,
compilation, a source evaluator, native execution, kernel proof or an axiom
audit. It supplies no new general conversion, custody, resource-bound or
archive/controller integration proof, no current-Core qualification and no
release acceptance. No compiler stop is claimed to have been cleared by this
structural gate.

The combined producer-and-Stream typed source head must be qualified separately
through the original Model and OwnedProofs roots and the Stream obligations,
at unchanged stock settings. Full-project checks and locked-SDK `just vv`
remain the acceptance boundary. Any resulting concrete compiler or proof
failure must be retained as its own evidence and blocker.
