// Regenerates validation/integrity/canonical-current/evidence.json from real tool runs:
// one scoped lexlean verify of the 7 canonical and 3 transcript proof modules
// (kernel compile, leanchecker and exact axiom audit read from the attestation),
// the committed canonical runtime (node scripts/integrity-canonical-runtime.mjs),
// and `git show` snapshots for every historical pin that no longer matches.
// The frozen canonical/ and transcript/ receipts are only read.
// Checked by validateCurrentCanonicalReplay (scripts/integrity-current-context.mjs).
//   node scripts/integrity-canonical-current-regenerate.mjs [--plan]
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {repo, sha, requireFact, scratchProject, lexleanBinary, verify, proofReplay, closureSources, unchangedInRepository} from './integrity-consumer-regenerate.mjs';
import {currentCanonicalProofNames, validateCurrentCanonicalReplay} from './integrity-current-context.mjs';
import {sourceFiles as transcriptFiles, validateTranscriptEvidence} from './integrity-transcript.mjs';
import {validateCanonicalEvidence} from './integrity-canonical.mjs';
import {canonicalDriver} from './integrity-canonical-runtime.mjs';
import {leanDriver} from './integrity-binding-runtime.mjs';

const script = fileURLToPath(import.meta.url);
const base = 'validation/integrity/canonical-current';
const receiptFile = `${base}/evidence.json`, replayDir = `${base}/replay`;
const historical = ['validation/integrity/canonical/evidence.json', 'validation/integrity/transcript/evidence.json'];
const bindingScript = 'scripts/integrity-binding-runtime.mjs';
// Both values are fixed by validateCurrentCanonicalReplay; checked here before any build.
const requiredCompiler = 'c4b08084a27cd1d2992702ce2f2dd2fcae789aeb8d68f6eb01346da59fabefb2';
const sourceRef = '8188c3d041d00b268acd1f0af75025fa9b28ac4a';
const canonicalFiles = currentCanonicalProofNames.map(name => `src/Uorc/Integrity/${name}.lex.tex`);
const read = file => fs.readFileSync(path.join(repo, file));
function git(args, encoding) {
  const r = spawnSync('git', args, {cwd: repo, maxBuffer: 1 << 30, ...(encoding ? {encoding} : {})});
  requireFact(r.status === 0, `git ${args.join(' ')} failed: ${r.stderr}`);
  return r.stdout;
}
// One substitution row per historical pin whose bytes no longer match, from the pinned commit.
export function substitutions() {
  requireFact(git(['rev-parse', `${sourceRef.slice(0, 7)}^{commit}`], 'utf8').trim() === sourceRef, 'historical source commit not available');
  const pins = new Map();
  for (const receipt of historical) for (const [file, expected] of Object.entries(JSON.parse(read(receipt)).source_sha256)) {
    requireFact(!pins.has(file) || pins.get(file) === expected, `${file} pinned at two different historical digests`);
    pins.set(file, expected);
  }
  const rows = [];
  for (const [file, expected] of [...pins].sort(([a], [b]) => a < b ? -1 : 1)) {
    const current = sha(read(file));
    if (current === expected) continue;
    const shown = git(['show', `${sourceRef}:${file}`]);
    requireFact(sha(shown) === expected, `${file}: pin ${expected} is not the ${sourceRef} source; it needs a new replay, not a snapshot`);
    rows.push({file, snapshot: `${base}/historical-source/${file}.snapshot`, historical_sha256: expected, current_sha256: current, source_ref: sourceRef, bytes: shown});
  }
  return rows;
}
// Counts the executed driver enforces with Lean `check` gates, plus the partition count it prints.
export function driverCounts(driver, stdout) {
  const one = (re, what) => { const m = [...driver.matchAll(re)]; requireFact(m.length === 1, `driver must contain one ${what} gate`); return Number(m[0][1]); };
  const printed = [...stdout.matchAll(/; (\d+) span partitions;/g)];
  requireFact(printed.length === 1, 'runtime stdout omits its partition count');
  const owned = one(/ check \(count == (\d+)\) "complete finite OwnedHash canonical case inventory"\n/g, 'owned canonical case');
  return {owned_cases: owned, fixture_partitions: Number(printed[0][1]), profile_comparisons: owned + Number(printed[0][1]),
    stream_cases: one(/ check \(count == (\d+)\) "complete diagnostic StreamEngine case inventory"\n/g, 'stream case'),
    transcript_controls: one(/ check \(orderedChecks\.length == (\d+)\) "complete ordered source transcript inventory"\n/g, 'transcript control')};
}
// The committed canonical runtime, run in the repository; its outputs land in .prism/uorc/binding.*.
function runtime(s) {
  const prism = path.join(repo, '.prism/uorc'), listing = () => fs.existsSync(prism) ? fs.readdirSync(prism).filter(d => d.startsWith('binding.')) : [];
  const before = new Set(listing()), driver = canonicalDriver(repo);
  const r = spawnSync(process.execPath, ['scripts/integrity-canonical-runtime.mjs'], {cwd: repo, encoding: 'utf8', maxBuffer: 1 << 30, env: {...process.env, ...s.env}});
  requireFact(!r.error && !r.signal && r.status !== null, `canonical runtime did not complete: ${r.error ?? r.signal}`);
  const made = listing().filter(d => !before.has(d));
  requireFact(made.length === 1, `canonical runtime left ${made.length} output directories`);
  const out = path.join(prism, made[0]), file = f => fs.existsSync(path.join(out, f)) ? fs.readFileSync(path.join(out, f), 'utf8') : '';
  requireFact(r.status === 0, `canonical runtime exited ${r.status}: ${r.stderr}; see ${out}`);
  const stdout = file('runtime.stdout'), receipt = JSON.parse(file('receipt.json'));
  requireFact(receipt.driver_sha256 === sha(Buffer.from(driver)), 'executed driver is not the current canonical driver');
  requireFact(receipt.stdout_sha256 === sha(Buffer.from(stdout)), 'runtime receipt does not bind its stdout');
  return {exit_code: r.status, driver_sha256: receipt.driver_sha256, stdout_sha256: receipt.stdout_sha256, ...driverCounts(driver, stdout),
    files: {'runtime.stdout': stdout, 'runtime.stderr': file('runtime.stderr'), 'runtime-receipt.json': file('receipt.json'), 'runtime-node.stdout': r.stdout, 'runtime-node.stderr': r.stderr}};
}
const exact = row => (row.policy.kind === 'exact' || (row.policy.kind === 'none' && row.policy.axioms.length === 0)) && JSON.stringify([...row.observed].sort()) === JSON.stringify([...row.policy.axioms].sort());
export function regenerate() {
  const compiler = lexleanBinary();
  requireFact(compiler.sha256 === requiredCompiler, `lexlean on PATH (${compiler.file}) is ${compiler.sha256}; the checker requires compiler ${requiredCompiler}`);
  const rows = substitutions();
  const bindingSha = sha(read(bindingScript)), legacyDriverSha = sha(Buffer.from(leanDriver(repo)));
  const s = scratchProject('canonical-current-'), files = {};
  try {
    const v = verify(s, [...canonicalFiles, ...transcriptFiles], 'verify');
    requireFact(v.attestation.lexlean.executable_sha256 === compiler.sha256, 'attestation names a different lexlean binary');
    const sources = closureSources(s, v.manifest);
    const replay = file => proofReplay(v, file, fs.readFileSync(path.join(s.project, file), 'utf8'));
    const proofs = canonicalFiles.map(replay), transcript = transcriptFiles.map(replay);
    for (const [f, t] of [['verify.json', v.stdout], ['verify.stderr', v.stderr],
      ['attestation.json', fs.readFileSync(path.join(v.artifact, 'attestation.json'), 'utf8')],
      ['build-manifest.json', fs.readFileSync(path.join(v.artifact, 'build-manifest.json'), 'utf8')]]) files[f] = t;
    const run = runtime(s);
    Object.assign(files, run.files);
    unchangedInRepository(sources);
    requireFact(sha(read(bindingScript)) === bindingSha && sha(Buffer.from(leanDriver(repo))) === legacyDriverSha, 'binding script or driver changed during the run');
    const historicalCanonical = JSON.parse(read(historical[0]));
    const charge = transcript.find(p => p.file.endsWith('/ChargeContracts.lex.tex'));
    // Install outputs, then the receipt; the checkers judge the installed tree.
    fs.rmSync(path.join(repo, replayDir), {recursive: true, force: true}); fs.mkdirSync(path.join(repo, replayDir), {recursive: true});
    for (const [f, t] of Object.entries(files)) fs.writeFileSync(path.join(repo, replayDir, f), t);
    for (const row of rows) { fs.mkdirSync(path.dirname(path.join(repo, row.snapshot)), {recursive: true}); fs.writeFileSync(path.join(repo, row.snapshot), row.bytes); }
    const pinned = [...Object.keys(files).map(f => `${replayDir}/${f}`), ...rows.map(r => r.snapshot), historical[1], bindingScript];
    const receipt = {
      schema: 'uorc/canonical-current-source-replay/1',
      historical_receipt_sha256: sha(read(historical[0])), historical_transcript_receipt_sha256: sha(read(historical[1])),
      historical_declarations: historicalCanonical.proofs.reduce((n, p) => n + p.rows, 0),
      current_declarations: proofs.reduce((n, p) => n + p.rows, 0),
      source_sha256: sources,
      evidence_sha256: Object.fromEntries(pinned.map(f => [f, sha(read(f))])),
      historical_source_substitutions: rows.map(({bytes, ...row}) => row),
      proofs,
      current_binding_script_sha256: bindingSha, legacy_driver_sha256: legacyDriverSha,
      source_build: {command: 'verify', exit_code: v.status, compiler_sha256: v.attestation.lexlean.executable_sha256,
        compiler_version: v.attestation.lexlean.version, build_id: v.attestation.build_id, source_id: v.attestation.source_id,
        semantic_id: v.attestation.semantic_id, attestation_id: v.attestation.attestation_id, selection: [...canonicalFiles, ...transcriptFiles]},
      runtime: {exit_code: run.exit_code, profile_comparisons: run.profile_comparisons, stream_cases: run.stream_cases,
        owned_cases: run.owned_cases, fixture_partitions: run.fixture_partitions, driver_sha256: run.driver_sha256, stdout_sha256: run.stdout_sha256},
      transcript_current: {rows: transcript.reduce((n, p) => n + p.rows, 0), all_exact: transcript.every(p => p.audit.every(exact)),
        charge_leanchecker_exit: charge.leanchecker_exit, runtime_controls: run.transcript_controls, runtime_exit: run.exit_code, proofs: transcript},
      full_product_acceptance: false, original_pinned_sdk_acceptance: false,
      remaining: ['Original pinned-SDK, production preservation/export and whole-product acceptance remain unqualified.'],
    };
    const target = path.join(repo, receiptFile), previous = fs.readFileSync(target);
    fs.writeFileSync(target, JSON.stringify(receipt, null, 1) + '\n');
    try { validateCurrentCanonicalReplay(repo); validateCanonicalEvidence(repo); validateTranscriptEvidence(repo); }
    catch (error) {
      fs.writeFileSync(path.join(s.dir, 'rejected-evidence.json'), JSON.stringify(receipt, null, 1) + '\n');
      fs.writeFileSync(target, previous);
      throw new Error(`regenerated receipt rejected by the checker: ${error.message}; candidate at ${path.join(s.dir, 'rejected-evidence.json')}, previous receipt restored`);
    }
    fs.rmSync(s.dir, {recursive: true, force: true});
    console.log(`canonical-current: ${receipt.current_declarations} canonical + ${receipt.transcript_current.rows} transcript rows exact; ${rows.length} historical substitutions; runtime ${run.profile_comparisons}/${run.stream_cases}/${run.transcript_controls}`);
  } catch (error) { error.message += ` (scratch kept at ${s.dir})`; throw error; }
}
if (process.argv[1] && path.resolve(process.argv[1]) === script) {
  try {
    const args = process.argv.slice(2);
    requireFact(args.length === 0 || (args.length === 1 && args[0] === '--plan'), 'usage: integrity-canonical-current-regenerate.mjs [--plan]');
    if (args[0] === '--plan') {
      const compiler = lexleanBinary();
      console.log(`lexlean ${compiler.file} ${compiler.sha256}${compiler.sha256 === requiredCompiler ? '' : ` (checker requires ${requiredCompiler})`}`);
      console.log(`verify ${[...canonicalFiles, ...transcriptFiles].length} proof modules; run scripts/integrity-canonical-runtime.mjs`);
      for (const row of substitutions()) console.log(`substitute ${row.file} ${row.historical_sha256.slice(0, 12)} -> ${row.current_sha256.slice(0, 12)}`);
    } else regenerate();
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
