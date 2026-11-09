//! Reading a UORC archive's frame header and integrity seal without decoding
//! its body.
//!
//! The layout is the one `Uorc.Wire.FrameParse` accepts:
//!
//! ```text
//! "UORC" format profile scope representation
//! uleb(objectCount) uleb(objectLength)* uleb(bodyLength)
//! basis[32]          -- only when scope = 1 (conditional)
//! body[bodyLength]
//! digest[32]         -- UORC-SHA256-CHAIN-v1 over everything before it
//! ```
//!
//! Numbers are shortest-form ULEB128. A raw body's length equals the sum of
//! the object lengths. This module only reads that framing; the body is
//! decoded and checked by the certified decoder in [`crate::decompress`].

use sha2::{Digest, Sha256};

use crate::Error;

const MAGIC: &[u8; 4] = b"UORC";
const DIGEST_LEN: usize = 32;
const BASIS_LEN: usize = 32;
const CHAIN_TAG: &[u8] = b"UORC-SHA256-CHAIN-v1\0";

/// How an archive's body produces its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    /// The body is the output bytes, stored as is.
    Raw,
    /// The body is the ulz decoder kernel graph with an LZ + range-coded payload.
    UlzKernel,
    /// The body is any other UORC program graph (for example a search result).
    Program,
}

impl Representation {
    /// A short lowercase name: `raw`, `ulz-kernel` or `program`.
    pub fn name(self) -> &'static str {
        match self {
            Representation::Raw => "raw",
            Representation::UlzKernel => "ulz-kernel",
            Representation::Program => "program",
        }
    }
}

/// What an archive's frame header says, read without decoding the body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// Archive format version (2 for this client).
    pub format: u8,
    /// Machine profile (1 for this client).
    pub profile: u8,
    /// The basis identity of a conditional archive; `None` when self-contained.
    pub basis: Option<[u8; BASIS_LEN]>,
    /// How the body produces the output.
    pub representation: Representation,
    /// Byte length of each output object, in order.
    pub object_lengths: Vec<u64>,
    /// Total output length: the sum of `object_lengths`.
    pub output_len: u64,
    /// Length of the body.
    pub body_len: u64,
    /// Length of the whole archive.
    pub archive_len: u64,
    /// The sealed integrity digest at the end of the archive.
    pub digest: [u8; DIGEST_LEN],
    /// Whether the digest matches the archive's bytes.
    pub integrity_ok: bool,
}

impl Info {
    /// Name of the integrity profile that sealed the archive.
    pub fn integrity_profile(&self) -> &'static str {
        "UORC-SHA256-CHAIN-v1"
    }

    /// The digest as lowercase hex.
    pub fn digest_hex(&self) -> String {
        hex(&self.digest)
    }

    /// One JSON object describing the archive (no external JSON dependency).
    pub fn to_json(&self) -> String {
        let lengths: Vec<String> = self.object_lengths.iter().map(u64::to_string).collect();
        format!(
            "{{\"format\":{},\"profile\":{},\"scope\":\"{}\",\"basis\":{},\"representation\":\"{}\",\
             \"objects\":{},\"object_lengths\":[{}],\"output_bytes\":{},\"body_bytes\":{},\
             \"archive_bytes\":{},\"integrity\":\"{}\",\"digest\":\"{}\",\"integrity_ok\":{}}}",
            self.format,
            self.profile,
            if self.basis.is_some() { "conditional" } else { "self-contained" },
            self.basis.map_or_else(|| "null".to_owned(), |b| format!("\"{}\"", hex(&b))),
            self.representation.name(),
            self.object_lengths.len(),
            lengths.join(","),
            self.output_len,
            self.body_len,
            self.archive_len,
            self.integrity_profile(),
            self.digest_hex(),
            self.integrity_ok,
        )
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn byte(&mut self) -> Result<u8, Error> {
        let b = *self.data.get(self.at).ok_or(Error::Decode)?;
        self.at += 1;
        Ok(b)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self.at.checked_add(n).ok_or(Error::Decode)?;
        let s = self.data.get(self.at..end).ok_or(Error::Decode)?;
        self.at = end;
        Ok(s)
    }

    /// Shortest-form ULEB128 that fits in 64 bits.
    fn uleb(&mut self) -> Result<u64, Error> {
        let (mut value, mut shift) = (0u64, 0u32);
        loop {
            let b = self.byte()?;
            let part = u64::from(b & 0x7f);
            if shift >= 64 || (shift == 63 && part > 1) {
                return Err(Error::Decode);
            }
            value |= part << shift;
            if b & 0x80 == 0 {
                if b == 0 && shift > 0 {
                    return Err(Error::Decode); // non-shortest
                }
                return Ok(value);
            }
            shift += 7;
        }
    }
}

/// The ulz kernel graph's bytes before its payload length, taken from the
/// codec source so this check follows any regeneration of the kernel.
fn kernel_prefix() -> &'static [u8] {
    use std::sync::OnceLock;
    static PREFIX: OnceLock<Vec<u8>> = OnceLock::new();
    PREFIX.get_or_init(|| {
        let source = include_str!("../../../client/src/Uorc/Codec/Archive.lex.tex");
        let def = source
            .split("name: kernelPrefix,")
            .nth(1)
            .expect("kernelPrefix in Archive.lex.tex");
        let text = def
            .split("body: 0x")
            .nth(1)
            .expect("kernelPrefix body")
            .split(|c: char| !c.is_ascii_hexdigit())
            .next()
            .expect("hex literal");
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
            .collect()
    })
}

/// The UORC-SHA256-CHAIN-v1 digest of `preimage`.
pub(crate) fn chain_digest(preimage: &[u8]) -> [u8; DIGEST_LEN] {
    let mut c: [u8; DIGEST_LEN] = Sha256::new()
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

/// Where an archive's fields sit, from [`parse`].
pub(crate) struct Layout {
    /// Offset of the body-length field (just after the object lengths).
    body_len_at: usize,
    /// End of the sealed preimage (start of the digest).
    sealed: usize,
    /// End of the archive.
    end: usize,
}

/// Parse one archive at the start of `data`, which may continue past it.
fn parse(data: &[u8]) -> Result<(Info, Layout), Error> {
    let mut r = Reader { data, at: 0 };
    if r.take(4)? != MAGIC {
        return Err(Error::Decode);
    }
    let format = r.byte()?;
    let profile = r.byte()?;
    let conditional = match r.byte()? {
        0 => false,
        1 => true,
        _ => return Err(Error::Decode),
    };
    let program = match r.byte()? {
        0 => false,
        1 => true,
        _ => return Err(Error::Decode),
    };
    let count = r.uleb()?;
    // Each length takes at least one byte, which bounds the allocation.
    if count > data.len() as u64 {
        return Err(Error::Decode);
    }
    let mut object_lengths = Vec::with_capacity(count as usize);
    let mut output_len = 0u64;
    for _ in 0..count {
        let n = r.uleb()?;
        output_len = output_len.checked_add(n).ok_or(Error::Overflow)?;
        object_lengths.push(n);
    }
    let body_len_at = r.at;
    let body_len = r.uleb()?;
    if !program && body_len != output_len {
        return Err(Error::Decode);
    }
    let basis = if conditional {
        Some(r.take(BASIS_LEN)?.try_into().expect("32 bytes"))
    } else {
        None
    };
    let body = r.take(usize::try_from(body_len).map_err(|_| Error::Decode)?)?;
    let sealed = r.at;
    let digest: [u8; DIGEST_LEN] = r.take(DIGEST_LEN)?.try_into().expect("32 bytes");
    let end = r.at;
    let representation = if !program {
        Representation::Raw
    } else if body.starts_with(kernel_prefix()) {
        Representation::UlzKernel
    } else {
        Representation::Program
    };
    let info = Info {
        format,
        profile,
        basis,
        representation,
        object_lengths,
        output_len,
        body_len,
        archive_len: end as u64,
        digest,
        integrity_ok: chain_digest(&data[..sealed]) == digest,
    };
    Ok((
        info,
        Layout {
            body_len_at,
            sealed,
            end,
        },
    ))
}

/// Read one archive's frame header and check its integrity seal, without
/// decoding. The bytes must be exactly one archive.
pub fn inspect(archive: &[u8]) -> Result<Info, Error> {
    let (info, layout) = parse(archive)?;
    if layout.end != archive.len() {
        return Err(Error::Decode);
    }
    Ok(info)
}

/// Split a stream of archives written back to back into its archives. A
/// single archive is a stream of one; an empty stream has none.
pub fn frames(stream: &[u8]) -> Result<Vec<&[u8]>, Error> {
    let mut out = Vec::new();
    let mut rest = stream;
    while !rest.is_empty() {
        let (_, layout) = parse(rest)?;
        let (frame, tail) = rest.split_at(layout.end);
        out.push(frame);
        rest = tail;
    }
    Ok(out)
}

/// The same archive with its output split into `lengths` instead, resealed.
/// Only the object lengths and the seal change: the body, and so the bytes it
/// produces, stay as they are, which is why the lengths must keep their sum.
/// The original seal must be intact.
pub(crate) fn relabel(archive: &[u8], lengths: &[u64]) -> Result<Vec<u8>, Error> {
    let (info, layout) = parse(archive)?;
    let sum = lengths
        .iter()
        .try_fold(0u64, |a, &n| a.checked_add(n))
        .ok_or(Error::Overflow)?;
    if layout.end != archive.len() || !info.integrity_ok || sum != info.output_len {
        return Err(Error::Decode);
    }
    let mut out = archive[..8].to_vec();
    push_uleb(&mut out, lengths.len() as u64);
    for &n in lengths {
        push_uleb(&mut out, n);
    }
    out.extend_from_slice(&archive[layout.body_len_at..layout.sealed]);
    let digest = chain_digest(&out);
    out.extend_from_slice(&digest);
    Ok(out)
}

fn push_uleb(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}
