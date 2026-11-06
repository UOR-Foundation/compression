#!/usr/bin/env python3
"""Replay the complete source-owned equality fixture and its kernel audit."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

MODULE = "Uorc.Search.EqualityClassScenarios"
CASES = (
    "changed_state_offset", "changed_state_navigation_deep_field",
    "changed_arena_index_payload", "changed_arena_release_list", "same_state",
    "same_arena", "funding_same_ctor_resource_payload",
    "funding_nested_error_constructor", "funding_deep_sequence_heap_payload",
    "funding_produced_event_payload", "funding_outcome_constructor",
    "owner_same_ctor_resource_payload", "owner_same_ctor_heap_payload",
    "owner_nested_error_constructor", "owner_outcome_constructor",
    "identical_funding_produced", "identical_owner_failed_payload",
)
GLOBALS = (
    "fullFundingOutcomeEqual_iff", "fullOwnerOutcomeEqual_iff",
    "fullStateEqual_iff", "fullArenaEqual_iff",
)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "project", "helper", "lean", "runtime-root", "output"):
        ap.add_argument("--" + name, required=True, type=Path)
    ap.add_argument("--fixture", type=Path,
                    default=Path(__file__).with_name("EqualityClassScenarios.lex.tex"))
    ap.add_argument("--proof-file", type=Path,
                    default=Path(__file__).with_name("DefaultEqualityClassAudit32.lean"))
    args = ap.parse_args()
    compiler, helper, lean = (p.resolve(strict=True) for p in
                              (args.compiler, args.helper, args.lean))
    config = args.project.resolve(strict=True)
    if config.is_dir():
        config /= "lexlean.toml"
    fixture, proof = (p.resolve(strict=True) for p in (args.fixture, args.proof_file))
    runtime = args.runtime_root.resolve(strict=True) / "LexLeanRuntime"
    if not (runtime / "Portable.olean").is_file():
        raise ValueError("--runtime-root must contain official LexLeanRuntime/Portable.olean")
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", proof.stem):
        raise ValueError("proof filename must be a Lean module identifier")
    if re.search(r"\bset_option\b", proof.read_text()):
        raise ValueError("audit proof must use stock Lean options")
    settings = tomllib.loads(config.read_text())
    if settings.get("language") != "1.2":
        raise ValueError("prepare a language-1.2 project through the official SDK first")
    out = args.output.resolve()
    out.mkdir(parents=True)  # Refuse stale output or an existing workspace.
    workspace, lean_src, oleans = (out / n for n in ("project", "lean-src", "oleans"))
    workspace.mkdir()
    lean_src.mkdir()
    oleans.mkdir()
    runtime_only = out / "runtime"
    runtime_only.mkdir()
    (runtime_only / "LexLeanRuntime").symlink_to(runtime, target_is_directory=True)
    records: list[dict] = []

    def command(name: str, argv: list[str], *, stdin: str | None = None,
                lean_environment: bool = False) -> str:
        env = os.environ.copy()
        if lean_environment:
            env["LEAN_PATH"] = os.pathsep.join((str(oleans), str(runtime_only)))
        result = subprocess.run(argv, input=stdin, text=True, capture_output=True,
                                cwd=workspace, env=env, timeout=60)
        receipt = out / "commands" / name
        receipt.mkdir(parents=True)
        (receipt / "stdout").write_text(result.stdout)
        (receipt / "stderr").write_text(result.stderr)
        record = {"name": name, "argv": argv, "returncode": result.returncode}
        records.append(record)
        (out / "commands.json").write_text(json.dumps(records, indent=2) + "\n")
        if result.returncode:
            raise RuntimeError(f"{name} failed; see {receipt}")
        return result.stdout

    value = json.loads(command("read-fixture", [str(helper), "read"],
                               stdin=fixture.read_text()))
    declarations = value.get("declarations", [])
    if tuple(d.get("name") for d in declarations) != CASES:
        raise ValueError("fixture must contain all 17 expected cases, in order")
    if any(d.get("kind") != "definition" or d.get("parameters") != [] or
           d.get("result") != {"kind": "bool"} for d in declarations):
        raise ValueError("each case must remain a closed source-owned Bool definition")

    source_roots = settings["source_roots"]
    if not source_roots:
        raise ValueError("project has no configured source root")

    def relative(path: str) -> Path:
        result = Path(path)
        if result.is_absolute() or ".." in result.parts:
            raise ValueError("reproducer requires project-relative workspace paths")
        return result

    copied: dict[str, str] = {}
    for root in source_roots:
        root = relative(root)
        shutil.copytree(config.parent / root, workspace / root)
    files = {config.name, str(relative(settings.get("lockfile", "lexlean.lock")))}
    lean_workspace = relative(settings.get("lean_workspace", "."))
    for name in ("lakefile.toml", "lakefile.lean", "lake-manifest.json", "lean-toolchain"):
        path = lean_workspace / name
        if (config.parent / path).is_file():
            files.add(str(path))
    for name in sorted(files):
        source = config if name == config.name else config.parent / name
        target = workspace / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        copied[name] = digest(target)
    fixture_relative = relative(source_roots[0]) / Path(*MODULE.split("."))
    fixture_relative = fixture_relative.with_suffix(".lex.tex")
    target = workspace / fixture_relative
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists() and target.read_bytes() != fixture.read_bytes():
        raise ValueError("source fixture destination already has different content")
    shutil.copy2(fixture, target)
    configured = workspace / config.name
    answer = json.loads(command("build", [str(compiler), "--project", str(configured),
                               "--diagnostic-format", "json", "build", str(fixture_relative)]))
    if not answer.get("success") or len(answer.get("artifacts", [])) != 1:
        raise ValueError("compiler did not return one successful build artifact")
    artifact = workspace / answer["artifacts"][0]
    manifest = json.loads((artifact / "manifest.json").read_text())
    modules = {row["lean_module"]: artifact / "modules" /
               Path(*row["lean_module"].split(".")).with_suffix(".lean")
               for row in manifest["modules"]}
    fixture_rows = [r for r in manifest["modules"] if r["module"] == MODULE]
    if len(fixture_rows) != 1:
        raise ValueError("build manifest does not identify the exact fixture owner")
    fixture_owner = fixture_rows[0]["lean_module"]
    done: set[str] = set()
    active: set[str] = set()
    compiled: list[dict] = []

    def compile_module(name: str) -> None:
        if name in done:
            return
        if name in active:
            raise ValueError("cyclic generated Lean import graph")
        active.add(name)
        original = modules[name]
        imports = re.findall(r"^(?:public )?(?:meta )?import (\S+)\s*$",
                             original.read_text(), re.MULTILINE)
        for dependency in imports:
            if dependency in modules:
                compile_module(dependency)
        relative_name = Path(*name.split("."))
        source = lean_src / relative_name.with_suffix(".lean")
        target = oleans / relative_name.with_suffix(".olean")
        source.parent.mkdir(parents=True, exist_ok=True)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(original, source)  # Complete generated source, including every proof.
        command("compile-" + name, [str(lean), "-R", str(lean_src), "-o",
                                    str(target), str(source)], lean_environment=True)
        compiled.append({"module": name, "generated_sha256": digest(source)})
        active.remove(name)
        done.add(name)

    for name in sorted(modules):  # Every emitted module; no filtered dependency proofs.
        compile_module(name)
    harness = lean_src / "RunEqualityRegression.lean"
    harness.write_text("module\npublic meta import " + fixture_owner + "\n" + "".join(
        '#eval (do\n  if ' + fixture_owner + '.' + name + ' then IO.println "' +
        name + ':true"\n  else throw (IO.userError "' + name + ':false")\n  : IO Unit)\n'
        for name in CASES))
    output = command("execute-17-cases", [str(lean), "-R", str(lean_src), str(harness)],
                     lean_environment=True)
    if output.splitlines() != [name + ":true" for name in CASES]:
        raise ValueError("VM output omitted, duplicated or failed a case")
    proof_copy = lean_src / proof.name
    shutil.copy2(proof, proof_copy)
    command("kernel-audit", [str(lean), "-R", str(lean_src), "-o",
                              str(oleans / (proof.stem + ".olean")), str(proof_copy)],
            lean_environment=True)
    consumer = lean_src / "ImportEqualityRegression.lean"
    consumer.write_text("module\npublic import " + proof.stem +
                        "\npublic meta import Lean\n" + "".join(
                            "#check " + name + "\n#print axioms " + name + "\n"
                            for name in GLOBALS) +
                        'run_cmd do\n  let opts ← Lean.getOptions\n'
                        '  if Lean.maxRecDepth.get opts != 512 || Lean.maxHeartbeats.get opts != 200000 then\n'
                        '    throwError "expected stock Lean 4.32.1 recursion and heartbeat options"\n'
                        '  Lean.logInfo m!"effective maxRecDepth={Lean.maxRecDepth.get opts}; '
                        'effective maxHeartbeats={Lean.maxHeartbeats.get opts}"\n')
    command("fresh-kernel-import", [str(lean), "-R", str(lean_src), "-o",
                                    str(oleans / "ImportEqualityRegression.olean"), str(consumer)],
            lean_environment=True)
    if any(digest(workspace / path) != value for path, value in copied.items()):
        raise ValueError("copied configuration/workspace/lock bytes changed")
    summary = {"scope": "Complete generated fixture closure, 17 actual VM assertions and stock-option kernel audit; not full Controls or SDK/release acceptance",
               "fixture_sha256": digest(fixture), "proof_sha256": digest(proof),
               "tools": {str(p): digest(p) for p in (compiler, helper, lean)},
               "build_id": answer.get("build_id"), "modules": compiled,
               "cases": list(CASES), "all_cases_passed": True,
               "kernel_audit_and_fresh_import_passed": True,
               "copied_configuration_and_lock_sha256": copied,
               "runtime_files": {str(p.relative_to(runtime)): digest(p)
                                 for p in runtime.rglob("*") if p.is_file()},
               "all_generated_module_proofs_retained": True,
               "source_project_unchanged": True}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print("17 cases, complete generated closure, kernel audit and fresh import: PASS")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
