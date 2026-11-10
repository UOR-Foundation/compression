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
const LIBRARY_HEX: &str = include_str!("basis-library.hex");
/// The standard basis's one root (`Archive.libraryRoot`): the ulz and ncm
/// decoder library blocks.
pub static LIBRARY: [u8; hex_len(LIBRARY_HEX)] = hex_literal(LIBRARY_HEX);
const BASIS_TAG: &[u8] = b"UORC-BASIS-V1\0";
const WINDOW1_PREFIX_HEX: &str = include_str!("linked-ulz1-prefix.hex");
const WINDOW1_SUFFIX_HEX: &str = include_str!("linked-ulz1-suffix.hex");
const WINDOW2_PREFIX_HEX: &str = include_str!("linked-ulz2-prefix.hex");
const WINDOW2_SUFFIX_HEX: &str = include_str!("linked-ulz2-suffix.hex");
const WINDOW1_PREFIX: [u8; hex_len(WINDOW1_PREFIX_HEX)] = hex_literal(WINDOW1_PREFIX_HEX);
const WINDOW1_SUFFIX: [u8; hex_len(WINDOW1_SUFFIX_HEX)] = hex_literal(WINDOW1_SUFFIX_HEX);
const WINDOW2_PREFIX: [u8; hex_len(WINDOW2_PREFIX_HEX)] = hex_literal(WINDOW2_PREFIX_HEX);
const WINDOW2_SUFFIX: [u8; hex_len(WINDOW2_SUFFIX_HEX)] = hex_literal(WINDOW2_SUFFIX_HEX);
// The ncm entries differ from the ulz ones of the same shape only after the
// payload; the x86 entries (`x`, ncm only) undo the filter on the decoder's output.
const X86_PREFIX_HEX: &str = include_str!("linked-ncmx-prefix.hex");
const X86_PREFIX: [u8; hex_len(X86_PREFIX_HEX)] = hex_literal(X86_PREFIX_HEX);
const NCMX_SUFFIX_HEX: &str = include_str!("linked-ncmx-suffix.hex");
const NCMX_SUFFIX: [u8; hex_len(NCMX_SUFFIX_HEX)] = hex_literal(NCMX_SUFFIX_HEX);
const NCM1_SUFFIX_HEX: &str = include_str!("linked-ncm1-suffix.hex");
const NCM1_SUFFIX: [u8; hex_len(NCM1_SUFFIX_HEX)] = hex_literal(NCM1_SUFFIX_HEX);
const NCM2_SUFFIX_HEX: &str = include_str!("linked-ncm2-suffix.hex");
const NCM2_SUFFIX: [u8; hex_len(NCM2_SUFFIX_HEX)] = hex_literal(NCM2_SUFFIX_HEX);
const WINDOW1X_PREFIX_HEX: &str = include_str!("linked-ncm1x-prefix.hex");
const WINDOW1X_PREFIX: [u8; hex_len(WINDOW1X_PREFIX_HEX)] = hex_literal(WINDOW1X_PREFIX_HEX);
const WINDOW2X_PREFIX_HEX: &str = include_str!("linked-ncm2x-prefix.hex");
const WINDOW2X_PREFIX: [u8; hex_len(WINDOW2X_PREFIX_HEX)] = hex_literal(WINDOW2X_PREFIX_HEX);
const NCM1X_SUFFIX_HEX: &str = include_str!("linked-ncm1x-suffix.hex");
const NCM1X_SUFFIX: [u8; hex_len(NCM1X_SUFFIX_HEX)] = hex_literal(NCM1X_SUFFIX_HEX);
const NCM2X_SUFFIX_HEX: &str = include_str!("linked-ncm2x-suffix.hex");
const NCM2X_SUFFIX: [u8; hex_len(NCM2X_SUFFIX_HEX)] = hex_literal(NCM2X_SUFFIX_HEX);
// The v0.2 basis: its library is this one's first blocks, byte for byte, and
// v0.2 archives (which name it) decode as before.
const V02_BASIS_ID_HEX: &str = include_str!("basis-v02-id.hex");
/// Identity of the v0.2 standard basis, which v0.2 archives name; they still decode.
pub const V02_BASIS_ID: [u8; hex_len(V02_BASIS_ID_HEX)] = hex_literal(V02_BASIS_ID_HEX);
const V02_LIBRARY_HEX: &str = include_str!("basis-v02-library.hex");
const V02_LIBRARY: [u8; hex_len(V02_LIBRARY_HEX)] = hex_literal(V02_LIBRARY_HEX);
const V02_ULZ_SUFFIX_HEX: &str = include_str!("v02-linked-ulz-suffix.hex");
const V02_ULZ_SUFFIX: [u8; hex_len(V02_ULZ_SUFFIX_HEX)] = hex_literal(V02_ULZ_SUFFIX_HEX);
const V02_NCM_SUFFIX_HEX: &str = include_str!("v02-linked-ncm-suffix.hex");
const V02_NCM_SUFFIX: [u8; hex_len(V02_NCM_SUFFIX_HEX)] = hex_literal(V02_NCM_SUFFIX_HEX);
const V02_ULZ1_SUFFIX_HEX: &str = include_str!("v02-linked-ulz1-suffix.hex");
const V02_ULZ1_SUFFIX: [u8; hex_len(V02_ULZ1_SUFFIX_HEX)] = hex_literal(V02_ULZ1_SUFFIX_HEX);
const V02_ULZ2_SUFFIX_HEX: &str = include_str!("v02-linked-ulz2-suffix.hex");
const V02_ULZ2_SUFFIX: [u8; hex_len(V02_ULZ2_SUFFIX_HEX)] = hex_literal(V02_ULZ2_SUFFIX_HEX);
// The v0.3 basis: its library is this one's first blocks, byte for byte, and
// v0.3 archives (which name it) decode as before. Its entries have this
// basis's prefixes; their suffixes reach the run blocks at the v0.3 distance.
const V03_BASIS_ID_HEX: &str = include_str!("basis-v03-id.hex");
/// Identity of the v0.3 standard basis, which v0.3 archives name; they still decode.
pub const V03_BASIS_ID: [u8; hex_len(V03_BASIS_ID_HEX)] = hex_literal(V03_BASIS_ID_HEX);
const V03_LIBRARY_HEX: &str = include_str!("basis-v03-library.hex");
static V03_LIBRARY: [u8; hex_len(V03_LIBRARY_HEX)] = hex_literal(V03_LIBRARY_HEX);
macro_rules! hex_const {
    ($name:ident, $file:literal) => {
        const $name: [u8; hex_len(include_str!($file))] = hex_literal(include_str!($file));
    };
}
hex_const!(V03_ULZ_SUFFIX, "v03-linked-ulz-suffix.hex");
hex_const!(V03_NCM_SUFFIX, "v03-linked-ncm-suffix.hex");
hex_const!(V03_NCMX_SUFFIX, "v03-linked-ncmx-suffix.hex");
hex_const!(V03_ULZ1_SUFFIX, "v03-linked-ulz1-suffix.hex");
hex_const!(V03_ULZ2_SUFFIX, "v03-linked-ulz2-suffix.hex");
hex_const!(V03_NCM1_SUFFIX, "v03-linked-ncm1-suffix.hex");
hex_const!(V03_NCM2_SUFFIX, "v03-linked-ncm2-suffix.hex");
hex_const!(V03_NCM1X_SUFFIX, "v03-linked-ncm1x-suffix.hex");
hex_const!(V03_NCM2X_SUFFIX, "v03-linked-ncm2x-suffix.hex");
// Profiled ncm entries (`gen_basis.py` PCODECS): the body is
// `prefix uleb(len) payload 00 uleb(profile) tail`, the profile word a natural
// constant the entry passes to the library's `ncmRunProfile`.
hex_const!(NCMP_PREFIX, "linked-ncmp-prefix.hex");
hex_const!(NCMP_TAIL, "linked-ncmp-tail.hex");
hex_const!(NCMP1_PREFIX, "linked-ncmp1-prefix.hex");
hex_const!(NCMP1_TAIL, "linked-ncmp1-tail.hex");
hex_const!(NCMP2_PREFIX, "linked-ncmp2-prefix.hex");
hex_const!(NCMP2_TAIL, "linked-ncmp2-tail.hex");
hex_const!(NCMPX_PREFIX, "linked-ncmpx-prefix.hex");
hex_const!(NCMPX_TAIL, "linked-ncmpx-tail.hex");
hex_const!(NCMP1X_PREFIX, "linked-ncmp1x-prefix.hex");
hex_const!(NCMP1X_TAIL, "linked-ncmp1x-tail.hex");
hex_const!(NCMP2X_PREFIX, "linked-ncmp2x-prefix.hex");
hex_const!(NCMP2X_TAIL, "linked-ncmp2x-tail.hex");

/// Which standard basis an archive names: this one, or the v0.3 or v0.2 one,
/// whose libraries are this one's first blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Era {
    Current,
    V03,
    V02,
}

impl Era {
    const ALL: [Era; 3] = [Era::Current, Era::V03, Era::V02];

    fn of(identity: &[u8]) -> Option<Era> {
        Era::ALL.into_iter().find(|e| e.identity() == identity)
    }

    fn identity(self) -> &'static [u8; 32] {
        match self {
            Era::Current => &BASIS_ID,
            Era::V03 => &V03_BASIS_ID,
            Era::V02 => &V02_BASIS_ID,
        }
    }

    fn library(self) -> &'static [u8] {
        match self {
            Era::Current => &LIBRARY,
            Era::V03 => &V03_LIBRARY,
            Era::V02 => &V02_LIBRARY,
        }
    }
}

/// Most preceding frames a window frame reads (`Archive.windowPrefix`).
pub const WINDOW: usize = 2;

/// A decoder of the standard basis that a linked archive applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// The LZ + range coder (`Uorc.Codec.LzDecode`).
    Ulz,
    /// The v0.3 context-mixing coder (`Uorc.Codec.NcmDecode` under
    /// [`crate::ncm::V03`]).
    Ncm,
    /// [`Codec::Ncm`] over the x86 branch filter ([`crate::x86`]).
    NcmX86,
    /// The context-mixing coder under the profile word it carries
    /// ([`crate::ncm::valid_profile`]), which the entry names.
    Profile(u32),
    /// [`Codec::Profile`] over the x86 branch filter.
    ProfileX86(u32),
}

impl Codec {
    /// Every codec, in the order the model's decoders try them (`Archive` codec 0..2).
    pub const ALL: [Codec; 3] = [Codec::Ulz, Codec::Ncm, Codec::NcmX86];

    /// Whether the payload codes the x86-filtered bytes.
    pub fn x86(self) -> bool {
        matches!(self, Codec::NcmX86 | Codec::ProfileX86(_))
    }

    /// The ncm profile word the payload is coded under (`None` for ulz).
    pub fn ncm_profile(self) -> Option<u32> {
        match self {
            Codec::Ulz => None,
            Codec::Ncm | Codec::NcmX86 => Some(crate::ncm::V03),
            Codec::Profile(p) | Codec::ProfileX86(p) => Some(p),
        }
    }

    /// The profiled codec coding `profile`, over the x86 filter when `x86`.
    pub fn profiled(profile: u32, x86: bool) -> Codec {
        if x86 {
            Codec::ProfileX86(profile)
        } else {
            Codec::Profile(profile)
        }
    }
}

/// The bytes around the payload of an entry that applies `codec` over `k`
/// window frames in a basis of `era`: `prefix uleb(len) payload suffix`.
fn template(codec: Codec, k: usize, era: Era) -> Option<(&'static [u8], Vec<u8>)> {
    if let Codec::Profile(p) | Codec::ProfileX86(p) = codec {
        if era != Era::Current || !crate::ncm::valid_profile(p) {
            return None;
        }
        let (prefix, tail) = profile_template(k, codec.x86())?;
        let mut suffix = vec![0];
        push_uleb(&mut suffix, u64::from(p));
        suffix.extend_from_slice(tail);
        return Some((prefix, suffix));
    }
    let (prefix, suffix): (&[u8], &[u8]) = match (era, codec, k) {
        (Era::V02, Codec::Ulz, 0) => (&LINKED_PREFIX, &V02_ULZ_SUFFIX),
        (Era::V02, Codec::Ncm, 0) => (&LINKED_PREFIX, &V02_NCM_SUFFIX),
        (Era::V02, Codec::Ulz, 1) => (&WINDOW1_PREFIX, &V02_ULZ1_SUFFIX),
        (Era::V02, Codec::Ulz, 2) => (&WINDOW2_PREFIX, &V02_ULZ2_SUFFIX),
        (Era::V02, _, _) => return None,
        (Era::V03, Codec::Ulz, 0) => (&LINKED_PREFIX, &V03_ULZ_SUFFIX),
        (Era::V03, Codec::Ncm, 0) => (&LINKED_PREFIX, &V03_NCM_SUFFIX),
        (Era::V03, Codec::NcmX86, 0) => (&X86_PREFIX, &V03_NCMX_SUFFIX),
        (Era::V03, Codec::Ulz, 1) => (&WINDOW1_PREFIX, &V03_ULZ1_SUFFIX),
        (Era::V03, Codec::Ulz, 2) => (&WINDOW2_PREFIX, &V03_ULZ2_SUFFIX),
        (Era::V03, Codec::Ncm, 1) => (&WINDOW1_PREFIX, &V03_NCM1_SUFFIX),
        (Era::V03, Codec::Ncm, 2) => (&WINDOW2_PREFIX, &V03_NCM2_SUFFIX),
        (Era::V03, Codec::NcmX86, 1) => (&WINDOW1X_PREFIX, &V03_NCM1X_SUFFIX),
        (Era::V03, Codec::NcmX86, 2) => (&WINDOW2X_PREFIX, &V03_NCM2X_SUFFIX),
        (Era::Current, Codec::Ulz, 0) => (&LINKED_PREFIX, &ULZ_SUFFIX),
        (Era::Current, Codec::Ncm, 0) => (&LINKED_PREFIX, &NCM_SUFFIX),
        (Era::Current, Codec::NcmX86, 0) => (&X86_PREFIX, &NCMX_SUFFIX),
        (Era::Current, Codec::Ulz, 1) => (&WINDOW1_PREFIX, &WINDOW1_SUFFIX),
        (Era::Current, Codec::Ulz, 2) => (&WINDOW2_PREFIX, &WINDOW2_SUFFIX),
        (Era::Current, Codec::Ncm, 1) => (&WINDOW1_PREFIX, &NCM1_SUFFIX),
        (Era::Current, Codec::Ncm, 2) => (&WINDOW2_PREFIX, &NCM2_SUFFIX),
        (Era::Current, Codec::NcmX86, 1) => (&WINDOW1X_PREFIX, &NCM1X_SUFFIX),
        (Era::Current, Codec::NcmX86, 2) => (&WINDOW2X_PREFIX, &NCM2X_SUFFIX),
        _ => return None,
    };
    Some((prefix, suffix.to_vec()))
}

/// (prefix, tail) of the profiled entry over `k` window frames.
fn profile_template(k: usize, x86: bool) -> Option<(&'static [u8], &'static [u8])> {
    Some(match (k, x86) {
        (0, false) => (&NCMP_PREFIX, &NCMP_TAIL),
        (1, false) => (&NCMP1_PREFIX, &NCMP1_TAIL),
        (2, false) => (&NCMP2_PREFIX, &NCMP2_TAIL),
        (0, true) => (&NCMPX_PREFIX, &NCMPX_TAIL),
        (1, true) => (&NCMP1X_PREFIX, &NCMP1X_TAIL),
        (2, true) => (&NCMP2X_PREFIX, &NCMP2X_TAIL),
        _ => return None,
    })
}

/// The codec and payload of a body over `k` window frames in a basis of
/// `era`, when it is laid out exactly as this client writes one.
fn parse_body(body: &[u8], k: usize, era: Era) -> Option<(Codec, &[u8])> {
    // `prefix uleb(len) payload` with a shortest length: (payload, the rest)
    let split = |prefix: &[u8]| -> Option<(&[u8], &[u8])> {
        let rest = body.strip_prefix(prefix)?;
        let (p, after) = read_uleb(rest, 0)?;
        let p = usize::try_from(p).ok()?;
        if after != uleb_len(p as u64) || rest.len() < after.checked_add(p)? {
            return None;
        }
        Some(rest[after..].split_at(p))
    };
    let fixed = Codec::ALL.into_iter().find_map(|c| {
        let (prefix, suffix) = template(c, k, era)?;
        let (payload, rest) = split(prefix)?;
        (rest == &suffix[..]).then_some((c, payload))
    });
    if fixed.is_some() || era != Era::Current {
        return fixed;
    }
    [false, true].into_iter().find_map(|x86| {
        let (prefix, tail) = profile_template(k, x86)?;
        let (payload, rest) = split(prefix)?;
        let rest = rest.strip_prefix(&[0])?;
        let (p, after) = read_uleb(rest, 0)?;
        let p = u32::try_from(p).ok()?;
        let ok = after == uleb_len(u64::from(p))
            && crate::ncm::valid_profile(p)
            && &rest[after..] == tail;
        ok.then_some((Codec::profiled(p, x86), payload))
    })
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
    chain_parts(&[preimage])
}

fn chain_block(c: &[u8; 32], block: &[u8]) -> [u8; 32] {
    Sha256::new()
        .chain_update(CHAIN_TAG)
        .chain_update([1u8])
        .chain_update(c)
        .chain_update(block)
        .finalize()
        .into()
}

/// [`chain`] of the concatenation of `parts`, without joining them.
pub fn chain_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut c: [u8; 32] = Sha256::new()
        .chain_update(CHAIN_TAG)
        .chain_update([0u8])
        .finalize()
        .into();
    let mut carry = [0u8; 64];
    let mut held = 0usize;
    for part in parts {
        let mut rest = *part;
        if held > 0 {
            let take = rest.len().min(64 - held);
            carry[held..held + take].copy_from_slice(&rest[..take]);
            held += take;
            rest = &rest[take..];
            if held < 64 {
                continue;
            }
            c = chain_block(&c, &carry);
        }
        let mut blocks = rest.chunks_exact(64);
        for block in &mut blocks {
            c = chain_block(&c, block);
        }
        let tail = blocks.remainder();
        carry[..tail.len()].copy_from_slice(tail);
        held = tail.len();
    }
    Sha256::new()
        .chain_update(CHAIN_TAG)
        .chain_update([2u8])
        .chain_update(c)
        .chain_update([held as u8])
        .chain_update(&carry[..held])
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

fn body_len(codec: Codec, k: usize, p: u64) -> Option<u64> {
    let (prefix, suffix) = template(codec, k, Era::Current)?;
    Some(prefix.len() as u64 + uleb_len(p) as u64 + p + suffix.len() as u64)
}

/// `Archive.linkedArchive`: a conditional archive over the standard basis whose
/// entry applies `codec`'s decoder to `payload`, reconstructing objects of
/// `lengths` (one object in the model; several when the client relabels).
/// `None` for a profile word that is not a [`crate::ncm::valid_profile`].
pub fn linked_archive(codec: Codec, lengths: &[u64], payload: &[u8]) -> Option<Vec<u8>> {
    entry_archive(codec, 0, Era::Current, &BASIS_ID, lengths, payload)
}

/// The archive over the basis named `identity` (`era`'s library and the `k`
/// window frames) whose entry is `codec`'s template for `era`.
fn entry_archive(
    codec: Codec,
    k: usize,
    era: Era,
    identity: &[u8; 32],
    lengths: &[u64],
    payload: &[u8],
) -> Option<Vec<u8>> {
    let (prefix, suffix) = template(codec, k, era)?;
    let p = payload.len() as u64;
    let body_len = prefix.len() as u64 + uleb_len(p) as u64 + p + suffix.len() as u64;
    let mut pre = header(1, lengths, body_len);
    pre[6] = 1;
    pre.extend_from_slice(identity);
    pre.reserve(body_len as usize + SEAL);
    pre.extend_from_slice(prefix);
    push_uleb(&mut pre, p);
    pre.extend_from_slice(payload);
    pre.extend_from_slice(&suffix);
    Some(sealed(pre))
}

/// The plain codec a profiled frame keeps: ncm, or ncm over the x86 filter.
fn plain(codec: Codec) -> Codec {
    if codec.x86() {
        Codec::NcmX86
    } else {
        Codec::Ncm
    }
}

/// [`linked_archive`] of the profiled entry for `codec` ([`Codec::Ncm`] or
/// [`Codec::NcmX86`], whose x86 filter it keeps) under `profile`
/// (`Archive.profileCandidate` with no window). Profile [`crate::ncm::V03`] is the
/// v0.3 coder, so its frame is the v0.3 archive itself (`Archive.v03Candidate`).
pub fn linked_archive_profile(
    codec: Codec,
    profile: u32,
    lengths: &[u64],
    payload: &[u8],
) -> Option<Vec<u8>> {
    if profile == crate::ncm::V03 {
        return entry_archive(plain(codec), 0, Era::V03, &V03_BASIS_ID, lengths, payload);
    }
    linked_archive(Codec::profiled(profile, codec.x86()), lengths, payload)
}

/// [`window_archive`] of the profiled entry for `codec` under `profile`
/// (`Archive.profileCandidate` over the frames of `window`, whose current
/// [`window_identity`] is `identity`). Profile [`crate::ncm::V03`] writes the v0.3
/// window frame over the v0.3 library.
pub fn window_archive_profile(
    codec: Codec,
    profile: u32,
    window: &[&[u8]],
    identity: &[u8; 32],
    lengths: &[u64],
    payload: &[u8],
) -> Option<Vec<u8>> {
    if profile == crate::ncm::V03 {
        let v03 = window_identity_over(&V03_LIBRARY, window);
        return entry_archive(plain(codec), window.len(), Era::V03, &v03, lengths, payload);
    }
    window_archive(
        Codec::profiled(profile, codec.x86()),
        window.len(),
        identity,
        lengths,
        payload,
    )
}

/// Exact length of [`linked_archive`] for objects of `lengths` and a `p`-byte payload.
pub fn linked_length(codec: Codec, lengths: &[u64], p: u64) -> u64 {
    let body = body_len(codec, 0, p).unwrap_or(u64::MAX / 4);
    let fields: u64 = lengths.iter().map(|&l| uleb_len(l) as u64).sum();
    8 + uleb_len(lengths.len() as u64) as u64
        + fields
        + uleb_len(body) as u64
        + 32
        + body
        + SEAL as u64
}

/// `Archive.windowIdentity`: the identity of the basis `[library, window..]`,
/// the UORC-BASIS-V1 chain over its roots, oldest frame first.
pub fn window_identity(window: &[&[u8]]) -> [u8; 32] {
    window_identity_over(&LIBRARY, window)
}

fn window_identity_over(library: &[u8], window: &[&[u8]]) -> [u8; 32] {
    let mut head = BASIS_TAG.to_vec();
    push_uleb(&mut head, 1 + window.len() as u64);
    push_uleb(&mut head, library.len() as u64);
    let lengths: Vec<Vec<u8>> = window
        .iter()
        .map(|r| {
            let mut l = Vec::new();
            push_uleb(&mut l, r.len() as u64);
            l
        })
        .collect();
    let mut parts: Vec<&[u8]> = vec![&head, library];
    for (l, r) in lengths.iter().zip(window) {
        parts.push(l);
        parts.push(r);
    }
    chain_parts(&parts)
}

/// `Archive.windowArchive`: a conditional archive over `[library, the k
/// preceding frames]` (named by `identity`) whose entry decodes the `codec`
/// `payload` with those frames as history, reconstructing objects of `lengths`.
pub fn window_archive(
    codec: Codec,
    k: usize,
    identity: &[u8; 32],
    lengths: &[u64],
    payload: &[u8],
) -> Option<Vec<u8>> {
    if k == 0 {
        return None;
    }
    entry_archive(codec, k, Era::Current, identity, lengths, payload)
}

const REF_HEX: &str = include_str!("linked-ref.hex");
/// Body of a reference frame (`gen_basis.py` `REF_BODY`): one entry block that
/// returns basis root 1. Over the basis `[library, F]` it reproduces `F`, so an
/// earlier frame is named by its κ-label ([`frame_label`]) and stored once.
pub const REF_BODY: [u8; hex_len(REF_HEX)] = hex_literal(REF_HEX);

/// The κ-label of a frame's content: the identity of the basis `[library,
/// frame]` ([`window_identity`] of the one frame), which a reference frame
/// names to reproduce it.
pub fn frame_label(frame: &[u8]) -> [u8; 32] {
    window_identity(&[frame])
}

/// The κ-labels a reference frame may name for `frame`: its label over the
/// current library ([`frame_label`]) and over the v0.3 library, which a v0.3
/// stream's reference frames name.
pub fn frame_labels(frame: &[u8]) -> [[u8; 32]; 2] {
    [frame_label(frame), window_identity_over(&V03_LIBRARY, &[frame])]
}

/// A reference frame: a conditional archive over `[library, F]`, named by `F`'s
/// κ-label `label`, that reproduces `F` as objects of `lengths`.
pub fn reference_archive(label: &[u8; 32], lengths: &[u64]) -> Vec<u8> {
    let mut pre = header(1, lengths, REF_BODY.len() as u64);
    pre[6] = 1;
    pre.extend_from_slice(label);
    pre.extend_from_slice(&REF_BODY);
    sealed(pre)
}

/// Exact length of [`reference_archive`] for objects of `lengths`.
pub fn reference_length(lengths: &[u64]) -> u64 {
    let fields: u64 = lengths.iter().map(|&l| uleb_len(l) as u64).sum();
    let body = REF_BODY.len() as u64;
    8 + uleb_len(lengths.len() as u64) as u64
        + fields
        + uleb_len(body) as u64
        + 32
        + body
        + SEAL as u64
}

/// Where the fields of a conditional program archive sit: the object lengths,
/// the basis identity and the body. `None` for any other shape.
fn conditional_fields(data: &[u8]) -> Option<(Vec<u64>, [u8; 32], &[u8])> {
    if data.len() < 8 + 32 + SEAL || data[..6] != MAGIC[..] || data[6] != 1 || data[7] != 1 {
        return None;
    }
    let (count, mut at) = read_uleb(data, 8)?;
    if count > data.len() as u64 {
        return None;
    }
    let mut lengths = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (n, next) = read_uleb(data, at)?;
        lengths.push(n);
        at = next;
    }
    let (body_len, at) = read_uleb(data, at)?;
    let start = at.checked_add(32)?;
    let end = start.checked_add(usize::try_from(body_len).ok()?)?;
    if data.len() != end.checked_add(SEAL)? {
        return None;
    }
    Some((lengths, data[at..start].try_into().ok()?, &data[start..end]))
}

/// The κ-label a reference frame names, read from its fields (the seal is
/// checked when it is decoded); `None` when `data` is not one.
pub fn reference_of(data: &[u8]) -> Option<[u8; 32]> {
    let (_, label, body) = conditional_fields(data)?;
    (body == REF_BODY).then_some(label)
}

/// `Archive.windowDecode` of a reference frame, natively: `frame` (the frame
/// whose κ-label the archive names) when the archive is laid out exactly as
/// [`reference_archive`] writes it, its seal holds and its objects add up to
/// `frame`. The caller found `frame` by that label.
pub fn reference_decode(data: &[u8], frame: &[u8]) -> Option<Vec<u8>> {
    let (lengths, label, body) = conditional_fields(data)?;
    let n = lengths.iter().try_fold(0u64, |a, &l| a.checked_add(l))?;
    let (pre, seal) = data.split_at(data.len() - SEAL);
    let shortest = reference_length(&lengths) == data.len() as u64;
    (body == REF_BODY && shortest && n == frame.len() as u64 && seal == chain(pre)).then(|| {
        debug_assert!(frame_labels(frame).contains(&label));
        frame.to_vec()
    })
}

/// The basis identity a conditional archive names, if it is one.
pub fn basis_of(data: &[u8]) -> Option<[u8; 32]> {
    if data.len() < 8 || data[..6] != MAGIC[..] || data[6] != 1 {
        return None;
    }
    let (count, mut at) = read_uleb(data, 8)?;
    for _ in 0..count {
        at = read_uleb(data, at)?.1;
    }
    let at = read_uleb(data, at)?.1;
    data.get(at..at + 32)?.try_into().ok()
}

/// How many frames before it a frame reads: 0 for an archive that is not a
/// window frame, 1 or 2 for a window frame this client writes (its entry
/// names the count), and `None` for a window frame of another layout, which
/// reads up to [`WINDOW`] frames as the v0.2 encoder wrote them. The decoder
/// schedules from this alone: a frame that reads no window starts a block,
/// and blocks decode independently.
pub fn window_len(data: &[u8]) -> Option<usize> {
    let Some(basis) = basis_of(data) else {
        return Some(0);
    };
    if Era::of(&basis).is_some() || reference_of(data).is_some() {
        return Some(0);
    }
    let (count, mut at) = read_uleb(data, 8)?;
    for _ in 0..count {
        at = read_uleb(data, at)?.1;
    }
    let body = data.get(read_uleb(data, at)?.1 + 32..)?;
    [
        (&WINDOW1_PREFIX[..], 1),
        (&WINDOW1X_PREFIX[..], 1),
        (&WINDOW2_PREFIX[..], 2),
        (&WINDOW2X_PREFIX[..], 2),
        (&NCMP1_PREFIX[..], 1),
        (&NCMP1X_PREFIX[..], 1),
        (&NCMP2_PREFIX[..], 2),
        (&NCMP2X_PREFIX[..], 2),
    ]
    .into_iter()
    .find_map(|(prefix, k)| body.starts_with(prefix).then_some(k))
}

/// The codec, basis era and payload of a window body over `k` frames laid
/// out exactly as [`window_archive`] (or a v0.3 or v0.2 client) writes it.
/// The entries of the eras differ, so at most one matches.
fn window_payload(body: &[u8], k: usize) -> Option<(Codec, Era, &[u8])> {
    Era::ALL.into_iter().find_map(|era| {
        let (codec, payload) = parse_body(body, k, era)?;
        Some((codec, era, payload))
    })
}

/// The codec of a window frame and how many frames it reads, when it is laid
/// out exactly as [`window_archive`] writes it.
pub fn window_codec(data: &[u8]) -> Option<(Codec, usize)> {
    let k = window_len(data).filter(|&k| k > 0)?;
    let (_, _, body) = conditional_fields(data)?;
    Some((window_payload(body, k)?.0, k))
}

/// `Archive.windowDecode`, natively: the output of a window archive over
/// `window` laid out exactly as [`window_archive`] writes it, with a payload
/// the native decoder vouches for. The basis identity is checked on another
/// core while the payload decodes.
pub fn window_decode(data: &[u8], window: &[&[u8]]) -> Option<Vec<u8>> {
    let k = window.len();
    if !(1..=WINDOW).contains(&k) {
        return None;
    }
    if data.len() < 8 + 32 + SEAL || data[..6] != MAGIC[..] || data[6] != 1 || data[7] != 1 {
        return None;
    }
    let (count, mut at) = read_uleb(data, 8)?;
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
    let start = at.checked_add(32)?;
    if data.len() != start.checked_add(body_len)?.checked_add(SEAL)? {
        return None;
    }
    let (pre, seal) = data.split_at(data.len() - SEAL);
    let identity: [u8; 32] = pre[at..start].try_into().ok()?;
    let mut expected = header(1, &lengths, body_len as u64);
    expected[6] = 1;
    if pre[..at] != expected[..] {
        return None;
    }
    let (codec, era, payload) = window_payload(&pre[start..], k)?;
    let history = window.concat();
    let decode = || {
        if !codec.x86() {
            return decode_with(codec, &history, payload, n);
        }
        let h = crate::x86::filter(&history, false, 0).0;
        let out = decode_with(codec, &h, payload, n)?;
        Some(crate::x86::filter(&out, true, history.len() as u64).0)
    };
    let library = era.library();
    let check = || seal == chain(pre) && identity == window_identity_over(library, window);
    #[cfg(not(target_family = "wasm"))]
    let (output, ok) = std::thread::scope(|scope| {
        let checked = scope.spawn(check);
        let output = decode();
        (output, checked.join().unwrap_or(false))
    });
    #[cfg(target_family = "wasm")]
    let (output, ok) = (decode(), check());
    if ok {
        output
    } else {
        None
    }
}

/// The bytes a window payload codes (before any x86 inverse), with the native
/// decoder's guarantee that the payload is the encoder's.
fn decode_with(codec: Codec, history: &[u8], payload: &[u8], n: u64) -> Option<Vec<u8>> {
    match codec.ncm_profile() {
        None => crate::ulz::decode_with(history, payload, n),
        Some(p) => crate::ncm::decode_canonical_profile(history, payload, n, p),
    }
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
    // a linked archive names this basis or an earlier one
    let named = if linked { at - 32 } else { at };
    if pre[..named] != expected[..named] || seal != chain(pre) {
        return None;
    }
    let body = &pre[at..];
    if linked {
        return linked_decode(body, n, Era::of(&pre[named..at])?);
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

/// The library decoder a body applies over the standard basis named
/// `identity` (this one or an earlier one), when it is laid out exactly as
/// [`linked_archive`] writes it.
pub fn linked_codec(identity: &[u8], body: &[u8]) -> Option<Codec> {
    parse_body(body, 0, Era::of(identity)?).map(|(c, _)| c)
}

/// The output of a linked archive's body over a basis of `era`, laid out
/// exactly as [`linked_archive`] writes it, with the payload its encoder
/// writes for that output (`Archive.linkedDecode`). `None` sends it to the
/// reference decoder.
fn linked_decode(body: &[u8], n: u64, era: Era) -> Option<Vec<u8>> {
    let (codec, payload) = parse_body(body, 0, era)?;
    let out = match codec.ncm_profile() {
        None => crate::ulz::decode(payload, n),
        Some(p) => crate::ncm::decode_canonical_profile(&[], payload, n, p),
    }?;
    Some(if codec.x86() {
        crate::x86::filter(&out, true, 0).0
    } else {
        out
    })
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
        let u = linked_archive(Codec::Ulz, &[n], &crate::ulz::encode(&input)).expect("codec");
        let c = linked_archive(Codec::Ncm, &[n], &crate::ncm::encode(&input).expect("fits"))
            .expect("codec");
        // the model's archives for this text (client/gen/gen_basis.py, Archive.ulzLinkedArchive)
        assert_eq!((u.len(), c.len()), (166, 140));
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

    /// Archives the v0.2.0 and v0.3.0 release binaries wrote still decode
    /// natively, to the bytes they were written from.
    #[test]
    fn earlier_releases_decode() {
        let text = include_bytes!("../tests/compat/text.bin");
        let x86 = include_bytes!("../tests/compat/x86.bin");
        for (archive, original, era) in [
            (&include_bytes!("../tests/compat/text-v0.2.0.uorc")[..], &text[..], Era::V02),
            (include_bytes!("../tests/compat/text-v0.2.0-ncm.uorc"), text, Era::V02),
            (include_bytes!("../tests/compat/x86-v0.2.0.uorc"), x86, Era::V02),
            (include_bytes!("../tests/compat/text-v0.3.0.uorc"), text, Era::V03),
            (include_bytes!("../tests/compat/x86-v0.3.0.uorc"), x86, Era::V03),
        ] {
            assert_eq!(basis_of(archive).and_then(|b| Era::of(&b)), Some(era));
            assert_eq!(fast_decode(archive).as_deref(), Some(original));
            assert_eq!(window_len(archive), Some(0));
        }
    }

    /// A profiled entry carries its profile word; the archive decodes under
    /// that word alone, linked and over a window, and only a shortest word
    /// below 2^20 is read as one.
    #[test]
    fn profiled_archives_round_trip() {
        let a = b"profiled frames carry the word that names their coder. ".repeat(20);
        let b = b"the second frame reads the first through its window. ".repeat(20);
        let n = b.len() as u64;
        for profile in [crate::ncm::FULL, crate::ncm::V03, 0x1059e | crate::ncm::F_REP, 0] {
            for x86 in [false, true] {
                let codec = if x86 { Codec::NcmX86 } else { Codec::Ncm };
                let (history, data) = if x86 {
                    let h = crate::x86::filter(&a, false, 0).0;
                    let d = crate::x86::filter(&b, false, a.len() as u64).0;
                    (h, d)
                } else {
                    (a.clone(), b.clone())
                };
                let lone = if x86 { crate::x86::filter(&b, false, 0).0 } else { b.clone() };
                let p = crate::ncm::encode_profile(&[], &lone, profile).expect("fits");
                // the v0.3 coder's frame is the v0.3 archive: old-shape entry, v0.3 identity
                let v03 = profile == crate::ncm::V03;
                let (expect, era) = if v03 {
                    (codec, Era::V03)
                } else {
                    (Codec::profiled(profile, x86), Era::Current)
                };
                let linked = linked_archive_profile(codec, profile, &[n], &p).expect("profile");
                assert_eq!(fast_decode(&linked).as_ref(), Some(&b));
                if !v03 {
                    assert_eq!(linked.len() as u64, linked_length(expect, &[n], p.len() as u64));
                }
                let (_, _, body) = conditional_fields(&linked).expect("fields");
                assert_eq!(linked_codec(era.identity(), body), Some(expect));
                let w = crate::ncm::encode_profile(&history, &data, profile).expect("fits");
                let id = window_identity(&[&a]);
                let windowed =
                    window_archive_profile(codec, profile, &[&a], &id, &[n], &w).expect("profile");
                assert_eq!(window_len(&windowed), Some(1));
                assert_eq!(window_codec(&windowed), Some((expect, 1)));
                assert_eq!(window_decode(&windowed, &[&a]).as_ref(), Some(&b));
                // the payload under another word is not the encoder's
                let other = if profile == crate::ncm::FULL { crate::ncm::V03 } else { crate::ncm::FULL };
                let wrong = linked_archive_profile(codec, other, &[n], &p).expect("profile");
                assert_eq!(fast_decode(&wrong), None);
            }
        }
        let p = crate::ncm::encode(b"abc").expect("fits");
        assert_eq!(linked_archive(Codec::Profile(1 << 20), &[3], &p), None);
        // a profile word written in a longer form than the shortest
        let a = linked_archive(Codec::Profile(1), &[3], &p).expect("profile");
        let (_, _, body) = conditional_fields(&a).expect("fields");
        let at = body.len() - NCMP_TAIL.len() - 1;
        assert_eq!(body[at - 1..at + 1], [0, 1]);
        let mut long = body[..at].to_vec();
        long.extend_from_slice(&[0x81, 0x00]);
        long.extend_from_slice(&NCMP_TAIL);
        assert_eq!(parse_body(&long, 0, Era::Current), None);
    }

    #[test]
    fn linked_archive_names_the_basis() {
        let mut a = linked_archive(Codec::Ncm, &[3], &crate::ncm::encode(b"abc").expect("fits"))
            .expect("codec");
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
