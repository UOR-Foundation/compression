#!/usr/bin/env python3
"""Plant annotation loss and make the real official-read audit reject it."""

import argparse
import copy
import json
from pathlib import Path
import unittest

from audit import assert_same_module
from prepare import official


class AnnotationControls(unittest.TestCase):
    def setUp(self):
        self.source = self.source_path.read_text()
        self.module = json.loads(official(self.reader, "read", self.source))

    def assert_rejected(self, field, predicate, change):
        planted = copy.deepcopy(self.module)
        declaration = next(d for d in planted["declarations"]
                           if field in d and predicate(d[field]))
        change(declaration)
        rendered = official(self.reader, "write", json.dumps({
            "source": self.source, "value": planted,
        }))
        observed = json.loads(official(self.reader, "read", rendered))
        self.assertEqual(observed, planted, "official writer must retain the planted defect")
        with self.assertRaisesRegex(ValueError, "changed beyond root schema"):
            assert_same_module(self.module, observed, "planted annotation loss")

    def test_executable_annotation_loss_is_rejected(self):
        self.assert_rejected("executable", lambda v: v is True,
                             lambda d: d.pop("executable"))

    def test_exact_axiom_annotation_change_is_rejected(self):
        self.assert_rejected("axioms", bool, lambda d: d.update(axioms=[]))

    def test_production_annotation_loss_is_rejected(self):
        self.assert_rejected("production", bool, lambda d: d.pop("production"))

    def test_unchanged_module_passes(self):
        assert_same_module(self.module, self.module, "unchanged source")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reader", required=True, type=Path)
    parser.add_argument("--source", required=True, type=Path,
                        help="notation module with executable, axiom, and production annotations")
    options = parser.parse_args()
    AnnotationControls.reader = options.reader.resolve()
    AnnotationControls.source_path = options.source.resolve()
    unittest.main(argv=[__file__])
