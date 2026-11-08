import Uorc.Uorc.Core.RuntimeMachineEngineTests
import Uorc.Uorc.Core.RuntimeMachineSemanticPrefixTests
import Uorc.Uorc.Core.RuntimeMachineCompositionTests

-- Source-owned, unfiltered aggregates: 93 + 9 + 10 real control bodies.
-- Execution remains pending the unchanged Core proof dependency.
def main : IO Unit := do
  match Uorc.Uorc.Core.RuntimeMachineEngineTests.run with
  | .ok _ => IO.println "RuntimeMachineEngineTests.run93: pass"
  | .error _ => throw (IO.userError "RuntimeMachineEngineTests.run93: failed")
  match Uorc.Uorc.Core.RuntimeMachineSemanticPrefixTests.run 300000 with
  | .ok _ => IO.println "RuntimeMachineSemanticPrefixTests.run9: pass"
  | .error _ => throw (IO.userError "RuntimeMachineSemanticPrefixTests.run9: failed")
  match Uorc.Uorc.Core.RuntimeMachineCompositionTests.run 300000 with
  | .ok _ => IO.println "RuntimeMachineCompositionTests.run10: pass"
  | .error _ => throw (IO.userError "RuntimeMachineCompositionTests.run10: failed")
  IO.println "FutureCompositionRun112: pass"
