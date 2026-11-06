import Uorc.Uorc.Core.RuntimeMachineEngineTests
set_option maxRecDepth 1000000
set_option maxHeartbeats 1000000000

def main : IO Unit :=
  match Uorc.Uorc.Core.RuntimeMachineEngineTests.run with
  | .ok _ => IO.println "RuntimeMachineEngineTests.run: pass"
  | .error _ => throw (IO.userError "RuntimeMachineEngineTests.run failed")
