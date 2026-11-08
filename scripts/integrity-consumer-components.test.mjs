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
    // The kept mutant is exactly the planted source the receipt names.
    const mutant = JSON.parse(evidenceFile('.', `validation/integrity/consumer/${component}/mutant-source.json`));
    assert.equal(mutant.source_path, receipt.mutation.sourcePath);
    assert.equal(sha(mutant.notation), receipt.mutation.mutantSourceSha256);
    // The rejecting module is named, and its kernel exit is the recorded one.
    const law = `Uorc.Uorc.Integrity.${{reader: 'OwnedReadCoverageLaws', calls: 'ClosedArchiveCallLaws', trace: 'ArchiveIntegrityTraceLaws'}[component]}`;
    assert.equal(receipt.mutation.rejectingModule, component === 'trace' ? 'Uorc.Uorc.Integrity.ClosedArchiveCallLaws' : law);
    const exits = JSON.parse(evidenceFile('.', `validation/integrity/consumer/${component}/mutant-compile.json`));
    assert.deepEqual(exits.at(-1), {module: receipt.mutation.rejectingModule, exit_code: 1});
    if (receipt.mutation.rejectingModule !== law) assert.ok(!exits.some(e => e.module === law), `${law} must not be reported as compiled`);
    assert.equal(receipt.original_pinned_sdk_acceptance, false);
    assert.equal(receipt.full_product_acceptance, false);
  });
}
