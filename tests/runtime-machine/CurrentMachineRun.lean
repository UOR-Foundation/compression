import Uorc.Uorc.Core.RuntimeMachineEngineTests
import Uorc.Uorc.Core.RuntimeMachineSemanticPrefixTests

-- Both complete source-owned aggregates run against one imported Engine closure.
def main : IO Unit := do
  match Uorc.Uorc.Core.RuntimeMachineEngineTests.run with
  | .ok _ => IO.println "RuntimeMachineEngineTests.run93: pass"
  | .error _ => throw (IO.userError "RuntimeMachineEngineTests.run93: failed")
  match Uorc.Uorc.Core.RuntimeMachineSemanticPrefixTests.run 300000 with
  | .ok _ => IO.println "RuntimeMachineSemanticPrefixTests.run9: pass"
  | .error _ => throw (IO.userError "RuntimeMachineSemanticPrefixTests.run9: failed")
  IO.println "CurrentMachineRun102: pass"
