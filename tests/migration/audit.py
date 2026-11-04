#!/usr/bin/env python3
"""Compare original and candidate modules through the official SDK reader."""

import argparse
import json
from pathlib import Path
import tomllib

from prepare import digest, official, policy_inventory


def assert_same_module(original, candidate, label):
    expected = json.loads(json.dumps(original))
    schema = expected["spec"]
    if schema in ("lexlean/semantic-module/1", "lexlean/core-module/1"):
        expected["spec"] = schema[:-1] + "2"
    if candidate != expected:
        raise ValueError(f"{label}: parsed module changed beyond root schema")
    if policy_inventory(original) != policy_inventory(candidate):
        raise ValueError(f"{label}: declaration annotations changed")


def module_paths(root, configuration):
    return {
        path.relative_to(root)
        for source in configuration["source_roots"]
        for path in (root / source).rglob("*.lex.tex")
    }


def audit(source, candidate, reader):
    source, candidate, reader = (p.resolve() for p in (source, candidate, reader))
    original_config = tomllib.loads((source / "lexlean.toml").read_text())
    candidate_config = tomllib.loads((candidate / "lexlean.toml").read_text())
    expected_config = dict(original_config, language="1.2")
    if original_config["language"] != "1.1" or candidate_config != expected_config:
        raise ValueError("project configuration changed beyond language 1.1 to 1.2")
    paths = module_paths(source, original_config)
    if paths != module_paths(candidate, candidate_config):
        raise ValueError("candidate source module inventory differs")
    rows = []
    for relative in sorted(paths):
        original = (source / relative).read_text()
        transformed = (candidate / relative).read_text()
        before = json.loads(official(reader, "read", original))
        after = json.loads(official(reader, "read", transformed))
        assert_same_module(before, after, relative)
        restored = official(reader, "write", json.dumps({
            "source": transformed, "value": after,
        }))
        if restored != transformed:
            raise ValueError(f"{relative}: official notation write is not byte-idempotent")
        rows.append({
            "path": relative.as_posix(),
            "original_sha256": digest(original.encode()),
            "candidate_sha256": digest(transformed.encode()),
            "declarations": len(before["declarations"]),
        })
    return {"full_ir_and_annotation_audit": "passed", "modules": rows,
            "scope": "source equivalence; not compiler, installed SDK, or release acceptance"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--reader", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    options = parser.parse_args()
    result = audit(options.source, options.candidate, options.reader)
    options.report.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"full_ir_and_annotation_audit": "passed",
                      "modules": len(result["modules"])}))


if __name__ == "__main__":
    main()
