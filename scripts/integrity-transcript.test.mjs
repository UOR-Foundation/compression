// Nonproduction importer for actual source-generated transcript evidence.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {validateTranscriptEvidence, sourceFiles} from './integrity-transcript.mjs';
import {leanDriver} from './integrity-binding-runtime.mjs';
test('ordered transcript capability has actual source and bound execution evidence', () => {
  for (const file of sourceFiles) assert.ok(fs.existsSync(file), `missing ${file}`);
  assert.equal(validateTranscriptEvidence('.').runtime.controls.length, 12);
});
test('transcript importer rejects stale source, omitted controls and false observations', () => {
  const original = validateTranscriptEvidence('.');
  const files = [...Object.keys(original.source_sha256), ...Object.keys(original.evidence_sha256), 'scripts/integrity-binding-runtime.mjs'];
  for (const change of [r => r.runtime.controls.pop(), r => r.runtime.controls.push(r.runtime.controls[0]), r => { r.runtime.controls[5].passed = false; }, r => { r.proofs.owned.rows = 0; }, r => { r.mutations.overflow_reset = 'unvisited'; }, r => { r.combined.native_compiles = 0; }]) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ordered-transcript-test-'));
    try {
      for (const file of files) { const dest = path.join(root, file); fs.mkdirSync(path.dirname(dest), {recursive:true}); fs.copyFileSync(file, dest); }
      const receipt = structuredClone(original); change(receipt);
      const dest = path.join(root, 'validation/integrity/transcript/evidence.json'); fs.mkdirSync(path.dirname(dest), {recursive:true}); fs.writeFileSync(dest, JSON.stringify(receipt));
      assert.throws(() => validateTranscriptEvidence(root));
    } finally { fs.rmSync(root, {recursive:true, force:true}); }
  }
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ordered-transcript-stale-'));
  try {
    for (const file of files) { const dest = path.join(root, file); fs.mkdirSync(path.dirname(dest), {recursive:true}); fs.copyFileSync(file, dest); }
    const dest = path.join(root, 'validation/integrity/transcript/evidence.json'); fs.writeFileSync(dest, JSON.stringify(original));
    fs.appendFileSync(path.join(root, sourceFiles[0]), ' ');
    assert.throws(() => validateTranscriptEvidence(root), /stale source/);
  } finally { fs.rmSync(root, {recursive:true, force:true}); }
});
test('normal generated-model binding gate executes the complete transcript controls', () => {
  const source = fs.readFileSync('scripts/integrity-binding-runtime.mjs', 'utf8');
  for (const file of sourceFiles) assert.ok(source.includes(file), file);
  const driver = leanDriver('.');
  assert.equal(crypto.createHash('sha256').update(driver).digest('hex'), validateTranscriptEvidence('.').combined.driver_sha256);
  assert.ok(driver.includes('TranscriptControls.runtimeChecks'));
  assert.ok(driver.includes('orderedChecks.length == 12'));
  assert.ok(driver.includes('orderedChecks.all id'));
  assert.ok(fs.readFileSync('Justfile', 'utf8').includes('    node --test scripts/integrity-transcript.test.mjs\n'));
});
