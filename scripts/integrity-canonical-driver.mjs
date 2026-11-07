// Finite validation client over compiler-generated source models; no production hash.
const OWN_LENGTHS = Object.freeze([0, 1, 55, 56, 63, 64, 65, 91, 150, 209, 257]);
const OWN_WIDTHS = Object.freeze([1, 7, 64]);
const STREAM_LENGTHS = Object.freeze([0, 1, 72, 73, 74, 113, 114, 115, 154, 155, 156]);
const STREAM_WIDTHS = Object.freeze([1, 7, 64, 101]);
export const canonicalDriverInventory = Object.freeze({
  ownedLengths: OWN_LENGTHS, ownedWidths: OWN_WIDTHS,
  kinds: Object.freeze(['format2Chain', 'format3Stream']),
  purposes: Object.freeze(['inputBytes', 'basisObjects', 'checkpointBytes']),
  ownedCases: 198, existingFixtureCases: 60, profileComparisons: 258,
  streamLimit: 100, streamLengths: STREAM_LENGTHS, streamWidths: STREAM_WIDTHS,
  streamCases: 44,
});
const MAIN = 'private def main : IO Unit := do\n';
const HASH_CASE = 'private def hashCase (kind : Profile.Kind) (purpose : Profile.Purpose) (input expected : ByteArray) (width : Nat) : IO Unit := do\n';
const FIXTURE_CHECK = ' check (bytes==expected) "independent digest fixture"\n';
const STDOUT = ' IO.println s!"20 independent binding fixtures; {n} span partitions; 10 independent owned reconstructions; ticket, cancellation, copy, digest and source-owned positive/negative controls passed"\n';
function once(text, marker, label) {
  if (text.split(marker).length !== 2) throw new Error(`canonical driver requires exactly one ${label}`);
}
function array(values) { return `#[${values.join(',')}]`; }

export function extendCanonicalDriver(base) {
  if (typeof base !== 'string') throw new TypeError('canonical driver base must be a string');
  for (const [marker, label] of [[MAIN, 'main entry'], [HASH_CASE, 'actual OwnedHash hashCase'],
    [FIXTURE_CHECK, 'published image fixture comparison'], [STDOUT, 'unchanged stdout identity'],
    ['   hashCase kind purpose (hex input) (hex expected) width\n', 'existing fixture hashCase call'],
    [' let started ← get (OwnedHash.start image.arena kind purpose) \"start\"\n', 'actual source OwnedHash start'],
    [' let bytes ← readImage done.arena output 32\n', 'actual published image getter'],
    [' check (arena.ownedBytes==0 && arena.liveEntries==0) \"complete ownership cleanup\"\n', 'actual image cleanup'],
    [' check (orderedChecks.length == 12) "complete ordered source transcript inventory"\n', 'ordered source inventory'],
    [' check (orderedChecks.all id) "ordered source transcript controls"\n', 'ordered source gate'],
    ['end BindingOracle\n#eval BindingOracle.main\n', 'executed main']]) once(base, marker, label);
  if (base.includes('canonicalProfileComparison') || base.includes('canonicalOwnedCases') || base.includes('canonicalStreamCases')) {
    throw new Error('canonical driver was already extended or has conflicting validation names');
  }
  if (2 * 3 * OWN_LENGTHS.length * OWN_WIDTHS.length !== 198 ||
      STREAM_LENGTHS.length * STREAM_WIDTHS.length !== 44) throw new Error('canonical case inventory changed');

  const comparison = ` let canonicalProfileComparison ← get (Profile.hash kind purpose input) "canonical source Profile.hash"
 check (bytes==canonicalProfileComparison) "published OwnedHash image equals canonical Profile.hash"
${FIXTURE_CHECK}`;
  const helpers = `-- Finite diagnostic resources and fuel below are validation-client bounds.
-- These calls compare actual source engines; they do not establish a universal refinement.
private def canonicalPayload (length : Nat) : ByteArray := Id.run do
 let mut bytes := ByteArray.empty
 for i in [:length] do
  bytes := bytes.push (UInt8.ofNat (i % 256))
 return bytes
private def canonicalOwnedCases : IO Unit := do
 let mut count := 0
 for kind in #[Profile.Kind.format2Chain, Profile.Kind.format3Stream] do
  for purpose in #[Profile.Purpose.inputBytes, Profile.Purpose.basisObjects, Profile.Purpose.checkpointBytes] do
   for length in ${array(OWN_LENGTHS)} do
    let input := canonicalPayload length
    let expected ← get (Profile.hash kind purpose input) "finite canonical source Profile.hash"
    for width in ${array(OWN_WIDTHS)} do
     hashCase kind purpose input expected width
     count := count + 1
 check (count == 198) "complete finite OwnedHash canonical case inventory"
private def canonicalStreamCases : IO Unit := do
 let limit : UInt64 := 100
 let mut count := 0
 for length in ${array(STREAM_LENGTHS)} do
  let input := canonicalPayload length
  let expected ← get (StreamEngine.hashAt limit input) "diagnostic canonical StreamEngine.hashAt"
  for width in ${array(STREAM_WIDTHS)} do
   let initial ← get (StreamEngine.initAt limit) "diagnostic source initAt"
   let mut state ← get (StreamEngine.updateAt limit initial ByteArray.empty) "diagnostic empty updateAt"
   let mut offset := 0
   while offset < input.size do
    let size := min width (input.size - offset)
    state ← get (StreamEngine.updateAt limit state (input.extract offset (offset + size))) "diagnostic partition updateAt"
    offset := offset + size
   let closing ← get (StreamEngine.finalizeAt limit state) "diagnostic finalizeAt including empty terminal"
   let actual ← get (StreamEngine.runFinalizeAt 1200 limit closing) "diagnostic runFinalizeAt"
   check (actual == expected) "partitioned StreamEngine equals canonical hashAt"
   count := count + 1
 check (count == 44) "complete diagnostic StreamEngine case inventory"
`;
  const extended = base.replace(FIXTURE_CHECK, comparison).replace(MAIN,
    `${helpers}${MAIN} canonicalOwnedCases\n canonicalStreamCases\n`);
  for (const [marker, label] of [[STDOUT, 'preserved stdout identity'],
    [' canonicalOwnedCases\n canonicalStreamCases\n', 'mandatory canonical main entry'],
    [' check (count == 198) "complete finite OwnedHash canonical case inventory"\n', '198-case gate'],
    [' check (count == 44) "complete diagnostic StreamEngine case inventory"\n', '44-case gate'],
    [' check (bytes==canonicalProfileComparison) "published OwnedHash image equals canonical Profile.hash"\n', 'actual published-image comparison']]) once(extended, marker, label);
  return extended;
}
