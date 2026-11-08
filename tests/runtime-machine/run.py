#!/usr/bin/env python3
"""Scoped execution of source-owned scenarios through official generated Lean.

This driver has no decoder implementation and cannot set release acceptance.
It requires an explicitly prepared diagnostic project and generated SDK runtime.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOTS = (
    "arithmeticScenario", "allOpcodesScenario", "dispatchResultsScenario",
    "conversionScenario", "referenceErrorsScenario", "resourcePrefixScenario",
    "scanFinalUpdateScenario", "invalidReferenceScenario", "eagerChoiceScenario",
    "choiceTypeScenario", "captureArityScenario", "exactBoundaryScenario",
    "oneBasedReferenceScenario",
)


def command(argv, *, cwd, env=None):
    result = subprocess.run(argv, cwd=cwd, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError("command failed: " + " ".join(map(str, argv)) + "\n" + result.stdout + result.stderr)
    return result.stdout + result.stderr


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project", required=True, type=Path)
    parser.add_argument("--lexlean", required=True, type=Path)
    parser.add_argument("--lean", required=True, type=Path)
    parser.add_argument("--runtime-modules", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    project = args.project.resolve()
    source = project / "src/Uorc/Core/RuntimeMachineTests.lex.tex"
    if not source.is_file() or not (args.runtime_modules / "Portable.lean").is_file():
        raise RuntimeError("exact scenario source and generated SDK runtime are required")
    original = sha(source)
    log = command([str(args.lexlean.resolve()), "--project", str(project / "lexlean.toml"),
                   "build", "src/Uorc/Core/RuntimeMachineTests.lex.tex"], cwd=project)
    match = re.search(r"\.lexlean/build/([0-9a-f]{64})", log)
    if not match:
        raise RuntimeError("official build returned no content-addressed build ID")
    build = project / ".lexlean/build" / match[1]
    modules = build / "modules"
    manifest = json.loads((build / "manifest.json").read_text())
    source_inventory = [row for row in manifest["inputs"] if row["kind"] == "source"]
    if not source_inventory:
        raise RuntimeError("empty source inventory")

    def check_inputs():
        for row in source_inventory:
            path = (project / row["path"]).resolve()
            if not path.is_relative_to(project) or not path.is_file() or sha(path) != row["sha256"]:
                raise RuntimeError("changed or escaping source input: " + row["path"])

    check_inputs()
    runtime = modules / "LexLeanRuntime"
    if runtime.exists():
        shutil.rmtree(runtime)
    shutil.copytree(args.runtime_modules, runtime)
    lean = str(args.lean.resolve())
    env = dict(os.environ, LEAN_PATH=str(modules))
    generated = {".".join(p.relative_to(modules).with_suffix("").parts): p
                 for p in modules.rglob("*.lean")}
    compiled = set()

    def compile_module(name):
        if name in compiled:
            return
        path = generated[name]
        for dependency in re.findall(r"(?:public )?import ([A-Za-z0-9_.]+)", path.read_text()):
            if dependency in generated:
                compile_module(dependency)
        command([lean, "-o", str(path.with_suffix(".olean")), str(path)], cwd=modules, env=env)
        compiled.add(name)

    target = "Uorc.Uorc.Core.RuntimeMachineTests"
    compile_module(target)
    body = generated[target].read_text()
    for root in ROOTS:
        if not re.search(r"public def " + re.escape(root) + r"\b", body):
            raise RuntimeError("missing scenario root: " + root)
    rows = ", ".join(f'("{root}", {target}.{root})' for root in ROOTS)
    harness = modules / "RunMachineControls.lean"
    harness.write_text(
        f"import {target}\ndef main : IO UInt32 := do\n"
        f"  let results := [{rows}]\n"
        '  for (name, passed) in results do\n    IO.println s!"{name}: {passed}"\n'
        "  return if results.all (·.2) then 0 else 1\n"
    )
    result = subprocess.run([lean, "--run", str(harness)], cwd=modules, env=env,
                            text=True, capture_output=True)
    expected = [f"{root}: true" for root in ROOTS]
    if result.returncode or result.stdout.splitlines() != expected or result.stderr:
        raise RuntimeError("scenario assertion failed\n" + result.stdout + result.stderr)
    check_inputs()
    if sha(source) != original:
        raise RuntimeError("scenario source changed during execution")
    report = {
        "schema": "uorc/runtime-machine-diagnostic/1", "scope": "helper-controls",
        "implementationAccepted": False, "build": match[1], "scenario_source_sha256": original,
        "lexlean_sha256": sha(args.lexlean), "lean_sha256": sha(args.lean),
        "roots": [{"root": target + "." + root, "passed": True} for root in ROOTS],
        "compiled_modules": sorted(compiled), "execution": "generated_lean_vm",
        "source_inventory": source_inventory,
        "generated_runtime": [{"path": str(p.relative_to(runtime)), "sha256": sha(p)}
                              for p in sorted(runtime.rglob("*.lean"))],
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(result.stdout, end="")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
