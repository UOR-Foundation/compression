"""Materialize three exact public authoring modules for the certificate-A probe."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

REPO = Path(__file__).resolve().parents[2]
SOURCES = [f"src/Uorc/Core/{name}.lex.tex" for name in
           ["RuntimeValue", "Heap", "RuntimeSequence"]]
CONTROL = "e87cd22372037374f7ce28b6ab0ea3e619129610"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_commit", help="exact 40-character local Git commit")
    parser.add_argument("output", type=Path, help="new absolute directory")
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.source_commit):
        parser.error("source_commit must be an immutable full Git SHA")
    if not args.output.is_absolute() or args.output.exists():
        parser.error("output must be a new absolute directory")
    resolved = subprocess.check_output(
        ["git", "-C", str(REPO), "rev-parse", args.source_commit + "^{commit}"],
        text=True).strip()
    if resolved != args.source_commit:
        raise RuntimeError("source commit did not resolve exactly")
    files = {}
    for source in SOURCES:
        files[source] = subprocess.check_output(
            ["git", "-C", str(REPO), "show", f"{resolved}:{source}"])
    allowed = {"Uorc.Core.RuntimeValue", "Uorc.Core.Heap", "Uorc.Core.RuntimeSequence"}
    for source in SOURCES:
        imports = re.findall(rb"\\importmodule\{([^}]+)\}", files[source])
        if not {x.decode() for x in imports}.issubset(allowed):
            raise RuntimeError("fixture no longer contains the complete source closure")
    for name in ["lakefile.toml", "lake-manifest.json", "lean-toolchain"]:
        files[name] = subprocess.check_output(
            ["git", "-C", str(REPO), "show", f"{CONTROL}:tests/heap/{name}"])
    files["lexlean.toml"] = (REPO / "tests/heap/sequence-repro.toml").read_bytes()
    files["README.md"] = f"""# Exact Sequence certificate reproduction

The three .lex.tex files are byte-exact from public compressor commit
{resolved}. Only diagnostic configuration is supplied here. No source patch,
generated compiler artifact or replacement SDK is included. This language1.2
project is separate from the compressor's locked SDK release project.

Use the already-built official compiler; do not build or download another one.
From this directory, run the following with its absolute path substituted:

```sh
/absolute/existing/lexlean --version
/absolute/existing/lexlean --project lexlean.toml lock --diagnostic-format json > lock-result.json
timeout -k 10s 360s /absolute/existing/lexlean --project lexlean.toml verify src/Uorc/Core/RuntimeSequence.lex.tex --diagnostic-format json > verify-result.json 2> verify.stderr
```

Retain the compiler binary SHA256, generated lock, process exit status and both
verification streams. Exit124 denotes the outer timeout, not a kernel result.
The SDK child limit stays300 seconds and max_file_bytes stays4,194,304.
The standard verify command executes the unchanged fixed pipeline, including
ordinary source/preservation checks. A failure is not an accepted export.

Original Sequence c352ecaf... with public Heap8bb13182... is the confirmed
control: compiler71defb4's step certificateA reports __rel_16 kernel deep
recursion, and the mapped function is concatDown. The direct concatDown root
has a smaller closure and has NOT been established as a failing reproduction.
For factored sources, do not assume newer-compiler acceptance. Record the
actual failed relation and source identity. The local target-prefix probe is
narrower evidence and does not replace this full verification.
""".encode()
    manifest = {
        "schema": "uorc/sequence-certificate-repro/1",
        "repository": "https://github.com/UOR-Foundation/compression",
        "source_commit": resolved,
        "sources": [{"path": p, "sha256": sha(files[p])} for p in SOURCES],
        "files": [{"path": p, "sha256": sha(data)} for p, data in sorted(files.items())],
        "verification_input": "src/Uorc/Core/RuntimeSequence.lex.tex",
        "mapped_failed_function_in_control": "Uorc.Uorc.Core.RuntimeSequence.concatDown",
        "control_relation_in_step_program": "__rel_16",
        "max_file_bytes": 4194304,
        "child_timeout_ms": 300000,
        "compiler_bundled": False,
        "compiler_acceptance_claimed": False,
        "locked_product_sdk_acceptance_claimed": False,
    }
    files["fixture-manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    args.output.mkdir(parents=True)
    for relative, data in files.items():
        target = args.output / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    print(json.dumps({"directory": str(args.output), "source_commit": resolved,
                      "sources": manifest["sources"],
                      "manifest_sha256": sha(files["fixture-manifest.json"])}))


if __name__ == "__main__":
    main()
