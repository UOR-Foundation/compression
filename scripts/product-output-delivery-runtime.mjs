// Execute this actual source-owned runtime and its owning planted defects.
import {validate} from './product-machine-runtime.mjs';
try {
  if (process.argv.length !== 2) throw new Error('no arguments accepted');
  validate(process.cwd(), undefined, 'OutputDeliveryScenarios');
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
