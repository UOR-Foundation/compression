// Scoped evidence tests. The complete archive composition has a separate owner.
import test from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {evidenceFile, validateProofReplay} from './integrity-current-context.mjs';
import {sourceBound} from './language-migration.mjs';

const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
for (const [component, rows] of [['reader', 107], ['calls', 44], ['trace', 20]]) {
  test(`actual ${component} Source proof and semantic defect retain their exact scope`, () => {
    const receipt = JSON.parse(evidenceFile('.', `validation/integrity/consumer/${component}/evidence.json`));
    assert.equal(receipt.schema, 'uorc/consumer-component-evidence/1');
    assert.equal(receipt.proof.rows, rows);
    for (const [file, expected] of Object.entries(receipt.source_sha256)) assert.ok(sourceBound('.', file, expected), `stale component source ${file}`);
    for (const [file, expected] of Object.entries(receipt.evidence_sha256)) assert.equal(sha(evidenceFile('.', file)), expected, `changed component evidence ${file}`);
    validateProofReplay('.', receipt.proof);
    assert.equal(receipt.mutation.mutantReaderKernelExit, 0);
    assert.equal(receipt.mutation.unchangedProofKernelExit, 1);
    assert.equal(receipt.mutation.restoredProofKernelExit, 0);
    assert.equal(receipt.mutation.restoredLeancheckerExit, 0);
    assert.equal(receipt.mutation.restoredBuildIdentical, true);
    assert.ok(receipt.mutation.rejectedDeclarations.length > 0);
    assert.equal(receipt.original_pinned_sdk_acceptance, false);
    assert.equal(receipt.full_product_acceptance, false);
  });
}
