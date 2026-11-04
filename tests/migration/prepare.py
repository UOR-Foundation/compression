#!/usr/bin/env python3
"""Prepare and audit a separate language-1.2 source candidate with the SDK.

This is source-readiness tooling. It does not change the source checkout, select
an SDK, qualify production export, or replace the repository acceptance gates.
The supplied reader must implement the official LexLean authoring API; no
LexLean syntax is parsed here. Every module is read and written by that API.
"""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tomllib


def digest(data):
    return hashlib.sha256(data).hexdigest()


def official(reader, action, value):
    result = subprocess.run(
        [str(reader), action], input=value, text=True, capture_output=True,
        timeout=60, check=False,
    )
    if result.returncode:
        raise RuntimeError(f"official {action} failed: {result.stderr}")
    return result.stdout


def policy_inventory(module):
    """Describe annotations without treating absent fields as false/empty."""
    fields = ("axioms", "policy", "executable", "production")
    return [
        {
            "name": declaration.get("name"),
            "kind": declaration.get("kind"),
            "declaration_sha256": digest(json.dumps(
                declaration, sort_keys=True, separators=(",", ":"),
                ensure_ascii=False,
            ).encode()),
            "annotations": {
                field: {"present": field in declaration,
                        "value": declaration.get(field)}
                for field in fields
            },
        }
        for declaration in module["declarations"]
    ]


def retarget_glossaries(source):
    """Move only complete builtin glossary header lines, as migrate does."""
    result = []
    for line in source.splitlines(keepends=True):
        body = line.removesuffix("\n")
        if body.startswith("\\useglossary{lexlean.") and body.endswith("@1.1.0}"):
            body = body.removesuffix("@1.1.0}") + "@1.2.0}"
            line = body + ("\n" if line.endswith("\n") else "")
        result.append(line)
    return "".join(result)


def prepare(source, output, reader):
    source, output, reader = (p.resolve() for p in (source, output, reader))
    if not reader.is_file():
        raise ValueError("reader must be an existing official SDK executable")
    if output == source or source in output.parents:
        raise ValueError("output must be outside the source checkout")
    if output.exists():
        raise ValueError("output must not already exist")
    configuration = (source / "lexlean.toml").read_text()
    project = tomllib.loads(configuration)
    if project["language"] != "1.1":
        raise ValueError("this readiness transition starts from language 1.1")
    if any(row["kind"] != "builtin" for row in project.get("lexicon_source", [])):
        raise ValueError("path/Git lexicons require their own official migration")
    if configuration.count('language = "1.1"') != 1:
        raise ValueError("expected exactly one canonical project language row")
    files = sorted({
        path.relative_to(source)
        for root in project["source_roots"]
        for path in (source / root).rglob("*.lex.tex")
    })
    if not files:
        raise ValueError("source roots contain no modules")
    # Refuse escaping source roots, including symlinks, before creating output.
    for relative in files:
        path = source / relative
        if path.is_symlink() or not path.is_file() or source not in path.resolve().parents:
            raise ValueError(f"source is not a confined regular file: {relative}")
    output.mkdir(parents=True)
    candidate = output / "candidate"
    candidate.mkdir()
    (candidate / "lexlean.toml").write_text(
        configuration.replace('language = "1.1"', 'language = "1.2"', 1)
    )
    for name in ("lakefile.toml", "lake-manifest.json", "lean-toolchain"):
        shutil.copyfile(source / name, candidate / name)
    modules = []
    for relative in files:
        original = (source / relative).read_text()
        parsed = json.loads(official(reader, "read", original))
        target = json.loads(json.dumps(parsed))
        schema = target.get("spec")
        if schema in ("lexlean/semantic-module/1", "lexlean/core-module/1"):
            target["spec"] = schema[:-1] + "2"
            updated = official(reader, "write", json.dumps({
                "source": original, "value": target,
            }))
            updated = official(reader, "to-notation", updated)
        elif schema in ("lexlean/semantic-module/2", "lexlean/core-module/2"):
            updated = official(reader, "write", json.dumps({
                "source": original, "value": target,
            }))
        else:
            raise ValueError(f"unsupported source schema in {relative}: {schema}")
        updated = retarget_glossaries(updated)
        reread = json.loads(official(reader, "read", updated))
        if reread != target:
            raise ValueError(f"official parsed IR changed meaning: {relative}")
        rewritten = official(reader, "write", json.dumps({
            "source": updated, "value": reread,
        }))
        if rewritten != updated:
            raise ValueError(f"official notation write is not byte-idempotent: {relative}")
        if policy_inventory(parsed) != policy_inventory(reread):
            raise ValueError(f"declaration or annotation changed: {relative}")
        destination = candidate / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(updated)
        modules.append({
            "path": relative.as_posix(),
            "original_sha256": digest(original.encode()),
            "candidate_sha256": digest(updated.encode()),
            "original_schema": schema,
            "candidate_schema": target["spec"],
            "existing_notation_canonicalized": schema.endswith("/2") and updated != original,
            "full_ir_equal_except_root_schema": True,
            "official_notation_write_byte_idempotent": True,
            "declarations": policy_inventory(parsed),
        })
    report = {
        "schema": "uorc/language12-source-readiness/1",
        "scope": "source conversion only; not compiler, installed SDK, or release acceptance",
        "official_reader_sha256": digest(reader.read_bytes()),
        "original_configuration_sha256": digest(configuration.encode()),
        "candidate_configuration_sha256": digest((candidate / "lexlean.toml").read_bytes()),
        "original_lock_copied_or_changed": False,
        "source_modules": modules,
    }
    (output / "source-readiness.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--reader", required=True, type=Path)
    options = parser.parse_args()
    report = prepare(options.source, options.output, options.reader)
    print(json.dumps({
        "source_modules": len(report["source_modules"]),
        "declarations": sum(len(row["declarations"]) for row in report["source_modules"]),
        "scope": report["scope"],
    }))


if __name__ == "__main__":
    main()
