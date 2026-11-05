// Execute the actual owned framing-to-inspection source binding and its defects.
import {validate} from './product-machine-runtime.mjs';
try {
  if (process.argv.length !== 2) throw new Error('no arguments accepted');
  validate(process.cwd(), undefined, 'InspectionReportScenarios');
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
