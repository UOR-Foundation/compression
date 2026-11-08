// Evidence importer only. Hashing and transcript semantics are source-owned.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {canonicalSourceBound, validateCurrentCanonicalReplay} from './integrity-current-context.mjs';
import {leanDriver} from './integrity-binding-runtime.mjs';

export const controlNames = [
  'admittedPendingEarlyFinalize', 'splitHeaderDrain', 'emptySpanBetweenPartitions',
  'cancelDuringUpdate', 'cancelDuringReleaseDrain', 'actualReadFailureAfterPrefix',
  'latePreimageCorruption', 'lateStoredDigestMismatch', 'orderedReaderBoundary',
  'reorderedSourceAttempt', 'repeatedSourceAttempt', 'basisThenMainFailureCumulative',
];
export const sourceFiles = [
  'src/Uorc/Integrity/OwnedTranscript.lex.tex',
  'src/Uorc/Integrity/ChargeContracts.lex.tex',
  'src/Uorc/Integrity/TranscriptControls.lex.tex',
];
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const required = (ok, message) => { if (!ok) throw new Error(message); };
export function validateTranscriptEvidence(root) {
  const read = file => fs.readFileSync(path.join(root, file));
  const receipt = JSON.parse(read('validation/integrity/transcript/evidence.json'));
  const current = validateCurrentCanonicalReplay(root);
  required(receipt.schema === 'uorc/ordered-integrity-transcript-evidence/1', 'transcript schema');
  required(receipt.runtime.exit_code === 0, 'source runtime failed');
  required(JSON.stringify(receipt.runtime.controls.map(r => r.name)) === JSON.stringify(controlNames), 'complete ordered control inventory');
  for (const row of receipt.runtime.controls) required(row.passed === true, `failed ${row.name}`);
  for (const file of sourceFiles) required(Object.hasOwn(receipt.source_sha256, file), `missing source ${file}`);
  for (const [file, expected] of Object.entries(receipt.source_sha256)) {
    required(!path.isAbsolute(file) && !file.split('/').includes('..'), 'source outside project');
    required(canonicalSourceBound(root, file, expected), `stale source ${file}`);
  }
  for (const [file, expected] of Object.entries(receipt.evidence_sha256)) {
    required(!path.isAbsolute(file) && !file.split('/').includes('..'), 'evidence outside project');
    required(sha(read(file)) === expected, `changed evidence ${file}`);
  }
  required(receipt.proofs.owned.rows === 45 && receipt.proofs.owned.leanchecker_exit === 0 && receipt.proofs.owned.exact_axioms === true, 'owned transcript proof scope');
  required(receipt.proofs.charge.rows === 16 && receipt.proofs.charge.leanchecker_exit === 0 && receipt.proofs.charge.exact_axioms === true, 'charge proof scope');
  required(receipt.controls_policy_rows === 22, 'complete control axiom inventory');
  const combined = receipt.combined;
  required(combined.source_modules === 27 && combined.sdk_runtime_modules === 3 && combined.native_compiles === 30, 'combined native closure');
  required(combined.all_new_rows_exact === 85 && combined.source_and_driver_current_after_run === true, 'combined exact audit');
  required(combined.fixtures === 20 && combined.partitions === 60 && combined.byte_reconstructions === 10 && combined.transcript_controls === 12, 'combined runtime inventory');
  required(sha(read('scripts/integrity-binding-runtime.mjs')) === current.current_binding_script_sha256, 'stale current binding script');
  required(sha(Buffer.from(leanDriver(root))) === current.legacy_driver_sha256, 'changed legacy generated client');
  required(receipt.mutations.reader_origin === 'rejected-restored', 'reader origin control');
  required(receipt.mutations.overflow_reset === 'rejected-restored', 'overflow control');
  required(JSON.stringify(receipt.mutations.failure_usage_reset) === JSON.stringify(['actualReadFailureAfterPrefix', 'basisThenMainFailureCumulative']), 'failed-prefix mutation attribution');
  required(receipt.qualification === 'source build, kernel replay and finite generated-model controls; not full pinned SDK acceptance', 'qualification boundary');
  return receipt;
}
