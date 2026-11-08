// Regenerates validation/integrity/consumer/<component>/ from real tool runs.
// Every recorded field is read from lexlean/lean/leanchecker output; nothing is
// hand-entered. Checked by scripts/integrity-consumer-components.test.mjs.
//   node scripts/integrity-consumer-regenerate.mjs reader|calls|trace|all [--plan]
// --plan only plants each mutant in memory (SDK data form -> notation) and
// prints the selection; it runs no build.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {dataForm, notation, stageRuntime} from './semantic-source.mjs';
import {validateProofReplay} from './integrity-current-context.mjs';

const script = fileURLToPath(import.meta.url);
export const repo = path.resolve(path.dirname(script), '..');
export const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export const requireFact = (ok, why) => { if (!ok) throw new Error(why); };
const projectFiles = ['lexlean.toml', 'lexlean.lock', 'lakefile.toml', 'lake-manifest.json', 'lean-toolchain'];

// Walk a semantic expression tree; `visit` returns a replacement node or undefined.
function rewrite(node, visit) {
  if (Array.isArray(node)) return node.forEach((child, i) => { const r = visit(child); if (r !== undefined) node[i] = r; else rewrite(child, visit); });
  if (!node || typeof node !== 'object') return;
  for (const [key, child] of Object.entries(node)) { const r = visit(child); if (r !== undefined) node[key] = r; else rewrite(child, visit); }
}
const uint = (representation, value) => ({kind: 'integer', representation, value: String(value)});
// Planted semantic defects, re-derived against the language-1.2 data form.
export const components = {
  reader: {
    law: 'OwnedReadCoverageLaws', rows: 107,
    scope: 'Scoped lexlean verify of the OwnedReadCoverageLaws closure (kernel compile, leanchecker, per-declaration exact axiom audit from the lexlean attestation) and a planted OwnedRead.step defect. No full SDK verify, Rust preservation or product release claim.',
    mutation: {
      source: 'src/Uorc/Wire/OwnedRead.lex.tex', definition: 'step',
      fault: 'Actual OwnedRead.step sequence-leaf span count (the one UInt64 let in step) replaced by 0; unchanged proof source must be rejected.',
      apply(body) { let n = 0; rewrite(body, x => { if (x?.kind === 'let' && x.binder?.type?.kind === 'uint64') { n++; x.value = uint('uint64', 0); } }); return n; },
    },
  },
  calls: {
    law: 'ClosedArchiveCallLaws', rows: 44,
    scope: 'Actual call/phase effects only; full accepted trailer history and canonical consumer composition remain open. Scoped lexlean verify (kernel compile, leanchecker, exact axiom audit from the attestation) and a planted ArchiveIntegrity.byteAt defect. No full SDK verify or Rust preservation claim.',
    mutation: {
      source: 'src/Uorc/Wire/ArchiveIntegrity.lex.tex', definition: 'byteAt',
      fault: 'Actual ArchiveIntegrity.byteAt success payload replaced by constant 0; unchanged actual read bridge must reject.',
      apply(body) {
        let n = 0;
        for (const branch of body.kind === 'match' ? body.branches : []) {
          const b = branch.body;
          if (branch.constructor?.name === 'Result.ok' && b?.kind === 'constructor' && b.constructor?.name === 'Result.ok' && b.arguments?.length === 1 && b.arguments[0].kind === 'var' && b.arguments[0].name === branch.binders[0]) { b.arguments[0] = uint('uint8', 0); n++; }
        }
        return n;
      },
    },
  },
  trace: {
    law: 'ArchiveIntegrityTraceLaws', rows: 20,
    scope: 'Actual ArchiveIntegrity path failure persistence, snapshot and pre-trailer compared preservation, via scoped lexlean verify (kernel compile, leanchecker, exact axiom audit from the attestation) and a planted ArchiveIntegrity.adoptHash defect rejected inside the trace closure by the imported ClosedArchiveCallLaws, which unfolds every ArchiveIntegrity transition body; the trace law cannot elaborate over the defect. No full consumer/canonical acceptance claim.',
    mutation: {
      source: 'src/Uorc/Wire/ArchiveIntegrity.lex.tex', definition: 'adoptHash',
      fault: 'Actual ArchiveIntegrity.adoptHash pending branch increments compared by one; the unchanged trace closure must reject it, first at its imported ClosedArchiveCallLaws.',
      rejectedBy: 'Uorc.Uorc.Integrity.ClosedArchiveCallLaws',
      apply(body) {
        let n = 0;
        for (const branch of body.kind === 'let' && body.value?.kind === 'match' ? body.value.branches : []) {
          if (branch.constructor?.name !== 'Outcome.pending' || branch.body?.kind !== 'record') continue;
          const field = branch.body.fields.find(f => f.field === 'compared');
          if (field) { field.value = {kind: 'call', function: {module: 'Uorc.Core.Heap', name: 'wordAdd'}, arguments: [field.value, uint('uint64', 1)]}; n++; }
        }
        return n;
      },
    },
  },
};
export function parseArgs(argv) {
  const plan = argv.includes('--plan'), rest = argv.filter(a => a !== '--plan');
  requireFact(rest.length === 1 && (rest[0] === 'all' || Object.hasOwn(components, rest[0])), 'usage: integrity-consumer-regenerate.mjs reader|calls|trace|all [--plan]');
  return {plan, names: rest[0] === 'all' ? Object.keys(components) : [rest[0]]};
}
// The planted source, written back in notation through the SDK conversions.
export function plant(text, mutation) {
  const form = dataForm(text), rows = [...form.matchAll(/\\semanticdata\{(.*)\}\n/g)];
  requireFact(rows.length === 1, `one semantic module required in ${mutation.source}`);
  const module = JSON.parse(rows[0][1]), decls = module.declarations.filter(d => d.kind === 'definition' && d.name === mutation.definition);
  requireFact(decls.length === 1, `one ${mutation.definition} definition required`);
  requireFact(mutation.apply(decls[0].body) === 1, `mutation of ${mutation.definition} must change exactly one node`);
  const planted = notation(form.replace(rows[0][1], () => JSON.stringify(module)));
  requireFact(planted !== text && dataForm(planted) !== form, 'planted source unchanged');
  return planted;
}
// The checker's own declaration inventory (validateProofReplay), used to select audit rows.
export function declarationNames(file, source) {
  const base = path.basename(file, '.lex.tex');
  const semantic = [...source.matchAll(/\\semanticdata\{(.*)\}\n/g)];
  const semanticNotation = [...source.matchAll(/^(?:structure|class|instance|inductive|definition|theorem)\{\n  name: ([^,\n]+),/gm)];
  return semantic.length === 1 ? JSON.parse(semantic[0][1]).declarations.map(row => `Uorc.Uorc.Integrity.${base}.${row.name}`)
    : semanticNotation.length > 0 ? semanticNotation.map(m => `Uorc.Uorc.Integrity.${base}.${m[1]}`)
    : [...source.matchAll(/^(?:definition|theorem|opaque|abbrev|inductive|constructor|recursor)\s+(.+?)\s*:\s*@\d+/gm)].map(m => m[1].replace(/\.\{[^}]*\}$/, ''));
}
export const leanModule = file => 'Uorc.' + file.replace(/^src\//, '').replace(/\.lex\.tex$/, '').split('/').join('.');

// One child process; stdout/stderr kept under `out/label.*`.
export function execute(command, args, {cwd, out, label, env = {}}) {
  const r = spawnSync(command, args, {cwd, encoding: 'utf8', maxBuffer: 1 << 30, env: {...process.env, ...env}});
  requireFact(!r.error && !r.signal && r.status !== null, `${label}: ${command} did not complete: ${r.error ?? r.signal}`);
  fs.writeFileSync(path.join(out, label + '.stdout'), r.stdout ?? ''); fs.writeFileSync(path.join(out, label + '.stderr'), r.stderr ?? '');
  return r;
}
// A scratch copy of the Source project under the repository's .tmp/.
export function scratchProject(prefix) {
  fs.mkdirSync(path.join(repo, '.tmp'), {recursive: true});
  const dir = fs.mkdtempSync(path.join(repo, '.tmp', prefix));
  const project = path.join(dir, 'project'), out = path.join(dir, 'out'), tmp = path.join(dir, 'tmp');
  for (const d of [project, out, tmp]) fs.mkdirSync(d, {recursive: true});
  fs.cpSync(path.join(repo, 'src'), path.join(project, 'src'), {recursive: true});
  for (const f of projectFiles) fs.copyFileSync(path.join(repo, f), path.join(project, f));
  return {dir, project, out, tmp, env: {TMPDIR: tmp}};
}
export function lexleanBinary() {
  const found = spawnSync('sh', ['-c', 'command -v lexlean'], {encoding: 'utf8'});
  requireFact(found.status === 0 && found.stdout.trim(), 'lexlean not on PATH');
  const file = fs.realpathSync(found.stdout.trim());
  return {file, sha256: sha(fs.readFileSync(file))};
}
// `lexlean verify` of a selection: the official build, kernel compile, leanchecker and axiom audit.
export function verify(s, selected, label) {
  const r = execute('lexlean', ['--project', path.join(s.project, 'lexlean.toml'), '--diagnostic-format', 'json', 'verify', ...selected], {cwd: s.project, out: s.out, label, env: s.env});
  requireFact(r.status === 0, `${label}: lexlean verify exited ${r.status}; see ${path.join(s.out, label)}.stdout`);
  const result = JSON.parse(r.stdout);
  requireFact(result.success === true && result.artifacts?.length === 1, `${label}: one verified artifact required`);
  const artifact = path.resolve(s.project, result.artifacts[0]);
  requireFact(artifact.startsWith(path.resolve(s.project) + path.sep), `${label}: artifact escaped project`);
  const attestation = JSON.parse(fs.readFileSync(path.join(artifact, 'attestation.json'), 'utf8'));
  const manifest = JSON.parse(fs.readFileSync(path.join(artifact, 'build-manifest.json'), 'utf8'));
  requireFact(attestation.status === 'verified' && attestation.build_id === manifest.build_id, `${label}: attestation does not certify its build manifest`);
  return {status: r.status, stdout: r.stdout, stderr: r.stderr, result, artifact, attestation, manifest};
}
export function process1(attestation, tool, module, compile) {
  const rows = attestation.processes.filter(p => p.tool === tool && p.module === module && (!compile || p.argv.includes('-o')));
  requireFact(rows.length === 1, `attestation has ${rows.length} ${tool} runs for ${module}`);
  return rows[0];
}
// One proof replay row, every field from the verified attestation.
export function proofReplay(v, file, source) {
  const module = leanModule(file), names = declarationNames(file, source);
  const audit = names.map(name => {
    const rows = v.attestation.declarations.filter(d => d.name === name);
    requireFact(rows.length === 1, `attestation audits ${name} ${rows.length} times`);
    return {name, policy: rows[0].policy, observed: rows[0].observed, result: rows[0].result};
  });
  return {file, module, rows: names.length, source_build_passed: v.status === 0 && v.result.success === true,
    kernel_exit: process1(v.attestation, 'lean', module, true).exit_code,
    leanchecker_exit: process1(v.attestation, 'leanchecker', module, false).exit_code, audit};
}
// Source bytes the verify built, from its manifest, read from the scratch copy.
export function closureSources(s, manifest) {
  return Object.fromEntries(manifest.modules.map(m => m.source_path).sort().map(f => [f, sha(fs.readFileSync(path.join(s.project, f)))]));
}
export function unchangedInRepository(sources) {
  const changed = Object.entries(sources).filter(([f, h]) => sha(fs.readFileSync(path.join(repo, f))) !== h).map(([f]) => f);
  requireFact(changed.length === 0, `repository sources changed during the run: ${changed.join(', ')}`);
}
// Stock-lean kernel compile of a generated closure over the staged LexLean runtime.
function compileGenerated(s, artifact, root, label, rejectedBy) {
  const modules = path.join(artifact, 'modules'), stage = path.join(s.dir, label + '-model');
  const list = d => fs.readdirSync(d, {withFileTypes: true}).flatMap(e => e.isDirectory() ? list(path.join(d, e.name)) : e.name.endsWith('.lean') ? [path.join(d, e.name)] : []);
  const files = new Map(list(modules).map(p => [path.relative(modules, p).slice(0, -5).split(path.sep).join('.'), p]));
  requireFact(files.has(root), `${label}: ${root} not generated`);
  stageRuntime(stage);
  const exits = [], done = new Map();
  const compile = name => {
    if (done.has(name)) return done.get(name);
    done.set(name, null);
    const text = fs.readFileSync(files.get(name), 'utf8');
    for (const dep of [...text.matchAll(/^(?:public )?import (\S+)/gm)].map(m => m[1]).filter(d => files.has(d))) {
      const r = compile(dep);
      if (rejectedBy && dep === rejectedBy && r && r.status !== 0) { done.set(name, r); return r; }
      requireFact(r && r.status === 0, `${label}: dependency ${dep} of ${name} was not accepted; failure not attributable to ${root}`);
    }
    const target = path.join(stage, path.relative(modules, files.get(name)));
    fs.mkdirSync(path.dirname(target), {recursive: true}); fs.writeFileSync(target, text);
    const r = execute('lean', ['-R', stage, '-o', target.slice(0, -5) + '.olean', target], {cwd: s.project, out: s.out, label: `${label}-${name}`, env: {...s.env, LEAN_PATH: stage}});
    exits.push({module: name, exit_code: r.status});
    done.set(name, r); return r;
  };
  return {result: compile(root), exits};
}
export function regenerate(name) {
  const c = components[name], lawFile = `src/Uorc/Integrity/${c.law}.lex.tex`, lawModule = leanModule(lawFile);
  const s = scratchProject(`consumer-${name}-`), files = {};
  const keep = (file, text) => { files[file] = text; };
  const compiler = lexleanBinary();
  try {
    // 1. Base: scoped verify of the law module's closure.
    const base = verify(s, [lawFile], 'verify');
    const sources = closureSources(s, base.manifest);
    requireFact(Object.hasOwn(sources, lawFile) && Object.hasOwn(sources, c.mutation.source), 'closure omits the law module or the mutated module');
    requireFact(base.attestation.lexlean.executable_sha256 === compiler.sha256, 'attestation names a different lexlean binary');
    const proof = proofReplay(base, lawFile, fs.readFileSync(path.join(s.project, lawFile), 'utf8'));
    keep('verify.json', base.stdout); keep('verify.stderr', base.stderr);
    keep('build-manifest.json', fs.readFileSync(path.join(base.artifact, 'build-manifest.json'), 'utf8'));
    // 2. Mutant: planted source, lexlean build, stock-lean kernel compile of the unchanged proof.
    const sourceFile = path.join(s.project, c.mutation.source), original = fs.readFileSync(sourceFile);
    const mutant = plant(original.toString('utf8'), c.mutation);
    fs.writeFileSync(sourceFile, mutant);
    const built = execute('lexlean', ['--project', path.join(s.project, 'lexlean.toml'), '--diagnostic-format', 'json', 'build', lawFile], {cwd: s.project, out: s.out, label: 'mutant-build', env: s.env});
    requireFact(built.status === 0, `mutant lexlean build exited ${built.status}: ${built.stdout}${built.stderr}`);
    const mutantBuild = JSON.parse(built.stdout);
    requireFact(mutantBuild.success === true && mutantBuild.artifacts?.length === 1, 'one mutant build artifact required');
    const compiled = compileGenerated(s, path.resolve(s.project, mutantBuild.artifacts[0]), lawModule, 'mutant', c.mutation.rejectedBy);
    requireFact(!c.mutation.rejectedBy || compiled.exits.some(e => e.module === c.mutation.rejectedBy && e.exit_code === 1), `${name}: ${c.mutation.rejectedBy} did not reject the mutant`);
    const reader = leanModule(c.mutation.source), readerRun = compiled.exits.find(e => e.module === reader);
    requireFact(readerRun, `mutated module ${reader} not compiled`);
    const proofLog = compiled.result.stdout + compiled.result.stderr;
    const rejected = [...new Set([...proofLog.matchAll(/native core declaration '([^']+)'/g)].map(m => m[1]))];
    keep('mutant-source.json', JSON.stringify({source_path: c.mutation.source, notation: mutant}, null, 1) + '\n'); keep('mutant-build.json', built.stdout);
    keep('mutant-compile.json', JSON.stringify(compiled.exits, null, 1) + '\n'); keep('mutant-proof.log', proofLog);
    // 3. Restored: original bytes, fresh build root, verify again.
    fs.writeFileSync(sourceFile, original);
    for (const d of ['.lexlean', '.lake']) fs.rmSync(path.join(s.project, d), {recursive: true, force: true});
    const restored = verify(s, [lawFile], 'restored-verify');
    const restoredRuns = [process1(restored.attestation, 'lean', reader, true), process1(restored.attestation, 'lean', lawModule, true), process1(restored.attestation, 'leanchecker', lawModule, false)];
    keep('restored-verify.json', restored.stdout); keep('restored-processes.json', JSON.stringify(restoredRuns, null, 1) + '\n');
    const mutation = {
      fault: c.mutation.fault, sourcePath: c.mutation.source, definition: c.mutation.definition,
      originalSourceSha256: sha(original), mutantSourceSha256: sha(mutant),
      mutantBuildId: mutantBuild.build_id, mutantReaderModule: reader,
      mutantReaderKernelExit: readerRun.exit_code, unchangedProofKernelExit: compiled.result.status, rejectedDeclarations: rejected,
      rejectingModule: c.mutation.rejectedBy ?? lawModule,
      baseBuildId: base.attestation.build_id, restoredBuildId: restored.attestation.build_id,
      restoredBuildIdentical: restored.attestation.build_id === base.attestation.build_id,
      restoredSourceId: restored.attestation.source_id,
      restoredReaderKernelExit: restoredRuns[0].exit_code, restoredProofKernelExit: restoredRuns[1].exit_code, restoredLeancheckerExit: restoredRuns[2].exit_code,
      authoritativeRepositorySourceUntouched: sha(fs.readFileSync(path.join(repo, c.mutation.source))) === sha(original),
    };
    keep('mutation-evidence.json', JSON.stringify(mutation, null, 1) + '\n');
    unchangedInRepository(sources);
    const dir = `validation/integrity/consumer/${name}`;
    const receipt = {
      schema: 'uorc/consumer-component-evidence/1', component: name, scope: c.scope,
      compiler: {lexlean_sha256: base.attestation.lexlean.executable_sha256, lexlean_version: base.attestation.lexlean.version, toolchain: base.attestation.toolchain},
      source_sha256: sources,
      evidence_sha256: Object.fromEntries(Object.entries(files).map(([f, t]) => [`${dir}/${f}`, sha(Buffer.from(t))])),
      // The full attestation (~27 MB) stays out of the repository; its digest binds the audit rows above.
      attestation_sha256: sha(fs.readFileSync(path.join(base.artifact, 'attestation.json'))),
      proof, mutation, original_pinned_sdk_acceptance: false, full_product_acceptance: false,
    };
    // The checker's assertions, before anything in the repository is replaced.
    requireFact(receipt.proof.rows === c.rows, `${name}: ${receipt.proof.rows} declarations, checker expects ${c.rows}`);
    validateProofReplay(s.project, proof);
    for (const [k, v] of [['mutantReaderKernelExit', 0], ['unchangedProofKernelExit', 1], ['restoredProofKernelExit', 0], ['restoredLeancheckerExit', 0], ['restoredBuildIdentical', true]])
      requireFact(mutation[k] === v, `${name}: ${k} is ${mutation[k]}, checker requires ${v}; see ${s.dir}`);
    requireFact(rejected.length > 0, `${name}: kernel rejection names no declaration; see ${s.out}/mutant-${lawModule}.stderr`);
    fs.rmSync(path.join(repo, dir), {recursive: true, force: true}); fs.mkdirSync(path.join(repo, dir), {recursive: true});
    for (const [f, t] of Object.entries(files)) fs.writeFileSync(path.join(repo, dir, f), t);
    fs.writeFileSync(path.join(repo, dir, 'evidence.json'), JSON.stringify(receipt, null, 1) + '\n');
    fs.rmSync(s.dir, {recursive: true, force: true});
    console.log(`${name}: ${proof.rows} rows exact; mutant rejected (${rejected.length} declarations); restored build ${mutation.restoredBuildId}`);
  } catch (error) {
    error.message += ` (scratch kept at ${s.dir})`; throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === script) {
  try {
    const {plan, names} = parseArgs(process.argv.slice(2));
    for (const name of names) {
      const c = components[name];
      if (plan) { const text = fs.readFileSync(path.join(repo, c.mutation.source), 'utf8'); const planted = plant(text, c.mutation); console.log(`${name}: verify src/Uorc/Integrity/${c.law}.lex.tex; plant ${c.mutation.source}#${c.mutation.definition} (${sha(text).slice(0, 12)} -> ${sha(planted).slice(0, 12)})`); }
      else regenerate(name);
    }
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
