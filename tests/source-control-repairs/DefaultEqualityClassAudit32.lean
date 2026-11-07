module
public import Uorc.Uorc.Search.WaitingEquality
open Uorc.Uorc.Search.WaitingEquality
public theorem byteArray_beq_eq (a b : ByteArray) : (a == b) = true ↔ a = b := by
  change (a.data == b.data) = true ↔ a = b
  cases a
  cases b
  simp
#print axioms byteArray_beq_eq
public theorem waitingEqCoreRuntimeValueHandle_iff (left right : Uorc.Uorc.Core.RuntimeValue.Handle) : (waitingEqCoreRuntimeValueHandle left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeValueHandle, LexLeanRuntime.equal, and_assoc]
#print axioms waitingEqCoreRuntimeValueHandle_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeValueHandle
public theorem waitingEqSearchFixedTemplatesTemplate_iff (left right : Uorc.Uorc.Search.FixedTemplates.Template) : (waitingEqSearchFixedTemplatesTemplate left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchFixedTemplatesTemplate, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeValueHandle_iff]
#print axioms waitingEqSearchFixedTemplatesTemplate_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchFixedTemplatesTemplate
public theorem waitingEqSearchFixedTemplatesCursor_iff (left right : Uorc.Uorc.Search.FixedTemplates.Cursor) : (waitingEqSearchFixedTemplatesCursor left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchFixedTemplatesCursor, LexLeanRuntime.equal, and_assoc, waitingEqSearchFixedTemplatesTemplate_iff]
#print axioms waitingEqSearchFixedTemplatesCursor_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchFixedTemplatesCursor
public theorem waitingEqSearchTemplateStreamPhase_iff (left right : Uorc.Uorc.Search.TemplateStream.Phase) : (waitingEqSearchTemplateStreamPhase left right = true) ↔ left = right := by
  cases left <;> cases right <;> constructor <;> intro h <;> first | rfl | cases h
#print axioms waitingEqSearchTemplateStreamPhase_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchTemplateStreamPhase
public theorem waitingEqCoreRuntimeSequenceRef_iff (left right : Uorc.Uorc.Core.RuntimeSequence.Ref) : (waitingEqCoreRuntimeSequenceRef left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeSequenceRef, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeValueHandle_iff]
#print axioms waitingEqCoreRuntimeSequenceRef_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeSequenceRef
public theorem waitingEqCoreRuntimeSequenceInstruction_iff (left right : Uorc.Uorc.Core.RuntimeSequence.Instruction) : (waitingEqCoreRuntimeSequenceInstruction left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeSequenceInstruction, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeSequenceRef_iff]
#print axioms waitingEqCoreRuntimeSequenceInstruction_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeSequenceInstruction
public theorem waitingEqCoreRuntimeSequencePlan_iff (left right : Uorc.Uorc.Core.RuntimeSequence.Plan) : (waitingEqCoreRuntimeSequencePlan left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeSequencePlan, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeSequenceInstruction_iff, waitingEqCoreRuntimeSequenceRef_iff]
#print axioms waitingEqCoreRuntimeSequencePlan_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeSequencePlan
public theorem waitingEqCoreRuntimeSequenceCursor_iff (left right : Uorc.Uorc.Core.RuntimeSequence.Cursor) : (waitingEqCoreRuntimeSequenceCursor left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeSequenceCursor, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeValueHandle_iff, waitingEqCoreRuntimeSequencePlan_iff]
#print axioms waitingEqCoreRuntimeSequenceCursor_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeSequenceCursor
public theorem waitingEqOptionCoreRuntimeSequenceCursor_iff (left right : Option (Uorc.Uorc.Core.RuntimeSequence.Cursor)) : (waitingEqOptionCoreRuntimeSequenceCursor left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqOptionCoreRuntimeSequenceCursor, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeSequenceCursor_iff]
#print axioms waitingEqOptionCoreRuntimeSequenceCursor_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqOptionCoreRuntimeSequenceCursor
public theorem waitingEqCoreRuntimeValueRuntimeType_iff (left right : Uorc.Uorc.Core.RuntimeValue.RuntimeType) : (waitingEqCoreRuntimeValueRuntimeType left right = true) ↔ left = right := by
  cases left <;> cases right <;> constructor <;> intro h <;> first | rfl | cases h
#print axioms waitingEqCoreRuntimeValueRuntimeType_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeValueRuntimeType
public theorem waitingEqCoreRuntimeOpcodeOpcode_iff (left right : Uorc.Uorc.Core.RuntimeOpcode.Opcode) : (waitingEqCoreRuntimeOpcodeOpcode left right = true) ↔ left = right := by
  cases left <;> cases right <;> constructor <;> intro h <;> first | rfl | cases h
#print axioms waitingEqCoreRuntimeOpcodeOpcode_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeOpcodeOpcode
public theorem waitingEqCoreRuntimeProgramCaptureLane_iff (left right : Uorc.Uorc.Core.RuntimeProgram.CaptureLane) : (waitingEqCoreRuntimeProgramCaptureLane left right = true) ↔ left = right := by
  cases left <;> cases right <;> constructor <;> intro h <;> first | rfl | cases h
#print axioms waitingEqCoreRuntimeProgramCaptureLane_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeProgramCaptureLane
public theorem waitingEqCoreRuntimeProgramEvent_iff (left right : Uorc.Uorc.Core.RuntimeProgram.Event) : (waitingEqCoreRuntimeProgramEvent left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeProgramEvent, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeValueRuntimeType_iff, waitingEqCoreRuntimeOpcodeOpcode_iff, waitingEqCoreRuntimeProgramCaptureLane_iff, waitingEqCoreRuntimeValueHandle_iff]
#print axioms waitingEqCoreRuntimeProgramEvent_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeProgramEvent
public theorem waitingEqOptionCoreRuntimeProgramEvent_iff (left right : Option (Uorc.Uorc.Core.RuntimeProgram.Event)) : (waitingEqOptionCoreRuntimeProgramEvent left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqOptionCoreRuntimeProgramEvent, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeProgramEvent_iff]
#print axioms waitingEqOptionCoreRuntimeProgramEvent_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqOptionCoreRuntimeProgramEvent
public theorem waitingEqSearchTemplateStreamState_iff (left right : Uorc.Uorc.Search.TemplateStream.State) : (waitingEqSearchTemplateStreamState left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchTemplateStreamState, LexLeanRuntime.equal, and_assoc, waitingEqSearchFixedTemplatesCursor_iff, waitingEqSearchTemplateStreamPhase_iff, waitingEqCoreRuntimeValueHandle_iff, waitingEqOptionCoreRuntimeSequenceCursor_iff, waitingEqOptionCoreRuntimeProgramEvent_iff]
#print axioms waitingEqSearchTemplateStreamState_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchTemplateStreamState
public theorem waitingEqCoreHeapImageKind_iff (left right : Uorc.Uorc.Core.Heap.ImageKind) : (waitingEqCoreHeapImageKind left right = true) ↔ left = right := by
  cases left <;> cases right <;> constructor <;> intro h <;> first | rfl | cases h
#print axioms waitingEqCoreHeapImageKind_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreHeapImageKind
public theorem waitingEqCoreRuntimeValueRootRole_iff (left right : Uorc.Uorc.Core.RuntimeValue.RootRole) : (waitingEqCoreRuntimeValueRootRole left right = true) ↔ left = right := by
  cases left <;> cases right <;> constructor <;> intro h <;> first | rfl | cases h
#print axioms waitingEqCoreRuntimeValueRootRole_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeValueRootRole
public theorem waitingEqCoreRuntimeValueRuntimeValue_iff (left right : Uorc.Uorc.Core.RuntimeValue.RuntimeValue) : (waitingEqCoreRuntimeValueRuntimeValue left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeValueRuntimeValue, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeValueHandle_iff]
#print axioms waitingEqCoreRuntimeValueRuntimeValue_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeValueRuntimeValue
public theorem waitingEqCoreHeapNode_iff (left right : Uorc.Uorc.Core.Heap.Node) : (waitingEqCoreHeapNode left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [byteArray_beq_eq, waitingEqCoreHeapNode, LexLeanRuntime.equal, and_assoc, waitingEqCoreHeapImageKind_iff, waitingEqCoreRuntimeValueHandle_iff, waitingEqCoreRuntimeValueRootRole_iff, waitingEqCoreRuntimeValueRuntimeValue_iff]
#print axioms waitingEqCoreHeapNode_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreHeapNode
public theorem waitingEqCoreHeapEntry_iff (left right : Uorc.Uorc.Core.Heap.Entry) : (waitingEqCoreHeapEntry left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreHeapEntry, LexLeanRuntime.equal, and_assoc, waitingEqCoreHeapNode_iff]
#print axioms waitingEqCoreHeapEntry_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreHeapEntry
public theorem waitingEqCoreHeapIndex_iff (left right : Uorc.Uorc.Core.Heap.Index) : (waitingEqCoreHeapIndex left right = true) ↔ left = right := by
  induction left generalizing right <;> cases right <;> simp_all [waitingEqCoreHeapIndex, LexLeanRuntime.equal, and_assoc, waitingEqCoreHeapEntry_iff]
#print axioms waitingEqCoreHeapIndex_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreHeapIndex
public theorem waitingEqListCoreRuntimeValueHandle_iff (left right : List (Uorc.Uorc.Core.RuntimeValue.Handle)) : (waitingEqListCoreRuntimeValueHandle left right = true) ↔ left = right := by
  induction left generalizing right <;> cases right <;> simp_all [waitingEqListCoreRuntimeValueHandle, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeValueHandle_iff]
#print axioms waitingEqListCoreRuntimeValueHandle_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqListCoreRuntimeValueHandle
public theorem waitingEqCoreHeapReleaseCursor_iff (left right : Uorc.Uorc.Core.Heap.ReleaseCursor) : (waitingEqCoreHeapReleaseCursor left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreHeapReleaseCursor, LexLeanRuntime.equal, and_assoc, waitingEqListCoreRuntimeValueHandle_iff]
#print axioms waitingEqCoreHeapReleaseCursor_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreHeapReleaseCursor
public theorem waitingEqCoreHeapArena_iff (left right : Uorc.Uorc.Core.Heap.Arena) : (waitingEqCoreHeapArena left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreHeapArena, LexLeanRuntime.equal, and_assoc, waitingEqCoreHeapIndex_iff, waitingEqCoreHeapReleaseCursor_iff]
#print axioms waitingEqCoreHeapArena_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreHeapArena

-- These global statements quantify over all values, without validity or reachability assumptions.
public theorem fullStateEqual_iff (a b : Uorc.Uorc.Search.TemplateStream.State) :
    (waitingEqSearchTemplateStreamState a b = true) ↔ a = b :=
  waitingEqSearchTemplateStreamState_iff a b
#print axioms fullStateEqual_iff

public theorem fullArenaEqual_iff (a b : Uorc.Uorc.Core.Heap.Arena) :
    (waitingEqCoreHeapArena a b = true) ↔ a = b :=
  waitingEqCoreHeapArena_iff a b
#print axioms fullArenaEqual_iff

public theorem waitingEqCoreHeapHeapError_iff (left right : Uorc.Uorc.Core.Heap.HeapError) : (waitingEqCoreHeapHeapError left right = true) ↔ left = right := by
  cases left <;> cases right <;> constructor <;> intro h <;> first | rfl | cases h
#print axioms waitingEqCoreHeapHeapError_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreHeapHeapError

public theorem waitingEqCoreRuntimeResourcesError_iff (left right : Uorc.Uorc.Core.RuntimeResources.Error) : (waitingEqCoreRuntimeResourcesError left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeResourcesError, LexLeanRuntime.equal, and_assoc, waitingEqCoreHeapHeapError_iff]
#print axioms waitingEqCoreRuntimeResourcesError_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeResourcesError

public theorem waitingEqCoreRuntimeSequenceSequenceFailure_iff (left right : Uorc.Uorc.Core.RuntimeSequence.SequenceFailure) : (waitingEqCoreRuntimeSequenceSequenceFailure left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqCoreRuntimeSequenceSequenceFailure, LexLeanRuntime.equal, and_assoc, waitingEqCoreHeapHeapError_iff]
#print axioms waitingEqCoreRuntimeSequenceSequenceFailure_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqCoreRuntimeSequenceSequenceFailure

public theorem waitingEqSearchTemplateStreamError_iff (left right : Uorc.Uorc.Search.TemplateStream.Error) : (waitingEqSearchTemplateStreamError left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchTemplateStreamError, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeSequenceSequenceFailure_iff, waitingEqCoreHeapHeapError_iff]
#print axioms waitingEqSearchTemplateStreamError_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchTemplateStreamError

public theorem waitingEqSearchTemplateFundingError_iff (left right : Uorc.Uorc.Search.TemplateFunding.Error) : (waitingEqSearchTemplateFundingError left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchTemplateFundingError, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeResourcesError_iff, waitingEqSearchTemplateStreamError_iff]
#print axioms waitingEqSearchTemplateFundingError_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchTemplateFundingError

public theorem waitingEqSearchTemplateFundingOutcome_iff (left right : Uorc.Uorc.Search.TemplateFunding.Outcome) : (waitingEqSearchTemplateFundingOutcome left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchTemplateFundingOutcome, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeProgramEvent_iff, waitingEqSearchTemplateFundingError_iff]
#print axioms waitingEqSearchTemplateFundingOutcome_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchTemplateFundingOutcome

public theorem waitingEqSearchTemplateOwnerError_iff (left right : Uorc.Uorc.Search.TemplateOwner.Error) : (waitingEqSearchTemplateOwnerError left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchTemplateOwnerError, LexLeanRuntime.equal, and_assoc, waitingEqCoreRuntimeResourcesError_iff, waitingEqCoreHeapHeapError_iff]
#print axioms waitingEqSearchTemplateOwnerError_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchTemplateOwnerError

public theorem waitingEqSearchTemplateOwnerOutcome_iff (left right : Uorc.Uorc.Search.TemplateOwner.Outcome) : (waitingEqSearchTemplateOwnerOutcome left right = true) ↔ left = right := by
  cases left <;> cases right <;> simp [waitingEqSearchTemplateOwnerOutcome, LexLeanRuntime.equal, and_assoc, waitingEqSearchTemplateOwnerError_iff]
#print axioms waitingEqSearchTemplateOwnerOutcome_iff
#print axioms Uorc.Uorc.Search.WaitingEquality.waitingEqSearchTemplateOwnerOutcome

public theorem fullFundingOutcomeEqual_iff (a b : Uorc.Uorc.Search.TemplateFunding.Outcome) : (waitingEqSearchTemplateFundingOutcome a b = true) ↔ a = b :=
  waitingEqSearchTemplateFundingOutcome_iff a b
#print axioms fullFundingOutcomeEqual_iff

public theorem fullOwnerOutcomeEqual_iff (a b : Uorc.Uorc.Search.TemplateOwner.Outcome) : (waitingEqSearchTemplateOwnerOutcome a b = true) ↔ a = b :=
  waitingEqSearchTemplateOwnerOutcome_iff a b
#print axioms fullOwnerOutcomeEqual_iff
