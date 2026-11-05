// Nonproduction importer; canonical equations are authored in the Source model.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {validateCanonicalEvidence, proofSources} from './integrity-canonical.mjs';
import {leanDriver} from './integrity-binding-runtime.mjs';
import {canonicalDriver} from './integrity-canonical-runtime.mjs';
import {extendCanonicalDriver, canonicalDriverInventory} from './integrity-canonical-driver.mjs';

test('canonical integrity has all source declarations, exact replay and actual defects', () => {
  const receipt = validateCanonicalEvidence('.');
  assert.equal(receipt.proofs.reduce((sum, row) => sum + row.rows, 0), 534);
});
test('canonical importer rejects omitted, stale, incorrect and axiom-widened evidence', () => {
  const original = validateCanonicalEvidence('.');
  const files = [...Object.keys(original.source_sha256), ...Object.keys(original.evidence_sha256)];
  for (const change of [r => r.proofs.pop(), r => { r.proofs[0].audit.pop(); }, r => { r.proofs[0].audit[0].observed.push('Classical.choice'); }, r => { r.mutations[0].mutant_exit = -9; }, r => { r.runtime.stream_cases = 0; }, r => { r.source_sha256[proofSources[0]] = '0'.repeat(64); }]) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'canonical-evidence-test-'));
    try {
      for (const file of files) { const dest = path.join(root, file); fs.mkdirSync(path.dirname(dest), {recursive: true}); fs.copyFileSync(file, dest); }
      const receipt = structuredClone(original); change(receipt);
      const dest = path.join(root, 'validation/integrity/canonical/evidence.json'); fs.mkdirSync(path.dirname(dest), {recursive: true}); fs.writeFileSync(dest, JSON.stringify(receipt));
      assert.throws(() => validateCanonicalEvidence(root));
    } finally { fs.rmSync(root, {recursive: true, force: true}); }
  }
});
test('normal SDK gate executes one complete canonical generated client', () => {
  const receipt = validateCanonicalEvidence('.');
  const driver = canonicalDriver('.');
  assert.equal(crypto.createHash('sha256').update(driver).digest('hex'), receipt.runtime.driver_sha256);
  assert.equal(canonicalDriverInventory.profileComparisons, 258);
  assert.equal(canonicalDriverInventory.streamCases, 44);
  for (const changed of ['', leanDriver('.').replace('private def main : IO Unit := do\n', ''), leanDriver('.').replace(' let bytes ← readImage done.arena output 32\n', ''), driver]) assert.throws(() => extendCanonicalDriver(changed));
  const binding = fs.readFileSync('scripts/integrity-binding-runtime.mjs', 'utf8');
  for (const file of proofSources) assert.ok(binding.includes(file), file);
  const entry = fs.readFileSync('scripts/integrity-canonical-runtime.mjs', 'utf8');
  assert.ok(entry.includes('run(process.cwd(), canonicalDriver)'));
  const just = fs.readFileSync('Justfile', 'utf8');
  assert.ok(just.includes('    node --test scripts/integrity-canonical.test.mjs\n'));
  assert.equal(just.split('    node scripts/integrity-canonical-runtime.mjs\n').length, 2);
  assert.equal(just.split('    node scripts/integrity-binding-runtime.mjs\n').length, 1);
});
