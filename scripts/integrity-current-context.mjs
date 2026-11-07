// Evidence importer. Historical proof receipts retain their original source scope.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {sourceBound} from './language-migration.mjs';

const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const requireEvidence = (condition, why) => { if (!condition) throw new Error(why); };
const historicalReceiptSha = '88d16929238b65fb4f4ca5cf5fc52ed14913c916bf8ae94bbd0b3983abcf6e4b';
const historicalTranscriptSha = '5e5676c2a49f1c9cd5d755e9ee6bd26a125c4a2b28c3a0bff31703eda7c61d8a';
export const currentCanonicalProofNames = [
  'RetainedSourceLaws', 'ProfilePartitionLaws', 'ImagePublicationLaws',
  'CanonicalRefinement', 'HashControlCleanupLaws', 'ProfileFuelLaws', 'OwnedControlLifetime',
];
export function evidenceFile(root, file) {
  requireEvidence(typeof file === 'string' && file.length > 0 && !path.isAbsolute(file) && !file.split('/').includes('..'), 'evidence path outside project');
  return fs.readFileSync(path.join(root, file));
}
export function validateProofReplay(root, proof) {
  const source = evidenceFile(root, proof.file).toString();
  const semantic = [...source.matchAll(/\\semanticdata\{(.*)\}\n/g)];
  const semanticNotation = [...source.matchAll(/^(?:structure|class|instance|inductive|definition|theorem)\{\n  name: ([^,\n]+),/gm)];
  const names = semantic.length === 1
    ? JSON.parse(semantic[0][1]).declarations.map(row => `Uorc.Uorc.Integrity.${path.basename(proof.file, '.lex.tex')}.${row.name}`)
    : semanticNotation.length > 0 ? semanticNotation.map(match => `Uorc.Uorc.Integrity.${path.basename(proof.file, '.lex.tex')}.${match[1]}`)
    : [...source.matchAll(/^(?:definition|theorem|opaque|abbrev|inductive|constructor|recursor)\s+(.+?)\s*:\s*@\d+/gm)].map(match => match[1].replace(/\.\{[^}]*\}$/, ''));
  requireEvidence(names.length > 0 && new Set(names).size === names.length && names.length === proof.rows, `complete current declarations ${proof.file}`);
  requireEvidence(proof.kernel_exit === 0 && proof.leanchecker_exit === 0 && proof.source_build_passed === true, `actual current replay ${proof.file}`);
  requireEvidence(proof.audit.length === names.length && new Set(proof.audit.map(row => row.name)).size === names.length, `complete current axiom audit ${proof.file}`);
  for (const name of names) {
    const row = proof.audit.find(row => row.name === name);
    requireEvidence(row && row.policy.kind === 'exact', `current exact policy ${name}`);
    const observed = [...row.observed].sort(), declared = [...row.policy.axioms].sort();
    requireEvidence(JSON.stringify(observed) === JSON.stringify(declared), `current axiom mismatch ${name}`);
    requireEvidence(observed.every(axiom => ['Quot.sound', 'propext'].includes(axiom)), `unapproved current axiom ${name}`);
  }
}
export function validateCurrentCanonicalReplay(root) {
  const receipt = JSON.parse(evidenceFile(root, 'validation/integrity/canonical-current/evidence.json'));
  requireEvidence(receipt.schema === 'uorc/canonical-current-source-replay/1', 'current canonical receipt schema');
  requireEvidence(sha(evidenceFile(root, 'validation/integrity/canonical/evidence.json')) === historicalReceiptSha && receipt.historical_receipt_sha256 === historicalReceiptSha, 'historical receipt must remain unchanged');
  requireEvidence(sha(evidenceFile(root, 'validation/integrity/transcript/evidence.json')) === historicalTranscriptSha && receipt.historical_transcript_receipt_sha256 === historicalTranscriptSha, 'historical transcript receipt must remain unchanged');
  for (const [file, expected] of Object.entries(receipt.source_sha256)) requireEvidence(sourceBound(root, file, expected), `stale current source ${file}`);
  for (const [file, expected] of Object.entries(receipt.evidence_sha256)) requireEvidence(sha(evidenceFile(root, file)) === expected, `changed current evidence ${file}`);
  const expectedProofs = currentCanonicalProofNames.map(name => `src/Uorc/Integrity/${name}.lex.tex`);
  requireEvidence(JSON.stringify(receipt.proofs.map(proof => proof.file)) === JSON.stringify(expectedProofs), 'complete current canonical proof inventory');
  for (const proof of receipt.proofs) {
    requireEvidence(Object.hasOwn(receipt.source_sha256, proof.file), `unbound current proof ${proof.file}`);
    validateProofReplay(root, proof);
  }
  const declarations = receipt.proofs.reduce((total, proof) => total + proof.rows, 0);
  requireEvidence(receipt.historical_declarations === 534 && receipt.current_declarations === declarations, 'complete current declaration replay and separate historical count');
  requireEvidence(receipt.source_build.exit_code === 0 && receipt.source_build.compiler_sha256 === '38f8502e5c108a362265663253283a0f60f14538951e592b2d5dd4db9e67be0a', 'actual scoped source build');
  requireEvidence(receipt.runtime.exit_code === 0 && receipt.runtime.profile_comparisons === 258 && receipt.runtime.stream_cases === 44, 'actual current canonical controls');
  requireEvidence(receipt.transcript_current.rows === 85 && receipt.transcript_current.all_exact === true && receipt.transcript_current.charge_leanchecker_exit === 0, 'actual current transcript proof replay');
  requireEvidence(JSON.stringify(receipt.transcript_current.proofs.map(proof => proof.file)) === JSON.stringify(['OwnedTranscript', 'ChargeContracts', 'TranscriptControls'].map(name => `src/Uorc/Integrity/${name}.lex.tex`)), 'complete current transcript proof inventory');
  for (const proof of receipt.transcript_current.proofs) validateProofReplay(root, proof);
  requireEvidence(receipt.transcript_current.proofs.reduce((total, proof) => total + proof.rows, 0) === 85, 'complete current transcript declaration count');
  requireEvidence(receipt.transcript_current.runtime_controls === 12 && receipt.transcript_current.runtime_exit === 0, 'actual current transcript controls');
  requireEvidence(receipt.current_binding_script_sha256 === sha(evidenceFile(root, 'scripts/integrity-binding-runtime.mjs')), 'current SDK binding adapter');
  requireEvidence(receipt.full_product_acceptance === false && receipt.original_pinned_sdk_acceptance === false, 'current replay qualification boundary');
  return receipt;
}
// Whether a receipt pin still names this source: the same bytes, the bridged
// language-1.2 notation of the same model, or an explicit historical snapshot.
export function canonicalSourceBound(root, file, expected) {
  return sourceBound(root, file, expected) || sha(historicalCanonicalSource(root, file, expected)) === expected;
}
export function historicalCanonicalSource(root, file, expected) {
  const current = evidenceFile(root, file);
  if (sha(current) === expected || sourceBound(root, file, expected)) return current;
  const receipt = validateCurrentCanonicalReplay(root);
  const row = receipt.historical_source_substitutions.find(item => item.file === file);
  requireEvidence(row && row.historical_sha256 === expected && row.current_sha256 === sha(current), `missing explicit source transition ${file}`);
  requireEvidence(row.source_ref === '8188c3d041d00b268acd1f0af75025fa9b28ac4a', `wrong historical source scope ${file}`);
  const original = evidenceFile(root, row.snapshot);
  requireEvidence(sha(original) === expected, `changed historical source ${file}`);
  return original;
}
