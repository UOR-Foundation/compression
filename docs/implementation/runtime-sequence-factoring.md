# Sequence certificate factoring

RuntimeSequence.concatDown now keeps its ordered sequence-info lookups and
delegates height selection, left/right descent and fixed pending-result
construction to small source-owned functions. Runtime carriers, handles,
continuation fields, errors, ownership operations, Delta and logical work are
unchanged. The helpers introduce no Heap allocation, retain or release.

The exact previous source has SHA256
`c352ecaf1eb161df6df7c6f0fed7fc7868d00e4fc9fe3590271d40604785a29e`.
The factored source has SHA256
`ce9ae14fc5f4837858d67f2a06287fe502a2481ea65ab67e58c313e803ff89c9`.
The source-owned RuntimeSequenceConcatReference preserves the previous
concatDown body and the previous step body with only its concatDown call
redirected to that preserved body. Both reference functions are explicitly
non-executable and are outside the runtime dependency graph.

For every Arena and Cursor, two reflexivity theorems prove exact equality of
the new concatDown and step results to those references. Equality covers typed
errors and every returned arena/cursor/delta/work/outcome field; it assumes no
valid-lineage or finite test domain. Expanding the five added helpers also
reconstructs the previous concatDown semantic AST exactly. Every other old
declaration is unchanged. This is a general implementation-equivalence bridge,
not global ownership/representation preservation.

Official formatting, source checking/build and generated Lean kernel checks
passed against the fused Heap source `ee41c3fb...`. Exact axiom traces are
empty for concatCursor and concatPending, and Quot.sound plus propext for the
other helpers, changed function and reference theorems. No custom axiom or
sorry was introduced. Changing only concatPending.work from one to two leaves
the runtime source kernel-valid but makes both equality theorems fail. The
unchanged control passes. See the [source-bound receipt](runtime-sequence-factoring-evidence.json).

Certificate-A acceptance is separate and remains unqualified for this structural
source. The exact original step closure on diagnostic compiler71defb4 fails
its concatDown relation with kernel deep recursion. A first cursor-updater-only
candidate also failed on the fused Heap and was not published. Its public-Heap
probe ended with signal9 and no Lean diagnostic; an OOM attribution was not
established. No certificate limit, proof check, compiler or product lock was
changed. The completed nine-native-suite/577-policy receipt remains bound to
the earlier runtime source and is not relabeled as a new native execution.

The one complete certificate-A kernel probe of ce9 source ended with SIGKILL
after152 seconds, before its unchanged300-second bound, with no Lean diagnostic
or failed relation. Maximum Lean child RSS was12.24GiB; hierarchy OOM/kill
counters increased during that run. No PID-specific kernel attribution was
available. This is an unqualified resource termination, not an established
source/kernel rejection. The original complete wrapper/source were unchanged;
recorded certificate source is2,192,156 bytes, below4MiB. No retry was performed.

The earlier narrower prefix extraction omitted implicit generated implementation
companions. Its code-generation errors and dependency deep-recursion diagnostic
cannot qualify concatDown or substitute for complete A verification. The
[bounded outcome receipt](runtime-sequence-certificate-a-evidence.json) preserves
these distinctions. The [exact three-source ordinary probe](../../tests/heap/sequence-repro.md)
is available for the existing compiler-owning executor; no compiler build or
Library materialization is required.
