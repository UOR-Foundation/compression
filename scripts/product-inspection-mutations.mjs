// Apply to the official SDK read_module result, then write via official product-authoring.
export const mutationTargets = [
  { id: 'inspection-as-execution', definition: 'buildReport', control: 'executionPromotionControl' },
  { id: 'stale-inspection-completion', definition: 'complete', control: 'staleCompletionControl' },
  { id: 'stale-inspection-take', definition: 'take', control: 'staleTakeControl' }
];
export function mutateInspectionReport(module, id) {
  const result = structuredClone(module);
  const target = mutationTargets.find(x => x.id === id);
  if (!target) throw new Error(`Unknown inspection mutation: ${id}`);
  const declaration = result.declarations.find(x => x.name === target.definition);
  if (!declaration) throw new Error(`Missing definition ${target.definition}`);
  if (id === 'inspection-as-execution') {
    const facts = declaration.body.fields.find(x => x.field === 'facts').value;
    const executed = facts.fields.find(x => x.field === 'executed');
    if (executed.value.kind !== 'bool' || executed.value.value !== false) throw new Error('Unexpected executed field');
    executed.value.value = true;
  } else {
    const body = declaration.body;
    if (body.kind !== 'if' || body.condition.kind !== 'call' || body.condition.function.module !== 'Uorc.Product.Control' || body.condition.function.name !== 'owns') throw new Error('Unexpected current-run guard');
    body.condition = { kind: 'bool', value: true };
  }
  return result;
}
export function inspectionMutationSpecimens() {
  return mutationTargets.map(target => [target.id, 'InspectionReport', target.control, declarations => {
    const mutated = mutateInspectionReport({declarations}, target.id);
    declarations.splice(0, declarations.length, ...mutated.declarations);
  }]);
}
