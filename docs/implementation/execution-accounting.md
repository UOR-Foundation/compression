# Source execution-credit repair and diagnostic boundary

The authoritative behavior remains the semantic modules. This document records source diagnostics, not SDK/Lean acceptance, physical RuntimeProgram.Event ownership qualification, or a native benchmark. The general StepMachine bound proof remains open; its finite charge-callgraph derivation and tested examples are in the core worktree's runtime-admission implementation record.

## Three distinct measures

Resources.Usage.steps is canonical decoder-profile work. ExecutionCredit decoderUnits is actual canonical work incurred after its paid baseline. Search scheduling units also include prepaid unused escrow, paid bound planning and attempt dispatch. None is substituted for another.

For valid cursors:

- futureUsage, committedUsage and paidSteps describe the same already-computed canonical prefix. remainingDebt is zero.
- decoderUnits + availableCredit = prepaidUnits.
- consumedUnits = attemptUnits + prepaidUnits + planningUnits.
- remainingEventWork records the still-unpaid declared event bound. Cancellation retains it and unused escrow separately from canonical Usage.

The invalid/underfunded rejection case retains actual reserved Usage, actual decoder work, the last committed prefix and explicit remainingDebt. A source-bound defect is not turned into eligibility.

## Before computation

ProgramTrial, ArchiveTrial, GeneralFit and ModelFit now call startInitialization, which stores an initialization request and no initialized Machine. The request's declared work is paid before initializeAdmitted is invoked. The final payment is bound by a source let before the call. A canonical initialization budget failure stays a typed operational failure, with its actual Usage prefix and all unused scheduling escrow retained.

For an existing Machine, ExecutionCredit first pays StepMachine.planningWork before invoking plannedWorkBound. That probe is source-owned and does not dry-run step. It then funds/carries availableCredit to at least the declared bound before calling StepMachine.step. Actual canonical work is deducted afterward; unused escrow remains available and is reported. The bound is a scheduling reservation, never a replacement for the canonical decoder budget.

The old startReserved/fromInitialization adapters cannot turn an unpaid precomputed future into payable progress. Unpaid legacy cursors and direct terminal observation are rejected. Cancellation still reports their exact reserved prefix/debt for diagnosis.

## Evidence retention

ProgramTrial keeps its existing Step.rejected(error,events) shape. Finite executionAccounted, creditAccounted, outputTypeAccounted, mismatchAccounted and verificationRopeAccounted error variants retain the exact ExecutionCredit.Ledger. rejectionLedger retrieves it; preexecution errors return None.

ArchiveTrial.Ledger.executionCredit is None before execution and Some with the latest full ledger during initialization/execution and all later phases, success, failure and cancellation. Replay transports that field without changing certificate/coverage meaning. GeneralFit and ModelFit use delayed initialization and retain exact ledgers for direct execution/credit/output-type rejections.

## Frozen legacy reconciliation

The former one-byte Huffman trace was captured before repair, against exact source hashes. It was not normalized or declared accepted:

- Validation308797 + empty basis1 = paid baseline308798.
- Initialization257 + subsequent canonical step deltas217334 = decoderUnits217591.
- Usage526389 = baseline308798 + decoderUnits217591.
- Old consumed218639 = attempt1048 + decoder217591.
- ProgramTrial540056 ticks = emission1861 + build3372 + parse7374 + validation308797 + admission1 + execution218639 + comparison12.

That former scheduler computed a future before paying its delta. The audit is negative evidence for prepayment, even though its terminal tick withheld observation until debt was repaid. Its Python AST evaluation count and scratch wall time are not native timings. No larger Huffman diagnostic was run after the repair.

## Actual small same-source trace

ExecutionAccountingControls owns a sequence-returning graph whose first three instructions are naturalConstant1, naturalConstant0 and naturalDivide(2,1). Validation succeeds; execution reaches divisionByZero.

The repaired call-entry trace invokes initialization only with257 prepaid units and each of its three actual step calls only with641 units available. Actual canonical deltas are257,129,129,321. Failure Usage6149 and decoder836 remain unchanged. Scheduling consumed1162 = attempt3 + prepaid1156 + planning3; unused escrow320 is retained rather than relabeled decoder work. The public ProgramTrial count changes from6421 to6744 because scheduling now prepays actual work and its bound probes.

At256/257 initialization payments there is no Machine, no allocated initialization cell, one unit of declared work remains, and cancellation retains256 escrow. At full257 payment the Machine is materialized. A step likewise remains unexecuted with one escrow unit missing.

A canonical-budget-negative one-byte literal preserves Usage1691, decoder0 and unused escrow257 on OperationalError.stepLimit; it is not changed into a search-credit error.

Small GeneralFit and ModelFit source runs reach proposals. An actual ArchiveTrial cancellation one initialization unit short retains the exact EC ledger. Existing Archive/Replay closed checks pass21 of22 in the generic diagnostic evaluator; the remaining targeted-real-program check reaches its unchanged10-million-AST-term instrumentation guard. That is not a semantic failure or a passed check.

## Gates and limits

Focused structural diagnostics report96 changed/related declaration bodies with zero shape/type errors. Fifteen focused closed source controls pass in the diagnostic evaluator. Actual-source planted mutations are rejected for premature initialization, premature step execution, discarded cancellation escrow, premature terminal observation, discarded ProgramTrial rejection ledger and discarded ArchiveTrial ledger transport. Sources are restored byte-for-byte after every mutation and controls are rerun.

These checks do not replace the pinned SDK's full acceptance command. Owned-runtime integration must keep every physical root accounted for, pay declared event work before any physical implementation runs, retain exact failure/cancellation prefixes and escrow, and require fresh comparison plus no unpaid event before eligibility. A precomputed future held behind a token gate is not sufficient.
