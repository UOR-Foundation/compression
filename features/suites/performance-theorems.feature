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
