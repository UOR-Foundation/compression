export const mutationTargets = [
  { id: 'target-length-observation-free', definition: 'tickWork', control: 'lengthObservationPrepaymentControl' },
  { id: 'target-ticket-not-cleared', definition: 'afterTake', control: 'oneTimeSnapshotTransferControl' },
  { id: 'target-length-link-points-to-bytes', definition: 'paidStep', control: 'orderedLengthsSurviveCleanupControl' }
];
export function mutateTargetDescriptor(module, id) {
  const result = structuredClone(module);
  const target = mutationTargets.find(x => x.id === id);
  if (!target) throw new Error(`Unknown mutation ${id}`);
  const d = result.declarations.find(x => x.name === target.definition);
  if (!d) throw new Error(`Missing definition ${target.definition}`);
  let changed = 0;
  function walk(x, visit) {
    if (!x || typeof x !== 'object') return;
    visit(x);
    for (const v of Object.values(x)) {if (Array.isArray(v)) v.forEach(y => walk(y,visit)); else walk(v,visit);}
  }
  if (id === 'target-length-observation-free') {
    const branch = d.body.branches.find(x => x.constructor.name === 'Phase.lengths');
    if (branch.body.kind !== 'call' || branch.body.function.name !== 'fee') throw new Error('Unexpected work quote');
    branch.body = {kind:'integer',representation:'uint64',value:'0'};changed++;
  } else if (id === 'target-ticket-not-cleared') {
    walk(d.body,x => {
      if (x.kind !== 'record' || x.type?.name !== 'Cursor') return;
      const record = x.fields.find(f => f.field === 'record');
      const run = x.fields.find(f => f.field === 'run');
      if (record?.value?.kind === 'call' && record.value.function.name === 'nullHandle') {
        record.value = {kind:'project',value:structuredClone(run.value.value),field:'record'};changed++;
      }
    });
  } else {
    let sequence;
    walk(d.body,x => {
      if (x.kind === 'constructor' && x.constructor.module === 'Uorc.Core.Heap' && x.constructor.name === 'Node.metadataLink' && x.arguments[1]?.value === '3') sequence = structuredClone(x.arguments[2]);
    });
    if (!sequence) throw new Error('Missing source sequence link');
    walk(d.body,x => {
      if (x.kind === 'constructor' && x.constructor.module === 'Uorc.Core.Heap' && x.constructor.name === 'Node.metadataLink' && x.arguments[1]?.value === '4') {x.arguments[2] = structuredClone(sequence);changed++;}
    });
  }
  if (changed !== 1) throw new Error(`Expected exactly one change; observed ${changed}`);
  return result;
}
