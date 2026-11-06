export const mutationTargets = [
  { id: 'attachment-identity-bypass', definition: 'attachmentReply', control: 'wrongAttachmentTicketControl' },
  { id: 'premature-image-unregister', definition: 'imageUnregistered', control: 'unregisterBeforeReleaseControl' },
  { id: 'cancel-drops-reader-request', definition: 'abortCursor', control: 'pendingCancellationControl' }
];
export function mutateReadController(module, id) {
  const result = structuredClone(module);
  const target = mutationTargets.find(x => x.id === id);
  if (!target) throw new Error(`Unknown mutation ${id}`);
  const declaration = result.declarations.find(x => x.name === target.definition);
  if (!declaration) throw new Error(`Missing definition ${target.definition}`);
  let changed = 0;
  function walk(value) {
    if (!value || typeof value !== 'object') return;
    if (id === 'attachment-identity-bypass' && value.kind === 'call' && value.function?.name === 'sameAttachment') {
      for (const key of Object.keys(value)) delete value[key];
      Object.assign(value, { kind: 'bool', value: true }); changed++; return;
    }
    if (id === 'premature-image-unregister' && value.kind === 'if' && value.condition?.kind === 'project' && value.condition.field === 'unregistered') {
      value.condition = { kind: 'bool', value: true }; changed++;
    }
    for (const child of Object.values(value)) {
      if (Array.isArray(child)) child.forEach(walk); else walk(child);
    }
  }
  if (id === 'cancel-drops-reader-request') {
    if (declaration.body.kind !== 'record') throw new Error('Unexpected abortCursor body');
    const field = declaration.body.fields.find(x => x.field === 'reader');
    const reader = field.value;
    field.value = { kind: 'record', type: { module: 'Uorc.Product.ResourceIO', name: 'Reader' }, fields:
      ['run','source','quantum','offset','ticket','phase','request','held','eof','error'].map(name => ({ field: name, value:
        name === 'request' ? { kind: 'constructor', constructor: { name: 'Option.none' }, arguments: [], type_arguments: [{kind:'named',member:{module:'Uorc.Product.ResourceIO',name:'ReadRequest'},arguments:[]}] }
        : { kind: 'project', value: structuredClone(reader), field: name }
      })) };
    changed++;
  } else walk(declaration.body);
  if (changed !== 1) throw new Error(`Expected exactly one mutation, observed ${changed}`);
  return result;
}
export function readControllerMutationSpecimens() {
  return mutationTargets.map(target => [target.id, 'ReadController', target.control, declarations => {
    const mutated = mutateReadController({declarations}, target.id);
    declarations.splice(0, declarations.length, ...mutated.declarations);
  }]);
}
