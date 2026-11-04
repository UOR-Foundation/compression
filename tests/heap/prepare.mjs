// Diagnostic fixture only. Never a replacement for the locked SDK release gate.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, '../..');
const target = process.argv[2];
if (!target || !path.isAbsolute(target)) throw Error('pass a new absolute diagnostic output directory');
await fs.mkdir(target, {recursive: true});
if ((await fs.readdir(target)).length) throw Error('diagnostic output directory must be empty');
const names = ['RuntimeValue', 'Heap', 'HeapAudit', 'HeapTests', 'RuntimeTable', 'RuntimeTableTests'];
const sources = [];
await fs.mkdir(path.join(target, 'src/Uorc/Core'), {recursive: true});
for (const name of names) {
  const relative = `src/Uorc/Core/${name}.lex.tex`;
  const bytes = await fs.readFile(path.join(repo, relative));
  await fs.writeFile(path.join(target, relative), bytes);
  sources.push({path: relative, sha256: crypto.createHash('sha256').update(bytes).digest('hex')});
}
for (const name of ['lexlean.toml', 'lakefile.toml', 'lake-manifest.json', 'lean-toolchain'])
  await fs.copyFile(path.join(here, name), path.join(target, name));
await fs.writeFile(path.join(target, 'source-inventory.json'), JSON.stringify({schema:'uorc/heap-diagnostic-sources/1', sources}, null, 2)+'\n');
console.log(target);
