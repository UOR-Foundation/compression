// Evidence importer only; direct consumer semantics belong to the Source model.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {sourceBound} from './language-migration.mjs';

test('closed archive integrity has actual consumer and source-layout proof evidence', () => {
  const r = JSON.parse(fs.readFileSync('validation/integrity/consumer/evidence.json'));
  assert.equal(r.schema, 'uorc/closed-archive-consumer-evidence/1');
  for (const name of ['OwnedReadCoverageLaws', 'ClosedArchiveCallLaws', 'ArchiveConsumerRefinement']) {
    const file = `src/Uorc/Integrity/${name}.lex.tex`;
    assert.ok(Object.hasOwn(r.source_sha256, file), `missing actual source ${file}`);
  }
  for (const [file, expected] of Object.entries(r.source_sha256)) assert.ok(sourceBound('.', file, expected), `stale source ${file}`);
  assert.equal(r.kernel_exit, 0);
  assert.equal(r.all_axiom_policies_exact, true);
  assert.ok(r.actual_mutations.length > 0);
  for (const row of r.actual_mutations) { assert.equal(row.mutant_exit, 1); assert.equal(row.restored_exit, 0); }
});
