"""Independent negative controls for the cheap, non-accepting CI preflight."""
import importlib.util
import errno
import os
import re
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
from types import SimpleNamespace

REPO = Path(__file__).resolve().parents[2]
SCRIPT = REPO / 'scripts/sdk-source-format-preflight.py'
SPEC = importlib.util.spec_from_file_location('sdk_preflight', SCRIPT)
PREFLIGHT = importlib.util.module_from_spec(SPEC)
# The implementation does not exist when these representative defects are planted.
SPEC.loader.exec_module(PREFLIGHT)
RUN_SPEC = importlib.util.spec_from_file_location('sdk_preflight_tests', Path(__file__).parent / 'run.py')
RUNNER = importlib.util.module_from_spec(RUN_SPEC)
RUN_SPEC.loader.exec_module(RUNNER)
GUARD = """      - name: Reject known source-format incompatibility
        shell: bash
        run: |
          set -euo pipefail
          python3 -B tests/sdk-preflight/run.py
          python3 -B scripts/sdk-source-format-preflight.py
"""

HEADER = '\\begin{lexlean}{Example.Main}\n\\title{Parameters}\n\\begin{semanticmodule}\n'
FOOTER = '\n\\end{semanticmodule}\n\\end{lexlean}\n'
NATIVE = 'inductive{\n  name: Example,\n  type_parameters: [],\n  parameters: [],\n  constructors: []\n}'
DATA = '\\semanticdata{{"declarations":[],"spec":"lexlean/semantic-module/1"}}'


class SourceFormatTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.put('lexlean.toml', 'spec = "lexlean/project/1"\nlanguage = "1.1"\nsource_roots = ["src"]\n')
        self.put('src/Main.lex.tex', HEADER + DATA + FOOTER)

    def put(self, name, content):
        file = self.root / name
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(content)

    def check(self, **kwargs):
        return PREFLIGHT.inspect_project(self.root, **kwargs)

    def test_current_failure_has_exact_relative_path_line_and_reason(self):
        source = (Path(__file__).parent / 'fixtures/current-rejection.lex.tex').read_text()
        (self.root / 'src/Main.lex.tex').unlink()
        self.put('src/Uorc/Wire/ProgramParseScenarios.lex.tex', source)
        result = self.check()
        self.assertEqual(result.code, 'UCF001')
        self.assertEqual(result.path, 'src/Uorc/Wire/ProgramParseScenarios.lex.tex')
        self.assertEqual(result.line, 15)
        self.assertIn(r'language 1.1 requires \semanticdata', result.reason)
        self.assertIn('inductive{', result.reason)

    def test_valid_language_11_data_is_delegated(self):
        self.assertIsNone(self.check())

    def test_language_12_and_unknown_language_delegate_native_source(self):
        self.put('src/Main.lex.tex', HEADER + NATIVE + FOOTER)
        for language in ['1.2', '1.0', '9.9']:
            with self.subTest(language=language):
                self.put('lexlean.toml', f'spec = "lexlean/project/1"\nlanguage = "{language}"\nsource_roots = ["src"]\n')
                self.assertIsNone(self.check())

    def test_known_native_starts_only_after_unambiguous_header(self):
        for declaration in ['inductive', 'structure', 'definition']:
            with self.subTest(declaration=declaration):
                self.put('src/Main.lex.tex', HEADER + declaration + '{\n}' + FOOTER)
                self.assertEqual(self.check().code, 'UCF001')

    def test_comments_strings_and_expository_text_do_not_trigger(self):
        examples = [
            '% ' + HEADER + NATIVE + FOOTER,
            '\\text{' + HEADER + NATIVE + FOOTER + '}',
            HEADER + '% inductive{\n' + DATA + FOOTER,
            HEADER + '// inductive{\n' + DATA + FOOTER,
            HEADER + '"inductive{"' + FOOTER,
            HEADER + DATA.replace('[]', '["inductive{", "\\\\begin{semanticmodule}"]') + FOOTER,
            HEADER.replace('Parameters', 'inductive') + DATA + FOOTER,
            HEADER.replace('\\title{Parameters}\n', '% example\n\\title{Parameters}\n') + NATIVE + FOOTER,
        ]
        for source in examples:
            with self.subTest(source=source[:100]):
                self.put('src/Main.lex.tex', source)
                self.assertIsNone(self.check())

    def test_malformed_or_unrecognized_input_still_reaches_compiler(self):
        examples = [
            HEADER + 'inductive Example' + FOOTER,
            HEADER + 'inductive' + FOOTER,
            HEADER + 'inductiveName{' + FOOTER,
            HEADER + 'theorem{' + FOOTER,
            HEADER.replace('\\title{Parameters}\n', '') + NATIVE + FOOTER,
            'prefix\n' + HEADER + NATIVE + FOOTER,
            HEADER.replace('{Example.Main}', '{bad module}') + NATIVE + FOOTER,
            HEADER.replace('\n', '\r\n') + NATIVE + FOOTER,
        ]
        for source in examples:
            with self.subTest(source=source[:100]):
                self.put('src/Main.lex.tex', source)
                self.assertIsNone(self.check())
        for config in ['language = [', 'language = "1.1"\n', 'spec = "other"\nlanguage = "1.1"\nsource_roots = ["src"]\n']:
            self.put('lexlean.toml', config)
            self.assertIsNone(self.check())

    def test_inspection_budgets_delegate_instead_of_rejecting_product_size(self):
        self.put('src/Main.lex.tex', HEADER + NATIVE + FOOTER)
        for budgets in [dict(max_entries=0), dict(max_bytes=1), dict(prefix_bytes=4), dict(max_depth=0)]:
            with self.subTest(budgets=budgets):
                self.assertIsNone(self.check(**budgets))
        self.put('src/Main.lex.tex', HEADER + DATA + FOOTER + (' ' * 100_000))
        self.assertIsNone(self.check(prefix_bytes=4096))

    def test_directory_iteration_and_prefix_reads_are_independently_bounded(self):
        seen = []

        class Entries:
            def __enter__(self):
                return self

            def __exit__(self, *args):
                pass

            def __iter__(self):
                for number in range(1000000):
                    seen.append(number)
                    if len(seen) > 5:
                        raise AssertionError('unbounded directory iteration')
                    yield SimpleNamespace(name=f'{number}.txt')

        with mock.patch.object(PREFLIGHT.os, 'scandir', return_value=Entries()):
            self.assertIsNone(self.check(max_entries=4))
        self.assertEqual(len(seen), 5)
        self.put('src/Main.lex.tex', HEADER + DATA + FOOTER + (' ' * 100000))
        actual_read = os.read
        reads = []

        def counted_read(descriptor, amount):
            block = actual_read(descriptor, amount)
            reads.append(len(block))
            return block

        with mock.patch.object(PREFLIGHT.os, 'read', side_effect=counted_read):
            self.assertIsNone(self.check(prefix_bytes=1024))
        self.assertLessEqual(sum(reads), (self.root / 'lexlean.toml').stat().st_size + 1024)

    def test_symlinks_and_special_files_are_not_followed_or_read(self):
        target = self.root / 'src/Main.lex.tex'
        target.unlink()
        os.symlink('/does/not/exist', target)
        self.assertEqual(self.check().code, 'UCF002')
        target.unlink()
        os.mkfifo(target)
        self.assertEqual(self.check().code, 'UCF002')
        target.unlink()
        os.symlink('/does/not/exist', self.root / 'src/adjacent')
        self.assertEqual(self.check().code, 'UCF002')

    def test_config_and_source_root_symlink_ancestors_are_rejected(self):
        (self.root / 'lexlean.toml').unlink()
        os.symlink('/does/not/exist', self.root / 'lexlean.toml')
        self.assertEqual(self.check().code, 'UCF002')
        (self.root / 'lexlean.toml').unlink()
        self.put('lexlean.toml', 'spec = "lexlean/project/1"\nlanguage = "1.1"\nsource_roots = ["link/nested"]\n')
        os.symlink('/does/not/exist', self.root / 'link')
        self.assertEqual(self.check().code, 'UCF002')

    def test_filesystem_resource_errors_delegate_without_false_format_claim(self):
        for number in [errno.EACCES, errno.EMFILE, errno.EIO, errno.ENOENT]:
            with self.subTest(errno=number):
                with mock.patch.object(PREFLIGHT.os, 'open', side_effect=OSError(number, 'injected')):
                    self.assertIsNone(self.check())

    def test_escaping_or_noncanonical_source_paths_are_rejected(self):
        for source_root in ['../outside', '/tmp/outside', 'src/../src', './src', 'src//nested', 'src\\nested']:
            with self.subTest(source_root=source_root):
                self.put('lexlean.toml', f'spec = "lexlean/project/1"\nlanguage = "1.1"\nsource_roots = [\'{source_root}\']\n')
                self.assertEqual(self.check().code, 'UCF002')

    def test_cli_negative_stops_before_docker_and_delegation_continues(self):
        fake = self.root / 'bin'
        fake.mkdir()
        docker = fake / 'docker'
        docker.write_text('#!/bin/sh\necho reached > docker-called\nexit 73\n')
        docker.chmod(0o755)
        env = dict(os.environ, PATH=f'{fake}:{os.environ["PATH"]}')
        command = f'"{sys.executable}" -B "{SCRIPT}"; docker pull unchanged-sdk'
        self.put('src/Main.lex.tex', HEADER + NATIVE + FOOTER)
        rejected = subprocess.run(['bash', '-euc', command], cwd=self.root, env=env, capture_output=True, text=True)
        self.assertEqual(rejected.returncode, 1, rejected.stderr)
        self.assertIn('src/Main.lex.tex:4: UCF001:', rejected.stderr)
        self.assertFalse((self.root / 'docker-called').exists())
        self.put('src/Main.lex.tex', HEADER + DATA + FOOTER)
        delegated = subprocess.run(['bash', '-euc', command], cwd=self.root, env=env, capture_output=True, text=True)
        self.assertEqual(delegated.returncode, 73, delegated.stderr)
        self.assertTrue((self.root / 'docker-called').exists())
        self.assertIn('full SDK checks remain required', delegated.stdout)

    def audit_workflow(self, workflow):
        guard = 'python3 -B scripts/sdk-source-format-preflight.py'
        self.assertEqual(workflow.count(guard), 3)
        self.assertEqual(workflow.count('python3 -B tests/sdk-preflight/run.py'), 3)
        blocks = re.findall(r'^      - name: Reject known source-format incompatibility\n(?:(?!      - ).*\n)*', workflow, re.MULTILINE)
        self.assertEqual(blocks, [GUARD, GUARD, GUARD])
        regenerate, rest = workflow.split('  regenerate:\n')[1].split('  offline-setup:\n')
        setup, offline = rest.split('  offline:\n')
        for job in [regenerate, setup, offline]:
            self.assertLess(job.index('persist-credentials: false'), job.index(guard))
            self.assertLess(job.index('python3 -B tests/sdk-preflight/run.py'), job.index(guard))
            self.assertLess(job.index(guard), job.index('docker pull "$image"'))
            preflight_step = job[job.index('- name: Reject known source-format incompatibility'):job.index(guard) + len(guard)]
            self.assertIn('set -euo pipefail', preflight_step)
            self.assertNotIn('if:', preflight_step)
            self.assertNotIn('continue-on-error', job)
            self.assertNotIn('|| true', job)
        self.assertIn('lexlean fmt --check --all; just model-write; node scripts/offline-validation.mjs export-projections', workflow)
        self.assertIn('"$image" node scripts/offline-validation.mjs acquire .tmp/offline-acquisition', setup)
        self.assertIn('"$image" node scripts/offline-validation.mjs isolated "${{ matrix.root }}" .tmp/offline-acquisition', offline)
        self.assertIn('needs: offline-setup', offline)
        runner = (REPO / 'scripts/offline-validation.mjs').read_text()
        self.assertIn("OFFLINE_RECIPES = ['lexlean-authority', 'model']", runner)
        self.assertIn("root: [first, second-distinct-root]", workflow)
        self.assertIn('name: offline formal construction / ${{ matrix.root }}', workflow)
        self.assertIn('node scripts/offline-validation.mjs compare-offline .prism/offline/first .prism/offline/second-distinct-root', workflow)
        self.assertIn('name: review-only generated projections', workflow)
        self.assertEqual([line.strip() for line in workflow.splitlines() if 'if:' in line], ['if: always()', 'if: always()'])


    def test_acceptance_legs_run_every_vv_recipe_exactly_once(self):
        just = (REPO / 'Justfile').read_text()
        vv = next(line for line in just.splitlines() if line.startswith('vv:')).split()[1:]
        workflow = (REPO / '.github/workflows/bootstrap.yml').read_text()
        rows = re.findall(r'- \{ leg: ([a-z-]+), runner: (ubuntu-24\.04(?:-arm)?), recipes: "([^"]+)" \}', workflow)
        self.assertTrue(rows)
        normative = [recipe for leg, runner, recipes in rows if runner == 'ubuntu-24.04' for recipe in recipes.split()]
        self.assertEqual(sorted(normative), sorted(vv))
        self.assertEqual(len(normative), len(set(normative)))
        self.assertIn('exec just $LEG_RECIPES', workflow)
        self.assertIn('needs: template-gate', workflow)
        self.assertIn('run: test "$LEGS" = success', workflow)

    def test_workflow_runs_tests_and_preflight_before_every_project_pull(self):
        self.audit_workflow((REPO / '.github/workflows/offline-validation.yml').read_text())

    def test_workflow_audit_rejects_commented_or_conditional_guards(self):
        workflow = (REPO / '.github/workflows/offline-validation.yml').read_text()
        mutations = [
            workflow.replace('python3 -B scripts/sdk-source-format-preflight.py', '# python3 -B scripts/sdk-source-format-preflight.py'),
            workflow.replace('python3 -B tests/sdk-preflight/run.py', '# python3 -B tests/sdk-preflight/run.py'),
            workflow.replace('        shell: bash', '        if: false\n        shell: bash'),
            workflow.replace(GUARD, GUARD + '        if: false\n'),
            workflow.replace('python3 -B scripts/sdk-source-format-preflight.py', 'python3 -B scripts/sdk-source-format-preflight.py || true'),
            workflow.replace('python3 -B tests/sdk-preflight/run.py', 'python3 -B tests/sdk-preflight/run.py --filter current'),
            workflow.replace(GUARD, ''),
        ]
        for mutated in mutations:
            with self.subTest(mutation=mutations.index(mutated)):
                with self.assertRaises(AssertionError):
                    self.audit_workflow(mutated)

    def test_test_runner_rejects_missing_duplicate_and_skipped_results(self):
        with self.assertRaises(ValueError):
            RUNNER.require_inventory(unittest.TestSuite())
        case = unittest.FunctionTestCase(lambda: None)
        with self.assertRaises(ValueError):
            RUNNER.require_inventory(unittest.TestSuite([case, case]))
        result = dict(testsRun=len(RUNNER.EXPECTED), skipped=[], expectedFailures=[],
                      unexpectedSuccesses=[], wasSuccessful=lambda: True)
        self.assertTrue(RUNNER.complete(SimpleNamespace(**result)))
        for changed in [dict(testsRun=0), dict(testsRun=len(RUNNER.EXPECTED) - 1),
                        dict(skipped=[('case', 'reason')]), dict(expectedFailures=['case']),
                        dict(unexpectedSuccesses=['case']), dict(wasSuccessful=lambda: False)]:
            self.assertFalse(RUNNER.complete(SimpleNamespace(**(result | changed))))


if __name__ == '__main__':
    unittest.main()
