//! Native UORC archive framing: the wire header, the UORC-SHA256-CHAIN-v1
//! seal, and the two archive shapes the client writes, mirroring
//! `Uorc.Codec.Archive` (client/src/Uorc/Codec/Archive.lex.tex) byte for byte.
//!
//! An archive is `preimage || seal(preimage)` where the preimage is
//! `"UORC" 02 01 scope kind count lengths.. bodyLength [basis32] body`, every
//! number a shortest ULEB128. Because the body length is in the header, an
//! archive is self-delimiting, so a stream of archives laid end to end needs no
//! container: [`crate::frame_length`] finds where each one ends.
//!
//! The client writes linked archives: conditional archives (scope 1) over the
//! built-in standard basis, the ulz and ncm decoder libraries bound by
//! [`BASIS_ID`] (`client/gen/gen_basis.py`). Their body is one entry block that
//! applies a library decoder to the payload, so a small file carries no kernel.

use sha2::{Digest, Sha256};

const PREFIX_HEX: &str = include_str!("kernel-prefix.hex");
const SUFFIX_HEX: &str = include_str!("kernel-suffix.hex");
const PREFIX: [u8; hex_len(PREFIX_HEX)] = hex_literal(PREFIX_HEX);
const SUFFIX: [u8; hex_len(SUFFIX_HEX)] = hex_literal(SUFFIX_HEX);
/// The ulz decoder kernel program (`Archive.kernelPrefix`), before the payload length.
pub const KERNEL_PREFIX: &[u8] = &PREFIX;
/// The ulz decoder kernel program (`Archive.kernelSuffix`), after the payload.
pub const KERNEL_SUFFIX: &[u8] = &SUFFIX;

const BASIS_ID_HEX: &str = include_str!("basis-id.hex");
const LINKED_PREFIX_HEX: &str = include_str!("linked-prefix.hex");
const ULZ_SUFFIX_HEX: &str = include_str!("linked-ulz-suffix.hex");
const NCM_SUFFIX_HEX: &str = include_str!("linked-ncm-suffix.hex");
/// Identity of the built-in standard basis (`Archive.basisIdentity`): the
/// UORC-BASIS-V1 chain over its one root, the ulz and ncm decoder libraries.
pub const BASIS_ID: [u8; 32] = hex_literal(BASIS_ID_HEX);
const LINKED_PREFIX: [u8; hex_len(LINKED_PREFIX_HEX)] = hex_literal(LINKED_PREFIX_HEX);
const ULZ_SUFFIX: [u8; hex_len(ULZ_SUFFIX_HEX)] = hex_literal(ULZ_SUFFIX_HEX);
const NCM_SUFFIX: [u8; hex_len(NCM_SUFFIX_HEX)] = hex_literal(NCM_SUFFIX_HEX);

/// A decoder of the standard basis that a linked archive applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// The LZ + range coder (`Uorc.Codec.LzDecode`).
    Ulz,
    /// The context-mixing coder (`Uorc.Codec.NcmDecode`).
    Ncm,
}

impl Codec {
    fn suffix(self) -> &'static [u8] {
        match self {
            Codec::Ulz => &ULZ_SUFFIX,
            Codec::Ncm => &NCM_SUFFIX,
        }
    }
}

const CHAIN_TAG: &[u8] = b"UORC-SHA256-CHAIN-v1\0";
const MAGIC: &[u8] = b"UORC\x02\x01";
/// Length of the integrity seal at the end of every archive.
pub const SEAL: usize = 32;

const fn hex_value(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => panic!("kernel hex"),
    }
}

const fn hex_len(text: &str) -> usize {
    let b = text.as_bytes();
    let mut n = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'\n' {
            n += 1;
        }
        i += 1;
    }
    n / 2
}

const fn hex_literal<const N: usize>(text: &str) -> [u8; N] {
    let b = text.as_bytes();
    let mut out = [0u8; N];
    let (mut i, mut k) = (0, 0);
    while i < b.len() {
        if b[i] == b'\n' {
            i += 1;
            continue;
        }
        out[k] = hex_value(b[i]) * 16 + hex_value(b[i + 1]);
        k += 1;
        i += 2;
    }
    assert!(k == N, "kernel hex length");
    out
}

pub(crate) fn push_uleb(out: &mut Vec<u8>, mut n: u64) {
    while n >= 0x80 {
        out.push((n as u8 & 0x7F) | 0x80);
        n >>= 7;
    }
    out.push(n as u8);
}

/// Length of the shortest ULEB128 encoding of `n`.
pub fn uleb_len(mut n: u64) -> usize {
    let mut len = 1;
    while n >= 0x80 {
        n >>= 7;
        len += 1;
    }
    len
}

/// Read a ULEB128 number at `at`: the value and the offset after it.
pub(crate) fn read_uleb(data: &[u8], mut at: usize) -> Option<(u64, usize)> {
    let mut value = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *data.get(at)?;
        at += 1;
        let part = u64::from(byte & 0x7F);
        if shift == 63 && part > 1 {
            return None;
        }
        value |= part << shift;
        if byte < 0x80 {
            return Some((value, at));
        }
    }
    None
}

/// The UORC-SHA256-CHAIN-v1 digest of `preimage`.
pub fn chain(preimage: &[u8]) -> [u8; 32] {
    let mut c: [u8; 32] = Sha256::new()
        .chain_update(CHAIN_TAG)
        .chain_update([0u8])
        .finalize()
        .into();
    let mut blocks = preimage.chunks_exact(64);
    for block in &mut blocks {
        c = Sha256::new()
            .chain_update(CHAIN_TAG)
            .chain_update([1u8])
            .chain_update(c)
            .chain_update(block)
            .finalize()
            .into();
    }
    let tail = blocks.remainder();
    Sha256::new()
        .chain_update(CHAIN_TAG)
        .chain_update([2u8])
        .chain_update(c)
        .chain_update([tail.len() as u8])
        .chain_update(tail)
        .finalize()
        .into()
}

fn sealed(mut preimage: Vec<u8>) -> Vec<u8> {
    let digest = chain(&preimage);
    preimage.extend_from_slice(&digest);
    preimage
}

fn header(kind: u8, lengths: &[u64], body_len: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + 2 * lengths.len());
    out.extend_from_slice(MAGIC);
    out.push(0);
    out.push(kind);
    push_uleb(&mut out, lengths.len() as u64);
    for &n in lengths {
        push_uleb(&mut out, n);
    }
    push_uleb(&mut out, body_len);
    out
}

/// `Archive.rawArchive input`: the input stored raw, as one object.
pub fn raw_archive(input: &[u8]) -> Vec<u8> {
    raw_objects(input, &[input.len() as u64])
}

/// The raw archive of `input` split into objects of `lengths` (which must sum
/// to its length); with one length this is [`raw_archive`].
pub fn raw_objects(input: &[u8], lengths: &[u64]) -> Vec<u8> {
    let mut pre = header(0, lengths, input.len() as u64);
    pre.reserve(input.len() + SEAL);
    pre.extend_from_slice(input);
    sealed(pre)
}

/// Exact length of [`raw_objects`] for objects of `lengths`.
pub fn raw_length(lengths: &[u64]) -> u64 {
    let n: u64 = lengths.iter().sum();
    let fields: u64 = lengths.iter().map(|&l| uleb_len(l) as u64).sum();
    8 + uleb_len(lengths.len() as u64) as u64 + fields + uleb_len(n) as u64 + n + SEAL as u64
}

/// `Archive.programArchive n payload`: the ulz kernel graph with `payload` as
/// its sequence constant, reconstructing objects of `lengths` (one object of
/// `n` bytes in the model; several when the client relabels its output).
pub fn program_archive(lengths: &[u64], payload: &[u8]) -> Vec<u8> {
    let p = payload.len() as u64;
    let body_len = KERNEL_PREFIX.len() as u64 + uleb_len(p) as u64 + p + KERNEL_SUFFIX.len() as u64;
    let mut pre = header(1, lengths, body_len);
    pre.reserve(body_len as usize + SEAL);
    pre.extend_from_slice(KERNEL_PREFIX);
    push_uleb(&mut pre, p);
    pre.extend_from_slice(payload);
    pre.extend_from_slice(KERNEL_SUFFIX);
    sealed(pre)
}

fn linked_header(lengths: &[u64], body_len: u64) -> Vec<u8> {
    let mut out = header(1, lengths, body_len);
    out[6] = 1;
    out.extend_from_slice(&BASIS_ID);
    out
}

fn linked_body_len(codec: Codec, p: u64) -> u64 {
    LINKED_PREFIX.len() as u64 + uleb_len(p) as u64 + p + codec.suffix().len() as u64
}

/// `Archive.linkedArchive`: a conditional archive over the standard basis whose
/// entry applies `codec`'s decoder to `payload`, reconstructing objects of
/// `lengths` (one object in the model; several when the client relabels).
pub fn linked_archive(codec: Codec, lengths: &[u64], payload: &[u8]) -> Vec<u8> {
    let p = payload.len() as u64;
    let body_len = linked_body_len(codec, p);
    let mut pre = linked_header(lengths, body_len);
    pre.reserve(body_len as usize + SEAL);
    pre.extend_from_slice(&LINKED_PREFIX);
    push_uleb(&mut pre, p);
    pre.extend_from_slice(payload);
    pre.extend_from_slice(codec.suffix());
    sealed(pre)
}

/// Exact length of [`linked_archive`] for objects of `lengths` and a `p`-byte payload.
pub fn linked_length(codec: Codec, lengths: &[u64], p: u64) -> u64 {
    let body = linked_body_len(codec, p);
    let fields: u64 = lengths.iter().map(|&l| uleb_len(l) as u64).sum();
    8 + uleb_len(lengths.len() as u64) as u64
        + fields
        + uleb_len(body) as u64
        + 32
        + body
        + SEAL as u64
}

/// `Archive.fastDecode`, natively: the output of an archive this client wrote
/// (raw, ulz kernel, or linked ulz/ncm, with a payload the native decoder vouches for), sealed
/// and laid out exactly as the client writes it, with any object lengths. `None` sends the archive to
/// the exported reference decoder.
pub fn fast_decode(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 8 + SEAL || data[..6] != MAGIC[..] || data[6] > 1 || data[7] > 1 {
        return None;
    }
    let linked = data[6] == 1;
    if linked && data[7] != 1 {
        return None;
    }
    let (count, mut at) = read_uleb(data, 8)?;
    // Each length takes a byte, which bounds the allocation.
    if count > data.len() as u64 {
        return None;
    }
    let mut lengths = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (n, next) = read_uleb(data, at)?;
        lengths.push(n);
        at = next;
    }
    let n = lengths.iter().try_fold(0u64, |a, &l| a.checked_add(l))?;
    let (body_len, at) = read_uleb(data, at)?;
    let body_len = usize::try_from(body_len).ok()?;
    let at = if linked { at.checked_add(32)? } else { at };
    if data.len() != at.checked_add(body_len)?.checked_add(SEAL)? {
        return None;
    }
    let (pre, seal) = data.split_at(data.len() - SEAL);
    // The header (and for a linked archive, the basis identity) must be
    // exactly what the client writes.
    let expected = if linked {
        linked_header(&lengths, body_len as u64)
    } else {
        header(data[7], &lengths, body_len as u64)
    };
    if pre[..at] != expected[..] || seal != chain(pre) {
        return None;
    }
    let body = &pre[at..];
    if linked {
        return linked_decode(body, n);
    }
    if data[7] == 0 {
        return (n == body_len as u64).then(|| body.to_vec());
    }
    let rest = body.strip_prefix(KERNEL_PREFIX)?;
    let (p, after) = read_uleb(rest, 0)?;
    let p = usize::try_from(p).ok()?;
    if after != uleb_len(p as u64) || rest.len() != after.checked_add(p)? + KERNEL_SUFFIX.len() {
        return None;
    }
    let (payload, suffix) = rest[after..].split_at(p);
    if suffix != KERNEL_SUFFIX {
        return None;
    }
    crate::ulz::decode(payload, n)
}

/// The library decoder a body applies, when it is laid out exactly as
/// [`linked_archive`] writes it.
pub fn linked_codec(body: &[u8]) -> Option<Codec> {
    let rest = body.strip_prefix(&LINKED_PREFIX[..])?;
    let (p, after) = read_uleb(rest, 0)?;
    let p = usize::try_from(p).ok()?;
    if after != uleb_len(p as u64) || rest.len() < after.checked_add(p)? {
        return None;
    }
    let suffix = &rest[after + p..];
    [Codec::Ulz, Codec::Ncm]
        .into_iter()
        .find(|c| suffix == c.suffix())
}

/// The output of a linked archive's body, laid out exactly as
/// [`linked_archive`] writes it, with the payload its encoder writes for that
/// output (`Archive.linkedDecode`). `None` sends it to the reference decoder.
fn linked_decode(body: &[u8], n: u64) -> Option<Vec<u8>> {
    let codec = linked_codec(body)?;
    let rest = &body[LINKED_PREFIX.len()..];
    let (p, after) = read_uleb(rest, 0)?;
    let payload = &rest[after..after + usize::try_from(p).ok()?];
    match codec {
        Codec::Ulz => crate::ulz::decode(payload, n),
        Codec::Ncm => crate::ncm::decode_canonical(payload, n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_archive_is_the_model_layout() {
        let a = raw_archive(b"abc");
        assert_eq!(a.len() as u64, raw_length(&[3]));
        assert_eq!(&a[..14], b"UORC\x02\x01\x00\x00\x01\x03\x03abc");
        assert_eq!(crate::frame_length(&a), Ok(Some(a.len())));
        assert_eq!(fast_decode(&a).as_deref(), Some(&b"abc"[..]));
    }

    #[test]
    fn program_archive_round_trips() {
        let input = b"abcabcabcabcabcabcabcabc".repeat(40);
        let a = program_archive(&[input.len() as u64], &crate::ulz::encode(&input));
        assert_eq!(crate::frame_length(&a), Ok(Some(a.len())));
        assert_eq!(fast_decode(&a), Some(input));
    }

    #[test]
    fn object_lengths_are_relabels() {
        let input = b"abcabcabcabcabcabcabcabc".repeat(40);
        let payload = crate::ulz::encode(&input);
        let lengths = [0, 100, 0, input.len() as u64 - 100];
        for (one, many) in [
            (raw_archive(&input), raw_objects(&input, &lengths)),
            (
                program_archive(&[input.len() as u64], &payload),
                program_archive(&lengths, &payload),
            ),
        ] {
            assert_eq!(
                many,
                crate::frame::relabel(&one, &lengths).expect("relabel")
            );
            assert_eq!(fast_decode(&many).as_ref(), Some(&input));
        }
        assert_eq!(
            raw_objects(&input, &lengths).len() as u64,
            raw_length(&lengths)
        );
    }

    #[test]
    fn linked_archives_round_trip() {
        let input =
            b"the standard basis carries the kernels; archives name it by identity. ".repeat(3);
        let n = input.len() as u64;
        let u = linked_archive(Codec::Ulz, &[n], &crate::ulz::encode(&input));
        let c = linked_archive(Codec::Ncm, &[n], &crate::ncm::encode(&input).expect("fits"));
        // the model's archives for this text (client/gen/gen_basis.py, Archive.ulzLinkedArchive)
        assert_eq!((u.len(), c.len()), (165, 140));
        for a in [&u, &c] {
            assert_eq!(crate::frame_length(a), Ok(Some(a.len())));
            assert_eq!(fast_decode(a).as_deref(), Some(&input[..]));
        }
        let p = crate::ncm::encode(&input).expect("fits");
        assert_eq!(
            linked_length(Codec::Ncm, &[n], p.len() as u64),
            c.len() as u64
        );
    }

    #[test]
    fn linked_archive_names_the_basis() {
        let mut a = linked_archive(Codec::Ncm, &[3], &crate::ncm::encode(b"abc").expect("fits"));
        assert_eq!(fast_decode(&a).as_deref(), Some(&b"abc"[..]));
        // another basis identity, resealed so only the identity differs
        a[12] ^= 1;
        let end = a.len() - SEAL;
        let digest = chain(&a[..end]);
        a[end..].copy_from_slice(&digest);
        assert_eq!(fast_decode(&a), None);
    }

    #[test]
    fn basis_identity_binds_the_library() {
        let lib = hex_bytes(include_str!("basis-library.hex"));
        let mut pre = b"UORC-BASIS-V1\0".to_vec();
        push_uleb(&mut pre, 1);
        push_uleb(&mut pre, lib.len() as u64);
        pre.extend_from_slice(&lib);
        assert_eq!(chain(&pre), BASIS_ID);
    }

    fn hex_bytes(text: &str) -> Vec<u8> {
        let t: Vec<u8> = text.bytes().filter(|&c| c != b'\n').collect();
        t.chunks(2)
            .map(|c| hex_value(c[0]) * 16 + hex_value(c[1]))
            .collect()
    }

    #[test]
    fn seal_is_checked() {
        let mut a = raw_archive(b"hello");
        let last = a.len() - 1;
        a[last] ^= 1;
        assert_eq!(fast_decode(&a), None);
    }

    /// The kernel bytes here are the ones the codec source defines.
    #[test]
    fn kernel_is_the_model_kernel() {
        let source = include_str!("../../../client/src/Uorc/Codec/Archive.lex.tex");
        for (name, bytes) in [
            ("kernelPrefix", KERNEL_PREFIX),
            ("kernelSuffix", KERNEL_SUFFIX),
        ] {
            let text = source
                .split(&format!("name: {name},"))
                .nth(1)
                .and_then(|d| d.split("body: 0x").nth(1))
                .expect(name);
            let hex: String = text.chars().take_while(char::is_ascii_hexdigit).collect();
            let expected: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(hex.to_ascii_lowercase(), expected, "{name}");
        }
    }
}
