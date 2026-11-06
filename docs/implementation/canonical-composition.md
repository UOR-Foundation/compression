# Canonical event composition

These source functions complete composition decisions left open by the existing
logical-reference event helpers. They import those helpers and the independent
ReferenceSemantics authority. They do not introduce a second evaluator or cost
table. The current source checkpoint passes official formatting, linking and
build for22 modules. Its65 general relation statements and64 registered controls
are not yet kernel/execution qualified; see the exact dependency failure below.

## Calls and completed results

CanonicalCallComposition.capturePreparation(count) runs once per instruction,
before logical callee entry or loop initialization, including zero iterations.
Scan prepares observer captures and then updater captures. This follows the
existing eager resolveReferences calls in ReferenceSemantics.evaluateInstruction.
Physical rereads from a pinned immutable caller root are operational realization;
they do not imply a repeated semantic captureReadEvents prefix on every call.

resolvedCallPlan(lane,prefixArguments,capturedArguments) reuses callPlan with
prefix arguments followed by captures. Each invocation emits blockEnter then
parameter binding/allocation in that order. appliedCallPlan adds its one capture
prefix before this entry. The proposed relations quantify arbitrary argument
lists and bindings; tests include duplicate captures, scalar aliases and empty
sequences. The event tag alone cannot distinguish two reference-read lanes;
the argument/lane relation remains part of machine refinement.

CompletionOrigin distinguishes fresh and transferred bindings explicitly.
completeInstruction(fresh,binding,deadBindings) allocates its full logical payload
before last-use releases. The transferred case changes the role to instruction
and emits only last-use releases. Apply, final Fold state and completed
Tabulate/Scan output are transfers. Every other existing Action is fresh.
Scalar/null handles and equal payloads do not infer either origin or aliasing.

CanonicalComposition supplies applyEntry, tabulateStart, foldStart and scanStart.
They use the unchanged instructionPrefix, then eager capture preparation, then
the relevant accumulator start events. Scan starts output0 before its state
copy. finishTabulate and finishFold transfer the existing accumulator;
finishScan releases final state before output transfer and checked dead-binding
releases. No completed output-length allocation is added.

## Remaining bindings and loops

CanonicalRetirement.remainingFrom(count,slot,returnedSlot,liveAt) inspects logical
keys in ascending order. It skips the returned key before lookup, skips missing
slots, and includes each other live binding independently. remaining starts at0.
frameReturnPlan delegates returnPlan with that derived list. returnCoherent and
planRepresentsReturnedSlot state the actual returned-key/binding relationship;
value equality does not establish it. The mathematical lookup is
Nat -> Option Binding; an Engine realization must keep a bounded sparse-slot
cursor, not a copied reference binding list.

The existing tabulateIterationEvents, foldIterationEvents and scanIterationEvents
remain the append/state authorities. A returned byte causes byteWrite, growth1
allocation and returned-byte1 release. Old loop state is released before the
new returned state transfers; completedIteration follows replacement. Scan's
observer append precedes updater events, and loopUpdate follows updater
replacement. Native builder chunks and Heap cleanup add no logical byte events.

## Failed actions and prepaid attempts

CanonicalActionComposition.startIndex/startBasis use the existing instruction
prefix unchanged. The typed mathematical cursor consumes one event through
Schedule.decideEvent. Refusal returns the exact original cursor and accepted
usage; a separate paid-attempt stage prevents charging that observation again.

Index range failure follows the entire prefix and emits no byteRead. An in-range
attempt prepays one byteRead before ReferenceSemantics.indexSequence, including
the malformed-reference case where its read function returns None. Basis lookup
prepays one additional referenceRead before attempting the basis index, including
missing basis entries. Only successful results schedule fresh result allocation
and last-use releases. Reference failures retain their achieved semantic usage.

These are non-executable mathematical composition functions and drivers. The
machine owner separately owns the fixed-width instruction-prefix implementation.
The remaining Engine realization must preserve paid cursors, one shared semantic
ledger, bounded captures/retirement/byte state and explicit output/EOF handoff.
No runtime integration or semantic-budget acceptance is inferred here.

## Exact qualification boundary

Base integration is ad70ad9a32bff5cca1a039a60794ac5eb0e7c079; the supplied machine
audit is ad6aac30f6c73e5d9907c630054b30e58f2deb98. Their four shared canonical
authority modules retain identical existing definition bodies. Diagnostic copies
of the ten dependencies were migrated by the official SDK from language1.1 to1.2;
official reads verify exact declaration equality. Public authority files and
product configuration/locks are unchanged.

Six dependency kernels pass. ReferenceSemantics.leaf_index_step_refines_independent_sequence
fails its existing linear_arithmetic proof at generated line309. No proof was
removed, pruned or replaced. This blocks dependent kernel, actual axiom, scenario
and mutation qualification. The six-module axiom audit is230/232 exact: existing
RuntimeOpcode.fromCode and operandAt declare Quot.sound/propext but trace empty.
Neither metadata mismatch changes their passing source bodies or kernels.

The [source-bound receipt](canonical-composition-evidence.json) records these
outcomes, exact hashes and diagnostic configuration. No certificate/export,
native execution, whole-product refinement, pinned SDK or release acceptance is
claimed for this checkpoint.

Two later, explicitly modified-generated proof experiments are recorded in
[v1](../../tests/canonical-composition/reference-leaf-experiment-0e3/result.json)
and [v2](../../tests/canonical-composition/reference-leaf-experiment-v2/result.json).
V1 fails the positive case. V2 passes the unchanged-statement full-module kernel,
rejects both false controls, and matches all79 original Ref axiom policies.
The experiment copies are restored byte-exactly, reproducing the original
eight-error failure. No supported source route or official compiler change is
accepted by those experiments, so the composition source qualification remains
blocked. The diagnostic positive artifact is not installed as its foundation.
