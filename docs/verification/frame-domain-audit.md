<!-- uorc:non-authoritative -->
# Frame length domain audit

Source boundary: compression `f947c988c6769fac2d94a850afee19015d18653c`
(`Wire.Frame`, `FrameParse`, `GraphEmit`, `Cursor` and the registered varint
contract), plus unchanged Integrity binding
`2c929d7482b8c2b98a5b965c684790abf8b8995f`.

| Quantity | Format-2 reference frame source | Owned realization |
| --- | --- | --- |
| Object count | Natural, shortest base-128 header integer | UInt64 count; larger values cause an operational refusal |
| Individual object length | Natural, shortest base-128 header integer | UInt64 indexed length; larger values cause an operational refusal |
| Aggregate output length | Natural sum of the ordered lengths; no separate encoded total | Checked UInt64 sum |
| Body length | Natural, shortest base-128 header integer | Checked UInt64 length and body fence |

`Frame.objectLengths`, `Prepared.objectCount/outputLength/bodyLength` and
`FrameParse.Header` use Nat. Crucially, `FrameParse.step`'s `Stage.number` does
not call `Cursor.ulebStep`: it accumulates Nat value/factor/used, checks the
shortest terminal spelling, and stores the result without a ten-byte or U64
check. `Frame.preimageStart` serializes these natural header fields through
`GraphEmit`'s natural task. The generic `Cursor.ulebStep` and separately
registered `Uorc.Varint` are explicitly bounded U64 codecs. Their limits must
not be projected onto a frame parser that does not invoke them.

`FrameDomainScenarios.run` actually executes the independent reference parser
on compact prefixes whose count, individual length, aggregate length or body
length is `2^64`. All four headers reach the intended Nat field. These controls
stop once the field has been read; they neither allocate an enormous body nor
claim acceptance/reconstruction of such an archive. They are finite diagnostic
evidence, not a theorem over every natural number.

Format 3 changes the explicitly selected integrity identity to STREAM-MAX-v1.
`ArchiveBinding` states that its Descriptor's U64 fields are runtime admitted
words, not wire-level length limits, and delegates the remaining grammar to the
parser. It does not independently define or prove a new bounded header grammar.
The owned parser continues the format-2 natural header spelling subject to its
operational addressability; no new versioned U64 wire-language restriction is
introduced here. Format 3 still needs the complete registered frame/refinement
and qualification path; the old diagnostic `FrameParse` itself accepts format 2
only, so it is not format-3 acceptance evidence.

Malformed tags, unsupported format/profile, nonminimal encodings, truncation
and trailing bytes are format errors. In contrast, a shortest finite natural
header beyond the owned realization's UInt64 domain, or a UInt64 sum/address
that cannot be represented, is an operational refusal. A future decision to
bound a version's wire language would require an explicit source specification
change and distinct diagnostics; it cannot be inferred from runtime types.

No source-defined archive member, continuation, indefinite-length object or
object-chunk mechanism currently joins multiple members into one original
ordered object. `FrameParse.Stage.end` demands immediate EOF. Internal image,
Rope and sequence partitions do not change object identity; splitting one object
into multiple entries changes the declared ordered object collection. Sequence
concatenation inside a program also does not eliminate the outer length field.

There is therefore no demonstrated mathematical header-length cap in the
format-2 reference grammar, but the current owned runtime cannot process every
larger finite object/count/aggregate and has no existing continuation mechanism
to bridge that realization limit. That is a real implementation/refinement gap
for an unqualified arbitrary-length production claim, not proof that those
finite inputs are forbidden by the reference format. This audit changes no wire
identifier, integer encoding, checksum construction, object semantics or SDK
lock, and introduces no uncharged big-integer production state.
