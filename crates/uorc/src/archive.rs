//! Native UORC archive framing: the wire header, the UORC-SHA256-CHAIN-v1
//! seal, and the two archive shapes the client writes, mirroring
//! `Uorc.Codec.Archive` (client/src/Uorc/Codec/Archive.lex.tex) byte for byte.
//!
//! An archive is `preimage || seal(preimage)` where the preimage is
//! `"UORC" 02 01 scope kind count lengths.. bodyLength [basis32] body`, every
//! number a shortest ULEB128. Because the body length is in the header, an
//! archive is self-delimiting, so a stream of archives laid end to end needs no
//! container: [`frame_length`] finds where each one ends.

use sha2::{Digest, Sha256};

const PREFIX_HEX: &str = include_str!("kernel-prefix.hex");
const SUFFIX_HEX: &str = include_str!("kernel-suffix.hex");
const PREFIX: [u8; hex_len(PREFIX_HEX)] = hex_literal(PREFIX_HEX);
const SUFFIX: [u8; hex_len(SUFFIX_HEX)] = hex_literal(SUFFIX_HEX);
/// The ulz decoder kernel program (`Archive.kernelPrefix`), before the payload length.
pub const KERNEL_PREFIX: &[u8] = &PREFIX;
/// The ulz decoder kernel program (`Archive.kernelSuffix`), after the payload.
pub const KERNEL_SUFFIX: &[u8] = &SUFFIX;

const CHAIN_TAG: &[u8] = b"UORC-SHA256-CHAIN-v1\0";
const MAGIC: &[u8] = b"UORC\x02\x01";
const MAGIC_SELF_CONTAINED: &[u8] = b"UORC\x02\x01\x00";
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

fn push_uleb(out: &mut Vec<u8>, mut n: u64) {
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
fn read_uleb(data: &[u8], mut at: usize) -> Option<(u64, usize)> {
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

fn header(kind: u8, n: u64, body_len: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.extend_from_slice(MAGIC);
    out.push(0);
    out.push(kind);
    push_uleb(&mut out, 1);
    push_uleb(&mut out, n);
    push_uleb(&mut out, body_len);
    out
}

/// `Archive.rawArchive input`: the input stored as one raw object.
pub fn raw_archive(input: &[u8]) -> Vec<u8> {
    let mut pre = header(0, input.len() as u64, input.len() as u64);
    pre.reserve(input.len() + SEAL);
    pre.extend_from_slice(input);
    sealed(pre)
}

/// Exact length of [`raw_archive`] for an input of `n` bytes.
pub fn raw_length(n: u64) -> u64 {
    2 * uleb_len(n) as u64 + n + 41
}

/// `Archive.programArchive n payload`: the ulz kernel graph with `payload` as
/// its sequence constant, reconstructing one object of `n` bytes.
pub fn program_archive(n: u64, payload: &[u8]) -> Vec<u8> {
    let p = payload.len() as u64;
    let body_len = KERNEL_PREFIX.len() as u64 + uleb_len(p) as u64 + p + KERNEL_SUFFIX.len() as u64;
    let mut pre = header(1, n, body_len);
    pre.reserve(body_len as usize + SEAL);
    pre.extend_from_slice(KERNEL_PREFIX);
    push_uleb(&mut pre, p);
    pre.extend_from_slice(payload);
    pre.extend_from_slice(KERNEL_SUFFIX);
    sealed(pre)
}

/// Total length of the archive starting at `data[0]`, read from its header,
/// or `None` when the header is incomplete or not a UORC format-2 header. The
/// archive's integrity is not checked here.
pub fn frame_length(data: &[u8]) -> Option<usize> {
    if data.len() < 8 || &data[..6] != MAGIC || data[6] > 1 || data[7] > 1 {
        return None;
    }
    let (count, mut at) = read_uleb(data, 8)?;
    for _ in 0..count {
        at = read_uleb(data, at)?.1;
    }
    let (body, at) = read_uleb(data, at)?;
    let basis = if data[6] == 1 { 32 } else { 0 };
    let total = (at as u64)
        .checked_add(basis)?
        .checked_add(body)?
        .checked_add(SEAL as u64)?;
    usize::try_from(total).ok()
}

/// Minimum bytes [`frame_length`] may need to see a header.
pub const HEADER_MAX: usize = 8 + 10 * 3;

/// `Archive.fastDecode`, natively: the output of an archive this client wrote
/// (raw, or ulz kernel with a payload the native decoder vouches for), sealed
/// and laid out exactly as the client writes it. `None` sends the archive to
/// the exported reference decoder.
pub fn fast_decode(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 8 + SEAL || data[..7] != MAGIC_SELF_CONTAINED[..] || data[7] > 1 {
        return None;
    }
    let (count, at) = read_uleb(data, 8)?;
    if count != 1 {
        return None;
    }
    let (n, at) = read_uleb(data, at)?;
    let (body_len, at) = read_uleb(data, at)?;
    let body_len = usize::try_from(body_len).ok()?;
    if data.len() != at.checked_add(body_len)?.checked_add(SEAL)? {
        return None;
    }
    let (pre, seal) = data.split_at(data.len() - SEAL);
    // The header must be the shortest encoding the client writes.
    if pre[..at] != header(data[7], n, body_len as u64)[..] || seal != chain(pre) {
        return None;
    }
    let body = &pre[at..];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_archive_is_the_model_layout() {
        let a = raw_archive(b"abc");
        assert_eq!(a.len() as u64, raw_length(3));
        assert_eq!(&a[..14], b"UORC\x02\x01\x00\x00\x01\x03\x03abc");
        assert_eq!(frame_length(&a), Some(a.len()));
        assert_eq!(fast_decode(&a).as_deref(), Some(&b"abc"[..]));
    }

    #[test]
    fn program_archive_round_trips() {
        let input = b"abcabcabcabcabcabcabcabc".repeat(40);
        let a = program_archive(input.len() as u64, &crate::ulz::encode(&input));
        assert_eq!(frame_length(&a), Some(a.len()));
        assert_eq!(fast_decode(&a), Some(input));
    }

    #[test]
    fn seal_is_checked() {
        let mut a = raw_archive(b"hello");
        let last = a.len() - 1;
        a[last] ^= 1;
        assert_eq!(fast_decode(&a), None);
    }
}
