//! The uorc client library.
//!
//! A uorc stream is a sequence of standard self-contained UORC archives laid
//! end to end, one per frame of at most [`FRAME`] input bytes. Each archive is
//! self-delimiting, so a stream needs no container, and a one-frame stream is
//! exactly the archive the LexLean model (`client/src/Uorc/Client.lex.tex`)
//! writes for that input.
//!
//! Frames are compressed in parallel. Every frame after the first is a window
//! frame: a conditional archive over the standard basis plus the outputs of the
//! [`archive::WINDOW`] frames before it, which its ulz payload reads as history,
//! so matches reach back across frame boundaries. Frames therefore decode in
//! order, each while the previous two are held, and memory stays bounded by a
//! few frames, never by the input.
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
pub mod ncm;
pub mod tree;
pub mod ulz;

pub use frame::{frame_length, frames, inspect, object_lengths, Info, Representation, HEADER_HINT};

/// Input bytes per frame. Each frame becomes one self-contained archive.
pub const FRAME: usize = 8 << 20;

/// Search steps spent by [`compress`] when the caller names no budget. With
/// none, compress takes the smaller of the ulz kernel archive and the raw one;
/// a positive budget also runs the program search and keeps the smallest.
pub const DEFAULT_STEPS: u64 = 0;

/// Which coders compress tries. Archives name their decoder in the built-in
/// standard basis, so decompress needs no level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Level {
    /// Raw or ulz (LZ + range coder): fast both ways.
    #[default]
    Fast,
    /// Also the ncm context-mixing coder, kept when smaller: the best ratio,
    /// at well under 1 MB/s per core each way.
    Best,
}

impl Level {
    /// The model's `level` argument.
    fn model(self) -> u64 {
        match self {
            Level::Fast => 0,
            Level::Best => 1,
        }
    }
}

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

/// Compress one frame into its archive exactly as the model's
/// `compress input [] steps level`: the smallest of the linked ulz archive,
/// with [`Level::Best`] the linked ncm archive, and the raw archive (the
/// earlier wins a tie); with `steps > 0` the exported model also searches and
/// keeps the smallest.
pub fn compress_frame(frame: &[u8], steps: u64, level: Level) -> Result<Vec<u8>, Error> {
    compress_pieces(frame, &[frame.len() as u64], &[], steps, level)
}

/// [`compress_frame`] for a frame that follows the frames `window` (at most
/// [`archive::WINDOW`], oldest first), exactly as the model's
/// `compress input window steps level`: the ulz candidate is the window
/// archive whose payload reads those frames as history.
pub fn compress_frame_with(
    frame: &[u8],
    window: &[&[u8]],
    steps: u64,
    level: Level,
) -> Result<Vec<u8>, Error> {
    compress_pieces(frame, &[frame.len() as u64], window, steps, level)
}

/// [`compress_frame`] with the output split into objects of `lengths`, which
/// sum to the frame's length. Only the header differs from the one-object
/// archive, so the choice between kernel and raw is the same.
fn compress_pieces(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    steps: u64,
    level: Level,
) -> Result<Vec<u8>, Error> {
    if steps > 0 {
        let archive = on_reference_stack(|| reference_compress(frame, window, steps, level))?;
        return if lengths.len() == 1 {
            Ok(archive)
        } else {
            frame::relabel(&archive, lengths)
        };
    }
    let n = frame.len() as u64;
    let mut best: Option<Vec<u8>> = None;
    if window.is_empty() {
        let payload = ulz::encode(frame);
        // The model writes the coder output into a table of the next power of
        // two at or above 2n + 64 bytes; a payload that would not fit is no candidate.
        let fits = (payload.len() as u64 - 8) <= (2 * n + 64).next_power_of_two();
        if fits && ulz::decode(&payload, n).as_deref() == Some(frame) {
            best = Some(archive::linked_archive(
                archive::Codec::Ulz,
                lengths,
                &payload,
            ));
        }
    } else {
        let history = window.concat();
        let payload = ulz::encode_with(&history, frame);
        let fits = (payload.len() as u64 - 8) <= (2 * n + 64).next_power_of_two();
        if fits && ulz::decode_with(&history, &payload, n).as_deref() == Some(frame) {
            let identity = archive::window_identity(window);
            best = archive::window_archive(window.len(), &identity, lengths, &payload);
        }
    }
    if level == Level::Best {
        if let Some(payload) = ncm::encode(frame) {
            let len = archive::linked_length(archive::Codec::Ncm, lengths, payload.len() as u64);
            let smaller = best.as_ref().is_none_or(|b| len < b.len() as u64);
            // The native ncm encoder is byte-identical to the model
            // (`native_archives_are_the_model_archives`) and every encoder
            // output decodes to its input, so a self-decode would only repeat
            // the encode's whole cost.
            if smaller {
                best = Some(archive::linked_archive(
                    archive::Codec::Ncm,
                    lengths,
                    &payload,
                ));
            }
        }
    }
    match best {
        Some(linked) if (linked.len() as u64) < archive::raw_length(lengths) => Ok(linked),
        _ => Ok(archive::raw_objects(frame, lengths)),
    }
}

fn reference_compress(
    input: &[u8],
    window: &[&[u8]],
    steps: u64,
    level: Level,
) -> Result<Vec<u8>, Error> {
    use uorc_compress::{Bytes, List};
    match uorc_compress::run(
        Bytes::lit(input),
        window
            .iter()
            .rev()
            .fold(List::nil(), |list, r| List::cons(Bytes::lit(r), list)),
        steps,
        level.model(),
    ) {
        Ok(Some(archive)) => Ok(archive.octets().to_vec()),
        Ok(None) => Err(Error::Budget),
        Err(_) => Err(Error::Overflow),
    }
}

/// Reconstruct the bytes of one archive that follows no other frame (all its
/// objects, in order). Archives the native path vouches for (raw, ulz kernel,
/// and the linked ulz and ncm archives this client writes) decode natively;
/// every other archive is reconstructed by the exported reference decoder
/// within `work`.
pub fn decompress_frame(archive: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    decompress_frame_with(archive, &[], work)
}

/// Whether an archive is a window frame: conditional on a basis other than
/// the standard one, so it reads the frames before it.
fn reads_window(archive: &[u8]) -> bool {
    archive::basis_of(archive).is_some_and(|basis| basis != archive::BASIS_ID)
}

/// [`decompress_frame`] for an archive that follows the frames `window`
/// (oldest first, as many as [`compress_frame_with`] was given). A window
/// frame decodes against them; any other archive ignores them.
pub fn decompress_frame_with(
    archive: &[u8],
    window: &[&[u8]],
    work: u64,
) -> Result<Vec<u8>, Error> {
    let window = if reads_window(archive) { window } else { &[] };
    let fast = if window.is_empty() {
        archive::fast_decode(archive)
    } else {
        archive::window_decode(archive, window)
    };
    if let Some(output) = fast {
        return Ok(output);
    }
    let info = inspect(archive)?;
    if info.object_lengths.len() == 1 {
        return on_reference_stack(|| reference_decompress(archive, window, work));
    }
    // The certified decoder reads one-object frames. The same body under the
    // single total length produces the same bytes; the seal of the archive as
    // given is checked before it is relabelled.
    let single = frame::relabel(archive, &[info.output_len])?;
    on_reference_stack(|| reference_decompress(&single, window, work))
}

fn reference_decompress(archive: &[u8], window: &[&[u8]], work: u64) -> Result<Vec<u8>, Error> {
    use uorc_decompress::{Bytes, List};
    match uorc_decompress::run(
        Bytes::lit(archive),
        window
            .iter()
            .rev()
            .fold(List::nil(), |list, r| List::cons(Bytes::lit(r), list)),
        work,
    ) {
        Ok(Some(output)) => Ok(output.octets().to_vec()),
        Ok(None) => Err(Error::Decode),
        Err(_) => Err(Error::Overflow),
    }
}

/// The window frame `k` of `frames` reads: up to [`archive::WINDOW`] frames
/// before it, oldest first.
fn window_of<T: AsRef<[u8]>>(frames: &[T], k: usize) -> Vec<&[u8]> {
    frames[k.saturating_sub(archive::WINDOW)..k]
        .iter()
        .map(AsRef::as_ref)
        .collect()
}

/// Decode the archives of a stream in order, each against the outputs of the
/// frames before it.
fn decode_in_order(archives: &[&[u8]], work: u64) -> Result<Vec<Vec<u8>>, Error> {
    let mut outputs: Vec<Vec<u8>> = Vec::with_capacity(archives.len());
    for (k, archive) in archives.iter().enumerate() {
        let output = decompress_frame_with(archive, &window_of(&outputs, k), work)?;
        outputs.push(output);
    }
    Ok(outputs)
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
fn compress_planned(
    joined: &[u8],
    plan: &[Vec<u64>],
    steps: u64,
    level: Level,
) -> Result<Vec<u8>, Error> {
    let mut at = 0usize;
    let slices: Vec<(&[u8], &[u64])> = plan
        .iter()
        .map(|pieces| {
            let len = pieces.iter().sum::<u64>() as usize;
            let job = (&joined[at..at + len], pieces.as_slice());
            at += len;
            job
        })
        .collect();
    let inputs: Vec<&[u8]> = slices.iter().map(|(f, _)| *f).collect();
    let jobs: Vec<usize> = (0..slices.len()).collect();
    let mut out = Vec::new();
    for archive in parallel(&jobs, default_threads(), |&k| {
        let (f, l) = slices[k];
        compress_pieces(f, l, &window_of(&inputs, k), steps, level)
    }) {
        out.extend_from_slice(&archive?);
    }
    Ok(out)
}

/// Compress `input` into a uorc stream (one archive per [`FRAME`] bytes),
/// using every available core. Every archive is checked against its input
/// before it is returned.
pub fn compress(input: &[u8], steps: u64, level: Level) -> Result<Vec<u8>, Error> {
    compress_planned(input, &plan(&[input.len() as u64]), steps, level)
}

/// Reconstruct the original bytes of a uorc stream. A stream of several objects decodes to their concatenation (see
/// [`decompress_objects`] to get them apart).
pub fn decompress(stream: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    Ok(decode_frames(stream, work)?.concat())
}

fn decode_frames(stream: &[u8], work: u64) -> Result<Vec<Vec<u8>>, Error> {
    let frames = frames(stream)?;
    if frames.is_empty() {
        return Err(Error::Decode);
    }
    decode_in_order(&frames, work)
}

/// Compress an ordered collection of byte objects into one stream whose
/// headers record each object's length. The body codes the objects'
/// concatenation, so similar objects compress together. Any number of objects
/// is allowed, including none and empty ones.
pub fn compress_objects<T: AsRef<[u8]>>(
    objects: &[T],
    steps: u64,
    level: Level,
) -> Result<Vec<u8>, Error> {
    let lengths: Vec<u64> = objects.iter().map(|o| o.as_ref().len() as u64).collect();
    let joined = objects.iter().fold(Vec::new(), |mut all, o| {
        all.extend_from_slice(o.as_ref());
        all
    });
    compress_planned(&joined, &plan(&lengths), steps, level)
}

/// Reconstruct the ordered objects of a stream, split at the lengths its
/// headers record.
pub fn decompress_objects(stream: &[u8], work: u64) -> Result<Vec<Vec<u8>>, Error> {
    let frames = frames(stream)?;
    if frames.is_empty() {
        return Err(Error::Decode);
    }
    let outputs = decode_in_order(&frames, work)?;
    let mut objects: Vec<Vec<u8>> = Vec::new();
    for (k, (archive, output)) in frames.iter().zip(outputs).enumerate() {
        let lengths = object_lengths(archive)?;
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
pub fn compress_tree(entries: &[tree::Entry], steps: u64, level: Level) -> Result<Vec<u8>, Error> {
    compress_objects(&tree::to_objects(entries)?, steps, level)
}

/// Reconstruct the file tree a stream from [`compress_tree`] stores. Every
/// path is checked to be relative and free of `..`.
pub fn decompress_tree(stream: &[u8], work: u64) -> Result<Vec<tree::Entry>, Error> {
    tree::from_objects(decompress_objects(stream, work)?)
}

/// Check a stream end to end without keeping its output: every archive's
/// frame and integrity seal must be intact, it must decode, its output must
/// have the lengths its header declares and, when `original` is given, the
/// concatenated output must equal it.
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
    let mut outputs: Vec<Vec<u8>> = Vec::new();
    let mut infos = Vec::with_capacity(jobs.len());
    for (k, (archive, info, start)) in jobs.into_iter().enumerate() {
        if !info.integrity_ok {
            return Err(Error::Decode);
        }
        let output = decompress_frame_with(archive, &window_of(&outputs, k), work)?;
        if output.len() as u64 != info.output_len {
            return Err(Error::Decode);
        }
        if let Some(original) = original {
            let start = start as usize;
            if original[start..start + output.len()] != output[..] {
                return Err(Error::Mismatch);
            }
        }
        outputs.push(output);
        // Only the last frames are ever read again.
        if outputs.len() > archive::WINDOW {
            outputs[k - archive::WINDOW] = Vec::new();
        }
        infos.push(info);
    }
    Ok(infos)
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
/// `threads` frames plus the window whatever the input size.
pub fn compress_stream(
    mut reader: impl Read,
    mut writer: impl Write,
    steps: u64,
    level: Level,
    threads: usize,
) -> io::Result<Totals> {
    let threads = threads.max(1);
    let mut totals = Totals::default();
    // The frames before the batch that its first frames read.
    let mut before: Vec<Vec<u8>> = Vec::new();
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
        let all: Vec<&[u8]> = before.iter().chain(&batch).map(Vec::as_slice).collect();
        let jobs: Vec<usize> = (before.len()..all.len()).collect();
        let archives = parallel(&jobs, threads, |&k| {
            compress_frame_with(all[k], &window_of(&all, k), steps, level)
        });
        drop(all);
        for (frame, archive) in batch.iter().zip(archives) {
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
        before.extend(batch);
        let keep = before.len().saturating_sub(archive::WINDOW);
        before.drain(..keep);
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
/// archive in order (each window frame reads the frames before it), holding
/// only the last [`archive::WINDOW`] outputs. `threads` is kept for symmetry
/// with [`compress_stream`]; a window frame's identity check runs on a second
/// core while it decodes.
pub fn decompress_stream(
    mut reader: impl Read,
    mut writer: impl Write,
    work: u64,
    threads: usize,
) -> io::Result<Totals> {
    let _ = threads;
    let mut totals = Totals::default();
    let mut carry = Vec::new();
    let mut window: Vec<Vec<u8>> = Vec::new();
    while let Some(archive) = next_archive(&mut reader, &mut carry)? {
        let view: Vec<&[u8]> = window.iter().map(Vec::as_slice).collect();
        let output = decompress_frame_with(&archive, &view, work)?;
        writer.write_all(&output)?;
        totals.read += archive.len() as u64;
        totals.written += output.len() as u64;
        totals.frames += 1;
        window.push(output);
        if window.len() > archive::WINDOW {
            window.remove(0);
        }
    }
    if totals.frames == 0 {
        return Err(Error::Decode.into());
    }
    writer.flush()?;
    Ok(totals)
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
        let archive = compress(b"", 100_000, Level::Fast).expect("empty input has a raw archive");
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK).expect("decodes"),
            b""
        );
    }

    #[test]
    fn arbitrary_bytes_round_trip() {
        let input: Vec<u8> = (0..=255u8).chain([0, 255, 0]).collect();
        let archive = compress(&input, 40_000, Level::Fast).expect("archive");
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
        let archive = compress(b"abc", 0, Level::Fast).expect("raw archive");
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
        let archive = compress(&input, DEFAULT_STEPS, Level::Fast).expect("archive");
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
            let archive = compress_objects(&objects, DEFAULT_STEPS, Level::Fast).expect("archive");
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
        let mut archive =
            compress_objects(&[b"ab".to_vec(), b"cd".to_vec()], 0, Level::Fast).expect("archive");
        let last = archive.len() - 1;
        archive[last] ^= 1;
        assert_eq!(
            decompress(&archive, DEFAULT_DECODE_WORK),
            Err(Error::Decode)
        );
    }

    #[test]
    fn streams_verify_frame_by_frame() {
        let mut stream = compress(b"first frame ", 0, Level::Fast).expect("archive");
        stream.extend(compress(b"second frame", 0, Level::Fast).expect("archive"));
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
            for level in [Level::Fast, Level::Best] {
                let native = compress_frame(&input, 0, level).expect("native");
                let model = on_reference_stack(|| reference_compress(&input, &[], 0, level))
                    .expect("model");
                assert_eq!(native, model, "{} bytes, {level:?}", input.len());
                assert_eq!(archive::fast_decode(&native).as_ref(), Some(&input));
            }
        }
    }

    /// A linked archive whose payload the encoder would not write (one extra
    /// byte): the native path declines it, and the exported model links the
    /// standard basis and runs the result by the reference semantics. (An ncm
    /// archive takes the same path but needs ~20 min of reference execution, so
    /// only its decline is checked here.)
    #[test]
    fn linked_archives_decode_by_linking_the_basis() {
        let mut payload = ulz::encode(b"ab");
        let a = archive::linked_archive(archive::Codec::Ulz, &[2], &payload);
        assert_eq!(
            decompress(&a, DEFAULT_DECODE_WORK).as_deref(),
            Ok(&b"ab"[..])
        );
        payload.push(0);
        let padded = archive::linked_archive(archive::Codec::Ulz, &[2], &payload);
        assert_eq!(archive::fast_decode(&padded), None);
        assert_eq!(
            decompress(&padded, DEFAULT_DECODE_WORK).as_deref(),
            Ok(&b"ab"[..])
        );
        let mut payload = ncm::encode(b"ab").expect("fits");
        payload.push(255);
        let padded = archive::linked_archive(archive::Codec::Ncm, &[2], &payload);
        assert_eq!(archive::fast_decode(&padded), None);
    }

    #[test]
    fn best_level_beats_fast_on_text() {
        let input = text(20_000, 9);
        let fast = compress(&input, 0, Level::Fast).expect("fast");
        let best = compress(&input, 0, Level::Best).expect("best");
        assert!(best.len() < fast.len(), "{} vs {}", best.len(), fast.len());
        assert_eq!(
            decompress(&best, DEFAULT_DECODE_WORK).expect("decodes"),
            input
        );
        let objects = [&input[..7_000], &input[7_000..]];
        let many = compress_objects(&objects, 0, Level::Best).expect("objects");
        assert_eq!(
            decompress_objects(&many, DEFAULT_DECODE_WORK).expect("decodes"),
            objects.map(<[u8]>::to_vec)
        );
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
                    let reference = on_reference_stack(|| {
                        reference_decompress(&archive, &[], DEFAULT_DECODE_WORK)
                    });
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
        let totals = compress_stream(&input[..], &mut stream, 0, Level::Fast, 3).expect("compress");
        assert_eq!(totals.frames, 3);
        assert_eq!(totals.read, input.len() as u64);
        assert_eq!(frames(&stream).expect("frames").len(), 3);
        assert_eq!(stream, compress(&input, 0, Level::Fast).expect("in memory"));
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
        compress_stream(&b""[..], &mut stream, 0, Level::Fast, 4).expect("empty");
        assert_eq!(frames(&stream).expect("one").len(), 1);
        assert_eq!(decompress(&stream, DEFAULT_DECODE_WORK), Ok(Vec::new()));
        let input = vec![7u8; FRAME];
        let mut stream = Vec::new();
        assert_eq!(
            compress_stream(&input[..], &mut stream, 0, Level::Fast, 1)
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
            let stream = compress_objects(&objects, 0, Level::Fast).expect("stream");
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
            compress_objects(&[&big], 0, Level::Fast).expect("one"),
            compress(&big, 0, Level::Fast).expect("plain")
        );
    }

    /// A window frame is exactly the archive the exported model writes for
    /// that frame and window, and the native decoder accepts it.
    #[test]
    fn window_frames_are_the_model_archives() {
        let a = text(3000, 31);
        let b = text(2000, 32);
        let mut c = a[500..1500].to_vec();
        c.extend(text(700, 33));
        for window in [vec![&a[..]], vec![&a[..], &b[..]], vec![&b[..], &[][..]]] {
            let native = compress_frame_with(&c, &window, 0, Level::Fast).expect("native");
            let model = on_reference_stack(|| reference_compress(&c, &window, 0, Level::Fast))
                .expect("model");
            assert_eq!(native, model, "window of {}", window.len());
            assert!(reads_window(&native));
            assert_eq!(archive::window_decode(&native, &window), Some(c.clone()));
            assert_eq!(
                decompress_frame_with(&native, &window, DEFAULT_DECODE_WORK),
                Ok(c.clone())
            );
            // Another window names another basis: refused natively and by the model.
            let other = [&b"another window"[..]; 1];
            let k = window.len();
            let wrong: Vec<&[u8]> = other.iter().copied().cycle().take(k).collect();
            assert_eq!(archive::window_decode(&native, &wrong), None);
        }
        let alone = compress_frame(&c, 0, Level::Fast).expect("alone");
        let with = compress_frame_with(&c, &[&a], 0, Level::Fast).expect("window");
        assert!(
            with.len() < alone.len(),
            "{} vs {}",
            with.len(),
            alone.len()
        );
    }

    /// The reference decoder reconstructs a window frame against its window,
    /// as the native decoder does (this runs the graph, so the case is small).
    #[test]
    fn window_frames_decode_by_the_reference_semantics() {
        let a = text(64, 41);
        let mut c = a[8..40].to_vec();
        c.extend(text(16, 42));
        // Built directly: at this size the client would keep the raw archive.
        let payload = ulz::encode_with(&a, &c);
        let identity = archive::window_identity(&[&a]);
        let archive =
            archive::window_archive(1, &identity, &[c.len() as u64], &payload).expect("archive");
        assert!(reads_window(&archive));
        assert_eq!(archive::window_decode(&archive, &[&a]).as_ref(), Some(&c));
        assert_eq!(
            on_reference_stack(|| reference_decompress(&archive, &[&a], DEFAULT_DECODE_WORK)),
            Ok(c)
        );
    }

    #[test]
    fn chain_parts_is_chain_of_the_concatenation() {
        let data = text(1000, 51);
        for cuts in [
            vec![],
            vec![0],
            vec![1, 63],
            vec![64, 64, 65],
            vec![100, 900, 999],
        ] {
            let mut parts = Vec::new();
            let mut at = 0;
            for &c in &cuts {
                parts.push(&data[at..c]);
                at = c;
            }
            parts.push(&data[at..]);
            assert_eq!(
                archive::chain_parts(&parts),
                archive::chain(&data),
                "{cuts:?}"
            );
        }
    }

    /// Frames match into the frames before them: a stream that repeats one
    /// frame costs about one frame's archive, not three.
    #[test]
    fn windows_carry_matches_across_frames() {
        let block = text(FRAME, 61);
        let alone = compress_frame(&block, 0, Level::Fast)
            .expect("one frame")
            .len();
        let input = block.repeat(3);
        let stream = compress(&input, 0, Level::Fast).expect("stream");
        assert_eq!(frames(&stream).expect("frames").len(), 3);
        assert!(
            stream.len() < alone + alone / 16,
            "{} vs {alone}",
            stream.len()
        );
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("decodes") == input);
        let mut back = Vec::new();
        decompress_stream(&stream[..], &mut back, DEFAULT_DECODE_WORK, 1).expect("streams");
        assert!(back == input);
        verify(&stream, Some(&input), DEFAULT_DECODE_WORK).expect("verifies");
    }
}
