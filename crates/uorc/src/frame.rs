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
//! decoded and checked in [`crate::decompress_frame`].

use crate::archive::{chain, push_uleb, Codec, KERNEL_PREFIX, SEAL as DIGEST_LEN};
use crate::Error;

const MAGIC: &[u8; 4] = b"UORC";
const BASIS_LEN: usize = 32;

/// How an archive's body produces its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Representation {
    /// The body is the output bytes, stored as is.
    Raw,
    /// The body is the ulz decoder kernel graph with an LZ + range-coded payload.
    UlzKernel,
    /// The body applies the standard basis's ulz decoder to an LZ + range-coded payload.
    Ulz,
    /// The body applies the standard basis's ncm (context-mixing) decoder to its payload.
    Ncm,
    /// [`Representation::Ncm`] over the x86 branch filter.
    NcmX86,
    /// The body reproduces earlier bytes of the stream (a run of content-defined
    /// units of an earlier frame), named by their κ-label (the basis identity).
    Reference,
    /// The body is any other UORC program graph (for example a search result).
    Program,
}

impl Representation {
    /// A short lowercase name: `raw`, `ulz`, `ncm`, `ncm-x86`, `reference`, `ulz-kernel` or `program`.
    pub fn name(self) -> &'static str {
        match self {
            Representation::Raw => "raw",
            Representation::UlzKernel => "ulz-kernel",
            Representation::Ulz => "ulz",
            Representation::Ncm => "ncm",
            Representation::NcmX86 => "ncm-x86",
            Representation::Reference => "reference",
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
    /// How many frames before it a window frame reads as history (its basis
    /// is the standard library plus those frames); 0 for any other archive.
    pub window: usize,
    /// The ncm profile word the payload is coded under
    /// ([`crate::ncm::valid_profile`]; [`crate::ncm::V03`] for the v0.3
    /// entries), `None` when the body applies no ncm decoder.
    pub ncm_profile: Option<u32>,
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
            "{{\"format\":{},\"profile\":{},\"scope\":\"{}\",\"basis\":{},\"representation\":\"{}\",\"ncm_profile\":{},\"window\":{},\
             \"objects\":{},\"object_lengths\":[{}],\"output_bytes\":{},\"body_bytes\":{},\
             \"archive_bytes\":{},\"integrity\":\"{}\",\"digest\":\"{}\",\"integrity_ok\":{},\"steps\":{}}}",
            self.format,
            self.profile,
            if self.basis.is_some() { "conditional" } else { "self-contained" },
            self.basis.map_or_else(|| "null".to_owned(), |b| format!("\"{}\"", hex(&b))),
            self.representation.name(),
            self.ncm_profile
                .map_or_else(|| "null".to_owned(), |p| format!("\"0x{p:05x}\"")),
            self.window,
            self.object_lengths.len(),
            lengths.join(","),
            self.output_len,
            self.body_len,
            self.archive_len,
            self.integrity_profile(),
            self.digest_hex(),
            self.integrity_ok,
            crate::performance::frame_steps(self).map_or_else(|| "null".to_owned(), |n| n.to_string()),
        )
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
    /// Set when a read ran past the end of `data`: more bytes might parse.
    short: bool,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Reader {
            data,
            at: 0,
            short: false,
        }
    }

    fn byte(&mut self) -> Result<u8, Error> {
        let Some(&b) = self.data.get(self.at) else {
            self.short = true;
            return Err(Error::Decode);
        };
        self.at += 1;
        Ok(b)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self.at.checked_add(n).ok_or(Error::Decode)?;
        let Some(s) = self.data.get(self.at..end) else {
            self.short = true;
            return Err(Error::Decode);
        };
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

/// Where an archive's fields sit, from [`parse`].
pub(crate) struct Layout {
    /// Offset of the body-length field (just after the object lengths).
    body_len_at: usize,
    /// End of the sealed preimage (start of the digest).
    sealed: usize,
    /// End of the archive.
    end: usize,
}

/// An archive header: everything before the body.
struct Head {
    format: u8,
    profile: u8,
    program: bool,
    basis: Option<[u8; BASIS_LEN]>,
    object_lengths: Vec<u64>,
    output_len: u64,
    body_len_at: usize,
    body_len: u64,
}

fn head(r: &mut Reader<'_>) -> Result<Head, Error> {
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
    if count > (r.data.len() - r.at) as u64 {
        r.short = true;
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
    Ok(Head {
        format,
        profile,
        program,
        basis,
        object_lengths,
        output_len,
        body_len_at,
        body_len,
    })
}

/// Total length of the archive that starts at `data[0]`, read from its header
/// alone: `Ok(None)` when `data` ends before the header does.
pub fn frame_length(data: &[u8]) -> Result<Option<usize>, Error> {
    let mut r = Reader::new(data);
    match head(&mut r) {
        Ok(h) => usize::try_from(h.body_len)
            .ok()
            .and_then(|b| r.at.checked_add(b)?.checked_add(DIGEST_LEN))
            .map(Some)
            .ok_or(Error::Decode),
        Err(_) if r.short => Ok(None),
        Err(e) => Err(e),
    }
}

/// Bytes of header worth reading before a first [`frame_length`] attempt: a
/// one-object header always fits.
pub const HEADER_HINT: usize = 8 + 10 * 3;

/// Parse one archive at the start of `data`, which may continue past it.
fn parse(data: &[u8]) -> Result<(Info, Layout), Error> {
    let mut r = Reader::new(data);
    let h = head(&mut r)?;
    let body = r.take(usize::try_from(h.body_len).map_err(|_| Error::Decode)?)?;
    let sealed = r.at;
    let digest: [u8; DIGEST_LEN] = r.take(DIGEST_LEN)?.try_into().expect("32 bytes");
    let end = r.at;
    // A window frame names its own basis (the library plus its window).
    let windowed = crate::archive::window_codec(&data[..end]);
    let representation = if !h.program {
        Representation::Raw
    } else if body.starts_with(KERNEL_PREFIX) {
        Representation::UlzKernel
    } else if let Some(codec) = h
        .basis
        .and_then(|b| crate::archive::linked_codec(&b, body))
        .or(windowed.map(|(c, _)| c))
    {
        match codec {
            Codec::Ulz => Representation::Ulz,
            c if c.x86() => Representation::NcmX86,
            _ => Representation::Ncm,
        }
    } else if h.basis.is_some() && body == crate::archive::REF_BODY {
        Representation::Reference
    } else {
        Representation::Program
    };
    let ncm_profile = h
        .basis
        .and_then(|b| crate::archive::linked_codec(&b, body))
        .or(windowed.map(|(c, _)| c))
        .and_then(Codec::ncm_profile);
    let window = windowed.map_or(0, |(_, k)| k);
    let info = Info {
        format: h.format,
        profile: h.profile,
        basis: h.basis,
        representation,
        window,
        ncm_profile,
        object_lengths: h.object_lengths,
        output_len: h.output_len,
        body_len: h.body_len,
        archive_len: end as u64,
        digest,
        integrity_ok: chain(&data[..sealed]) == digest,
    };
    Ok((
        info,
        Layout {
            body_len_at: h.body_len_at,
            sealed,
            end,
        },
    ))
}

/// The object lengths an archive's header declares, from the header alone.
pub fn object_lengths(archive: &[u8]) -> Result<Vec<u64>, Error> {
    Ok(head(&mut Reader::new(archive))?.object_lengths)
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
        let end = frame_length(rest)?
            .filter(|&end| end <= rest.len())
            .ok_or(Error::Decode)?;
        let (frame, tail) = rest.split_at(end);
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
    let digest = chain(&out);
    out.extend_from_slice(&digest);
    Ok(out)
}
