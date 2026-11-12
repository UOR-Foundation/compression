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

## codec-engine

| ID | Level | Statement |
| --- | --- | --- |
| `UC-ENG-01` | `build` | The uorc engine moves the codec through its space without an LLM in the loop, under the laws of Uorc.Engine, which are kernel-checked: rule 30 on a 64-cell ring stays in the ring, search and portfolio weights stay within 1..16 under the bounded Hebbian rule (+2, -1), the winner of a proved dominance is scheduled before the incumbent it beat, dominance in bytes and steps is irreflexive, asymmetric and transitive and never costs bytes, a claim whose kappa is decided is never minted again, and the generated workflow's schedule and budgets are valid. Each tick mints candidates next to each family's incumbent by the rule 30 register and the search weights, decides them over the kappa-named corpus, and ships a moved portfolio only when it dominates the shipped one; the shipped table and the workflow are generated from the engine state and the model. |

## complete-synthesis

| ID | Level | Statement |
| --- | --- | --- |
| `UC-SYN-01` | `build` | Program search under a step budget keeps a candidate only as a fully checked archive: more effort can change which archive wins, never whether the result decodes to the input. |

## conditional-basis

| ID | Level | Statement |
| --- | --- | --- |
| `UC-BAS-01` | `build` | Conditional archives name the client's standard basis, the ulz and ncm decoder libraries, by its UORC-BASIS-V1 identity and decode only against that actual basis. |

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

## neuromorphic-standard

| ID | Level | Statement |
| --- | --- | --- |
| `UC-NEU-01` | `build` | The ncm coder's plasticity is stated as the Prism neuromorphic standard in Uorc.Neuromorphic, with kernel-checked bounds: every mixer weight stays within plus or minus 2^22 under the bounded Hebbian rule, potentiation never lowers and depression never raises a weight, the plasticity lies between 24 and 128 per 65536 (128 at the first byte, 24 in the limit), one bit's weight change is at most 16376 and every product fits a 32-bit lane, a context slot never steps past its gap to the coded bit and consolidates at count 127, and every probability-map point stays within 16 bits; the native coder reads the model's constants. |

## ordered-objects

| ID | Level | Statement |
| --- | --- | --- |
| `UC-OBJ-01` | `build` | Archives of ordered objects preserve the exact object boundary vector, including zero objects and distinct zero-length objects. |

## pages-client

| ID | Level | Statement |
| --- | --- | --- |
| `UC-WEB-01` | `build` | The Pages client is generated from and conforms to its Prism product model in Uorc.Page: stakeholders, features with their calls and acceptance cases, journeys, the page's UML state machine, operations, fields, controls, examples, error hints, sections, usage, links and quality measures. The generated page holds exactly the modeled elements, and a real browser walks every journey, takes every modeled transition, checks every control's enabled state in every state it reaches and holds every gated quality measure. |
| `UC-WEB-02` | `build` | The product shows the codec as a point of its modelled space: uorc inspect, the library's Info JSON and the Pages client name each frame's codec point (basis, ncm profile word, consolidated model state, lanes and its κ) and the exact steps its decode takes, as Uorc.Performance defines them, from the header alone; the page's model view shows the release's performance theorems, its closed claims with codec point, corpus κ, bytes, steps and the factory's decision, and the engine's tick, rule30 register and weights, exactly as the release publishes them in model.json. |

## performance-theorems

| ID | Level | Statement |
| --- | --- | --- |
| `UC-PERF-02` | `build` | For every input length n, the settled frame of Uorc.Performance is no larger than the raw frame and no larger than a coded kernel candidate, and the raw frame is two length varints of at most 10 bytes each, the n input bytes and 41 fixed bytes, so no frame of one object exceeds n + 61 bytes (archiveBound), and a stream of k such frames is within n + 61 * k bytes; the native client keeps the same rule and the same constants. |
| `UC-PERF-03` | `build` | For every list of trials, the trial the portfolio keeps (Uorc.Performance best) is no larger in bytes than any coded trial in the list (bestNoLargerThanEvery), so a portfolio archive is no larger than the archive of any strategy it scheduled. |
| `UC-PERF-04` | `build` | A step is one model input evaluated for one coded bit. For every profile word the input count lies between 3 and 24 (17 context bits, 3 fixed inputs, 2 for each of the short-match and repeat features), a frame of n bytes costs 8 * n * inputs steps, and for every frame kind (raw, reference, ncm, ncm from a named state) decode takes at most 192 steps per modelled byte (decodeStepsBound); the full profile costs 192 steps per byte and the v0.3 profile 160. The measure counts model inputs, not machine time. |
| `UC-PERF-05` | `build` | A trial carries its defect, the count of bytes at which its decode differs from the input. Admission fails every trial with a nonzero defect, keeping never raises a defect above its operands', and the trial kept from any admitted list has defect 0 (bestOfAdmittedIsExact): whatever codec point or named model state produced a trial, it can change the archive's size and never its meaning. |
| `UC-PERF-06` | `build` | A candidate dominates an incumbent when it is no larger in bytes and no larger in steps, and its gain is the drop in bytes plus steps. The gain never exceeds the drop in cost (dominanceLowersCost), no trial gains over itself, gains compose (a positive gain of a over b and of b over c gives at least their sum of a over c), and a positive gain implies no growth in bytes or in steps: dominance is a strict partial order, so consolidating winners cannot cycle. |
| `UC-PERF-07` | `build` | A trial abandoned at a bound is modelled as a trial of at most the bound's bytes. For every list of trials and every raw length, the portfolio of trials abandoned at the raw length settles to the same size as the portfolio of complete trials (boundedPortfolioSettlesTheSame), and a trial no smaller than one already kept changes neither the kept bytes nor the kept steps (skippedTrial theorems): a search that stops a candidate once it cannot win, or skips one known to be no smaller, writes an archive of the same size for fewer compress steps. |
| `UC-PERF-08` | `build` | Choosing by fewer bytes and then fewer steps, from the raw frame and any list of trials, keeps a trial that is no later in that order than every trial in the list and than the raw frame (chosenIsLeast, chosenIsNoWorseThanRaw), has exactly the size the bytes-only rule settles to (chosenKeepsTheSize), and is dominated by no trial it is no later than (leastIsUndominated); a trial no earlier than the kept one changes nothing (excludedTrial theorems). The rule lowers decode steps on byte ties at no cost in size. The client's choice is this rule, in the model (Client.pick) and natively (compress_is_the_chosen_trial_uc_perf_08). |
| `UC-PERF-09` | `build` | The potential of a codec point is its bytes plus its steps. Consolidating a candidate into an incumbent lowers the potential by exactly the candidate's gain (consolidationSpendsItsGain), which is positive exactly when the candidate dominates, and over any sequence of candidates the potential never rises (potentialNeverRises). The potential is a natural number, so an engine that consolidates only dominating candidates never regresses and consolidates at most as many times as its first potential. |
| `UC-PERF-10` | `build` | The encoder's stop is a ledger over its coder events: its count of emitted bytes saturates at the cap (ledgerSaturatesAtTheCap), so a payload below the cap is exactly the unbounded payload, four header bytes, the emitted bytes and one final byte (stopKeepsEveryPayloadBelowTheCap), and a candidate is abandoned exactly when its emitted bytes reach the cap and at no other point (stopAbandonsExactlyAtTheCap). A coded bit at a 12-bit probability narrows the coder's width by at least one and by at least one part in 4096 (aOneNarrowsTheWidth, aOneSpendsAFraction, aZeroNarrowsTheWidth, aZeroSpendsAFraction). No exact rule decides a raw frame earlier than its ledger does (exactRawDecision): until a candidate's ledger reaches the cap, a continuation the model codes in few bytes can still make the coded frame the smaller one. Measured: random bytes code to 0.28 to 0.30 percent over raw under every shipped profile, so on random input the earliest exact stop comes at about 99.7 percent of the frame. |
| `UC-PERF-11` | `build` | Input the model cannot shrink costs one coder pass per scheduled candidate, so that cost falls only through passes that write the same bytes in less time. Strategies whose profiles differ only in mixer selection read the same contexts, slots, match and repeat state and mixers 2 and 3, so when no core is spare they are coded in one shared pass that writes each strategy's own payload (ncm::encode_pair_below). It changes no byte: the shared pass equals the two separate passes in both directions of the pair and when either side is abandoned, and the client's archives do not depend on the thread count. |
| `UC-PERF-16` | `build` | A frame's window facets step on its chosen archive (Client.stepWindow): its coding is recoded over one more earlier frame of its block at a time, then with one more match table bit at a time, and each step is kept only while the archive gets strictly shorter and the decoder fits the declared decode host; each facet stops at the first step that does not pay or does not fit, so no depth or table size is fixed. A frame is therefore the portfolio's archive byte for byte or a strictly shorter one, and it decodes to the frame. |
| `UC-PERF-17` | `build` | The profiled window entry over k earlier frames is one form for every k of at least 1 (Archive.profilePrefix, Archive.profileTail): three graph nodes per frame concatenate basis roots 1 to k in order, and the call to the coder reaches its payload and profile word three nodes further back per frame. For one and two frames it is the fixed entry byte for byte, and the decoder reads k from the entry, so window depth is a facet of the archive rather than a constant of the format. |

## product-interfaces

| ID | Level | Statement |
| --- | --- | --- |
| `UC-API-01` | `build` | The uorc library, CLI and WebAssembly exports run the same compress, decompress, inspect and verify operations over arbitrary bytes, with typed budget, overflow, decode and mismatch failures. |

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
| `UC-OPT-01` | `build` | Compress returns the smallest of the archives it checked, which are the raw archive, the ncm archive (over the x86 branch filter for x86 code) and any search candidate within the step budget; no minimum beyond that candidate set is claimed. |

## source-authority

| ID | Level | Statement |
| --- | --- | --- |
| `UC-CHR-01` | `build` | UORC charter, claim-boundary, evidence-boundary, and research-position records are accepted by the locked LexLean project and generate the committed project documentation projections. |
| `UC-CHR-02` | `build` | Handwritten project prose cannot define a second UORC specification or BCP-14 semantic rule, while non-authoritative evidence and planning prose remain permitted. |
| `UC-PERF-01` | `open` | closed public-corpus claims with a target: every release runs crates/frontier over the Canterbury and Silesia corpora, each pinned by its κ in frontier.toml, records the exact archive bytes and decode steps of its codec point as closed claims the factory decides, and publishes them in the release notes and the release's frontier.toml; the target is that on every public corpus the archive is no larger than each pinned comparator's (gzip, bzip2, xz, zstd, brotli, zpaq and 7-Zip PPMd from the v0.3.0 release run), decided per comparator as a bound claim; open with the target unmet while any bound is refuted (Silesia against zpaq -m5, 41726363 against 39112562 bytes at v0.3.0); scripts/bench.mjs still reports times against the host's core count and memory copy rate |
| `UC-REC-01` | `open` | externally accepted compression record; open unless the external authority actually accepts it |

## varint

| ID | Level | Statement |
| --- | --- | --- |
| `UC-VAR-01` | `build` | Shortest U64 ULEB128 encoding and strict decoding have source-owned domain and rejection rules, complete boundary vectors, and kernel-checked round-trip and size theorems. |

## wire-format

| ID | Level | Statement |
| --- | --- | --- |
| `UC-WIRE-01` | `build` | The client decodes only complete UORC archives in their canonical layout, self-contained or over the standard basis: truncated archives, trailing bytes and unsupported format identities are typed decode failures. |

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
| `UC-PERF-01` | `open` | `benchmark_target_unmet` | `measured_target_unmet` | closed public-corpus claims with a target: every release runs crates/frontier over the Canterbury and Silesia corpora, each pinned by its κ in frontier.toml, records the exact archive bytes and decode steps of its codec point as closed claims the factory decides, and publishes them in the release notes and the release's frontier.toml; the target is that on every public corpus the archive is no larger than each pinned comparator's (gzip, bzip2, xz, zstd, brotli, zpaq and 7-Zip PPMd from the v0.3.0 release run), decided per comparator as a bound claim; open with the target unmet while any bound is refuted (Silesia against zpaq -m5, 41726363 against 39112562 bytes at v0.3.0); scripts/bench.mjs still reports times against the host's core count and memory copy rate |
| `UC-REC-01` | `open` | `none` | `not_measured` | externally accepted compression record; open unless the external authority actually accepts it |
| `UC-REP-01` | `build` | `build_evidence` | `not_measured` | Source-derived charter projections and complete LexLean build and verification artifacts are byte-identical across clean absolute roots under the locked SDK. |
| `UC-REP-02` | `build` | `build_evidence` | `not_measured` | Dependency and execution-boundary failures are explicit and planted SDK-lock, source-lock, adjacent-checkout, and host-PATH fallbacks are rejected. |
| `UC-HON-01` | `build` | `build_evidence` | `not_measured` | Evidence classes and research dispositions remain distinct in the LexLean claim policy. |
| `UC-HON-02` | `build` | `build_evidence` | `not_measured` | The honesty ledger and claim summaries are exact source projections with open research claims protected against promotion. |
| `UC-VAR-01` | `build` | `kernel_theorem` | `not_measured` | Shortest U64 ULEB128 encoding and strict decoding have source-owned domain and rejection rules, complete boundary vectors, and kernel-checked round-trip and size theorems. |
| `UC-MACH-01` | `build` | `build_evidence` | `not_measured` | The client's reference-semantics evaluator decodes program archives, including search-found programs and the ulz kernel graph when the certified fast path declines, to their exact original bytes. |
| `UC-MACH-02` | `build` | `build_evidence` | `not_measured` | Decoding is charged against an explicit decoder work budget, and exhausting the budget is a typed refusal with no partial output. |
| `UC-WIRE-01` | `build` | `build_evidence` | `not_measured` | The client decodes only complete UORC archives in their canonical layout, self-contained or over the standard basis: truncated archives, trailing bytes and unsupported format identities are typed decode failures. |
| `UC-OBJ-01` | `build` | `build_evidence` | `not_measured` | Archives of ordered objects preserve the exact object boundary vector, including zero objects and distinct zero-length objects. |
| `UC-INT-01` | `build` | `build_evidence` | `not_measured` | Every archive the client decodes is checked against its UORC-SHA256-CHAIN-v1 integrity trailer, so changing any single bit of an archive makes it fail to decode. |
| `UC-BAS-01` | `build` | `build_evidence` | `not_measured` | Conditional archives name the client's standard basis, the ulz and ncm decoder libraries, by its UORC-BASIS-V1 identity and decode only against that actual basis. |
| `UC-SYN-01` | `build` | `build_evidence` | `not_measured` | Program search under a step budget keeps a candidate only as a fully checked archive: more effort can change which archive wins, never whether the result decodes to the input. |
| `UC-OPT-01` | `build` | `build_evidence` | `not_measured` | Compress returns the smallest of the archives it checked, which are the raw archive, the ncm archive (over the x86 branch filter for x86 code) and any search candidate within the step budget; no minimum beyond that candidate set is claimed. |
| `UC-API-01` | `build` | `build_evidence` | `not_measured` | The uorc library, CLI and WebAssembly exports run the same compress, decompress, inspect and verify operations over arbitrary bytes, with typed budget, overflow, decode and mismatch failures. |
| `UC-REAL-01` | `build` | `fixture_result` | `not_measured` | Real archives that uorc compress writes for text, structured binary, noise and edge-case inputs are independently decoded and compared against every original byte. |
| `UC-SHA-01` | `build` | `fixture_result` | `not_measured` | Incremental SHA-256 and the separately named bounded chain have source-owned streaming semantics, typed failures and independently locked byte-oriented CAVP oracle evidence. |
| `AUTH-NIST-SHA256-SHORT` | `some-true` | `imported_fact` | `not_measured` | The locked NIST CAVP SHA-256 short byte-oriented response data supplies 65 independent expected digests, reproduced as finite oracle checks. |
| `AUTH-NIST-SHA256-LONG` | `some-true` | `imported_fact` | `not_measured` | The locked NIST CAVP SHA-256 long byte-oriented response data supplies 64 independent expected digests, reproduced as finite oracle checks. |
| `UC-SHA-02` | `build` | `fixture_result` | `not_measured` | The distinct UORC-SHA256-STREAM-MAX-v1 construction streams arbitrary finite input with standard-domain frame bounds, typed cancellation, explicit terminal framing and independently checked reduced-capacity segmentation controls. |
| `UC-INT-02` | `build` | `fixture_result` | `not_measured` | Format3/machine1 explicitly binds STREAM-MAX-v1 to charged image spans, preserves format2 CHAIN-v1 semantics, and seals input, basis and checkpoint identity domains without replacing exact reconstruction comparison. |
| `UC-INT-03` | `build` | `build_evidence` | `not_measured` | Source-owned checker soundness and constructive finite backward reduction are kernel checked for actual canonical framed messages and shape-valid source-SHA-authenticated traces, without global hash injectivity or an assumed frame codec. The decoder-to-authenticated-trace bridge and full product acceptance remain distinct. |
| `UC-INT-04` | `build` | `build_evidence` | `not_measured` | Source-owned actual-call relations establish ordered hash admissions, drained selected-view EOF and checked cumulative basis/main work; composed generated-model controls reject failed-prefix resets and late corruption. Consumer snapshot preservation, canonical digest refinement and complete archive EOF remain separate obligations. |
| `UC-INT-05` | `build` | `build_evidence` | `not_measured` | Source-owned completed-view and continuous publication proofs derive the actual chosen-profile digest and emitted-image reads, discharging canonical driver termination and OwnedHash control/cleanup lifetime for both explicit format identities and arbitrary finite source input. Initial source-layout correspondence and full product acceptance remain separate consumer obligations. |
| `UC-NEU-01` | `build` | `kernel_theorem` | `not_measured` | The ncm coder's plasticity is stated as the Prism neuromorphic standard in Uorc.Neuromorphic, with kernel-checked bounds: every mixer weight stays within plus or minus 2^22 under the bounded Hebbian rule, potentiation never lowers and depression never raises a weight, the plasticity lies between 24 and 128 per 65536 (128 at the first byte, 24 in the limit), one bit's weight change is at most 16376 and every product fits a 32-bit lane, a context slot never steps past its gap to the coded bit and consolidates at count 127, and every probability-map point stays within 16 bits; the native coder reads the model's constants. |
| `UC-ENG-01` | `build` | `kernel_theorem` | `not_measured` | The uorc engine moves the codec through its space without an LLM in the loop, under the laws of Uorc.Engine, which are kernel-checked: rule 30 on a 64-cell ring stays in the ring, search and portfolio weights stay within 1..16 under the bounded Hebbian rule (+2, -1), the winner of a proved dominance is scheduled before the incumbent it beat, dominance in bytes and steps is irreflexive, asymmetric and transitive and never costs bytes, a claim whose kappa is decided is never minted again, and the generated workflow's schedule and budgets are valid. Each tick mints candidates next to each family's incumbent by the rule 30 register and the search weights, decides them over the kappa-named corpus, and ships a moved portfolio only when it dominates the shipped one; the shipped table and the workflow are generated from the engine state and the model. |
| `UC-WEB-01` | `build` | `build_evidence` | `not_measured` | The Pages client is generated from and conforms to its Prism product model in Uorc.Page: stakeholders, features with their calls and acceptance cases, journeys, the page's UML state machine, operations, fields, controls, examples, error hints, sections, usage, links and quality measures. The generated page holds exactly the modeled elements, and a real browser walks every journey, takes every modeled transition, checks every control's enabled state in every state it reaches and holds every gated quality measure. |
| `UC-PERF-02` | `build` | `kernel_theorem` | `not_measured` | For every input length n, the settled frame of Uorc.Performance is no larger than the raw frame and no larger than a coded kernel candidate, and the raw frame is two length varints of at most 10 bytes each, the n input bytes and 41 fixed bytes, so no frame of one object exceeds n + 61 bytes (archiveBound), and a stream of k such frames is within n + 61 * k bytes; the native client keeps the same rule and the same constants. |
| `UC-PERF-03` | `build` | `kernel_theorem` | `not_measured` | For every list of trials, the trial the portfolio keeps (Uorc.Performance best) is no larger in bytes than any coded trial in the list (bestNoLargerThanEvery), so a portfolio archive is no larger than the archive of any strategy it scheduled. |
| `UC-PERF-04` | `build` | `kernel_theorem` | `not_measured` | A step is one model input evaluated for one coded bit. For every profile word the input count lies between 3 and 24 (17 context bits, 3 fixed inputs, 2 for each of the short-match and repeat features), a frame of n bytes costs 8 * n * inputs steps, and for every frame kind (raw, reference, ncm, ncm from a named state) decode takes at most 192 steps per modelled byte (decodeStepsBound); the full profile costs 192 steps per byte and the v0.3 profile 160. The measure counts model inputs, not machine time. |
| `UC-PERF-05` | `build` | `kernel_theorem` | `not_measured` | A trial carries its defect, the count of bytes at which its decode differs from the input. Admission fails every trial with a nonzero defect, keeping never raises a defect above its operands', and the trial kept from any admitted list has defect 0 (bestOfAdmittedIsExact): whatever codec point or named model state produced a trial, it can change the archive's size and never its meaning. |
| `UC-PERF-06` | `build` | `kernel_theorem` | `not_measured` | A candidate dominates an incumbent when it is no larger in bytes and no larger in steps, and its gain is the drop in bytes plus steps. The gain never exceeds the drop in cost (dominanceLowersCost), no trial gains over itself, gains compose (a positive gain of a over b and of b over c gives at least their sum of a over c), and a positive gain implies no growth in bytes or in steps: dominance is a strict partial order, so consolidating winners cannot cycle. |
| `UC-PERF-07` | `build` | `kernel_theorem` | `not_measured` | A trial abandoned at a bound is modelled as a trial of at most the bound's bytes. For every list of trials and every raw length, the portfolio of trials abandoned at the raw length settles to the same size as the portfolio of complete trials (boundedPortfolioSettlesTheSame), and a trial no smaller than one already kept changes neither the kept bytes nor the kept steps (skippedTrial theorems): a search that stops a candidate once it cannot win, or skips one known to be no smaller, writes an archive of the same size for fewer compress steps. |
| `UC-PERF-08` | `build` | `kernel_theorem` | `not_measured` | Choosing by fewer bytes and then fewer steps, from the raw frame and any list of trials, keeps a trial that is no later in that order than every trial in the list and than the raw frame (chosenIsLeast, chosenIsNoWorseThanRaw), has exactly the size the bytes-only rule settles to (chosenKeepsTheSize), and is dominated by no trial it is no later than (leastIsUndominated); a trial no earlier than the kept one changes nothing (excludedTrial theorems). The rule lowers decode steps on byte ties at no cost in size. The client's choice is this rule, in the model (Client.pick) and natively (compress_is_the_chosen_trial_uc_perf_08). |
| `UC-PERF-09` | `build` | `kernel_theorem` | `not_measured` | The potential of a codec point is its bytes plus its steps. Consolidating a candidate into an incumbent lowers the potential by exactly the candidate's gain (consolidationSpendsItsGain), which is positive exactly when the candidate dominates, and over any sequence of candidates the potential never rises (potentialNeverRises). The potential is a natural number, so an engine that consolidates only dominating candidates never regresses and consolidates at most as many times as its first potential. |
| `UC-PERF-10` | `build` | `kernel_theorem` | `not_measured` | The encoder's stop is a ledger over its coder events: its count of emitted bytes saturates at the cap (ledgerSaturatesAtTheCap), so a payload below the cap is exactly the unbounded payload, four header bytes, the emitted bytes and one final byte (stopKeepsEveryPayloadBelowTheCap), and a candidate is abandoned exactly when its emitted bytes reach the cap and at no other point (stopAbandonsExactlyAtTheCap). A coded bit at a 12-bit probability narrows the coder's width by at least one and by at least one part in 4096 (aOneNarrowsTheWidth, aOneSpendsAFraction, aZeroNarrowsTheWidth, aZeroSpendsAFraction). No exact rule decides a raw frame earlier than its ledger does (exactRawDecision): until a candidate's ledger reaches the cap, a continuation the model codes in few bytes can still make the coded frame the smaller one. Measured: random bytes code to 0.28 to 0.30 percent over raw under every shipped profile, so on random input the earliest exact stop comes at about 99.7 percent of the frame. |
| `UC-PERF-11` | `build` | `build_evidence` | `not_measured` | Input the model cannot shrink costs one coder pass per scheduled candidate, so that cost falls only through passes that write the same bytes in less time. Strategies whose profiles differ only in mixer selection read the same contexts, slots, match and repeat state and mixers 2 and 3, so when no core is spare they are coded in one shared pass that writes each strategy's own payload (ncm::encode_pair_below). It changes no byte: the shared pass equals the two separate passes in both directions of the pair and when either side is abandoned, and the client's archives do not depend on the thread count. |
| `UC-PERF-16` | `build` | `build_evidence` | `not_measured` | A frame's window facets step on its chosen archive (Client.stepWindow): its coding is recoded over one more earlier frame of its block at a time, then with one more match table bit at a time, and each step is kept only while the archive gets strictly shorter and the decoder fits the declared decode host; each facet stops at the first step that does not pay or does not fit, so no depth or table size is fixed. A frame is therefore the portfolio's archive byte for byte or a strictly shorter one, and it decodes to the frame. |
| `UC-PERF-17` | `build` | `build_evidence` | `not_measured` | The profiled window entry over k earlier frames is one form for every k of at least 1 (Archive.profilePrefix, Archive.profileTail): three graph nodes per frame concatenate basis roots 1 to k in order, and the call to the coder reaches its payload and profile word three nodes further back per frame. For one and two frames it is the fixed entry byte for byte, and the decoder reads k from the entry, so window depth is a facet of the archive rather than a constant of the format. |
| `UC-WEB-02` | `build` | `build_evidence` | `not_measured` | The product shows the codec as a point of its modelled space: uorc inspect, the library's Info JSON and the Pages client name each frame's codec point (basis, ncm profile word, consolidated model state, lanes and its κ) and the exact steps its decode takes, as Uorc.Performance defines them, from the header alone; the page's model view shows the release's performance theorems, its closed claims with codec point, corpus κ, bytes, steps and the factory's decision, and the engine's tick, rule30 register and weights, exactly as the release publishes them in model.json. |
