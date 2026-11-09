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
| `UC-INT-01` | `build` | Every archive the client decodes is checked against its UORC-SHA256-CHAIN-v1 integrity trailer, so changing any single bit of an archive makes it fail to decode. |
| `UC-INT-02` | `build` | Format3/machine1 explicitly binds STREAM-MAX-v1 to charged image spans, preserves format2 CHAIN-v1 semantics, and seals input, basis and checkpoint identity domains without replacing exact reconstruction comparison. |
| `UC-INT-03` | `build` | Source-owned checker soundness and constructive finite backward reduction are kernel checked for actual canonical framed messages and shape-valid source-SHA-authenticated traces, without global hash injectivity or an assumed frame codec. The decoder-to-authenticated-trace bridge and full product acceptance remain distinct. |
| `UC-INT-04` | `build` | Source-owned actual-call relations establish ordered hash admissions, drained selected-view EOF and checked cumulative basis/main work; composed generated-model controls reject failed-prefix resets and late corruption. Consumer snapshot preservation, canonical digest refinement and complete archive EOF remain separate obligations. |
| `UC-INT-05` | `build` | Source-owned completed-view and continuous publication proofs derive the actual chosen-profile digest and emitted-image reads, discharging canonical driver termination and OwnedHash control/cleanup lifetime for both explicit format identities and arbitrary finite source input. Initial source-layout correspondence and full product acceptance remain separate consumer obligations. |

## claim-dispositions

| ID | Level | Statement |
| --- | --- | --- |
| `UC-HON-01` | `build` | Evidence classes and research dispositions remain distinct in the LexLean claim policy. |
| `UC-HON-02` | `build` | The honesty ledger and claim summaries are exact source projections with open research claims protected against promotion. |

## complete-synthesis

| ID | Level | Statement |
| --- | --- | --- |
| `UC-SYN-01` | `build` | Program search under a step budget keeps a candidate only as a fully checked archive: more effort can change which archive wins, never whether the result decodes to the input. |

## conditional-basis

| ID | Level | Statement |
| --- | --- | --- |
| `UC-BAS-01` | `open` | Conditional archives bind the actual supplied ordered basis using a source-owned length-delimited domain-separated identity, and execute against that admitted basis. Open: the shipped client writes and reads self-contained archives only; conditional-basis archives are not shipped yet. |

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
| `UC-MACH-02` | `build` | Decoding is charged against an explicit decoder work budget, and exhausting the budget is a typed refusal with no partial output. |

## machine-semantics

| ID | Level | Statement |
| --- | --- | --- |
| `UC-MACH-01` | `build` | The client's reference-semantics evaluator decodes program archives, including search-found programs and the ulz kernel graph when the certified fast path declines, to their exact original bytes. |

## ordered-objects

| ID | Level | Statement |
| --- | --- | --- |
| `UC-OBJ-01` | `open` | Decoded output preserves the exact ordered object boundary vector, including zero objects and distinct zero-length objects. Open: the shipped client archives one object per archive; multi-object archives are not shipped yet. |

## product-interfaces

| ID | Level | Statement |
| --- | --- | --- |
| `UC-API-01` | `build` | The uorc library, CLI and WebAssembly exports run the same exported compress and decompress over arbitrary bytes, with typed budget, overflow and decode failures; inspect and verify operations are not part of this interface yet. |

## real-execution

| ID | Level | Statement |
| --- | --- | --- |
| `UC-REAL-01` | `build` | Real archives that uorc compress writes for text, structured binary, noise and edge-case inputs are independently decoded and compared against every original byte. |

## reproducibility

| ID | Level | Statement |
| --- | --- | --- |
| `UC-REP-01` | `build` | Source-derived charter projections and complete LexLean build and verification artifacts are byte-identical across clean absolute roots under the locked SDK. |
| `UC-REP-02` | `build` | Dependency and execution-boundary failures are explicit and planted SDK-lock, source-lock, adjacent-checkout, and host-PATH fallbacks are rejected. |

## scoped-optimality

| ID | Level | Statement |
| --- | --- | --- |
| `UC-OPT-01` | `build` | Compress returns the smallest of the archives it checked, which are the ulz kernel archive, the raw archive and any search candidate within the step budget; no minimum beyond that candidate set is claimed. |

## source-authority

| ID | Level | Statement |
| --- | --- | --- |
| `UC-CHR-01` | `build` | UORC charter, claim-boundary, evidence-boundary, and research-position records are accepted by the locked LexLean project and generate the committed project documentation projections. |
| `UC-CHR-02` | `build` | Handwritten project prose cannot define a second UORC specification or BCP-14 semantic rule, while non-authoritative evidence and planning prose remain permitted. |
| `UC-PERF-01` | `open` | measured public-corpus compression: every release runs scripts/bench.mjs on the Canterbury and Silesia corpora against gzip, bzip2, xz, zstd and brotli and publishes archive sizes and times in the release notes and on the Pages site; open because the measurement sets no target |
| `UC-REC-01` | `open` | externally accepted compression record; open unless the external authority actually accepts it |

## varint

| ID | Level | Statement |
| --- | --- | --- |
| `UC-VAR-01` | `build` | Shortest U64 ULEB128 encoding and strict decoding have source-owned domain and rejection rules, complete boundary vectors, and kernel-checked round-trip and size theorems. |

## wire-format

| ID | Level | Statement |
| --- | --- | --- |
| `UC-WIRE-01` | `build` | The client decodes only complete self-contained UORC archives in their canonical layout: truncated archives, trailing bytes and unsupported format identities are typed decode failures. |

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
| `UC-PERF-01` | `open` | `none` | `not_measured` | measured public-corpus compression: every release runs scripts/bench.mjs on the Canterbury and Silesia corpora against gzip, bzip2, xz, zstd and brotli and publishes archive sizes and times in the release notes and on the Pages site; open because the measurement sets no target |
| `UC-REC-01` | `open` | `none` | `not_measured` | externally accepted compression record; open unless the external authority actually accepts it |
| `UC-REP-01` | `build` | `build_evidence` | `not_measured` | Source-derived charter projections and complete LexLean build and verification artifacts are byte-identical across clean absolute roots under the locked SDK. |
| `UC-REP-02` | `build` | `build_evidence` | `not_measured` | Dependency and execution-boundary failures are explicit and planted SDK-lock, source-lock, adjacent-checkout, and host-PATH fallbacks are rejected. |
| `UC-HON-01` | `build` | `build_evidence` | `not_measured` | Evidence classes and research dispositions remain distinct in the LexLean claim policy. |
| `UC-HON-02` | `build` | `build_evidence` | `not_measured` | The honesty ledger and claim summaries are exact source projections with open research claims protected against promotion. |
| `UC-VAR-01` | `build` | `kernel_theorem` | `not_measured` | Shortest U64 ULEB128 encoding and strict decoding have source-owned domain and rejection rules, complete boundary vectors, and kernel-checked round-trip and size theorems. |
| `UC-MACH-01` | `build` | `build_evidence` | `not_measured` | The client's reference-semantics evaluator decodes program archives, including search-found programs and the ulz kernel graph when the certified fast path declines, to their exact original bytes. |
| `UC-MACH-02` | `build` | `build_evidence` | `not_measured` | Decoding is charged against an explicit decoder work budget, and exhausting the budget is a typed refusal with no partial output. |
| `UC-WIRE-01` | `build` | `build_evidence` | `not_measured` | The client decodes only complete self-contained UORC archives in their canonical layout: truncated archives, trailing bytes and unsupported format identities are typed decode failures. |
| `UC-OBJ-01` | `open` | `none` | `not_measured` | Decoded output preserves the exact ordered object boundary vector, including zero objects and distinct zero-length objects. Open: the shipped client archives one object per archive; multi-object archives are not shipped yet. |
| `UC-INT-01` | `build` | `build_evidence` | `not_measured` | Every archive the client decodes is checked against its UORC-SHA256-CHAIN-v1 integrity trailer, so changing any single bit of an archive makes it fail to decode. |
| `UC-BAS-01` | `open` | `none` | `not_measured` | Conditional archives bind the actual supplied ordered basis using a source-owned length-delimited domain-separated identity, and execute against that admitted basis. Open: the shipped client writes and reads self-contained archives only; conditional-basis archives are not shipped yet. |
| `UC-SYN-01` | `build` | `build_evidence` | `not_measured` | Program search under a step budget keeps a candidate only as a fully checked archive: more effort can change which archive wins, never whether the result decodes to the input. |
| `UC-OPT-01` | `build` | `build_evidence` | `not_measured` | Compress returns the smallest of the archives it checked, which are the ulz kernel archive, the raw archive and any search candidate within the step budget; no minimum beyond that candidate set is claimed. |
| `UC-API-01` | `build` | `build_evidence` | `not_measured` | The uorc library, CLI and WebAssembly exports run the same exported compress and decompress over arbitrary bytes, with typed budget, overflow and decode failures; inspect and verify operations are not part of this interface yet. |
| `UC-REAL-01` | `build` | `fixture_result` | `not_measured` | Real archives that uorc compress writes for text, structured binary, noise and edge-case inputs are independently decoded and compared against every original byte. |
| `UC-SHA-01` | `build` | `fixture_result` | `not_measured` | Incremental SHA-256 and the separately named bounded chain have source-owned streaming semantics, typed failures and independently locked byte-oriented CAVP oracle evidence. |
| `AUTH-NIST-SHA256-SHORT` | `some-true` | `imported_fact` | `not_measured` | The locked NIST CAVP SHA-256 short byte-oriented response data supplies 65 independent expected digests, reproduced as finite oracle checks. |
| `AUTH-NIST-SHA256-LONG` | `some-true` | `imported_fact` | `not_measured` | The locked NIST CAVP SHA-256 long byte-oriented response data supplies 64 independent expected digests, reproduced as finite oracle checks. |
| `UC-SHA-02` | `build` | `fixture_result` | `not_measured` | The distinct UORC-SHA256-STREAM-MAX-v1 construction streams arbitrary finite input with standard-domain frame bounds, typed cancellation, explicit terminal framing and independently checked reduced-capacity segmentation controls. |
| `UC-INT-02` | `build` | `fixture_result` | `not_measured` | Format3/machine1 explicitly binds STREAM-MAX-v1 to charged image spans, preserves format2 CHAIN-v1 semantics, and seals input, basis and checkpoint identity domains without replacing exact reconstruction comparison. |
| `UC-INT-03` | `build` | `build_evidence` | `not_measured` | Source-owned checker soundness and constructive finite backward reduction are kernel checked for actual canonical framed messages and shape-valid source-SHA-authenticated traces, without global hash injectivity or an assumed frame codec. The decoder-to-authenticated-trace bridge and full product acceptance remain distinct. |
| `UC-INT-04` | `build` | `build_evidence` | `not_measured` | Source-owned actual-call relations establish ordered hash admissions, drained selected-view EOF and checked cumulative basis/main work; composed generated-model controls reject failed-prefix resets and late corruption. Consumer snapshot preservation, canonical digest refinement and complete archive EOF remain separate obligations. |
| `UC-INT-05` | `build` | `build_evidence` | `not_measured` | Source-owned completed-view and continuous publication proofs derive the actual chosen-profile digest and emitted-image reads, discharging canonical driver termination and OwnedHash control/cleanup lifetime for both explicit format identities and arbitrary finite source input. Initial source-layout correspondence and full product acceptance remain separate consumer obligations. |
