// The executed control modules of the shipped search and step machine, and the Lean driver that runs one.
const requireFact=(b,m)=>{if(!b)throw new Error(m);};
export const controlModules=['Core/Tests','Search/ArchiveControls','Search/HuffmanControls','Search/ReplayControls','Search/ExecutionAccountingControls','Search/CertificationControls'];
const leanModule=suite=>`Uorc.Uorc.${suite.split('/').join('.')}`;
export function driver(only,suite){
 requireFact(controlModules.includes(suite),'unknown executed control module');
 if(only)return `import ${leanModule(suite)}\ndef main : IO Unit := do\n if !${leanModule(suite)}.${only} then throw (IO.userError "FAIL ${only}")\n IO.println "PASS ${only}"\n`;
 return `import ${leanModule(suite)}\ndef main : IO Unit := do\n let mut passed := 0\n let mut total := 0\n let mut names : List String := []\n for observation in ${leanModule(suite)}.observations do\n  if names.contains observation.id then throw (IO.userError "duplicate observation")\n  names := observation.id :: names\n  total := total + 1\n  if observation.passed then passed := passed + 1 else IO.println s!"FAIL {observation.id}"\n IO.println s!"{passed}/{total} executed controls passed"\n if total == 0 || passed != total then throw (IO.userError "binding controls failed")\n`;
}
