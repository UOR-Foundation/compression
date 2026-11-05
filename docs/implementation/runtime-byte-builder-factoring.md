# ByteBuilder source factoring

The builder shares fixed-record updates and its phase dispatcher through six
source-owned helpers. The public carrier layout, control reservation, 4096-byte
block bound, reserve-before-copy ordering and balanced singleton-byte decision
tree are unchanged. The helpers add no allocation, retain or release operation.

The original RuntimeByteBuilder source has SHA256
`c7f67542e9f7a4f05f6161fe5fc23b26cf85769c3538d89f0e69f35990eac00d`.
The factored source has SHA256
`fc41954a33f2b396f694714de0e3daf7af612d6bd271abd48f0ca93761cbd77a`.
RuntimeByteBuilderFactorReference preserves the three changed original bodies:
clearCleanupTarget, cancelStep and step. Its definitions are non-executable and
outside the production closure. Three general reflexivity theorems equate the
actual new results with those references for every cursor and arena. They cover
typed failures, returned arena/cursor/outcome, deltas, copied bytes and work.
They establish implementation equivalence, not the global ownership invariant.

Official source build and all seven generated Lean modules pass kernel checking
against current Sequence `ce9ae14f...` and Heap `ee41c3fb...`. All44 builder and
reference axiom policies match actual kernel traces exactly. Native Lean/C
execution passes all35 existing builder controls and the aggregate suite.
Sequence, builder, tests and the assertion driver were compiled anew; reused
dependencies were accepted only after fresh generated-C byte comparisons.
The [source-bound receipt](runtime-byte-builder-factoring-evidence.json) records
that distinction and all44 exact policies. These checks use the retained language1.2
diagnostic project; the product's locked language1.1 project is unchanged.

Two planted defects leave the runtime source kernel-valid while making the
general bridge fail: retaining the old builder instead of its replacement
breaks all three equalities; changing terminal work from one to two breaks the
whole-step equality. Exact positive sources/configurations were restored and
the positive generated modules kernel-checked again. The existing VM control
run was still active when this checkpoint was published; native success is not
reported as its completion.

The official parsed-source body for clearCleanupTarget shrinks from2322 to466
dictionary objects, cancelStep from388 to246, and step from396 to66. Factored
helpers retain their own bodies. These are structural source metrics, not
certificate row sizes, native timing or physical memory measurements.

Product commit `97cc526d0a1f39a61c307514c89467003aaa65e5` separately reports a
RuntimeByteBuilder.step certificate-B resource refusal:7,530,370 source bytes
against the unchanged4,194,304-byte limit. Its verified builder source is the
original c7f version, with the older Sequence c352 version. The published
receipt does not identify the failing compiler executable, exact command or
diagnostic project configuration. Those missing identities remain with the
reporting owner. No verifier limit or proof statement changes.

Later Product commit e0a00895a74f0504e331eee6e82341997b174016 reports a
separately scoped full-verification refusal for the current fc419 builder:
certificateB source6,703,926 bytes against the same4,194,304-byte limit.
Its [receipt](https://github.com/UOR-Foundation/compression/blob/e0a00895a74f0504e331eee6e82341997b174016/validation/product/payload-image-core-compatibility.json)
supersedes the initial untested status for that exact current-source consumer.
Compiler/configuration/root-set reconciliation remains explicit; the numerical
difference from the earlier old-source receipt is not attributed to this
factoring without a matched context. No accepted export follows.

The newly reported Heap.releaseStep certificate-A deep-recursion failure and
the historical Sequence.step A probe are separate root-specific scopes. A
relation index cannot be transferred between their emitted programs. Neither
is the ByteBuilder certificate-B size refusal.
