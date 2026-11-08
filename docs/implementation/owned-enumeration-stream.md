<!-- uorc:non-authoritative -->
# EnumerationStream custody adapter: implementation progress

`Uorc.Search.EnumerationStream` adds the Search-owned correlation, prepayment
and image-lease boundary between an outstanding owned byte-word ticket and a
future real archive trial. It is not an archive controller or membership
verdict. The producer and the two frozen proof-source modules are unchanged.

## Source behavior and caller obligations

- `start` and normal funded events validate the producer's exact outstanding
  owner/root/length ticket. The adapter never calls producer advancement or
  acknowledgment. The producer retains the sequence root for the entire trial.
- One fresh charged control root identifies this adapter generation. A fixed
  cursor holds that root, the source ticket, shared reader fields, one span,
  its logical position, custody/cancellation flags and partial prepayment.
  There is no second Arena, list/Graph/Rope backing or private traversal stack.
- Shared `Wire.OwnedRead.begin/step` performs the traversal. A borrowed span is
  staged, its image is actually retained, and only then is an offer returned.
  Exact acknowledgment checks generation owner, source ticket, logical position,
  image, offset and count. Waiting replay does not retain again or charge work.
- Acknowledgment schedules release of only this image lease. The reader resumes
  after release completes. EOF releases adapter control custody; it leaves the
  producer ticket outstanding. EOF and lease acknowledgment prove no coverage.
- Every effect is preceded by the shared RuntimeWork quote, including reader
  and retain/release steps. Split prepayment is retained. Budget refusal leaves
  offered credits and the unexecuted prefix unchanged. A real failed primitive
  consumes its prepaid attempt and retains unresolved custody.
- Cancellation may clean up even after the caller revokes the source ticket.
  It discards unused partial prepayment without refunding prior work, preserves
  in-progress release phases and repeated-cancel prepayment, drains adapter
  roots, and never releases or acknowledges the producer's ticket.
- `observeResult` preserves achieved Arena/Cursor/Delta on post-effect failures.
  Logical counters are not a physical CPU/RSS bound.

The caller must keep the candidate ticket pinned through the entire independent
trial and all borrowed spans. It must feed the real ArchiveParse/integrity/
admission/Engine path and compare ordered output and all bytes. Parser acceptance
or a TemplateAdmission event is not a canonical-membership decision. The caller
must retain unresolved predecessors. Fair discovery needs an independent lane;
ordinary best-known compression must not wait for exhaustive certification.

## Authentic reader provenance

The committed diagnostic cache is an exact copy of public cache commit
`f3d3f619cad5f60c632201dae0a7f3b854226769`, path
`tests/heap/semantic-cache/wire-native-d27be3e.json`, SHA256
`58e49bc558d38d0414b69a3e27c0380dfaf0f768fce968cb271a8f5f1637ee88`.
Its source checkpoint is `d27be3e8f7dba73440ec29fa8dba236275162ca7`.

Both OwnedRead and FrameEventContract source/blob/import/parser hashes and
byte-exact repeat-read/notation-writer records were verified. Official reader
provenance matches the frozen authoritative Core parser bundle. This consumes
the recorded official parser evidence; it does not re-run the reader or imply
linked compilation, kernel verification or native execution.

Only OwnedRead's exact parsed declarations are injected and evaluated. Its
source bytes already match the reviewed base. FrameEventContract has different
base bytes: its public parse evidence is verified externally, but neither its
source nor parsed declarations are overlaid or exercised here. No unrelated
Core/Wire ancestry is merged to obtain this cache.

## Executed finite controls and their limits

UC-SYN-01's initial scenario/test failed before the adapter existed with
`AssertionError: UC-SYN-01 stream custody adapter is not implemented`.
The complete current commands are:

```
PYTHONDONTWRITEBYTECODE=1 python -B tests/enumeration/stream_diagnostic_test.py
PYTHONDONTWRITEBYTECODE=1 python -B tests/enumeration/stream_mutations.py
PYTHONDONTWRITEBYTECODE=1 python -B tests/enumeration/test_stream_qualification.py
PYTHONDONTWRITEBYTECODE=1 python -B tests/enumeration/qualify_stream.py --repository . --output /tmp/stream-evidence.json
```

The reproduction needs the exact public base, Wire source/cache and frozen Core
source/cache Git objects. It copies exact source bytes into isolated temporary
unions; it does not edit the caller's repository. The receipt lists every
source hash, authority, parsed input, evaluator binding and actual command output.

Each actual run covers eight byte fixtures, 111 operation/cancellation prefixes,
11 spans and 114 funded partial-prepayment cancellation forks. At every positive
execution/cancellation prefix, `HeapAudit.auditRoots` checks the joint producer
and stream root multiset, including Heap's pending release custody. Zero,
refused and split payments, acknowledgment prepayment, repeated cancellation,
replay, stale generation/position correlation and final independent producer
cleanup are exercised. Four mutations alter actual loaded source IR: missing
image retain, unpaid execution, omitted generation comparison and omitted logical
position comparison. Each reaches its intended semantic assertion and passes
again after restoration.

Fourteen harness controls reject missing/extra/duplicate/reordered/skipped
outputs, wrong mutant failures, omitted restoration, process errors/stderr,
wrong cache, optimized Python and filter arguments. They are finite self-quality
evidence, not a source or proof acceptance gate.

The exact receipt preserves two independent runs:

1. Public base `9ca824709b0c6ffe4dbb647cb11a050b04247c19` plus the exact new adapter,
   tests and authentic reader cache.
2. That same union with only 34 verified Core source/cache pairs overlaid from
   source `3609a3624f7bf0f0d7c83b181fd88317868bdad5` and cache
   `eb56990bf53cd0c483b40df19557bbe0bfa120be`. The sole evaluator edit is its
   exact Core-cache fingerprint. This is not the whole Core checkout.

Nonempty fixtures use real ByteBuilder sequences inserted into a producer state;
they test adapter custody, not end-to-end enumeration completeness. Heap refusal
covers actual initial allocation exhaustion, not all retain/release failure
sites. The overflow case deliberately supplies an invalid MAX-offset reader to
`borrowedSpan`; it is a defensive helper boundary, not a reachable valid-reader
overflow. A defensive observer case supplies a real achieved allocation with a
deliberately mismatched observation budget and verifies retained state/cleanup.
Full `fund` rejects that mismatched budget before effects; no reachable
post-allocation budget-refusal path is invented.

## Qualification still required

No official source check, build, generated-native root, kernel/refinement proof,
archive-membership trial, fair controller, full SDK acceptance or physical
resource measurement is claimed. The frozen proof-source modules are not
expanded by this adapter. All general producer/Arena refinement obligations,
actual consumer handoff, canonical membership and unchanged complete exact-SDK
`just vv` acceptance remain open. No compiler bootstrap, lock/policy alteration,
merge, tag, release or mainstream compression benchmark is included.
