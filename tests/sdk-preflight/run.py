#!/usr/bin/env python3
"""Run the complete operational preflight controls, rejecting omission/skips."""
from pathlib import Path
import sys
import unittest

EXPECTED = {
    'test_cli_negative_stops_before_docker_and_delegation_continues',
    'test_comments_strings_and_expository_text_do_not_trigger',
    'test_config_and_source_root_symlink_ancestors_are_rejected',
    'test_current_failure_has_exact_relative_path_line_and_reason',
    'test_directory_iteration_and_prefix_reads_are_independently_bounded',
    'test_escaping_or_noncanonical_source_paths_are_rejected',
    'test_filesystem_resource_errors_delegate_without_false_format_claim',
    'test_inspection_budgets_delegate_instead_of_rejecting_product_size',
    'test_known_native_starts_only_after_unambiguous_header',
    'test_language_12_and_unknown_language_delegate_native_source',
    'test_malformed_or_unrecognized_input_still_reaches_compiler',
    'test_symlinks_and_special_files_are_not_followed_or_read',
    'test_test_runner_rejects_missing_duplicate_and_skipped_results',
    'test_valid_language_11_data_is_delegated',
    'test_workflow_audit_rejects_commented_or_conditional_guards',
    'test_workflow_runs_tests_and_preflight_before_every_project_pull',
}


def _tests(suite):
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from _tests(item)
        else:
            yield item


def require_inventory(suite):
    names = [test.id().rsplit('.', 1)[-1] for test in _tests(suite)]
    if len(names) != len(EXPECTED) or set(names) != EXPECTED:
        raise ValueError('preflight test inventory is missing, duplicated, or unexpected')


def complete(result):
    return (result.wasSuccessful() and result.testsRun == len(EXPECTED)
            and not result.skipped and not result.expectedFailures
            and not result.unexpectedSuccesses)


def main():
    if len(sys.argv) != 1:
        print('preflight tests do not accept filters or arguments', file=sys.stderr)
        return 1
    suite = unittest.defaultTestLoader.discover(str(Path(__file__).parent), pattern='test_*.py')
    try:
        require_inventory(suite)
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1
    return 0 if complete(unittest.TextTestRunner(verbosity=2).run(suite)) else 1


if __name__ == '__main__':
    sys.exit(main())
