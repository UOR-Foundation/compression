<!-- @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`. -->
<!-- Do not edit. R1: the LexLean graph is the project authority. -->

# CONFORMANCE

Every registered ID has one generated projection here, one scenario, and
at least one executed test whose name ends in the normalized ID.

The three honesty levels (R2):

| Level | Meaning |
| --- | --- |
| `some-true` | A fact reproduced from an authority. **Not established here.** |
| `build` | Constructed here and validated against its oracle. Evidence, not a proof. |
| `open` | Measured and reported. **Never asserted.** |

## archive-integrity

| ID | Level | Statement |
| --- | --- | --- |
| `UC-INT-01` | `open` | Archive and basis integrity use an explicitly identified bounded streaming construction with standard SHA-256 components; digest agreement never replaces exact byte comparison. |
| `UC-INT-02` | `build` | Format3/machine1 explicitly binds STREAM-MAX-v1 to charged image spans, preserves format2 CHAIN-v1 semantics, and seals input, basis and checkpoint identity domains without replacing exact reconstruction comparison. |
| `UC-INT-03` | `build` | Finite reverse-trace candidates remain untrusted until actual successful SHA computations independently validate a collision witness; generic acceptance-guard reflection does not claim global SHA injectivity or reduction completeness. |

## checkpoint-replay

| ID | Level | Statement |
| --- | --- | --- |
| `UC-CHK-01` | `open` | Imported checkpoints are untrusted replay requests, not authority for a queue, incumbent, coverage, source identity or resource receipt. |

## claim-dispositions

| ID | Level | Statement |
| --- | --- | --- |
| `UC-HON-01` | `build` | Evidence classes and research dispositions remain distinct in the LexLean claim policy. |
| `UC-HON-02` | `build` | The honesty ledger and claim summaries are exact source projections with open research claims protected against promotion. |

## complete-synthesis

| ID | Level | Statement |
| --- | --- | --- |
| `UC-SYN-01` | `open` | Fair synthesis combines input-derived algebra, recurrence, residuals, dictionary, typed sharing and complete syntax enumeration without making heuristic proposals the admissible universe. |

## conditional-basis

| ID | Level | Statement |
| --- | --- | --- |
| `UC-BAS-01` | `open` | Conditional archives bind the actual supplied ordered basis using a source-owned length-delimited domain-separated identity, and execute against that admitted basis. |

## integrity

| ID | Level | Statement |
| --- | --- | --- |
| `UC-SHA-01` | `build` | Incremental SHA-256 and the separately named bounded chain have source-owned streaming semantics, typed failures and independently locked byte-oriented CAVP oracle evidence. |
| `AUTH-NIST-SHA256-SHORT` | `some-true` | The locked NIST CAVP SHA-256 short byte-oriented response data supplies 65 independent expected digests, reproduced as finite oracle checks. |
| `AUTH-NIST-SHA256-LONG` | `some-true` | The locked NIST CAVP SHA-256 long byte-oriented response data supplies 64 independent expected digests, reproduced as finite oracle checks. |
| `UC-SHA-02` | `build` | The distinct UORC-SHA256-STREAM-MAX-v1 construction streams arbitrary finite input with standard-domain frame bounds, typed cancellation, explicit terminal framing and independently checked reduced-capacity segmentation controls. |

## machine-resources

| ID | Level | Statement |
| --- | --- | --- |
| `UC-MACH-02` | `open` | Execution accounts for logical work and runtime-owned bytes under the explicit uorc.logical-owned-bytes/1 profile, including aliases, metadata, closures, transient copies and cancellation; old diagnostic live-cell counts remain distinct from bytes and neither is a claim of physical CPU time or resident memory. |

## machine-semantics

| ID | Level | Statement |
| --- | --- | --- |
| `UC-MACH-01` | `open` | Every admitted graph has the source-defined meaning of all 39 wire operations, including eager typed failures, prior-block calls and observer-before-update Scan with its final updater. |

## ordered-objects

| ID | Level | Statement |
| --- | --- | --- |
| `UC-OBJ-01` | `open` | Decoded output preserves the exact ordered object boundary vector, including zero objects and distinct zero-length objects. |

## product-interfaces

| ID | Level | Statement |
| --- | --- | --- |
| `UC-API-01` | `open` | The same product semantics serve native library, CLI and browser interfaces, with explicitly versioned streaming ownership rather than invented compatibility with bounded whole-buffer adapters. |

## product-lifecycle

| ID | Level | Statement |
| --- | --- | --- |
| `UC-PROD-01` | `open` | The source-owned product lifecycle handles demand, EOF, cancellation, edits, retries, stale completion and publication without exposing unfinished results as completed files. |

## real-execution

| ID | Level | Statement |
| --- | --- | --- |
| `UC-REAL-01` | `open` | A real nonliteral archive is independently parsed, executed and compared against every original byte before encoder publication. |

## reproducibility

| ID | Level | Statement |
| --- | --- | --- |
| `UC-REP-01` | `build` | Source-derived charter projections and complete LexLean build and verification artifacts are byte-identical across clean absolute roots under the locked SDK. |
| `UC-REP-02` | `build` | Dependency and execution-boundary failures are explicit and planted SDK-lock, source-lock, adjacent-checkout, and host-PATH fallbacks are rejected. |

## scoped-optimality

| ID | Level | Statement |
| --- | --- | --- |
| `UC-OPT-01` | `open` | The product minimizes complete archive byte length and unsigned lexicographic order within an explicitly sealed wire, basis and decoder-resource scope. |

## source-authority

| ID | Level | Statement |
| --- | --- | --- |
| `UC-CHR-01` | `build` | UORC charter, claim-boundary, evidence-boundary, and research-position records are accepted by the locked LexLean project and generate the committed project documentation projections. |
| `UC-CHR-02` | `build` | Handwritten project prose cannot define a second UORC specification or BCP-14 semantic rule, while non-authoritative evidence and planning prose remain permitted. |
| `UC-PERF-01` | `open` | measured public-corpus compression improvement, with exact target and scope; open until actually measured |
| `UC-REC-01` | `open` | externally accepted compression record; open unless the external authority actually accepts it |

## varint

| ID | Level | Statement |
| --- | --- | --- |
| `UC-VAR-01` | `build` | Shortest U64 ULEB128 encoding and strict decoding have source-owned domain and rejection rules, complete boundary vectors, and kernel-checked round-trip and size theorems. |

## wire-format

| ID | Level | Statement |
| --- | --- | --- |
| `UC-WIRE-01` | `open` | The source-declared unshipped archive and machine profile identities have canonical source-owned layouts, strict incremental parsing, exact body lengths and complete EOF checks, without asserting compatibility with an unavailable older draft. |

## Cited authorities

| Authority | Citation | Evidence here |
| --- | --- | --- |
| `NIST-SHA256-SHORT` | NIST CAVS 11.0, 2011-03-15; SHA256ShortMsg.rsp from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/shabytetestvectors.zip; acquired from existing public mirror https://github.com/mad4j/rusted-sha256/blob/0f7d17484d5d77c06f2f2f3e4d61745ce23a0c58/shabytetestvectors.zip; NIST public-information terms: https://www.nist.gov/copyrights-disclaimers | `UC-SHA-01` |
| `NIST-SHA256-LONG` | NIST CAVS 11.0, 2011-03-15; SHA256LongMsg.rsp from https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/shs/shabytetestvectors.zip; acquired from existing public mirror https://github.com/mad4j/rusted-sha256/blob/0f7d17484d5d77c06f2f2f3e4d61745ce23a0c58/shabytetestvectors.zip; NIST public-information terms: https://www.nist.gov/copyrights-disclaimers | `UC-SHA-01` |

## Registered claim dispositions

| ID | Level | Evidence class | Research disposition | Claim |
| --- | --- | --- | --- | --- |
| `UC-CHR-01` | `build` | `build_evidence` | `not_measured` | UORC charter, claim-boundary, evidence-boundary, and research-position records are accepted by the locked LexLean project and generate the committed project documentation projections. |
| `UC-CHR-02` | `build` | `build_evidence` | `not_measured` | Handwritten project prose cannot define a second UORC specification or BCP-14 semantic rule, while non-authoritative evidence and planning prose remain permitted. |
| `UC-PERF-01` | `open` | `none` | `not_measured` | measured public-corpus compression improvement, with exact target and scope; open until actually measured |
| `UC-REC-01` | `open` | `none` | `not_measured` | externally accepted compression record; open unless the external authority actually accepts it |
| `UC-REP-01` | `build` | `build_evidence` | `not_measured` | Source-derived charter projections and complete LexLean build and verification artifacts are byte-identical across clean absolute roots under the locked SDK. |
| `UC-REP-02` | `build` | `build_evidence` | `not_measured` | Dependency and execution-boundary failures are explicit and planted SDK-lock, source-lock, adjacent-checkout, and host-PATH fallbacks are rejected. |
| `UC-HON-01` | `build` | `build_evidence` | `not_measured` | Evidence classes and research dispositions remain distinct in the LexLean claim policy. |
| `UC-HON-02` | `build` | `build_evidence` | `not_measured` | The honesty ledger and claim summaries are exact source projections with open research claims protected against promotion. |
| `UC-VAR-01` | `build` | `kernel_theorem` | `not_measured` | Shortest U64 ULEB128 encoding and strict decoding have source-owned domain and rejection rules, complete boundary vectors, and kernel-checked round-trip and size theorems. |
| `UC-MACH-01` | `open` | `none` | `not_measured` | Every admitted graph has the source-defined meaning of all 39 wire operations, including eager typed failures, prior-block calls and observer-before-update Scan with its final updater. |
| `UC-MACH-02` | `open` | `none` | `not_measured` | Execution accounts for logical work and runtime-owned bytes under the explicit uorc.logical-owned-bytes/1 profile, including aliases, metadata, closures, transient copies and cancellation; old diagnostic live-cell counts remain distinct from bytes and neither is a claim of physical CPU time or resident memory. |
| `UC-WIRE-01` | `open` | `none` | `not_measured` | The source-declared unshipped archive and machine profile identities have canonical source-owned layouts, strict incremental parsing, exact body lengths and complete EOF checks, without asserting compatibility with an unavailable older draft. |
| `UC-OBJ-01` | `open` | `none` | `not_measured` | Decoded output preserves the exact ordered object boundary vector, including zero objects and distinct zero-length objects. |
| `UC-INT-01` | `open` | `none` | `not_measured` | Archive and basis integrity use an explicitly identified bounded streaming construction with standard SHA-256 components; digest agreement never replaces exact byte comparison. |
| `UC-BAS-01` | `open` | `none` | `not_measured` | Conditional archives bind the actual supplied ordered basis using a source-owned length-delimited domain-separated identity, and execute against that admitted basis. |
| `UC-SYN-01` | `open` | `none` | `not_measured` | Fair synthesis combines input-derived algebra, recurrence, residuals, dictionary, typed sharing and complete syntax enumeration without making heuristic proposals the admissible universe. |
| `UC-OPT-01` | `open` | `none` | `not_measured` | The product minimizes complete archive byte length and unsigned lexicographic order within an explicitly sealed wire, basis and decoder-resource scope. |
| `UC-CHK-01` | `open` | `none` | `not_measured` | Imported checkpoints are untrusted replay requests, not authority for a queue, incumbent, coverage, source identity or resource receipt. |
| `UC-PROD-01` | `open` | `none` | `not_measured` | The source-owned product lifecycle handles demand, EOF, cancellation, edits, retries, stale completion and publication without exposing unfinished results as completed files. |
| `UC-API-01` | `open` | `none` | `not_measured` | The same product semantics serve native library, CLI and browser interfaces, with explicitly versioned streaming ownership rather than invented compatibility with bounded whole-buffer adapters. |
| `UC-REAL-01` | `open` | `none` | `not_measured` | A real nonliteral archive is independently parsed, executed and compared against every original byte before encoder publication. |
| `UC-SHA-01` | `build` | `fixture_result` | `not_measured` | Incremental SHA-256 and the separately named bounded chain have source-owned streaming semantics, typed failures and independently locked byte-oriented CAVP oracle evidence. |
| `AUTH-NIST-SHA256-SHORT` | `some-true` | `imported_fact` | `not_measured` | The locked NIST CAVP SHA-256 short byte-oriented response data supplies 65 independent expected digests, reproduced as finite oracle checks. |
| `AUTH-NIST-SHA256-LONG` | `some-true` | `imported_fact` | `not_measured` | The locked NIST CAVP SHA-256 long byte-oriented response data supplies 64 independent expected digests, reproduced as finite oracle checks. |
| `UC-SHA-02` | `build` | `fixture_result` | `not_measured` | The distinct UORC-SHA256-STREAM-MAX-v1 construction streams arbitrary finite input with standard-domain frame bounds, typed cancellation, explicit terminal framing and independently checked reduced-capacity segmentation controls. |
| `UC-INT-02` | `build` | `fixture_result` | `not_measured` | Format3/machine1 explicitly binds STREAM-MAX-v1 to charged image spans, preserves format2 CHAIN-v1 semantics, and seals input, basis and checkpoint identity domains without replacing exact reconstruction comparison. |
| `UC-INT-03` | `build` | `fixture_result` | `not_measured` | Finite reverse-trace candidates remain untrusted until actual successful SHA computations independently validate a collision witness; generic acceptance-guard reflection does not claim global SHA injectivity or reduction completeness. |
