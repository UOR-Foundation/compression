// One SDK verification/native pass, extended with canonical comparison controls.
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {run, leanDriver} from './integrity-binding-runtime.mjs';
import {extendCanonicalDriver} from './integrity-canonical-driver.mjs';

export function canonicalDriver(root) {
  return extendCanonicalDriver(leanDriver(root));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 2) throw new Error('no arguments accepted');
    run(process.cwd(), canonicalDriver);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
