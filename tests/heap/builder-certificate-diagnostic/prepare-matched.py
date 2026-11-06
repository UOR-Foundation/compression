#!/usr/bin/env python3
"""Prepare the full, unchanged 21-source diagnostic pair; run no compiler."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import tarfile
import tempfile

ARCHIVE_SHA256 = "b87eaa03a33bf55279852efd1e0f3960670f90d5525461d7b5a0786fbacabc81"
CANDIDATE_SHA256 = "415d8c8d8183b3787611cf96f5620009177332c71f2a198861c1a77df32bb075"
BUILDER = "src/Uorc/Core/RuntimeByteBuilder.lex.tex"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def prepare(archive, candidate, output):
    if output.exists():
        raise ValueError("output already exists; no files replaced")
    if sha(archive.read_bytes()) != ARCHIVE_SHA256:
        raise ValueError("archive SHA-256 mismatch")
    candidate_bytes = candidate.read_bytes()
    if sha(candidate_bytes) != CANDIDATE_SHA256:
        raise ValueError("discarded candidate SHA-256 mismatch")
    expected = json.loads(Path(__file__).with_name("matched-inputs.json").read_text())
    if len(expected) != 25 or sum(p.endswith(".lex.tex") for p in expected) != 21:
        raise ValueError("fixture inventory must contain all 21 sources and four configuration files")
    payloads = {}
    with tarfile.open(archive, "r:gz") as tf:
        members = tf.getmembers()
        for relative, digest in expected.items():
            matches = [m for m in members if m.name == "diagnostic-project/" + relative]
            if len(matches) != 1 or not matches[0].isfile():
                raise ValueError("missing, duplicated or non-regular input: " + relative)
            data = tf.extractfile(matches[0]).read()
            if sha(data) != digest:
                raise ValueError("input SHA-256 mismatch: " + relative)
            payloads[relative] = data
    # Only fixed inventory members are written; archive names are never extracted.
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix="builder-matched-", dir=output.parent))
    try:
        hashes = {}
        for case in ["baseline-fc419", "candidate-415d8"]:
            hashes[case] = {}
            for relative, original in payloads.items():
                data = candidate_bytes if case == "candidate-415d8" and relative == BUILDER else original
                dest = staging / case / relative
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_bytes(data)
                hashes[case][relative] = sha(data)
        difference = [p for p in expected if hashes["baseline-fc419"][p] != hashes["candidate-415d8"][p]]
        if difference != [BUILDER]:
            raise ValueError("pair differs outside its one declared source change")
        receipt = {
            "schema": "uorc/builder-certificate-matched-preparation/1",
            "purpose": "diagnostic preparation only; no source, certificate or release acceptance",
            "archive_sha256": ARCHIVE_SHA256,
            "candidate_sha256": CANDIDATE_SHA256,
            "input_count_per_case": 25,
            "source_count_per_case": 21,
            "source_difference": difference,
            "files_sha256": hashes,
            "compiler_executed": False,
            "limits_or_production_annotations_changed": False,
        }
        (staging / "preparation-receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
        os.rename(staging, output)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("discarded_candidate_source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        result = prepare(args.archive, args.discarded_candidate_source, args.output)
    except (ValueError, OSError, tarfile.TarError) as error:
        parser.exit(1, str(error) + "\n")
    print(json.dumps({"sources_per_case": result["source_count_per_case"], "changed": result["source_difference"]}))
