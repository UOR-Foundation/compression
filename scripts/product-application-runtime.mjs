// Execute this source-owned typed suite; no product implementation lives here.
import {validate} from './product-machine-runtime.mjs';
try {
  if (process.argv.length !== 2) throw new Error('no arguments accepted');
  validate(process.cwd(), undefined, 'ApplicationScenarios');
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
