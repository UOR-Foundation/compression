"""Pack exact public e87 authoring sources; diagnostic tools are separate."""
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile

REPO = Path(__file__).resolve().parents[2]
SOURCE = "e87cd22372037374f7ce28b6ab0ea3e619129610"
COMPILER = "3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e"

def git_bytes(relative):
    return subprocess.check_output(["git", "-C", str(REPO), "show", f"{SOURCE}:{relative}"])

def sha(data):
    return hashlib.sha256(data).hexdigest()

def main():
    if len(sys.argv) != 2 or not Path(sys.argv[1]).is_absolute():
        raise SystemExit("pass an absolute new output .tar.gz path")
    target = Path(sys.argv[1])
    if target.exists():
        raise SystemExit("refuse to replace an existing deliverable")
    cache = json.loads(git_bytes("tests/heap/semantic-cache/runtime-owners.json"))
    files = {}
    sources = []
    roots = []
    for module in cache["modules"]:
        relative = module["source_path"]
        data = git_bytes(relative)
        assert sha(data) == module["source_sha256"]
        files[relative] = data
        sources.append({"path": relative, "sha256": sha(data)})
        for definition in module["definitions"]:
            if "production" in definition:
                roots.append("Uorc." + module["module"] + "." + definition["name"])
    assert len(sources) == 14 and len(set(roots)) == 21
    for name in ["lexlean.toml", "lakefile.toml", "lake-manifest.json", "lean-toolchain"]:
        files[name] = git_bytes("tests/heap/" + name)
    for name in ["request.mjs", "run-export.mjs"]:
        files["tools/" + name] = (REPO / "tests/heap" / name).read_bytes()
    files["README.md"] = f"""# Exact e87 production consumer fixture

All fourteen `.lex.tex` files and four project files are byte-exact from public
UOR-Foundation/compression commit `{SOURCE}`. This is the isolated language-1.2
diagnostic project, not the compressor's pinned SDK release project.

Use the existing official compiler at `{COMPILER}` in the wire executor; do not
build or pull another compiler. Extract into a fresh directory and run:

```sh
node tools/run-export.mjs /absolute/existing/lexlean "$PWD"
```

The runner verifies every supplied byte, records the compiler binary digest and
version, locks/checks/formats these exact sources, obtains the compiler's actual
rust-std target digest, requests all 21 sorted production roots and replays the
accepted bundle. It preserves every command's stdout, stderr and exit status in
`consumer-results/`. An existing result directory is refused rather than erased.
No failed export is promoted to an accepted result. No compiler/source/axiom or
preservation check is patched or weakened.

`fixture-manifest.json` contains all source/tool hashes and the complete root
inventory. Diagnostic request generation includes the SDK-required ascending
root ordering correction; it does not change any e87 authoring source.

The previous seven native Lean suites and 406 exact axiom checks are distinct
evidence. Export/replay here does not establish physical RSS/copy bounds, general
refinement or full digest-pinned SDK `just vv`/release acceptance.
""".encode()
    manifest = {
        "schema": "uorc/e87-production-consumer-fixture/1",
        "repository": "https://github.com/UOR-Foundation/compression",
        "source_commit": SOURCE,
        "requested_compiler_commit": COMPILER,
        "sources": sorted(sources, key=lambda row: row["path"]),
        "production_roots": sorted(roots),
        "files": [{"path": p, "sha256": sha(b)} for p, b in sorted(files.items())],
        "fixture_tools_are_diagnostic": True,
        "production_export_claimed": False,
    }
    files["fixture-manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode="w", format=tarfile.USTAR_FORMAT) as archive:
                for relative, data in sorted(files.items()):
                    entry = tarfile.TarInfo(relative)
                    entry.size = len(data)
                    entry.mode = 0o644
                    entry.mtime = 0
                    archive.addfile(entry, io.BytesIO(data))
    print(json.dumps({"path": str(target), "sha256": sha(target.read_bytes()),
                      "bytes": target.stat().st_size, "sources": len(sources), "roots": len(roots)}))

if __name__ == "__main__":
    main()
