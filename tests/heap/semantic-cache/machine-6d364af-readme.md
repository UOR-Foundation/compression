<!-- uorc:non-authoritative -->
# Exact machine semantic-adapter parsed-source transfer

machine-6d364af.json is unchanged official reader output for these three actual
public modules at compressor commit
`6d364afcbb09396c78fdb213d179798a2d09eacb`:

| Module | Source SHA256 |
| --- | --- |
| RuntimeMachineRefinement | 068ca58001ae3ffb6eb5afd082399f23aa060661621da802e7be37826505c00c |
| RuntimeMachineEngine | ad6a4507f5f4ac1b3637ed5b28b0003b8329295c2b14a3830bd20f83d768bf69 |
| RuntimeMachineSemantic | d6ada095bed76a5985c2d6598e640fc07a84836ea1959c73c63d473bde7839ac |

Each UTF8 source is checked against the complete commit tree's Git blob SHA1
and length. The existing official PrismPM helper reads each source twice with
byte-identical output, writes its notation byte-exactly, then reads the same
IR again. All three modules satisfy both byte-exact source roundtrip and
byte-exact canonical-read identity. The unchanged parsed_module objects retain
all declarations, imports, executable and axiom metadata.

The actual unit-cache declarations reside inside the supplied Semantic and
Engine modules; no missing adapter or cache was invented. The parser needs no
linked closure. This cache therefore establishes authentic grammar/reader
output only, with no linking, kernel, execution, export or release claim.

Bundle size: 1,441,827 bytes. SHA256:
`f6f0e7958306bca940e0b2b5aceb6bf6968916345e109485fb236374d8e8f5a7`.
Reader source/binary/package identities and reproduction command are included
in the bundle and export-machine-cache.mjs. The earlier machine-72f3.json and
its original source hashes remain unchanged. Source is authoritative and
neither cache is a production dependency.
