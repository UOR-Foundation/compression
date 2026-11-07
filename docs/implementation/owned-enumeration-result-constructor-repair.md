<!-- uorc:non-authoritative -->
# Exact producer Result constructor annotations

This is source-only progress against public checkpoint
`48b9436b2443c1fd3df725572cdea10863f11f39`. That checkpoint includes the separate
Stream Phase repair. Its producer, conversion helper, Model and OwnedProofs are
byte-identical to `134db36039927f559b43caa28f744c8338699857`.

The retained official qualifier reported OwnedProofs source-admission failure
`LLT4001`: `constructor Result.ok requires explicit success and error type
arguments`. That result was forwarded by the parent. This repair does not
independently rerun or remeasure the retained receipt. The qualifier's exact
first-error localization, also forwarded by the parent, is `appendStep` at
`/declarations/23/body/branches/1/body/arguments/2`: `Result.ok built` as the
third argument of `builderResult`, requiring `[BuilderTransition,
BuilderFailure]`. The corresponding `finishStep` site is a confirmed
same-class finding. It and the two `acknowledge` sites are statically complete
source-audit findings, not additional independent compiler stops.

## Compiler and repository evidence

Read-only inspection of pinned LexLean commit
`71defb4c663aaec5b9a3aab04cbc6a03e7106227` establishes the actual constructor
schema and argument order in `crates/lexlean/src/ir/semantic.rs`:

- Lines 463–468 define constructor `type_arguments` as `Vec<SemanticType>`.
- Lines 7669–7688 require exactly `[ok, error]` for both local `Result.ok` and
  `Result.error`, construct `Result { ok, error }`, and select the corresponding
  success or error payload type.
- The inspected compiler source SHA-256 is
  `c5309614b5693bed5e5f4c75aa28bcb5c98e88443b0e8095d00605e5a7edef21`.

No compiler checkout, bootstrap or build was performed. A broad initial Git
search encountered a missing partial-clone object and a lazy-fetch attempt;
it was cancelled, with `Proxy CONNECT aborted`. The required compiler source
was available for targeted local-object reads. No compiler execution result
is inferred from those reads.

Existing repository examples independently exhibit this schema:

- `Uorc.Core.ByteBuilder.start`, declaration 7, uses JSON `type_arguments`
  `[Builder, Error]` in both `Result.error` and `Result.ok` branches, matching
  its declared `Result Builder Error` return type.
- `Uorc.Core.RuntimeByteBuilder.beginAppendOwned` and `beginFinishOwned`
  return `Result BuilderTransition BuilderFailure` and use the surface form
  `[BuilderTransition, BuilderFailure]` in both constructors.

The gate verifies those example files remain byte-identical to the baseline,
checks the relevant signatures and annotations, and records their hashes. It
does not compile them or use a replacement SDK implementation.

## Exactly four source changes

Only `src/Uorc/Search/OwnedEnumeration.lex.tex` changes among source files.
The JSON type nodes use `kind: named`, an unchanged member reference, and an
empty `arguments` array. Imported builder types have their full existing
`Uorc.Core.RuntimeByteBuilder` module qualifier. `Cursor` and `Error` remain
local names.

1. `/declarations/23/body/branches/1/body/arguments/2`, `appendStep`:
   add `[Uorc.Core.RuntimeByteBuilder.BuilderTransition,
   Uorc.Core.RuntimeByteBuilder.BuilderFailure]` to `Result.ok built`.
   This is the retained compiler's localized first error.
2. `/declarations/24/body/branches/1/body/arguments/2`, `finishStep`:
   add the same pair to `Result.ok built`, a confirmed same-class source finding.
3. `/declarations/27/body/then_value`, `acknowledge`:
   add `[Cursor, Error]` to `Result.ok`, a same-class source finding.
4. `/declarations/27/body/else_value`, `acknowledge`:
   add `[Cursor, Error]` to `Result.error`, a same-class source finding.

The first two terms are the third argument of `builderResult`, whose exact
third parameter is `Result BuilderTransition BuilderFailure`. Each `built`
still refers to its original successful match binder. The other pair is fixed
by `acknowledge`'s unchanged declared `Result Cursor Error` return type.
Match patterns are not constructor terms and are not modified.

Before producer SHA-256:
`1fac8abfab28e1d3e50e893af515547b4a5a9798ed7b9fefeb9dcee9b2ec71d6`.

After producer SHA-256:
`a736436cc5c54a4740d1660a0d06f8b5b548ed5051f356d29de22c1e4117cdd0`.

No computation, payload, branch, declared result or parameter type, binder,
call argument order, record field, custody transition, import, declaration
policy, theorem statement or proof changes. The complete helper, Model and
OwnedProofs sources remain byte-identical, including all 45 Model and 18
OwnedProofs theorem declarations. Stream and every historical receipt remain
untouched by this repair. Nothing is relabelled as qualified.

## Executed focused gate

Run without filters or Python optimization:

```
PYTHONDONTWRITEBYTECODE=1 python tests/enumeration/result_constructor_equivalence.py
```

The receipt is `owned-enumeration-result-constructor-evidence.json`. It binds
the baseline, exact before/after source bytes, gate bytes, compiler source
audit, repository examples, four site/type/payload/declaration hash maps,
preserved helper and proof sources, and every complete theorem hash.

The gate checks the entire canonical producer AST and its non-JSON surface,
then erases only the four named annotation arrays. The result must recover
every original source byte. No blanket `type_arguments` erasure is allowed.
The producer's complete Result-constructor inventory must be exactly those
four sites.

Twelve small constructor-fragment evaluations compare the four actual terms
over three distinct symbolic payload environments. These verify that both
`built` payloads retain object identity, the success acknowledgement retains
all existing cursor fields except its original phase/resume/release-slot
assignments, and the error remains `Error.invalidTicket`. They do not execute
the surrounding Arena, producer state machine, ticket predicate or native
runtime. Symbolic payloads are not claimed to be valid heap states. Exact
whole-source erasure is the stronger structural result; the twelve cases are
only focused evaluator diagnostics.

Executed negative controls reject:

- 20 actual source-IR mutations: absent, swapped, wrong, one-element and extra
  type pairs independently at each of the four sites.
- Seven further source-IR mutations: wrong `built` reference, wrong match
  binder, changed acknowledgement error, changed declared return type,
  changed custody phase, reordered call arguments, and an unrequested fifth
  annotation.
- Two changed complete theorem declarations, one from each original root.
- Seven real subprocess controls: Python `-O`, a command-line filter, omitted
  exact-source validation, omitted complete theorem inventory, omitted
  fragment equivalence, a wrong source-file selection, and omitted mutations.

Each source-IR defect is applied to a complete canonical candidate and passed
to the same source checker; it must fail with the expected diagnostic. Each
restored candidate must pass afterward. The tracked source is never replaced
with a mutant. Process controls execute temporary altered gate files and
must fail before producing a success receipt. A normalized Python AST seal
covers the complete gate implementation, including its own validation logic;
there are no removable `assert` checks. The seal is an audit against accidental
or deliberate omissions, not an authentication claim against an actor who can
rewrite both the gate and its expected seal.

## Still unqualified

This gate has not run official LexLean source checking, build, verification,
kernel proof or an axiom audit. It supplies no general conversion law,
whole-Core compatibility result, native execution, archive/controller
integration result, full-project qualification or locked-SDK acceptance.
No broad runtime rerun is substituted for source admission.

The retained official executor must now recheck these exact new producer
bytes through the original Model and OwnedProofs roots, at unchanged stock
settings. The repaired Stream's own official obligations also remain. Both
original roots, all full-project checks, and exact-SDK `just vv` remain the
acceptance boundary. Any subsequent observed source or proof failure must be
retained honestly and handled as a separate concrete blocker.
