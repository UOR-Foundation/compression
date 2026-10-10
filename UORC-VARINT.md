<!-- @generated from the UORC LexLean authority graph. -->
# UORC ULEB128 contract

## authority

The native core declarations, computational definitions, and proof terms in Uorc.Varint are LexLean source. Verification reconstructs them from the source graph without any handwritten Lean file or precompiled product object as an input.

## axiomPolicy

Computational entry points have empty axiom sets. Proof declarations carry their exact kernel-audited subset of Lean foundations propext, Quot.sound, and Classical.choice. No additional axiom or admitted proof is allowed.

## decoding

Prefix decoding consumes at most ten bytes and returns the numeric value, consumed length, and untouched suffix. Exact decoding additionally rejects any suffix. The prefix is accepted only if the bounded encoder reproduces its exact bytes.

## domain

Unsigned 64-bit integers: natural values from 0 through 18446744073709551615. The public encoder rejects larger values before encoding. Byte inputs are natural values checked against 0..255.

## encoding

Shortest unsigned little-endian base-128 encoding. Each non-final byte carries seven payload bits and sets bit 7; the final byte clears bit 7. Zero is the single byte 00. A successful U64 encoding occupies one through ten bytes.

## errorOrder

At each consumed byte: exhausted ten-byte allowance, absent byte, invalid byte domain, then termination/continuation. After termination: U64 overflow, non-shortest spelling, then (for exact decoding) trailing bytes. Prefix decoding does not inspect suffix bytes.

## offsets

Offsets are zero-based relative to the varint start. Truncation identifies the first missing byte; an invalid byte identifies itself; overflow or non-shortest spelling identifies the final consumed byte; trailing bytes identify the first unconsumed byte; encoder domain rejection uses offset zero.

## proofScope

The two public round trips quantify over all domain values or all accepted inputs. Additional theorems establish exact size, a one-to-ten-byte bound, minimum possible base-128 representation length, and preservation of prefix/suffix concatenation. Closed vectors are separate exact observations.

## Typed failures

| Variant | Meaning | Negative family |
| --- | --- | --- |
| `truncated` | Input ends before a terminating byte. | `truncated_` |
| `overflow` | The ten-byte allowance or U64 numeric range is exceeded. | `overflow_` |
| `nonShortest` | The consumed bytes are not the shortest spelling of their value. | `overlong_` |
| `invalidByte` | A consumed input element is outside the byte domain. | `invalid_byte_` |
| `trailingBytes` | Exact decoding finds a suffix after a valid prefix. | `trailing_` |
| `valueOutOfRange` | The encoder input is outside the U64 domain. | `encode_overflow_` |

## General kernel roots

- `Uorc.Varint.uleb_encode_decode`
- `Uorc.Varint.uleb_decode_encode`
- `Uorc.Varint.uleb_size_exact`
- `Uorc.Varint.uleb_size_bounded`
- `Uorc.Varint.shortest`
- `Uorc.Varint.scan_reconstruct`
- `Uorc.Varint.chunk_encode`
- `Uorc.Varint.fuel_eq`
- `Uorc.Varint.bounded_eq`

236 closed vector theorems are inventoried in `model/varint.toml`.

