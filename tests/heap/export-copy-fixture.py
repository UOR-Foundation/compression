"""Pack four immutable public Copy sources; never invoke a compiler or export."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib

REPO = Path(__file__).resolve().parents[2]
SOURCE = "bd8c62063bd568c7f513042475e33a36bdccdeda"
TOOLS = REPO / "tests/heap/copy-export"


def git_bytes(relative):
    return subprocess.check_output(["git", "-C", str(REPO), "show", f"{SOURCE}:{relative}"])


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path, help="absolute NEW fixture directory")
    parser.add_argument("--archive", type=Path, help="optional absolute NEW source-only .tar.gz")
    args = parser.parse_args()
    if not args.destination.is_absolute() or args.destination.exists():
        parser.error("destination must be an absolute new directory")
    if args.archive and (not args.archive.is_absolute() or args.archive.exists()):
        parser.error("archive must be an absolute new file")
    inventory = json.loads((TOOLS / "root-inventory.json").read_text())
    assert inventory["source_commit"] == SOURCE and len(inventory["sources"]) == 4
    roots = [row["root"] for row in inventory["roots"]]
    assert len(roots) == 5 and roots == sorted(set(roots))
    files = {}
    modules = set()
    for row in inventory["sources"]:
        data = git_bytes(row["path"])
        assert sha(data) == row["sha256"]
        assert len(data) == row["bytes"]
        files[row["path"]] = data
        text = data.decode()
        modules.add(re.search(r"\\begin\{lexlean\}\{([^}]+)\}", text)[1])
    imports = set()
    for row in inventory["sources"]:
        imports.update(re.findall(r"\\importmodule\{([^}]+)\}", files[row["path"]].decode()))
    assert imports <= modules, "four-source declared module closure is not closed"
    qualified = "docs/implementation/runtime-image-copy-diagnostic-config/"
    names = ["lexlean.toml", "lexlean.lock", "lakefile.toml", "lake-manifest.json", "lean-toolchain"]
    for name in names:
        data = git_bytes(qualified + name)
        files["profiles/qualified/" + name] = data
        if name not in ["lexlean.toml", "lexlean.lock"]:
            files[name] = data
    original_config = files["profiles/qualified/lexlean.toml"].decode()
    export_config = original_config.replace(
        'name = "uorc-runtime-ownership-final-fusion"',
        'name = "uorc-runtime-image-copy-export"',
    ).replace(
        'entrypoints = ["src/Uorc/Core/RuntimeImageCopyTests.lex.tex"]',
        'entrypoints = ["src/Uorc/Core/RuntimeImageCopy.lex.tex"]',
    )
    old, new = tomllib.loads(original_config), tomllib.loads(export_config)
    assert {k for k in old if old[k] != new[k]} == {"name", "entrypoints"}
    assert new["language"] == "1.2" and new["limits"]["max_file_bytes"] == 4194304
    assert new["limits"]["child_timeout_ms"] == 300000
    files["lexlean.toml"] = export_config.encode()
    for name in ["request.mjs", "run-export.mjs"]:
        files["tools/" + name] = (TOOLS / name).read_bytes()
    files["root-inventory.json"] = (TOOLS / "root-inventory.json").read_bytes()
    evidence = json.loads(git_bytes("docs/implementation/runtime-image-copy-evidence.json"))
    files["README.md"] = f"""# Four-source Copy production export fixture

Authoring sources are byte-exact public UOR-Foundation/compression commit {SOURCE}:
Heap b342, Value8b3, Image7a and Copyaa130. There are exactly four source modules.
The five requested Copy roots and exact official-reader flags/signatures are in
root-inventory.json. The parser's metadata is not a replacement implementation.

lexlean.toml is the explicit language-1.2 export profile with entry Copy Main.
It changes only project name/entrypoint from the preserved qualified profile.
The unchanged 4MiB/300s limits and all imported production annotations remain.
No initial export lock is supplied: each existing compiler creates its own lock.
profiles/qualified preserves the original exact configuration/lock separately.
That is reference-only here: its original six-module test/proof fixture is not
reconstructed from this four-source production closure.

Run from this fixture with the compiler already present in its owning executor:
node tools/run-export.mjs /absolute/existing/original-lexlean "$PWD" original
node tools/run-export.mjs /absolute/existing/new-lexlean "$PWD" new

Each lane refuses an existing result directory and gets its own isolated project.
It records compiler SHA/version, generated lock, every command/stdout/stderr,
actual rust-std descriptor/digest, sorted request, export and artifact replay.
The lane label does not prove a compiler Git revision; bind the executable to
its source/build provenance in the owning executor. This package downloads or
builds no compiler and does not supply/qualify a new compiler candidate.

The pipeline is standard lock, format check, source check, production target
discovery, production build and accepted artifact replay. A failed stage records
accepted_export_and_replay=false. No result is promoted from source checking or
unqualified rendering. All diagnostics and partially generated artifacts remain.

The four immutable files also retain ten inherited Heap/Image production roots.
Standard verification may fail an inherited root before the five requested Copy
exports. The request is not a verifier filter: no annotations or checks are
removed. Heap A, Builder B and whole-product outcomes stay separate; this fixture
cannot establish full-product or digest-pinned SDK release acceptance.
The original source/kernel/native/48-policy receipts remain separate evidence.
"""
    files["README.md"] = files["README.md"].encode()
    manifest = {
        "schema": "uorc/runtime-image-copy-production-fixture/1",
        "repository": "https://github.com/UOR-Foundation/compression",
        "source_commit": SOURCE,
        "sources": inventory["sources"],
        "production_roots": roots,
        "inherited_declared_production_roots": [
            row["root"] for row in inventory["inherited_declared_production_roots"]
        ],
        "source_declared_module_closure": sorted(modules),
        "export_profile": {"path": "lexlean.toml", "language": "1.2",
                           "entrypoint": new["entrypoints"][0], "limits": new["limits"]},
        "qualified_profile": {
            "path": "profiles/qualified/lexlean.toml",
            "lock_path": "profiles/qualified/lexlean.lock",
            "reference_only_in_four_source_package": True,
            "original_source_build": evidence["source_build"],
            "original_six_source_sha256": evidence["source_sha256"],
        },
        "compiler_comparison": {
            "original_reference_sdk_commit": "71defb4c663aaec5b9a3aab04cbc6a03e7106227",
            "new_compiler_supplied_and_identified_by_existing_owner": True,
            "results_separate": ["consumer-results/original", "consumer-results/new"],
        },
        "tools_adapted_from": ["tests/heap/export-e87-fixture.py",
                               "tests/heap/request.mjs", "tests/heap/run-export.mjs"],
        "packer_sha256": sha(Path(__file__).read_bytes()),
        "files": [{"path": p, "sha256": sha(b)} for p, b in sorted(files.items())],
        "export_run_here": False,
        "production_export_claimed": False,
        "whole_product_acceptance_claimed": False,
    }
    files["fixture-manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    args.destination.mkdir(parents=True)
    for relative, data in sorted(files.items()):
        output = args.destination / relative
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(data)
    result = {"directory": str(args.destination), "source_count": 4, "root_count": 5,
              "file_count": len(files), "manifest_sha256": sha(files["fixture-manifest.json"])}
    if args.archive:
        args.archive.parent.mkdir(parents=True, exist_ok=True)
        with args.archive.open("wb") as raw:
            with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as zipped:
                with tarfile.open(fileobj=zipped, mode="w", format=tarfile.USTAR_FORMAT) as archive:
                    for relative, data in sorted(files.items()):
                        entry = tarfile.TarInfo(relative)
                        entry.size, entry.mode, entry.mtime = len(data), 0o644, 0
                        archive.addfile(entry, io.BytesIO(data))
        result.update(archive=str(args.archive), archive_sha256=sha(args.archive.read_bytes()),
                      archive_bytes=args.archive.stat().st_size)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
