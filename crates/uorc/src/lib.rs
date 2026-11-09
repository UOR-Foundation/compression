//! The uorc client library.
//!
//! A uorc stream is a sequence of standard self-contained UORC archives laid
//! end to end, one per frame of at most [`FRAME`] input bytes. Each archive is
//! self-delimiting, so a stream needs no container, and a one-frame stream is
//! exactly the archive the LexLean model (`client/src/Uorc/Client.lex.tex`)
//! writes for that input.
//!
//! Frames are compressed and decompressed in parallel and independently, so
//! memory is bounded by the frame size and the worker count, never by the input.
//!
//! The hot paths are native Rust ports of the model: the ulz codec
//! ([`ulz`]), the archive layout and the UORC-SHA256-CHAIN-v1 seal
//! ([`archive`]). They write byte-identical archives, and the decoder only
//! accepts what it can vouch for. Everything else (archives the native path
//! declines, and the program search behind `steps > 0`) runs the certified
//! Rust that `lexlean production build` exports from the model
//! (`generated/rust/uorc_{compress,decompress}`, see `scripts/client-export.mjs`).
//!
//! Ordered objects (and so file trees, see [`tree`]) are the same stream: each
//! frame's header lists the pieces of objects it holds, and the first piece of
//! every frame after the first continues the last object of the frame before
//! (an object that ends exactly at a frame boundary is continued by an empty
//! piece). A plain byte stream is the one-object case.

use std::fmt;
use std::io::{self, Read, Write};

pub mod archive;
mod frame;
pub mod tree;
pub mod ulz;

pub use frame::{frame_length, frames, inspect, object_lengths, Info, Representation, HEADER_HINT};

/// Input bytes per frame. Each frame becomes one self-contained archive.
pub const FRAME: usize = 8 << 20;

/// Search steps spent by [`compress`] when the caller names no budget. With
/// none, compress takes the smaller of the ulz kernel archive and the raw one;
/// a positive budget also runs the program search and keeps the smallest.
pub const DEFAULT_STEPS: u64 = 0;

/// Decoder work allowed per archive the reference decoder reconstructs.
pub const DEFAULT_DECODE_WORK: u64 = 1 << 40;

/// A failed client operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The search ended its step budget before any checked archive existed.
    Budget,
    /// A 64-bit natural-number operation in the exported code overflowed.
    Overflow,
    /// The bytes are not a stream of self-contained UORC archives this client decodes.
    Decode,
    /// The archive decodes, but not to the original it was checked against.
    Mismatch,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::Budget => "no checked archive was found within the step budget",
            Error::Overflow => "a natural-number operation overflowed 64 bits",
            Error::Decode => "input is not a decodable stream of self-contained UORC archives",
            Error::Mismatch => "the archive does not decode to the given original",
        })
    }
}

impl std::error::Error for Error {}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, e)
    }
}

impl Error {
    /// The client error an I/O error from a streaming operation carries, if any.
    pub fn of(error: &io::Error) -> Option<Error> {
        error.get_ref()?.downcast_ref::<Error>().copied()
    }
}

/// Stack reserved for the exported reference code, which recurses structurally
/// over archives and byte ropes. Only the pages actually touched are committed,
/// and native paths never run on it.
#[cfg(not(target_family = "wasm"))]
const REFERENCE_STACK: usize = 1 << 30;

#[cfg(not(target_family = "wasm"))]
fn on_reference_stack<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("uorc-reference".to_owned())
            .stack_size(REFERENCE_STACK)
            .spawn_scoped(scope, work)
            .expect("spawn the uorc reference thread")
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

#[cfg(target_family = "wasm")]
fn on_reference_stack<T>(work: impl FnOnce() -> T) -> T {
    work()
}

/// Worker threads used when the caller does not say.
pub fn default_threads() -> usize {
    std::thread::available_parallelism().map_or(1, usize::from)
}

/// Compress one frame into its archive: the smaller of the ulz kernel archive
/// and the raw archive, exactly as the model's `compress input 0`; with
/// `steps > 0`, the exported model also searches and keeps the smallest.
pub fn compress_frame(frame: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    compress_pieces(frame, &[frame.len() as u64], steps)
}

/// [`compress_frame`] with the output split into objects of `lengths`, which
/// sum to the frame's length. Only the header differs from the one-object
/// archive, so the choice between kernel and raw is the same.
fn compress_pieces(frame: &[u8], lengths: &[u64], steps: u64) -> Result<Vec<u8>, Error> {
    if steps > 0 {
        let archive = on_reference_stack(|| reference_compress(frame, steps))?;
        return if lengths.len() == 1 {
            Ok(archive)
        } else {
            frame::relabel(&archive, lengths)
        };
    }
    let n = frame.len() as u64;
    let payload = ulz::encode(frame);
    // The model writes the coder output into a table of the next power of two
    // at or above 2n + 64 bytes and falls back to raw if it would not fit.
    let fits = (payload.len() as u64 - 8) <= (2 * n + 64).next_power_of_two();
    if fits && ulz::decode(&payload, n).as_deref() == Some(frame) {
        let kernel = archive::program_archive(lengths, &payload);
        if (kernel.len() as u64) < archive::raw_length(lengths) {
            return Ok(kernel);
        }
    }
    Ok(archive::raw_objects(frame, lengths))
}

fn reference_compress(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    match uorc_compress::run(uorc_compress::Bytes::lit(input), steps) {
        Ok(Some(archive)) => Ok(archive.octets().to_vec()),
        Ok(None) => Err(Error::Budget),
        Err(_) => Err(Error::Overflow),
    }
}

/// Reconstruct the bytes of one self-contained archive (all its objects, in
/// order). Archives the native path vouches for (raw, and ulz kernel archives
/// this client writes) decode natively; every other archive is reconstructed
/// by the exported reference decoder within `work`.
pub fn decompress_frame(archive: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    if let Some(output) = archive::fast_decode(archive) {
        return Ok(output);
    }
    let info = inspect(archive)?;
    if info.object_lengths.len() == 1 {
        return on_reference_stack(|| reference_decompress(archive, work));
    }
    // The certified decoder reads one-object frames. The same body under the
    // single total length produces the same bytes; the seal of the archive as
    // given is checked before it is relabelled.
    let single = frame::relabel(archive, &[info.output_len])?;
    on_reference_stack(|| reference_decompress(&single, work))
}

fn reference_decompress(archive: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    match uorc_decompress::run(uorc_decompress::Bytes::lit(archive), work) {
        Ok(Some(output)) => Ok(output.octets().to_vec()),
        Ok(None) => Err(Error::Decode),
        Err(_) => Err(Error::Overflow),
    }
}

/// Apply `f` to every item on up to `threads` workers, keeping order.
fn parallel<T: Sync, U: Send>(items: &[T], threads: usize, f: impl Fn(&T) -> U + Sync) -> Vec<U> {
    if threads <= 1 || items.len() <= 1 {
        return items.iter().map(f).collect();
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut slots: Vec<Option<U>> = std::iter::repeat_with(|| None).take(items.len()).collect();
    let results = std::sync::Mutex::new(&mut slots);
    std::thread::scope(|scope| {
        for _ in 0..threads.min(items.len()) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(item) = items.get(i) else { break };
                let value = f(item);
                results
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)[i] = Some(value);
            });
        }
    });
    slots.into_iter().flatten().collect()
}

/// Where each frame of a stream of `lengths` objects ends and the pieces of
/// objects it holds (see the crate documentation).
fn plan(lengths: &[u64]) -> Vec<Vec<u64>> {
    let frame = FRAME as u64;
    let mut plan = vec![Vec::new()];
    let mut used = 0u64;
    for &n in lengths {
        if used == frame {
            plan.push(vec![0]);
            used = 0;
        }
        let mut left = n;
        loop {
            let take = left.min(frame - used);
            plan.last_mut().expect("a frame").push(take);
            used += take;
            left -= take;
            if left == 0 {
                break;
            }
            plan.push(Vec::new());
            used = 0;
        }
    }
    plan
}

/// Compress the concatenation `joined` of objects with the given frame plan.
fn compress_planned(joined: &[u8], plan: &[Vec<u64>], steps: u64) -> Result<Vec<u8>, Error> {
    let mut at = 0usize;
    let jobs: Vec<(&[u8], &[u64])> = plan
        .iter()
        .map(|pieces| {
            let len = pieces.iter().sum::<u64>() as usize;
            let job = (&joined[at..at + len], pieces.as_slice());
            at += len;
            job
        })
        .collect();
    let mut out = Vec::new();
    for archive in parallel(&jobs, default_threads(), |(f, l)| {
        compress_pieces(f, l, steps)
    }) {
        out.extend_from_slice(&archive?);
    }
    Ok(out)
}

/// Compress `input` into a uorc stream (one archive per [`FRAME`] bytes),
/// using every available core. Every archive is checked against its input
/// before it is returned.
pub fn compress(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    compress_planned(input, &plan(&[input.len() as u64]), steps)
}

/// Reconstruct the original bytes of a uorc stream, using every available
/// core. A stream of several objects decodes to their concatenation (see
/// [`decompress_objects`] to get them apart).
pub fn decompress(stream: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    Ok(decode_frames(stream, work)?.concat())
}

fn decode_frames(stream: &[u8], work: u64) -> Result<Vec<Vec<u8>>, Error> {
    let frames = frames(stream)?;
    if frames.is_empty() {
        return Err(Error::Decode);
    }
    parallel(&frames, default_threads(), |a| decompress_frame(a, work))
        .into_iter()
        .collect()
}

/// Compress an ordered collection of byte objects into one stream whose
/// headers record each object's length. The body codes the objects'
/// concatenation, so similar objects compress together. Any number of objects
/// is allowed, including none and empty ones.
pub fn compress_objects<T: AsRef<[u8]>>(objects: &[T], steps: u64) -> Result<Vec<u8>, Error> {
    let lengths: Vec<u64> = objects.iter().map(|o| o.as_ref().len() as u64).collect();
    let joined = objects.iter().fold(Vec::new(), |mut all, o| {
        all.extend_from_slice(o.as_ref());
        all
    });
    compress_planned(&joined, &plan(&lengths), steps)
}

/// Reconstruct the ordered objects of a stream, split at the lengths its
/// headers record.
pub fn decompress_objects(stream: &[u8], work: u64) -> Result<Vec<Vec<u8>>, Error> {
    let frames = frames(stream)?;
    if frames.is_empty() {
        return Err(Error::Decode);
    }
    let decoded = parallel(&frames, default_threads(), |a| {
        let lengths = object_lengths(a)?;
        let output = decompress_frame(a, work)?;
        Ok::<_, Error>((lengths, output))
    });
    let mut objects: Vec<Vec<u8>> = Vec::new();
    for (k, part) in decoded.into_iter().enumerate() {
        let (lengths, output) = part?;
        if lengths.iter().sum::<u64>() != output.len() as u64 {
            return Err(Error::Decode);
        }
        let mut rest = output.as_slice();
        for (i, &n) in lengths.iter().enumerate() {
            let (piece, tail) = rest.split_at(n as usize);
            rest = tail;
            match objects.last_mut() {
                Some(last) if k > 0 && i == 0 => last.extend_from_slice(piece),
                _ if k > 0 && i == 0 => return Err(Error::Decode),
                _ => objects.push(piece.to_vec()),
            }
        }
        if k > 0 && lengths.is_empty() {
            return Err(Error::Decode);
        }
    }
    Ok(objects)
}

/// Compress a file tree (see [`tree`]) into one stream.
pub fn compress_tree(entries: &[tree::Entry], steps: u64) -> Result<Vec<u8>, Error> {
    compress_objects(&tree::to_objects(entries)?, steps)
}

/// Reconstruct the file tree a stream from [`compress_tree`] stores. Every
/// path is checked to be relative and free of `..`.
pub fn decompress_tree(stream: &[u8], work: u64) -> Result<Vec<tree::Entry>, Error> {
    tree::from_objects(decompress_objects(stream, work)?)
}

/// Check a stream end to end without keeping its output: every archive's
/// frame and integrity seal must be intact, it must decode, its output must
/// have the lengths its header declares and, when `original` is given, the
/// concatenated output must equal it. Archives are checked in parallel.
pub fn verify(stream: &[u8], original: Option<&[u8]>, work: u64) -> Result<Vec<Info>, Error> {
    let frames = frames(stream)?;
    if frames.is_empty() {
        return Err(Error::Decode);
    }
    let mut jobs = Vec::with_capacity(frames.len());
    let mut at = 0u64;
    for archive in frames {
        let info = inspect(archive)?;
        let start = at;
        at = at.checked_add(info.output_len).ok_or(Error::Mismatch)?;
        jobs.push((archive, info, start));
    }
    if original.is_some_and(|o| o.len() as u64 != at) {
        return Err(Error::Mismatch);
    }
    let checked = parallel(&jobs, default_threads(), |(archive, info, start)| {
        if !info.integrity_ok {
            return Err(Error::Decode);
        }
        let output = decompress_frame(archive, work)?;
        if output.len() as u64 != info.output_len {
            return Err(Error::Decode);
        }
        if let Some(original) = original {
            let start = *start as usize;
            if original[start..start + output.len()] != output[..] {
                return Err(Error::Mismatch);
            }
        }
        Ok(info.clone())
    });
    checked.into_iter().collect()
}

/// Totals of a streaming operation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Totals {
    /// Bytes read.
    pub read: u64,
    /// Bytes written.
    pub written: u64,
    /// Archives (frames) read or written.
    pub frames: u64,
}

fn read_full(reader: &mut impl Read, buf: &mut Vec<u8>, want: usize) -> io::Result<()> {
    let have = buf.len();
    if have >= want {
        return Ok(());
    }
    reader.take((want - have) as u64).read_to_end(buf)?;
    Ok(())
}

/// Compress everything `reader` yields into a uorc stream on `writer`, one
/// [`FRAME`] at a time with `threads` frames in flight. Memory stays bounded by
/// `threads` frames whatever the input size.
pub fn compress_stream(
    mut reader: impl Read,
    mut writer: impl Write,
    steps: u64,
    threads: usize,
) -> io::Result<Totals> {
    let threads = threads.max(1);
    let mut totals = Totals::default();
    loop {
        let mut batch = Vec::with_capacity(threads);
        let mut end = false;
        while batch.len() < threads && !end {
            let mut frame = Vec::new();
            read_full(&mut reader, &mut frame, FRAME)?;
            end = frame.len() < FRAME;
            // Empty input still becomes one (empty) archive.
            if !frame.is_empty() || (totals.frames == 0 && batch.is_empty()) {
                batch.push(frame);
            }
        }
        for (frame, archive) in batch
            .iter()
            .zip(parallel(&batch, threads, |f| compress_frame(f, steps)))
        {
            let archive = archive?;
            writer.write_all(&archive)?;
            totals.read += frame.len() as u64;
            totals.written += archive.len() as u64;
            totals.frames += 1;
        }
        if end {
            writer.flush()?;
            return Ok(totals);
        }
    }
}

/// Read the next archive of a stream, or `None` at a clean end of input.
fn next_archive(reader: &mut impl Read, carry: &mut Vec<u8>) -> io::Result<Option<Vec<u8>>> {
    let mut frame = std::mem::take(carry);
    let mut want = HEADER_HINT;
    let len = loop {
        read_full(reader, &mut frame, want)?;
        if frame.is_empty() {
            return Ok(None);
        }
        match frame_length(&frame)? {
            Some(len) => break len,
            None if frame.len() < want => return Err(Error::Decode.into()),
            None => want *= 2,
        }
    };
    if len < frame.len() {
        // The header read ran into the next archive: keep those bytes.
        *carry = frame.split_off(len);
    }
    read_full(reader, &mut frame, len)?;
    if frame.len() < len {
        return Err(Error::Decode.into());
    }
    Ok(Some(frame))
}

/// Decompress the uorc stream `reader` yields onto `writer`, archive by
/// archive with `threads` archives in flight.
pub fn decompress_stream(
    mut reader: impl Read,
    mut writer: impl Write,
    work: u64,
    threads: usize,
) -> io::Result<Totals> {
    let threads = threads.max(1);
    let mut totals = Totals::default();
    let mut carry = Vec::new();
    loop {
        let mut batch = Vec::with_capacity(threads);
        while batch.len() < threads {
            match next_archive(&mut reader, &mut carry)? {
                Some(a) => batch.push(a),
                None => break,
            }
        }
        if batch.is_empty() {
            if totals.frames == 0 {
                return Err(Error::Decode.into());
            }
            writer.flush()?;
            return Ok(totals);
        }
        for (archive, output) in batch
            .iter()
            .zip(parallel(&batch, threads, |a| decompress_frame(a, work)))
        {
            let output = output?;
            writer.write_all(&output)?;
            totals.read += archive.len() as u64;
            totals.written += output.len() as u64;
            totals.frames += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 200 zero bytes as a 64-byte program archive, found by the same exported
    /// search at 9.3M steps. Decoding it exercises the program path, not raw.
    const ZEROS_200: &str = "554f52430201000101c80114020100010101000100030200c8012401010001010b1420bec29aa29cd5b2136b604ed97ba986b96e842955bf7feb8e396389b2b5";

    fn hex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
            .collect()
    }

    #[test]
    fn empty_round_trip() {
        let archive = compress(b"", 100_000).expect("empty input has a raw archive");
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
            b""
        );
    }

    #[test]
    fn arbitrary_bytes_round_trip() {
        let input: Vec<u8> = (0..=255u8).chain([0, 255, 0]).collect();
        let archive = compress(&input, 40_000).expect("archive");
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
            input
        );
    }

    #[test]
    fn program_archive_decodes_smaller_than_its_output() {
        let archive = hex(ZEROS_200);
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
            vec![0u8; 200]
        );
    }

    #[test]
    fn zero_budget_still_archives() {
        let archive = compress(b"abc", 0).expect("raw archive");
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
            b"abc"
        );
    }

    #[test]
    fn text_takes_the_kernel_archive() {
        let input: Vec<u8> = b"the quick brown fox jumps over the lazy dog. "
            .iter()
            .copied()
            .cycle()
            .take(20_000)
            .collect();
        let archive = compress(&input, DEFAULT_STEPS).expect("archive");
        assert!(archive.len() < 4_000, "{} bytes", archive.len());
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
            input
        );
    }

    /// The ulz kernel archive of the empty input with one extra payload byte the
    /// encoder would not write: the fast path declines it, so this decodes the
    /// kernel graph by the reference semantics.
    #[test]
    fn kernel_graph_decodes_by_the_reference_semantics() {
        let archive = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/kernel-empty-padded.uorc"
        ))
        .expect("fixture");
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
            b""
        );
    }

    #[test]
    fn ordered_objects_round_trip() {
        let text: Vec<u8> = b"objects share one body. ".repeat(200);
        for objects in [
            vec![],
            vec![Vec::new()],
            vec![Vec::new(), Vec::new(), Vec::new()],
            vec![
                text.clone(),
                Vec::new(),
                (0..=255u8).collect(),
                text.clone(),
            ],
        ] {
            let archive = compress_objects(&objects, DEFAULT_STEPS).expect("archive");
            let info = inspect(&archive).expect("header");
            assert!(info.integrity_ok);
            assert_eq!(info.object_lengths.len(), objects.len());
            assert_eq!(
                decompress_objects(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
                objects
            );
            verify(&archive, Some(&objects.concat()), DEFAULT_DECODE_WORK).expect("verifies");
        }
    }

    #[test]
    fn relabelled_archive_with_a_broken_seal_is_rejected() {
        let mut archive = compress_objects(&[b"ab".to_vec(), b"cd".to_vec()], 0).expect("archive");
        let last = archive.len() - 1;
        archive[last] ^= 1;
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK),
            Err(Error::Decode)
        );
    }

    #[test]
    fn streams_verify_frame_by_frame() {
        let mut stream = compress(b"first frame ", 0).expect("archive");
        stream.extend(compress(b"second frame", 0).expect("archive"));
        assert_eq!(frames(&stream).expect("frames").len(), 2);
        let infos = verify(
            &stream,
            Some(b"first frame second frame"),
            DEFAULT_DECODE_WORK,
        )
        .expect("verifies");
        assert_eq!(infos.len(), 2);
        assert_eq!(
            verify(
                &stream,
                Some(b"first frame second frame!"),
                DEFAULT_DECODE_WORK
            ),
            Err(Error::Mismatch)
        );
    }

    #[test]
    fn garbage_is_a_decode_error() {
        assert_eq!(
            decompress(b"not an archive", DEFAULT_DECODE_WORK),
            Err(Error::Decode)
        );
    }

    #[test]
    fn corrupted_archive_is_rejected() {
        let mut archive = hex(ZEROS_200);
        let last = archive.len() - 1;
        archive[last] ^= 1;
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK),
            Err(Error::Decode)
        );
    }

    fn text(n: usize, seed: u64) -> Vec<u8> {
        let mut x = seed;
        let words: [&[u8]; 7] = [
            b"frame ",
            b"archive ",
            b"kernel ",
            b"the ",
            b"of ",
            b"\n",
            b"uorc ",
        ];
        let mut out = Vec::new();
        while out.len() < n {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            if x.is_multiple_of(7) {
                out.push((x >> 32) as u8);
            } else {
                out.extend_from_slice(words[(x % 7) as usize]);
            }
        }
        out.truncate(n);
        out
    }

    /// The native compressor writes exactly the archive the exported model
    /// writes, and the native decoder accepts it.
    #[test]
    fn native_archives_are_the_model_archives() {
        let mut inputs = vec![Vec::new(), b"a".to_vec(), b"abc".to_vec(), vec![0; 3000]];
        inputs.extend([(64, 1), (777, 2), (4096, 3)].map(|(n, s)| text(n, s)));
        inputs.push((0..=255u8).cycle().take(2000).collect());
        // The exported model is slow unoptimized; release runs add a larger input.
        if !cfg!(debug_assertions) {
            inputs.push(text(20_000, 4));
        }
        for input in inputs {
            let native = compress_frame(&input, 0).expect("native");
            let model = on_reference_stack(|| reference_compress(&input, 0)).expect("model");
            assert_eq!(native, model, "{} bytes", input.len());
            assert_eq!(archive::fast_decode(&native), Some(input));
        }
    }

    /// Wherever the native decoder accepts a kernel archive, it reconstructs
    /// what the exported reference decoder reconstructs. The payloads here are
    /// damaged encoder output, resealed, so most leave the encoder's language.
    #[test]
    fn native_decoder_agrees_with_the_reference_where_it_accepts() {
        let input = text(64, 11);
        let payload = ulz::encode(&input);
        let (mut accepted, mut declined) = (0, 0);
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        for round in 0..48 {
            let mut p = payload.clone();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let at = 13 + (x as usize % (p.len() - 13));
            match round % 3 {
                0 => p[at] ^= 1 << ((x >> 40) % 8),
                1 => p[at] = (x >> 24) as u8,
                _ => p.truncate(at),
            }
            let archive = archive::program_archive(&[input.len() as u64], &p);
            match archive::fast_decode(&archive) {
                Some(native) => {
                    accepted += 1;
                    let reference =
                        on_reference_stack(|| reference_decompress(&archive, DEFAULT_DECODE_WORK));
                    assert_eq!(Ok(native), reference, "round {round}");
                }
                None => declined += 1,
            }
            // Each accepted case runs the graph by the reference semantics,
            // which is slow; two of them make the point.
            let enough = if cfg!(debug_assertions) { 1 } else { 2 };
            if accepted >= enough && declined >= enough {
                break;
            }
        }
        assert!(
            accepted > 0 && declined > 0,
            "{accepted} accepted, {declined} declined"
        );
    }

    #[test]
    fn streams_are_frames_end_to_end() {
        let input = text(2 * FRAME + 12_345, 5);
        let mut stream = Vec::new();
        let totals = compress_stream(&input[..], &mut stream, 0, 3).expect("compress");
        assert_eq!(totals.frames, 3);
        assert_eq!(totals.read, input.len() as u64);
        assert_eq!(frames(&stream).expect("frames").len(), 3);
        assert_eq!(stream, compress(&input, 0).expect("in memory"));
        let mut back = Vec::new();
        let totals =
            decompress_stream(&stream[..], &mut back, DEFAULT_DECODE_WORK, 2).expect("decompress");
        assert_eq!(totals.frames, 3);
        assert!(back == input);
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("in memory") == input);
    }

    #[test]
    fn empty_and_exact_frame_streams() {
        let mut stream = Vec::new();
        compress_stream(&b""[..], &mut stream, 0, 4).expect("empty");
        assert_eq!(frames(&stream).expect("one").len(), 1);
        assert_eq!(decompress(&stream, DEFAULT_DECODE_WORK), Ok(Vec::new()));
        let input = vec![7u8; FRAME];
        let mut stream = Vec::new();
        assert_eq!(
            compress_stream(&input[..], &mut stream, 0, 1)
                .expect("one frame")
                .frames,
            1
        );
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("decodes") == input);
        assert_eq!(
            decompress(&stream[..stream.len() - 1], DEFAULT_DECODE_WORK),
            Err(Error::Decode)
        );
    }

    #[test]
    fn frame_plans_continue_objects_across_frames() {
        let f = FRAME as u64;
        assert_eq!(plan(&[]), vec![Vec::<u64>::new()]);
        assert_eq!(plan(&[0]), vec![vec![0]]);
        assert_eq!(plan(&[f]), vec![vec![f]]);
        assert_eq!(plan(&[2 * f + 1]), vec![vec![f], vec![f], vec![1]]);
        assert_eq!(plan(&[f, 0, 3]), vec![vec![f], vec![0, 0, 3]]);
        assert_eq!(plan(&[5, f]), vec![vec![5, f - 5], vec![5]]);
    }

    #[test]
    fn objects_stream_across_frames() {
        let big = text(FRAME + 5_000, 7);
        for objects in [
            vec![big.clone(), Vec::new(), b"tail".to_vec()],
            vec![big[..FRAME].to_vec(), Vec::new(), big[FRAME..].to_vec()],
            vec![b"head".to_vec(), big.clone()],
        ] {
            let stream = compress_objects(&objects, 0).expect("stream");
            assert_eq!(frames(&stream).expect("frames").len(), 2);
            assert_eq!(
                decompress_objects(&stream, DEFAULT_DECODE_WORK).expect("objects"),
                objects
            );
            let joined = objects.concat();
            assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("bytes") == joined);
            assert_eq!(
                verify(&stream, Some(&joined), DEFAULT_DECODE_WORK)
                    .expect("verifies")
                    .len(),
                2
            );
        }
        // One object is a plain stream.
        assert_eq!(
            compress_objects(&[&big], 0).expect("one"),
            compress(&big, 0).expect("plain")
        );
    }
}
