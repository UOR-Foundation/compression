import Uorc.Uorc.Core.RuntimeMachineSemanticPrefixTests

open Uorc.Uorc.Core.RuntimeMachineSemanticPrefixTests

-- All registered roots execute through the source-owned aggregate.
def main : IO Unit :=
  match run 300000 with
  | .ok _ => IO.println "RuntimeMachineSemanticPrefixTests.run300000: pass"
  | .error _ => throw (IO.userError "RuntimeMachineSemanticPrefixTests.run300000: failed")
