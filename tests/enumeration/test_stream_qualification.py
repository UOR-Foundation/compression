"""Negative controls of the finite stream receipt gate, not product acceptance."""
from pathlib import Path
import subprocess
import sys
import unittest
import qualify_stream as q


class StreamQualificationTests(unittest.TestCase):
    def result(self, name='stream_diagnostic_test.py'):
        return {'returncode': 0, 'stdout': '\n'.join(
            q.CONTROL_LINES if name == 'stream_diagnostic_test.py' else q.MUTANT_LINES) + '\n', 'stderr': ''}

    def rejected(self, result, name='stream_diagnostic_test.py'):
        with self.assertRaises(ValueError):
            q.verify_result(name, result)

    def test_complete_controls_and_mutants(self):
        for name in ['stream_diagnostic_test.py', 'stream_mutations.py']:
            q.verify_result(name, self.result(name))

    def test_each_missing_control_is_rejected(self):
        for index in range(len(q.CONTROL_LINES)):
            result = self.result()
            result['stdout'] = '\n'.join(q.CONTROL_LINES[:index] + q.CONTROL_LINES[index+1:]) + '\n'
            self.rejected(result)

    def test_each_duplicate_control_is_rejected(self):
        for line in q.CONTROL_LINES:
            result = self.result(); result['stdout'] += line + '\n'; self.rejected(result)

    def test_reordered_controls_are_rejected(self):
        result = self.result(); result['stdout'] = '\n'.join(reversed(q.CONTROL_LINES)) + '\n'
        self.rejected(result)

    def test_unknown_extra_or_skip_result_is_rejected(self):
        for extra in ['SKIPPED byte specimen', 'PASS optional fixture', 'IGNORED failure']:
            result = self.result(); result['stdout'] += extra + '\n'; self.rejected(result)

    def test_failed_process_or_stderr_is_rejected(self):
        result = self.result(); result['returncode'] = 1; self.rejected(result)
        result = self.result(); result['stderr'] = 'warning'; self.rejected(result)

    def test_unknown_command_is_rejected(self):
        self.rejected(self.result(), 'unregistered.py')

    def test_each_missing_mutant_or_restoration_is_rejected(self):
        for index in range(len(q.MUTANT_LINES)):
            result = self.result('stream_mutations.py')
            result['stdout'] = '\n'.join(q.MUTANT_LINES[:index] + q.MUTANT_LINES[index+1:]) + '\n'
            self.rejected(result, 'stream_mutations.py')

    def test_each_duplicate_mutant_or_restoration_is_rejected(self):
        for line in q.MUTANT_LINES:
            result = self.result('stream_mutations.py'); result['stdout'] += line + '\n'
            self.rejected(result, 'stream_mutations.py')

    def test_wrong_semantic_failure_is_rejected(self):
        for original in q.MUTANT_LINES[1:-1]:
            result = self.result('stream_mutations.py')
            result['stdout'] = result['stdout'].replace(original, 'REJECTED wrong unrelated allocation error')
            self.rejected(result, 'stream_mutations.py')

    def test_reordered_mutants_are_rejected(self):
        result = self.result('stream_mutations.py')
        result['stdout'] = '\n'.join(reversed(q.MUTANT_LINES)) + '\n'
        self.rejected(result, 'stream_mutations.py')

    def test_wrong_wire_cache_is_rejected_before_source_or_git_read(self):
        called = []
        def source(path):
            called.append(path); raise AssertionError('source read must not occur')
        with self.assertRaisesRegex(ValueError, 'wrong authentic Wire cache bytes'):
            q.verify_wire(Path('/missing-repository'), b'{}', source)
        self.assertEqual(called, [])

    def test_optimized_runner_is_rejected(self):
        result = subprocess.run([sys.executable, '-O', str(Path(q.__file__)), '--repository', '.',
                                 '--output', '/tmp/should-not-create-stream-evidence.json'],
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('optimized Python disables stream assertions', result.stderr)

    def test_filtered_runner_is_rejected(self):
        result = subprocess.run([sys.executable, '-B', str(Path(q.__file__)), '--only', 'empty'],
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('error:', result.stderr)


if __name__ == '__main__':
    if not __debug__ or sys.flags.optimize != 0:
        raise SystemExit('optimized Python disables qualification self-tests')
    if sys.argv[1:]:
        raise SystemExit('qualification self-tests accept no filters or arguments')
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(StreamQualificationTests)
    if suite.countTestCases() != 14:
        raise SystemExit('self-test inventory is incomplete')
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if result.skipped or result.expectedFailures or result.unexpectedSuccesses:
        raise SystemExit('self-test results were skipped or weakened')
    raise SystemExit(0 if result.wasSuccessful() else 1)
