"""Negative controls for the diagnostic union harness, not product acceptance."""
import copy
import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest

PATH = Path(__file__).with_name('qualify_current_core.py')
SPEC = importlib.util.spec_from_file_location('qualify_current_core', PATH)
QUALIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(QUALIFY)


def result(lines):
    return {'returncode': 0, 'stdout': '\n'.join(lines) + '\n', 'stderr': ''}


class QualificationControls(unittest.TestCase):
    def test_complete_diagnostic_inventory(self):
        QUALIFY.verify_result('diagnostic_test.py', result(QUALIFY.DIAGNOSTIC_MARKERS))

    def test_diagnostic_omission_is_rejected(self):
        for index in range(len(QUALIFY.DIAGNOSTIC_MARKERS)):
            with self.subTest(index=index), self.assertRaises(ValueError):
                QUALIFY.verify_result('diagnostic_test.py',
                                     result(QUALIFY.DIAGNOSTIC_MARKERS[:index] +
                                            QUALIFY.DIAGNOSTIC_MARKERS[index + 1:]))

    def test_diagnostic_duplicates_are_rejected(self):
        for marker in QUALIFY.DIAGNOSTIC_MARKERS:
            with self.subTest(marker=marker), self.assertRaises(ValueError):
                QUALIFY.verify_result('diagnostic_test.py',
                                     result(QUALIFY.DIAGNOSTIC_MARKERS + [marker]))

    def test_failed_command_or_stderr_is_rejected(self):
        good = result(QUALIFY.DIAGNOSTIC_MARKERS)
        for patch in [{'returncode': 1}, {'stderr': 'failure'}, {'returncode': -9}]:
            with self.subTest(patch=patch), self.assertRaises(ValueError):
                QUALIFY.verify_result('diagnostic_test.py', dict(good, **patch))

    def test_unregistered_command_is_rejected(self):
        with self.assertRaises(ValueError):
            QUALIFY.verify_result('other.py', result([]))

    def mutant_lines(self):
        return ['restored exact-source diagnostic probes passed'] + [
            'REJECTED ' + name + ' ' + message for name, message in QUALIFY.MUTANTS.items()
        ] + ['all four planted defects rejected and restored probes passed']

    def test_complete_intended_mutants_and_restorations(self):
        QUALIFY.verify_result('mutations.py', result(self.mutant_lines()))

    def test_omitted_duplicate_or_reordered_mutant_is_rejected(self):
        lines = self.mutant_lines()
        for changed in [lines[:2] + lines[3:], lines + [lines[1]],
                        lines[:1] + list(reversed(lines[1:5])) + lines[5:]]:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                QUALIFY.verify_result('mutations.py', result(changed))

    def test_wrong_semantic_failure_is_rejected(self):
        for index in range(1, 5):
            lines = self.mutant_lines()
            name = list(QUALIFY.MUTANTS)[index - 1]
            lines[index] = 'REJECTED ' + name + ' unrelated assertion failure'
            with self.subTest(name=name), self.assertRaises(ValueError):
                QUALIFY.verify_result('mutations.py', result(lines))

    def test_missing_or_duplicate_restoration_is_rejected(self):
        lines = self.mutant_lines()
        for changed in [lines[1:], lines[:-1], lines + [lines[0]], lines + [lines[-1]]]:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                QUALIFY.verify_result('mutations.py', result(changed))

    def test_wrong_cache_rejected_before_any_source_read(self):
        def no_source_read(path):
            self.fail('untrusted cache reached source reader')
        with self.assertRaisesRegex(ValueError, 'wrong frozen cache bytes'):
            QUALIFY.verify_cache(b'{}', no_source_read)

    def test_optimized_python_is_rejected(self):
        process = subprocess.run([sys.executable, '-O', '-B', str(PATH), '--help'],
                                 capture_output=True, text=True, check=False)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn('optimized Python disables diagnostic assertions', process.stderr)


if __name__ == '__main__':
    if not __debug__:
        raise SystemExit('optimized Python disables harness assertions')
    if len(sys.argv) != 1:
        raise SystemExit('harness controls accept no filters or arguments')
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(QualificationControls)
    names = [test._testMethodName for test in suite]
    expected = {
        'test_complete_diagnostic_inventory', 'test_diagnostic_omission_is_rejected',
        'test_diagnostic_duplicates_are_rejected', 'test_failed_command_or_stderr_is_rejected',
        'test_unregistered_command_is_rejected', 'test_complete_intended_mutants_and_restorations',
        'test_omitted_duplicate_or_reordered_mutant_is_rejected',
        'test_wrong_semantic_failure_is_rejected', 'test_missing_or_duplicate_restoration_is_rejected',
        'test_wrong_cache_rejected_before_any_source_read', 'test_optimized_python_is_rejected',
    }
    if len(names) != len(expected) or set(names) != expected:
        raise SystemExit('missing, duplicated or extra harness controls')
    outcome = unittest.TextTestRunner(verbosity=2).run(suite)
    raise SystemExit(0 if outcome.wasSuccessful() and outcome.testsRun == len(expected)
                     and not outcome.skipped and not outcome.expectedFailures
                     and not outcome.unexpectedSuccesses else 1)
