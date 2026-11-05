// Evidence importer only. All hashing, progress and lifetime laws are Source-owned.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {historicalCanonicalSource} from './integrity-current-context.mjs';

export const proofSources = [
  'RetainedSourceLaws', 'ProfilePartitionLaws', 'ImagePublicationLaws',
  'CanonicalRefinement', 'HashControlCleanupLaws', 'ProfileFuelLaws',
  'OwnedControlLifetime',
].map(name => `src/Uorc/Integrity/${name}.lex.tex`);
export const mutationNames = [
  'drop-node-equality', 'profile-consuming-offset', 'commit-empty-payload',
  'owned-purpose-prefix', 'cleanup-empty-index', 'canonical-zero-final-fuel',
  'control-reservation-kind',
];
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const required = (ok, why) => { if (!ok) throw new Error(why); };
function checkedFile(root, file) {
  required(typeof file === 'string' && !path.isAbsolute(file) && !file.split('/').includes('..'), 'file outside project');
  return fs.readFileSync(path.join(root, file));
}
export function validateCanonicalEvidence(root) {
  const receipt = JSON.parse(checkedFile(root, 'validation/integrity/canonical/evidence.json'));
  required(receipt.schema === 'uorc/canonical-integrity-refinement-evidence/1', 'canonical evidence schema');
  for (const file of proofSources) required(Object.hasOwn(receipt.source_sha256, file), `missing proof source ${file}`);
  for (const [file, expected] of Object.entries(receipt.source_sha256)) required(sha(historicalCanonicalSource(root, file, expected)) === expected, `stale source ${file}`);
  for (const [file, expected] of Object.entries(receipt.evidence_sha256)) required(sha(checkedFile(root, file)) === expected, `changed evidence ${file}`);
  required(JSON.stringify(receipt.proofs.map(row => row.file)) === JSON.stringify(proofSources), 'complete ordered proof inventory');
  for (const proof of receipt.proofs) {
    const source = historicalCanonicalSource(root, proof.file, receipt.source_sha256[proof.file]).toString();
    const names = [...source.matchAll(/^(?:definition|theorem|opaque|abbrev)\s+(.+?)\s*:\s*@\d+/gm)].map(match => match[1].replace(/\.\{[^}]*\}$/, ''));
    required(names.length === proof.rows && new Set(names).size === names.length && names.length > 0, `complete source declarations ${proof.file}`);
    required(proof.source_build_passed === true && proof.kernel_exit === 0 && proof.leanchecker_exit === 0, `proof replay ${proof.file}`);
    required(proof.audit.length === names.length && new Set(proof.audit.map(row => row.name)).size === names.length, `complete audit ${proof.file}`);
    for (const name of names) {
      const row = proof.audit.find(row => row.name === name);
      required(row && row.policy.kind === 'exact', `missing exact policy ${name}`);
      const observed = [...row.observed].sort(), declared = [...row.policy.axioms].sort();
      required(JSON.stringify(observed) === JSON.stringify(declared), `axiom policy mismatch ${name}`);
      required(observed.every(axiom => ['Quot.sound', 'propext'].includes(axiom)), `unapproved axiom ${name}`);
    }
  }
  required(receipt.proofs.reduce((total, row) => total + row.rows, 0) === 534, 'complete 534-declaration scope');
  required(JSON.stringify(receipt.mutations.map(row => row.name)) === JSON.stringify(mutationNames), 'complete mutation inventory');
  for (const mutation of receipt.mutations) {
    required(mutation.mutant_exit === 1 && mutation.restored_exit === 0, `semantic rejection/restoration ${mutation.name}`);
    required(checkedFile(root, mutation.diagnostic).toString().includes('(kernel)'), `actual kernel diagnostic ${mutation.name}`);
  }
  required(receipt.runtime.exit_code === 0 && receipt.runtime.profile_comparisons === 258 && receipt.runtime.stream_cases === 44, 'actual complete canonical native controls');
  required(receipt.runtime.first_payload_boundary === 73 && receipt.runtime.continuation_payload_capacity === 41, 'actual reduced frame boundaries');
  required(receipt.remaining.includes('original source-layout correspondence') && receipt.remaining.includes('original pinned-SDK and whole-product acceptance'), 'scope boundary');
  required(receipt.qualification === 'Source/kernel proofs and finite generated-model controls; not original pinned-SDK or whole-product acceptance.', 'qualification boundary');
  return receipt;
}
