#!/usr/bin/env python3
"""Execute the source-owned product controls; this is a component diagnostic, not vv."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MODULES = ('Session', 'Evidence', 'StreamProtocol', 'Api', 'InputModel',
           'Control', 'Workspace', 'Presentation', 'Scenarios', 'Storage',
           'ResourceIO', 'Selection', 'HostContract')
HARNESS = '''import Uorc.Uorc.Product.Scenarios
open Uorc.Uorc.Product

def main : IO Unit := do
  let mut failures : Nat := 0
  let mut total : Nat := 0
  for c in Scenarios.cases do
    total := total + 1
    if !(Scenarios.runCase c) then
      failures := failures + 1
      IO.println s!"FAIL {c.id}"
  for o in Scenarios.observations do
    total := total + 1
    if !o.passed then
      failures := failures + 1
      IO.println s!"FAIL {o.id}"
  IO.println s!"{total - failures}/{total} product control and presentation scenarios passed"
  if total == 0 then throw (IO.userError "empty scenario inventory")
  if failures != 0 then throw (IO.userError "product scenarios failed")
'''


def sha(data):
    return hashlib.sha256(data).hexdigest()


def xtask(task, text):
    """LexLean SPEC 17.21.1: 1.2 sources are notation; convert only via the SDK."""
    subprocess.run(['cargo', 'build', '--locked', '-q', '-p', 'xtask'], cwd=ROOT, check=True)
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
    return subprocess.run([str(target / 'debug/xtask'), task], input=text, capture_output=True,
                          text=True, check=True).stdout


def read_module(path):
    text = xtask('data-form', path.read_text())
    return text, json.loads(re.search(r'\\semanticdata\{(.*)\}', text).group(1))


def write_module(path, text, data):
    path.write_text(xtask('notation', re.sub(r'\\semanticdata\{.*\}', lambda _: '\\semanticdata{' +
                          json.dumps(data, sort_keys=True, separators=(',', ':')) + '}', text)))


def run(args, cwd, env=None, expected=0):
    result = subprocess.run([str(a) for a in args], cwd=cwd, env=env,
                            capture_output=True, text=True)
    if expected is not None and result.returncode != expected:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lexlean', default='lexlean')
    parser.add_argument('--lean', default='lean')
    parser.add_argument('--diagnostic-compatibility', action='store_true',
                        help='In the temporary language-1.1 projection only, omit existing '
                             'executable metadata rejected by the newer compiler. No terms change.')
    parser.add_argument('--output', type=Path, default=ROOT / 'validation/product/evidence.json')
    args = parser.parse_args()
    compiler = shutil.which(args.lexlean) or args.lexlean
    lean = shutil.which(args.lean) or args.lean
    run(['python3', ROOT / 'scripts/product-host-projection.py'], ROOT)
    probe_source = (ROOT / 'tests/product-control/user-notice-forgery.lex.tex').read_bytes()
    sources = {name: (ROOT / f'src/Uorc/Product/{name}.lex.tex').read_bytes()
               for name in MODULES}
    evidence = {
        'schema': 'uorc/product-control-diagnostic/1',
        'scope': 'Source linking, kernel verification, exact axiom audit and generated Lean VM '
                 'execution of the component model and its planted defects. Synthetic Report '
                 'fixtures do not establish archive, native-library, browser or release acceptance.',
        'pinned_sdk_acceptance': False,
        'compiler': run([compiler, '--version'], ROOT).stdout.strip(),
        'lean': run([lean, '--version'], ROOT).stdout.strip(),
        'source_sha256': {name: sha(data) for name, data in sources.items()},
        'diagnostic_annotation_omissions': [],
        'mutations': [],
        'boundary_negatives': [],
        'gate_sha256': sha(Path(__file__).read_bytes()),
        'harness_sha256': sha(HARNESS.encode()),
        'boundary_probe_sha256': sha(probe_source),
        'host_projection_sha256': sha((ROOT / 'validation/product/host-contract.json').read_bytes()),
        'host_projection_gate_sha256': sha((ROOT / 'scripts/product-host-projection.py').read_bytes()),
    }
    with tempfile.TemporaryDirectory(prefix='uorc-product-') as temp:
        project = Path(temp)
        src = project / 'src/Uorc/Product'
        src.mkdir(parents=True)
        for name, data in sources.items():
            p = src / f'{name}.lex.tex'
            p.write_bytes(data)
            if args.diagnostic_compatibility:
                text, module = read_module(p)
                for declaration in module['declarations']:
                    if declaration.pop('executable', None) is not None:
                        evidence['diagnostic_annotation_omissions'].append(name + '.' + declaration['name'])
                write_module(p, text, module)
        config = (ROOT / 'lexlean.toml').read_text().replace('uorc-authority', 'product-control-diagnostic')
        config = re.sub(r'entrypoints = .*', 'entrypoints = ["src/Uorc/Product/Scenarios.lex.tex"]', config)
        (project / 'lexlean.toml').write_text(config)
        for name in ('lakefile.toml', 'lean-toolchain'):
            shutil.copyfile(ROOT / name, project / name)
        # Initialize Lake's own manifest before the compiler binds the workspace files.
        lake = Path(lean).with_name('lake')
        if lake.is_file():
            run([lake, 'update'], project)
        run([compiler, 'lock'], project)
        originals = {name: (src / f'{name}.lex.tex').read_bytes() for name in MODULES}

        def verify(expected_owner=None):
            result = run([compiler, 'verify', '--all'], project, expected=None)
            message = result.stdout + result.stderr
            if expected_owner:
                if result.returncode == 0:
                    raise RuntimeError('Planted defect was accepted: ' + message)
                built = json.loads(run([compiler, '--diagnostic-format', 'json', 'build', '--all'], project).stdout)
                location = re.search(r'generated location: .*?/lean-src/(.*?\.lean):(\d+):', message)
                if not location:
                    raise RuntimeError('No generated failing declaration location: ' + message)
                generated = project / '.lexlean/build' / built['build_id'] / 'modules' / location.group(1)
                lines = generated.read_text().splitlines()[:int(location.group(2))]
                owners = [re.match(r'public theorem (\w+)', line).group(1)
                          for line in lines if re.match(r'public theorem (\w+)', line)]
                if not owners or owners[-1] != expected_owner:
                    raise RuntimeError('Planted defect failed at another declaration: ' + message)
                return {'exit': result.returncode, 'owner': owners[-1], 'diagnostic': 'LLV7002'}
            if result.returncode:
                raise RuntimeError(message)
            match = re.search(r'attestation ([a-f0-9]{64})', message)
            if not match:
                raise RuntimeError('No complete verification attestation: ' + message)
            path = project / '.lexlean/verified' / match.group(1) / 'attestation.json'
            attestation = json.loads(path.read_text())
            if any(d['result'] != 'ok' for d in attestation['declarations']):
                raise RuntimeError('An audited declaration did not pass')
            return attestation

        def execute(attestation, expected_failure=None):
            generated = project / '.lexlean/build' / attestation['build_id'] / 'modules'
            runtime = project / ('runtime-' + attestation['attestation_id'])
            runtime.mkdir()
            files = list(generated.rglob('*.lean'))
            completed = set()
            env = dict(os.environ, LEAN_PATH=str(runtime))
            while files:
                ready = next((p for p in files if set(re.findall(r'public import (Uorc\.[\w.]+)',
                                                                                p.read_text())) <= completed), None)
                if ready is None:
                    raise RuntimeError('Generated dependency closure is incomplete')
                relative = ready.relative_to(generated)
                target = (runtime / relative).with_suffix('.olean')
                target.parent.mkdir(parents=True, exist_ok=True)
                run([lean, '-R', generated, '-o', target, ready], project, env)
                completed.add('.'.join(relative.with_suffix('').parts))
                files.remove(ready)
            harness = project / 'RunProductControls.lean'
            harness.write_text(HARNESS)
            result = run([lean, '--run', harness], project, env, expected=None)
            message = result.stdout + result.stderr
            if expected_failure:
                if result.returncode == 0 or ('FAIL ' + expected_failure) not in message:
                    raise RuntimeError('Runtime mutation did not fail at its owner: ' + message)
            elif result.returncode:
                raise RuntimeError(message)
            return {'exit': result.returncode, 'transcript': message.strip()}

        probe = src / 'UserNoticeForgery.lex.tex'
        probe.write_bytes(probe_source)
        rejected = run([compiler, 'check', '--all'], project, expected=None)
        diagnostic = rejected.stdout + rejected.stderr
        if rejected.returncode == 0 or 'LLT4001' not in diagnostic or 'UserNoticeForgery' not in diagnostic:
            raise RuntimeError('User intent admitted a forged runtime report: ' + diagnostic)
        probe.unlink()
        evidence['boundary_negatives'].append({'id': 'user-notice-forgery',
                                              'exit': rejected.returncode, 'diagnostic': 'LLT4001'})
        print('forged runtime report rejected at the user-intent type boundary', flush=True)
        before = verify()
        evidence['restored_control'] = execute(before)
        print(evidence['restored_control']['transcript'], flush=True)
        path = src / 'Control.lex.tex'
        text, module = read_module(path)
        declaration = next(d for d in module['declarations'] if d['name'] == 'completed')
        declaration['body']['arguments'][-1] = {'kind': 'bool', 'value': True}
        write_module(path, text, module)
        evidence['mutations'].append({'id': 'stale-guard-bypass',
                                     'verification': verify('completion_guard_binding')})
        path.write_bytes(originals['Control'])
        print('stale-guard-bypass rejected at completion_guard_binding', flush=True)
        path = src / 'Api.lex.tex'
        text, module = read_module(path)
        declaration = next(d for d in module['declarations'] if d['name'] == 'evidenceView')
        declaration['body']['arguments'][1] = {'kind': 'bool', 'value': True}
        write_module(path, text, module)
        mutated = verify()
        evidence['mutations'].append({'id': 'invented-original-availability',
                                     'runtime': execute(mutated, 'decompression-does-not-claim-equality')})
        path.write_bytes(originals['Api'])
        print('invented-original-availability rejected by generated runtime observation', flush=True)
        path = src / 'Storage.lex.tex'
        text, module = read_module(path)
        declaration = next(d for d in module['declarations'] if d['name'] == 'fits')
        declaration['body'] = {'kind': 'bool', 'value': True}
        write_module(path, text, module)
        mutated = verify()
        evidence['mutations'].append({'id': 'storage-quota-bypass',
                                     'runtime': execute(mutated, 'storage-admission-achieved-failure-transfer')})
        path.write_bytes(originals['Storage'])
        print('storage-quota-bypass rejected by storage runtime owner', flush=True)
        path = src / 'ResourceIO.lex.tex'
        text, module = read_module(path)
        declaration = next(d for d in module['declarations'] if d['name'] == 'cancelledWriteAccepted')
        declaration['body'] = declaration['body']['branches'][1]['body']
        write_module(path, text, module)
        mutated = verify()
        evidence['mutations'].append({'id': 'cancellation-before-write-ack',
                                     'runtime': execute(mutated, 'double-cancel-waits-for-achieved-write')})
        path.write_bytes(originals['ResourceIO'])
        print('cancellation-before-write-ack rejected by bounded I/O runtime owner', flush=True)
        path = src / 'ResourceIO.lex.tex'
        text, module = read_module(path)
        declaration = next(d for d in module['declarations'] if d['name'] == 'cleanupPreparation')
        declaration['body'] = declaration['body']['then_value']
        write_module(path, text, module)
        mutated = verify()
        evidence['mutations'].append({'id': 'delete-spool-before-read-receipt',
                                     'runtime': execute(mutated, 'cancelled-read-blocks-spool-deletion')})
        path.write_bytes(originals['ResourceIO'])
        print('delete-spool-before-read-receipt rejected by the read custody owner', flush=True)
        if any((src / f'{name}.lex.tex').read_bytes() != data for name, data in originals.items()):
            raise RuntimeError('A planted defect was not restored byte-for-byte')
        restored = verify()
        # A clean compilation directory prevents retained VM artifacts from satisfying the restored run.
        shutil.rmtree(project / ('runtime-' + restored['attestation_id']), ignore_errors=True)
        evidence['restored_control'] = execute(restored)
        evidence['attestation_id'] = restored['attestation_id']
        evidence['build_id'] = restored['build_id']
        evidence['audited_declarations'] = len(restored['declarations'])
        evidence['observed_axioms'] = sorted({a for d in restored['declarations'] for a in d['observed']})
        evidence['project_source_id'] = restored['source_id']
        evidence['semantic_id'] = restored['semantic_id']
        evidence['restored_byte_for_byte'] = True
        counts = re.search(r'(\d+)/(\d+) product control', evidence['restored_control']['transcript'])
        if not counts or int(counts[1]) != int(counts[2]) or int(counts[2]) == 0:
            raise RuntimeError('Incomplete runtime inventory')
        evidence['runtime_passed'] = int(counts[1])
        evidence['runtime_total'] = int(counts[2])
    if any((ROOT / f'src/Uorc/Product/{name}.lex.tex').read_bytes() != data
           for name, data in sources.items()):
        raise RuntimeError('Repository source changed during the diagnostic')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(evidence, indent=2, sort_keys=True) + '\n')
    print(evidence['restored_control']['transcript'], flush=True)
    print('Evidence written; pinned-SDK, native and browser qualification remain separate.', flush=True)


if __name__ == '__main__':
    main()
