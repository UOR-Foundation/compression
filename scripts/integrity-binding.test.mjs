// Acceptance infrastructure only; production behavior resides in LexLean.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
test('owned integrity adapter and explicit profile source exist',()=>{
 for(const name of ['Profile','OwnedHash','ArchiveBinding'])assert.ok(fs.readFileSync(`src/Uorc/Integrity/${name}.lex.tex`,'utf8').includes('\\begin{semanticmodule}'));
});
