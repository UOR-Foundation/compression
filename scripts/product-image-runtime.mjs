// Source-owned payload admission controls use the same generated-model gate.
import {validate} from './product-machine-runtime.mjs';
try {
  if (process.argv.length !== 2) throw new Error('no arguments accepted');
  validate(process.cwd(), undefined, 'ReadImageScenarios');
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
