// Intended actual source mutations; UNEXECUTED while the upstream Core proof blocks VM compilation.
export const mutationTargets = [
  { id: 'reset-accepted-semantic-ledger', definition: 'start', control: 'acceptedLedgerStartControl' },
  { id: 'bypass-terminal-receipt-barrier', definition: 'detachTerminal', control: 'pendingTerminalReceiptCannotDetach' },
  { id: 'duplicate-terminal-receipt', definition: 'takeTerminalReceipt', control: 'onceOnlyTerminalReceiptTransfer' },
  { id: 'drop-terminal-semantic-refusal', definition: 'finishDrain', control: 'terminalSemanticRefusalStatusControl' }
];
export function mutateMachineSemanticMigration(module, id) {
  const result = structuredClone(module);
  const target = mutationTargets.find(x => x.id === id);
  if (!target) throw new Error(`Unknown mutation ${id}`);
  const declaration = result.declarations.find(x => x.name === target.definition);
  if (!declaration) throw new Error(`Missing definition ${target.definition}`);
  let changed = 0;
  function walk(node) {
    if (!node || typeof node !== 'object') return;
    if (id === 'reset-accepted-semantic-ledger' && node.kind === 'call' && node.function?.module === 'Uorc.Core.RuntimeMachineEngine' && node.function.name === 'startWithSemantic') {
      if (node.arguments.length !== 4) throw new Error('Unexpected startWithSemantic shape');
      node.arguments[3] = {kind:'call',function:{module:'Uorc.Core.RuntimeMachineSemantic',name:'zeroLedger'},arguments:[]};changed++;
    }
    if (id === 'bypass-terminal-receipt-barrier' && node.kind === 'match' && node.scrutinee?.kind === 'project' && node.scrutinee.field === 'receipt') {
      const some = node.branches.find(b => b.constructor.name === 'Option.some');
      const none = node.branches.find(b => b.constructor.name === 'Option.none');
      if (some?.body?.constructor?.name === 'Detach.awaitReceipt' && none) {some.body = structuredClone(none.body);changed++;}
    }
    if (id === 'duplicate-terminal-receipt' && node.kind === 'record' && node.type?.name === 'Terminal') {
      const receipt = node.fields.find(f => f.field === 'receipt');
      const run = node.fields.find(f => f.field === 'run');
      if (receipt?.value?.constructor?.name === 'Option.none' && run?.value?.kind === 'project') {
        receipt.value = {kind:'project',value:structuredClone(run.value.value),field:'receipt'};changed++;
      }
    }
    if (id === 'drop-terminal-semantic-refusal' && node.kind === 'record' && node.type?.name === 'State') {
      const owner = node.fields.find(f => f.field === 'owner');
      const refusal = node.fields.find(f => f.field === 'semanticRefusal');
      if (owner?.value?.constructor?.name === 'Owner.terminal' && refusal) {
        refusal.value = {kind:'constructor',constructor:{name:'Option.none'},arguments:[],type_arguments:[{kind:'named',member:{module:'Uorc.Core.RuntimeMachineSemantic',name:'Error'},arguments:[]}]};changed++;
      }
    }
    for (const value of Object.values(node)) {if (Array.isArray(value)) value.forEach(walk); else walk(value);}
  }
  walk(declaration.body);
  if (changed !== 1) throw new Error(`Expected exactly one mutation; observed ${changed}`);
  return result;
}
