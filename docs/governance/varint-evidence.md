<!-- uorc:non-authoritative -->
# Bounded ULEB128 verification evidence (issue #10)

The authorized specification is the repository LexLean graph. The missing codec
semantics have been authored there from the issue requirements. The generated
[UORC-VARINT.md](../../UORC-VARINT.md) is the reviewable contract projection.
This evidence record supplies no independent wire-format rule.

## Source and implementation

`Uorc.Varint` contains the actual encoder, bounded prefix scanner, exact decoder,
mathematical reference encoder, and general proof terms in LexLean's closed
native-core representation. `Uorc.VarintContract` owns the contract summaries,
typed-failure inventory, proof roots, and planted mutations. Registry ownership,
the scenario, ledger, model projection, README, verification summary, and
conformance table are updated together.

Every acceptance invocation reconstructs the native declarations from LexLean
source with the exact SDK. No handwritten Lean file, external product object,
compiler replacement, or host implementation is an acceptance input. The six
computational entry/helper roots have exact empty axiom sets. Each proof records
its actual subset of the standard Lean foundations `propext`, `Quot.sound`, and
`Classical.choice`; other axioms, including admitted proofs, are rejected.

The mathematical encoder uses decreasing division. The executable encoder uses
a ten-step structurally recursive implementation; `fuel_eq` and `bounded_eq`
connect them for every admitted U64 value. The public domain check and refinement
proof prevent fuel exhaustion from producing a truncated successful encoding.
The prefix scanner also has a ten-byte allowance and preserves the untouched
suffix. Exact decoding checks that suffix after validating the consumed prefix.
All six failures have a distinct typed constructor and a byte offset.

## General proofs and closed observations

The proof inventory includes both public round trips, exact length, the
one-to-ten-byte bound, minimum representation length, scanner reconstruction,
prefix/suffix preservation, and executable/reference encoder refinement. These
quantify over arbitrary inputs in their stated domains. They are not a finite
sample advertised as a universal theorem.

The 236 closed vectors are independent literal observations checked by kernel
reflexivity against the computational definitions. They cover encode/decode
pairs for 36 values, including every seven-bit width transition and U64 maximum;
ten truncation lengths; nine overlong spellings; every overflowing terminal
payload 2–127 in byte ten; continuation overflow; invalid byte positions; trailing
suffixes; and inputs above the encoder domain. The generated inventory binds
every vector to its source theorem.

## Planted defects and controls

The initial registered conformance test failed because `Varint.lex.tex` did not
exist. The completed source and projection tests pass. The required
`just varint-proofs` runs every source-owned mutation in a fresh source-only root.
Each changes only the selected computational definition's expression graph;
shared original nodes, proof terms, and vector expectations remain unchanged.

| Defect | Expected rejecting kernel declaration |
| --- | --- |
| Encoder radix 128 changed to 129 | `Uorc.Varint.fuel_eq` |
| Decoder allowance reduced from ten bytes to nine | `Uorc.Varint.chunk_encode` |
| Encoder maximum widened to include 2^64 | `Uorc.Varint.encode_overflow_18446744073709551616` |

A successful negative requires LLV7002 attributed to the exact native declaration
and a kernel application/declaration type mismatch. Syntax failures, missing
constants, unrelated proofs, wrong source files, missing tools, signals, and
successful exits do not qualify. Every mutant is followed by a fresh successful
verification of the restored complete graph. Logs and source hashes, owning
roots, exit codes, restored build IDs and attestations are retained under
`.prism/uorc/varint.*` and uploaded with offline CI evidence.

The harness tests also plant wrong-owner diagnostics, omitted/filtered gates,
unknown arguments, and accidental shared-node mutation. The full unfiltered
harness test file is invoked by the Rust workspace tests. Full `just vv` remains
the acceptance boundary, including claim-disposition mutations and complete
clean-root reproduction. This primitive does not claim archive/machine execution
or completion of M0's separate SDK capability reconciliation.
