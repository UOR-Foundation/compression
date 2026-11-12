# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: performance-theorems

  @UC-PERF-02 @build
  Scenario: For every input length n, the settled frame of Uorc.Performance is no larger than the raw frame and no larger than a coded kernel candidate, and the raw frame is two length varints of at most 10 bytes each, the n input bytes and 41 fixed bytes, so no frame of one object exceeds n + 61 bytes (archiveBound), and a stream of k such frames is within n + 61 * k bytes; the native client keeps the same rule and the same constants.
    Given the Uorc.Performance module and the native client's frame writer
    When the locked SDK verifies the module and the native test compresses inputs of several lengths and kinds, including incompressible ones
    Then archiveBound and its lemmas are kernel-checked, every native raw frame has the model's length, and every native frame is within n + 61 bytes

  @UC-PERF-03 @build
  Scenario: For every list of trials, the trial the portfolio keeps (Uorc.Performance best) is no larger in bytes than any coded trial in the list (bestNoLargerThanEvery), so a portfolio archive is no larger than the archive of any strategy it scheduled.
    Given the Uorc.Performance module and the native client's strategy portfolio
    When the locked SDK verifies the module and the native test codes sample frames under each scheduled strategy and under the portfolio
    Then bestNoLargerThanEvery is kernel-checked and the native portfolio frame is no larger than the frame of every strategy it scheduled

  @UC-PERF-04 @build
  Scenario: A step is one model input evaluated for one coded bit. For every profile word the input count lies between 3 and 24 (17 context bits, 3 fixed inputs, 2 for each of the short-match and repeat features), a frame of n bytes costs 8 * n * inputs steps, and for every frame kind (raw, reference, ncm, ncm from a named state) decode takes at most 192 steps per modelled byte (decodeStepsBound); the full profile costs 192 steps per byte and the v0.3 profile 160. The measure counts model inputs, not machine time.
    Given the Uorc.Performance module and the native step measure in uorc::performance
    When the locked SDK verifies the module and the native test compares the native measure with the model's constants and closed vectors for sample profiles
    Then the step bound theorems are kernel-checked, every native constant equals the model's, and the native step count of every sampled frame is within 192 steps per byte

  @UC-PERF-05 @build
  Scenario: A trial carries its defect, the count of bytes at which its decode differs from the input. Admission fails every trial with a nonzero defect, keeping never raises a defect above its operands', and the trial kept from any admitted list has defect 0 (bestOfAdmittedIsExact): whatever codec point or named model state produced a trial, it can change the archive's size and never its meaning.
    Given the Uorc.Performance module and the native client's candidate admission
    When the locked SDK verifies the module and the native test decodes every frame it produced under each strategy
    Then bestOfAdmittedIsExact and its lemmas are kernel-checked and every native frame decodes to its input

  @UC-PERF-06 @build
  Scenario: A candidate dominates an incumbent when it is no larger in bytes and no larger in steps, and its gain is the drop in bytes plus steps. The gain never exceeds the drop in cost (dominanceLowersCost), no trial gains over itself, gains compose (a positive gain of a over b and of b over c gives at least their sum of a over c), and a positive gain implies no growth in bytes or in steps: dominance is a strict partial order, so consolidating winners cannot cycle.
    Given the Uorc.Performance module
    When the locked SDK verifies the module and the native test reads the module's theorem inventory
    Then the four dominance theorems are kernel-checked and present

  @UC-PERF-07 @build
  Scenario: A trial abandoned at a bound is modelled as a trial of at most the bound's bytes. For every list of trials and every raw length, the portfolio of trials abandoned at the raw length settles to the same size as the portfolio of complete trials (boundedPortfolioSettlesTheSame), and a trial no smaller than one already kept changes neither the kept bytes nor the kept steps (skippedTrial theorems): a search that stops a candidate once it cannot win, or skips one known to be no smaller, writes an archive of the same size for fewer compress steps.
    Given the Uorc.Performance module and the native encoder's bound
    When the locked SDK verifies the module and the native test abandons each scheduled strategy at several bounds
    Then the bounded search theorems are kernel-checked, a native payload abandoned at a bound is the complete payload or is absent only when the complete payload reaches the bound, and the native frame is no larger than any complete scheduled frame or the raw frame

  @UC-PERF-08 @build
  Scenario: Choosing by fewer bytes and then fewer steps, from the raw frame and any list of trials, keeps a trial that is no later in that order than every trial in the list and than the raw frame (chosenIsLeast, chosenIsNoWorseThanRaw), has exactly the size the bytes-only rule settles to (chosenKeepsTheSize), and is dominated by no trial it is no later than (leastIsUndominated); a trial no earlier than the kept one changes nothing (excludedTrial theorems). The rule lowers decode steps on byte ties at no cost in size. The released client breaks byte ties by schedule order until its model export adopts this rule.
    Given the Uorc.Performance module and the native choice functions in uorc::performance
    When the locked SDK verifies the module and the native test runs the native choice over every list of three small trials
    Then the choice theorems are kernel-checked and the native choice is least in bytes then steps, has the bytes-only size, and is dominated by no listed trial

  @UC-PERF-09 @build
  Scenario: The potential of a codec point is its bytes plus its steps. Consolidating a candidate into an incumbent lowers the potential by exactly the candidate's gain (consolidationSpendsItsGain), which is positive exactly when the candidate dominates, and over any sequence of candidates the potential never rises (potentialNeverRises). The potential is a natural number, so an engine that consolidates only dominating candidates never regresses and consolidates at most as many times as its first potential.
    Given the Uorc.Performance module and the native consolidation in uorc::performance
    When the locked SDK verifies the module and the native test consolidates every pair and sequence of small trials
    Then the potential theorems are kernel-checked and every native consolidation lowers bytes plus steps by exactly its gain

  @UC-PERF-10 @build
  Scenario: The encoder's stop is a ledger over its coder events: its count of emitted bytes saturates at the cap (ledgerSaturatesAtTheCap), so a payload below the cap is exactly the unbounded payload, four header bytes, the emitted bytes and one final byte (stopKeepsEveryPayloadBelowTheCap), and a candidate is abandoned exactly when its emitted bytes reach the cap and at no other point (stopAbandonsExactlyAtTheCap). A coded bit at a 12-bit probability narrows the coder's width by at least one and by at least one part in 4096 (aOneNarrowsTheWidth, aOneSpendsAFraction, aZeroNarrowsTheWidth, aZeroSpendsAFraction). No exact rule decides a raw frame earlier than its ledger does (exactRawDecision): until a candidate's ledger reaches the cap, a continuation the model codes in few bytes can still make the coded frame the smaller one. Measured: random bytes code to 0.28 to 0.30 percent over raw under every shipped profile, so on random input the earliest exact stop comes at about 99.7 percent of the frame.
    Given the Uorc.Performance module and the native ncm encoder
    When the locked SDK verifies the module and the native test abandons each shipped profile at every stop around its payload length and codes random bytes
    Then the ledger theorems are kernel-checked, the native encoder returns the unbounded payload below the cap and nothing at or past it, and random bytes code to more than raw

  @UC-PERF-11 @build
  Scenario: Input the model cannot shrink costs one coder pass per scheduled candidate, so that cost falls only through passes that write the same bytes in less time. Strategies whose profiles differ only in mixer selection read the same contexts, slots, match and repeat state and mixers 2 and 3, so when no core is spare they are coded in one shared pass that writes each strategy's own payload (ncm::encode_pair_below). It changes no byte: the shared pass equals the two separate passes in both directions of the pair and when either side is abandoned, and the client's archives do not depend on the thread count.
    Given the native ncm encoder and the client's portfolio
    When the native test codes text and random bytes under each twin pair in one shared pass and in separate passes, with each side abandoned in turn, and compresses with one thread and with several
    Then every shared-pass payload equals its separate pass and the archives are the same at every thread count

  @UC-PERF-16 @build
  Scenario: A frame's window facets step on its chosen archive (Client.stepWindow): its coding is recoded over one more earlier frame of its block at a time, then with one more match table bit at a time, and each step is kept only while the archive gets strictly shorter and the decoder fits the declared decode host; each facet stops at the first step that does not pay or does not fit, so no depth or table size is fixed. A frame is therefore the portfolio's archive byte for byte or a strictly shorter one, and it decodes to the frame.
    Given the native client's window step and its decoders
    When the native test compresses a frame that repeats the oldest frame of its block and a frame no earlier frame holds
    Then the first is coded over all three earlier frames and is strictly smaller than over the last two, the second is the portfolio's archive or a strictly shorter one, and both decode

  @UC-PERF-17 @build
  Scenario: The profiled window entry over k earlier frames is one form for every k of at least 1 (Archive.profilePrefix, Archive.profileTail): three graph nodes per frame concatenate basis roots 1 to k in order, and the call to the coder reaches its payload and profile word three nodes further back per frame. For one and two frames it is the fixed entry byte for byte, and the decoder reads k from the entry, so window depth is a facet of the archive rather than a constant of the format.
    Given the native window entry template and the native window decoder
    When the native test builds entries over 1 to 6 frames, with and without the x86 filter
    Then the one- and two-frame entries equal the fixed ones, and every deeper entry names its depth and decodes to its frame
