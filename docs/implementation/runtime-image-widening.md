# Scalar-offset widening investigation

The unchanged public RuntimeImage source7a converts a UInt64 byte offset with a
64-level recursive decoder. A one-expression candidate indexes with
`(prim to_nat offset : Nat)` instead of `(wordToNatAux 64 offset)`.
The [official SDK71 contract](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/SPEC.md#widening-conversions-language-12)
defines to_nat as a total exact unsigned widening conversion, elaborated into
bit tests and balanced sums. It is a language1.2 facility. The product is still
locked to language1.1, so the public getter and product configuration remain
unchanged. This is a diagnostic candidate, not an alternate shipped path.

Candidate source SHA256 is
`01b437150e5b2ffb9d3dd36fed1fc4399c58770d9818dc4ceef3d5582398875e`.
Its source/kernel checks, all67 exact production/proof/test policies and nine
source scenarios pass. Two semantic-resolution proof statements change to
name the primitive index; the other30 declarations are unchanged. Masking the
offset's top bit leaves the runtime source valid but fails the scenario suite;
restoring the candidate passes. These results do not prove general equality
with the old handwritten decoder.

Official Std lowering/rendering succeeds for readByte/readSpanByte, reducing
their reachable definitions from11/13 to8/10. The generated primitive initially
contains64 bit tests and balanced sums. In the inspected optimized x86_64
Rust1.97.1 output, nodeByte checks and indexes the original scalar offset
directly, with no conversion loop or call. The old conversion remains
recursive in its separately inspected output. This is a generated-code
observation, not a timing result or accepted preservation export.

A single fresh diagnostic render combines the unchanged source-owned setup
with the candidate getter in one generated Arena type. The unchanged allocator
harness passes all12 reads: image lengths1,1024,1048576, each at zero, last,
end and UInt64.max. Valid reads return the expected byte and invalid reads a
range error. All measured alloc/zeroed/realloc/dealloc/requested-byte counters
are zero. Actual native Arena/image owners stay live throughout each read;
argument header clones are measured. Setup, payload construction, printing
and final owner destruction are excluded. The old getter probe was not rerun.

The [source-bound receipt](runtime-image-widening-evidence.json) and
[observations](runtime-image-widening-allocations.csv) retain exact identities.
The exact diagnostic project matches the existing runtime-image-probe snapshot.
No SDK build, certificate/export, physical RSS/timing measurement, universal
ownership bound or language1.1 acceptance is inferred. Adoption needs aligned
language/SDK qualification and a separately scoped conversion/refinement review.
