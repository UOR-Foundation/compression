# Singleton-byte source factoring

This is a retained negative optimization experiment. Matched standard
verification on SDK71 produced certificateB6,703,926 bytes for fc419 and
6,866,519 bytes for415d8, against the unchanged4,194,304-byte guard.
Only Builder changed in the identical21-source/21-root context. The split
therefore did not improve the export result; production restores fc419.
The complete candidate remains in immutable commit
69f7b8986388fc70c2bec090b1b44fc92611bc68, with its original proof/test evidence.

The retained reference and independent256 literal controls pass fresh
qualification against restored fc419: three affected kernels, all256 Lean-VM
controls and all298 exact policies. No limits were raised and standard
verification was not repeated for that restoration. See the disjoint
[matched experiment and restored-source receipts](runtime-byte-builder-matched-qualification/README.md).

The discarded candidate retained singletonBytes' original top four decisions and
delegated each remaining depth-four subtree to one of16 source helpers.
Expanding those calls reconstructs the exact fc419 singleton body in official
parsed form. All other existing declarations are unchanged. Each invocation
still makes eight ordered byte comparisons and selects the same one-byte
literal; no table, extra backing carrier or new input restriction is introduced.

Original builder SHA256:
`fc41954a33f2b396f694714de0e3daf7af612d6bd271abd48f0ca93761cbd77a`.
Discarded candidate builder:
`415d8c8d8183b3787611cf96f5620009177332c71f2a198861c1a77df32bb075`.
RuntimeByteBuilderSingletonReference preserves the original helper as an
explicitly non-executable definition. Its generally quantified source-owned
reflexivity theorem proves actual Bytes equality for every UInt8 argument,
without a premise or finite input restriction. The earlier three-body whole-step
reference is not asserted to preserve this separately changed helper.

Official formatting, source checking/build and all three affected kernel modules
pass against b342 Heap, ce9 Sequence and8b3 RuntimeValue. Every reused dependency
has fresh generated-Lean byte equality to its qualified input. All314 axiom
policies match exactly:54 builder definitions,2 reference/theorem declarations
and258 test definitions. All256 independently expected raw-byte controls pass in
Lean VM. This is source/VM execution, not new native C execution or a rerun of the
earlier35-control builder suite.

A single literal7e-to7f defect keeps the runtime source kernel-valid but rejects
the universal equality and the independent singleton_7e control/suite. Exact
source restoration passes all three kernels and all256 controls again. The
[receipt](runtime-byte-builder-singleton-evidence.json) preserves identities,
commands, declared/actual policies and mutation outcomes.

The main helper's parsed body changes from2296 dictionary objects/255 conditions
to168 objects/15 conditions plus16 calls. Each extracted helper has136 objects
and15 conditions. Total conditions remain255; these are per-function structural
measurements, not a total-size reduction, runtime speed or certificate-size
measurement. Candidate helpers were executable; references/tests are not.

The closed primitive registry at the inspected SDK71 has no direct dynamic
UInt8-to-Bytes constructor or byte update. Slice uses an existing backing; the
UTF8 encoder has different semantics. This scoped inspection is not a claim
about every newer SDK and introduced no new language primitive.

The original source/proof/VM qualification above preceded the matched export
experiment. Its former untested status is superseded by the exact refusal at
6,866,519 bytes for415d8. No accepted certificate or export follows. The
[historical Product scope](runtime-byte-builder-current-b-scope.json) retains its
different compiler binary identity; the new matched control reproduces its
baseline byte count without claiming binary identity. No limit, proof check,
compiler or product lock changes.
