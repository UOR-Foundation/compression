//! The uorc client library.
//!
//! A uorc stream is a sequence of standard self-contained UORC archives laid
//! end to end, one per frame of at most [`FRAME`] input bytes. Each archive is
//! self-delimiting, so a stream needs no container, and a one-frame stream is
//! exactly the archive the LexLean model (`client/src/Uorc/Client.lex.tex`)
//! writes for that input.
//!
//! Frames are compressed in parallel, in blocks of [`BLOCK`] frames. Every
//! frame of a block after its first is a window frame: a conditional archive
//! over the standard basis plus the outputs of up to [`archive::WINDOW`]
//! frames before it in the block, which its payload reads as history, so the
//! model's contexts and matches reach back across frame boundaries. A frame
//! that reads no window starts a block, so the decoder finds the blocks from
//! the frame headers alone and decodes them on separate cores, frames in order
//! inside each block. Memory stays bounded by a few frames per core, never by
//! the input.
//!
//! The hot paths are native Rust ports of the model: the ncm codec ([`ncm`]),
//! the x86 filter ([`x86`]), the ulz decoder for archives earlier versions
//! wrote ([`ulz`]), the archive layout and the UORC-SHA256-CHAIN-v1 seal
//! ([`archive`]). They write byte-identical archives, and the decoder only
//! accepts what it can vouch for. Everything else (archives the native path
//! declines, and the program search behind `steps > 0`) runs the certified
//! Rust that `lexlean production build` exports from the model
//! (`generated/rust/uorc_{compress,decompress}`, see `scripts/client-export.mjs`).
//!
//! Every operation streams, in memory bounded by a few frames per core:
//! [`compress_stream`], [`compress_objects_stream`], [`decompress_stream`],
//! [`decompress_objects_stream`] and [`verify_stream`] run on readers and
//! writers, the in-memory functions run them over buffers, and
//! [`FrameEncoder`] and [`FrameDecoder`] code the same streams one frame at a
//! time on the caller's thread.
//!
//! Ordered objects (and so file trees, see [`tree`]) are the same stream: each
//! frame's header lists the pieces of objects it holds, and the first piece of
//! every frame after the first continues the last object of the frame before
//! (an object that ends exactly at a frame boundary is continued by an empty
//! piece). A plain byte stream is the one-object case.
//!
//! Every frame has a κ-label, the identity of the basis `[library, frame]`
//! ([`archive::frame_label`], a SHA-256 chain over the frame's content). A
//! frame identical to an earlier one is written as a reference frame
//! ([`archive::reference_archive`]): a conditional archive over that basis
//! whose 10-byte body returns the earlier frame, so identical frames are coded,
//! stored and decoded once. A reference frame reads no window, so it may start
//! a block. The decoder reads the headers first and keeps only the frames a
//! reference names, until their last reference; a reference whose frame
//! another block is still decoding waits for it.

use std::collections::HashSet;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub mod archive;
mod frame;
pub mod ncm;
pub mod portfolio;
mod store;
pub mod tree;
pub mod ulz;
pub mod x86;

pub use frame::{frame_length, frames, inspect, object_lengths, Info, Representation, HEADER_HINT};

/// Input bytes per frame. Each frame becomes one self-contained archive.
pub const FRAME: usize = 8 << 20;

/// Frames per block, the encoder's schedule: the first frame of every block
/// reads no window, and the others read up to [`archive::WINDOW`] frames
/// before them in the same block. The stream does not store this number. The
/// decoder finds blocks from the frame headers alone (a frame that reads no
/// window starts one) and decodes them on separate cores, so any block
/// length decodes, including a v0.2 stream, which is one block.
pub const BLOCK: usize = 4;

/// Search steps spent by [`compress`] when the caller names no budget. With
/// none, compress takes the smaller of the linked ncm archive and the raw one;
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

/// Compress one frame into its archive exactly as the model's
/// `compress input [] steps`: the smallest linked ncm archive of the
/// strategies the frame's family schedules first ([`portfolio`]), or the raw
/// archive when that is not smaller; with `steps > 0` the exported model also searches and keeps the
/// smallest.
pub fn compress_frame(frame: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    compress_pieces(frame, &[frame.len() as u64], &[], steps)
}

/// [`compress_frame`] for a frame that follows the frames `window` (at most
/// [`archive::WINDOW`], oldest first), exactly as the model's
/// `compress input window steps`: the ncm candidate is the window archive
/// whose payload reads those frames as history.
pub fn compress_frame_with(frame: &[u8], window: &[&[u8]], steps: u64) -> Result<Vec<u8>, Error> {
    compress_pieces(frame, &[frame.len() as u64], window, steps)
}

/// [`compress_frame`] with the output split into objects of `lengths`, which
/// sum to the frame's length. Only the header differs from the one-object
/// archive, so the choice between linked and raw is the same.
fn compress_pieces(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    steps: u64,
) -> Result<Vec<u8>, Error> {
    if steps > 0 {
        let archive = on_reference_stack(|| reference_compress(frame, window, steps))?;
        return if lengths.len() == 1 {
            Ok(archive)
        } else {
            frame::relabel(&archive, lengths)
        };
    }
    let history = window.concat();
    let identity = (!window.is_empty()).then(|| archive::window_identity(window));
    // The strategy portfolio (`Client.portfolio`): code the frame by the
    // first BUDGET strategies its family's schedule orders and keep the
    // smallest archive, the earlier one on a tie. A filtered strategy codes
    // the filtered frame after the filtered window.
    let code = |strategy: portfolio::Strategy, learn: bool| -> Option<Vec<u8>> {
        let filtered;
        let (data, past) = if strategy.x86 {
            filtered = (
                x86::filter(frame, false, history.len() as u64).0,
                x86::filter(&history, false, 0).0,
            );
            (&filtered.0[..], &filtered.1[..])
        } else {
            (frame, &history[..])
        };
        let codec = if strategy.x86 {
            archive::Codec::NcmX86
        } else {
            archive::Codec::Ncm
        };
        // The native ncm encoder is byte-identical to the model
        // (`native_archives_are_the_model_archives`) and every encoder output
        // decodes to its input, so a self-decode would only repeat its cost.
        let frame_of = |profile: u32, payload: &[u8]| match &identity {
            None => archive::linked_archive_profile(codec, profile, lengths, payload),
            Some(identity) => {
                archive::window_archive_profile(codec, profile, window, identity, lengths, payload)
            }
        };
        if !learn {
            return ncm::encode_profile(past, data, strategy.profile)
                .and_then(|payload| frame_of(strategy.profile, &payload));
        }
        // The learned profile (`Archive.learnedCandidate`): the strategy's
        // profile without the contexts that saved nothing on this frame,
        // coded too and kept when no larger. Fewer contexts decode faster.
        let (payload, learned) = ncm::encode_learn(past, data, strategy.profile)?;
        let plain = frame_of(strategy.profile, &payload);
        if learned == strategy.profile {
            return plain;
        }
        let pruned = ncm::encode_profile(past, data, learned).and_then(|p| frame_of(learned, &p));
        match (plain, pruned) {
            (Some(p), Some(l)) => Some(if l.len() <= p.len() { l } else { p }),
            (p, l) => l.or(p),
        }
    };
    // The scheduled strategies code side by side (a lone frame then uses two
    // cores); the choice is the same in any order. The first also learns its
    // context mask (`Client.portfolioLearned`).
    let [first, second] = portfolio::plan(frame);
    #[cfg(not(target_family = "wasm"))]
    let (a, b) = std::thread::scope(|s| {
        let other = s.spawn(|| code(second, false));
        (code(first, true), other.join().expect("strategy thread"))
    });
    #[cfg(target_family = "wasm")]
    let (a, b) = (code(first, true), code(second, false));
    let best = match (a, b) {
        (Some(a), Some(b)) => Some(if b.len() < a.len() { b } else { a }),
        (a, b) => a.or(b),
    };
    match best {
        Some(linked) if (linked.len() as u64) < archive::raw_length(lengths) => Ok(linked),
        _ => Ok(archive::raw_objects(frame, lengths)),
    }
}

fn reference_compress(input: &[u8], window: &[&[u8]], steps: u64) -> Result<Vec<u8>, Error> {
    use uorc_compress::{Bytes, List};
    match uorc_compress::run(
        Bytes::lit(input),
        window
            .iter()
            .rev()
            .fold(List::nil(), |list, r| List::cons(Bytes::lit(r), list)),
        steps,
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

/// Whether an archive is a window frame: it reads the frames before it.
#[cfg(test)]
fn reads_window(archive: &[u8]) -> bool {
    archive::window_len(archive) != Some(0)
}

/// [`decompress_frame`] for an archive that follows the frames `window`
/// (oldest first, as many as [`compress_frame_with`] was given). A window
/// frame decodes against them; any other archive ignores them.
pub fn decompress_frame_with(
    archive: &[u8],
    window: &[&[u8]],
    work: u64,
) -> Result<Vec<u8>, Error> {
    // The archive names how many frames it reads; it reads the newest ones.
    let window = match archive::window_len(archive) {
        Some(k) => &window[window.len().saturating_sub(k)..],
        None => window,
    };
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

/// The window the encoder gives frame `k` of `frames`, whose first frame is
/// frame `first` of the stream: up to [`archive::WINDOW`] frames before it in
/// its [`BLOCK`], oldest first.
fn window_of<T: AsRef<[u8]>>(frames: &[T], first: usize, k: usize) -> Vec<&[u8]> {
    let at = first + k;
    let block = (at - at % BLOCK).saturating_sub(first);
    frames[block.max(k.saturating_sub(archive::WINDOW))..k]
        .iter()
        .map(AsRef::as_ref)
        .collect()
}

/// Whether frame `i` starts a block: it reads no window, and none of the
/// frames after it (`next`, those whose windows could reach before it) reads
/// a frame before it. A window frame of unknown layout reads up to
/// [`archive::WINDOW`] frames.
fn starts_block(reads: Option<usize>, next: impl Iterator<Item = Option<usize>>) -> bool {
    reads == Some(0)
        && next
            .take(archive::WINDOW - 1)
            .enumerate()
            .all(|(d, r)| r.unwrap_or(archive::WINDOW) <= d + 1)
}

/// Decode `archive`, frame `index` of a stream, against `window` (the outputs
/// of the frames before it in its block, at most [`archive::WINDOW`]), then
/// make its output the newest frame of the window. A failure is reported to
/// `store`, so that references waiting for this block's frames give up.
fn decode_into(
    window: &mut Vec<Vec<u8>>,
    archive: &[u8],
    index: usize,
    store: &store::Shared,
    work: u64,
) -> Result<Vec<u8>, Error> {
    let view: Vec<&[u8]> = window.iter().map(Vec::as_slice).collect();
    let output = decode_next(archive, &view, index, store, work).inspect_err(|_| store.fail())?;
    window.push(output.clone());
    if window.len() > archive::WINDOW {
        window.remove(0);
    }
    Ok(output)
}

/// Decode frame `index` of a stream: a reference frame reproduces the frame
/// `store` holds under its label, any other archive decodes against the
/// frames before it (`window`). Either way the output is offered to `store`.
fn decode_next(
    archive: &[u8],
    window: &[&[u8]],
    index: usize,
    store: &store::Shared,
    work: u64,
) -> Result<Vec<u8>, Error> {
    let io = |e: io::Error| Error::of(&e).unwrap_or(Error::Decode);
    let output = match archive::reference_of(archive) {
        Some(label) => {
            let frame = store.take(index, &label).map_err(io)?;
            match archive::reference_decode(archive, &frame) {
                Some(output) => output,
                None => decompress_frame_with(archive, &[&frame], work)?,
            }
        }
        None => decompress_frame_with(archive, window, work)?,
    };
    store.finish(index, &output).map_err(io)?;
    Ok(output)
}

/// A reference frame for `label` when an earlier frame of the stream has it
/// (`seen` holds their labels) and the reference is smaller than the raw archive.
fn reference_for(
    seen: &mut HashSet<[u8; 32]>,
    label: [u8; 32],
    lengths: &[u64],
) -> Option<Vec<u8>> {
    if seen.insert(label) {
        return None;
    }
    let smaller = archive::reference_length(lengths) < archive::raw_length(lengths);
    smaller.then(|| archive::reference_archive(&label, lengths))
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

/// The result of an in-memory operation, which runs the streaming one over
/// buffers: its only failures are client errors.
fn in_memory<T>(result: io::Result<T>) -> Result<T, Error> {
    result.map_err(|e| Error::of(&e).unwrap_or(Error::Decode))
}

/// Compress `input` into a uorc stream (one archive per [`FRAME`] bytes),
/// using every available core. See [`compress_stream`].
pub fn compress(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    in_memory(compress_stream(input, &mut out, steps, default_threads()))?;
    Ok(out)
}

/// Reconstruct the original bytes of a uorc stream. A stream of several
/// objects decodes to their concatenation (see [`decompress_objects`] to get
/// them apart).
pub fn decompress(stream: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    in_memory(decompress_stream(
        io::Cursor::new(stream),
        &mut out,
        work,
        default_threads(),
    ))?;
    Ok(out)
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
    let mut out = Vec::new();
    in_memory(compress_objects_stream(
        &lengths,
        &joined[..],
        &mut out,
        steps,
        default_threads(),
    ))?;
    Ok(out)
}

/// Reconstruct the ordered objects of a stream, split at the lengths its
/// headers record.
pub fn decompress_objects(stream: &[u8], work: u64) -> Result<Vec<Vec<u8>>, Error> {
    let mut objects: Vec<Vec<u8>> = Vec::new();
    in_memory(decompress_objects_stream(
        io::Cursor::new(stream),
        work,
        default_threads(),
        |starts, piece| {
            match objects.last_mut() {
                Some(last) if !starts => last.extend_from_slice(piece),
                _ => objects.push(piece.to_vec()),
            }
            Ok(())
        },
    ))?;
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

/// Check a stream end to end without keeping its output (see
/// [`verify_stream`]).
pub fn verify(stream: &[u8], original: Option<&[u8]>, work: u64) -> Result<Vec<Info>, Error> {
    in_memory(verify_stream(
        io::Cursor::new(stream),
        original,
        work,
        default_threads(),
    ))
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

/// The next frame to code and the pieces of objects it holds, or `None` after
/// the last one.
type Frames<'a> = dyn FnMut() -> io::Result<Option<(Vec<u8>, Vec<u64>)>> + 'a;

/// The frames of one plain byte stream: [`FRAME`] bytes each, the last one
/// shorter, and one empty frame for empty input.
fn plain_frames(mut reader: impl Read) -> impl FnMut() -> io::Result<Option<(Vec<u8>, Vec<u64>)>> {
    let (mut started, mut end) = (false, false);
    move || {
        if end {
            return Ok(None);
        }
        let mut frame = Vec::new();
        read_full(&mut reader, &mut frame, FRAME)?;
        end = frame.len() < FRAME;
        if frame.is_empty() && started {
            return Ok(None);
        }
        started = true;
        let n = frame.len() as u64;
        Ok(Some((frame, vec![n])))
    }
}

/// The frames of objects of `lengths` that `reader` yields back to back, as
/// [`plan`] lays them out. A reader that ends early is an error.
fn planned_frames(
    lengths: &[u64],
    mut reader: impl Read,
) -> impl FnMut() -> io::Result<Option<(Vec<u8>, Vec<u64>)>> {
    let mut plan = plan(lengths).into_iter();
    move || {
        let Some(pieces) = plan.next() else {
            return Ok(None);
        };
        let len = pieces.iter().sum::<u64>() as usize;
        let mut frame = Vec::with_capacity(len);
        read_full(&mut reader, &mut frame, len)?;
        if frame.len() < len {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(Some((frame, pieces)))
    }
}

/// Code the frames `next` yields onto `writer`, `threads` frames at a time.
/// A frame identical to an earlier one becomes a reference frame; any other
/// frame reads the frames before it in its [`BLOCK`] as its window.
fn compress_frames(
    next: &mut Frames<'_>,
    writer: &mut dyn Write,
    steps: u64,
    threads: usize,
) -> io::Result<Totals> {
    let threads = threads.max(1);
    let mut totals = Totals::default();
    // The frames before the batch that its first frames read.
    let mut before: Vec<Vec<u8>> = Vec::new();
    // The κ-labels of every frame so far: a repeat becomes a reference frame.
    let mut seen = HashSet::new();
    loop {
        let mut batch: Vec<(Vec<u8>, Vec<u64>)> = Vec::with_capacity(threads);
        let mut end = false;
        while batch.len() < threads {
            match next()? {
                Some(frame) => batch.push(frame),
                None => {
                    end = true;
                    break;
                }
            }
        }
        let all: Vec<&[u8]> = before
            .iter()
            .map(Vec::as_slice)
            .chain(batch.iter().map(|(f, _)| f.as_slice()))
            .collect();
        let labels = parallel(&batch, threads, |(f, _)| archive::frame_label(f));
        let references: Vec<Option<Vec<u8>>> = labels
            .into_iter()
            .zip(&batch)
            .map(|(label, (_, pieces))| reference_for(&mut seen, label, pieces))
            .collect();
        let jobs: Vec<usize> = (before.len()..all.len())
            .filter(|&k| references[k - before.len()].is_none())
            .collect();
        let first = totals.frames as usize - before.len();
        let mut coded = parallel(&jobs, threads, |&k| {
            let pieces = &batch[k - before.len()].1;
            compress_pieces(all[k], pieces, &window_of(&all, first, k), steps)
        })
        .into_iter();
        drop(all);
        for ((frame, _), reference) in batch.iter().zip(references) {
            let archive = match reference {
                Some(archive) => archive,
                None => coded.next().expect("one archive per coded frame")?,
            };
            writer.write_all(&archive)?;
            totals.read += frame.len() as u64;
            totals.written += archive.len() as u64;
            totals.frames += 1;
        }
        if end {
            writer.flush()?;
            return Ok(totals);
        }
        before.extend(batch.into_iter().map(|(frame, _)| frame));
        let keep = before.len().saturating_sub(archive::WINDOW);
        before.drain(..keep);
    }
}

/// Compress everything `reader` yields into a uorc stream on `writer`, one
/// [`FRAME`] at a time with `threads` frames in flight. Memory stays bounded by
/// `threads` frames plus the window whatever the input size. Every archive is
/// written exactly as the model writes it for its frame and window.
pub fn compress_stream(
    reader: impl Read,
    mut writer: impl Write,
    steps: u64,
    threads: usize,
) -> io::Result<Totals> {
    compress_frames(&mut plain_frames(reader), &mut writer, steps, threads)
}

/// [`compress_stream`] for ordered objects of `lengths` that `reader` yields
/// back to back (a file tree's manifest and files, read one at a time): the
/// stream [`compress_objects`] writes for them, in bounded memory. `reader`
/// must yield at least the sum of `lengths` bytes; any more are not read.
pub fn compress_objects_stream(
    lengths: &[u64],
    reader: impl Read,
    mut writer: impl Write,
    steps: u64,
    threads: usize,
) -> io::Result<Totals> {
    compress_frames(
        &mut planned_frames(lengths, reader),
        &mut writer,
        steps,
        threads,
    )
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

/// The archives of a stream, read from `reader` one at a time (each is
/// checked only to be delimited by its header).
pub fn archives(mut reader: impl Read) -> impl Iterator<Item = io::Result<Vec<u8>>> {
    let mut carry = Vec::new();
    let mut done = false;
    std::iter::from_fn(move || {
        if done {
            return None;
        }
        let next = next_archive(&mut reader, &mut carry).transpose();
        done = !matches!(next, Some(Ok(_)));
        next
    })
}

/// Whether the archive whose first bytes (at least its header) are `head` has
/// the length of a reference frame, so that reading its headers alone, a
/// decoder must read it whole to see whether it is one.
pub fn may_be_reference(head: &[u8]) -> bool {
    match (frame_length(head), object_lengths(head)) {
        (Ok(Some(len)), Ok(lengths)) => len as u64 == archive::reference_length(&lengths),
        _ => false,
    }
}

/// Read the headers of the stream from `reader`'s position to its end,
/// skipping bodies, and note the reference frames: the store that decoding
/// it needs. `reader` is left where it started.
fn scan(reader: &mut (impl Read + Seek)) -> io::Result<store::Store> {
    let start = reader.stream_position()?;
    let mut store = store::Store::default();
    let mut head = Vec::new();
    loop {
        head.clear();
        let mut want = HEADER_HINT;
        let len = loop {
            read_full(reader, &mut head, want)?;
            if head.is_empty() {
                reader.seek(SeekFrom::Start(start))?;
                return Ok(store);
            }
            match frame_length(&head)? {
                Some(len) => break len,
                None if head.len() < want => return Err(Error::Decode.into()),
                None => want *= 2,
            }
        };
        if may_be_reference(&head) {
            read_full(reader, &mut head, len)?;
            store.want(&head[..len.min(head.len())]);
        }
        let past = head.len() as i64 - len as i64;
        reader.seek(SeekFrom::Current(-past))?;
    }
}

/// What a decoded frame hands on: the object lengths its header declares and
/// its output.
type Sink<'a> = dyn FnMut(&[u64], Vec<u8>) -> io::Result<()> + 'a;

/// Decode the uorc stream `reader` yields: blocks (see [`BLOCK`]) on up to
/// `threads` cores at once, frames in order inside each block. `on_archive`
/// sees every archive as it is read; `on_output` gets every frame's output in
/// stream order. Memory stays bounded by about `threads` blocks of output
/// whatever the stream length. The headers are read first (bodies are
/// skipped) to learn which frames a later reference frame names; only those
/// are kept, and a stream without reference frames keeps nothing more.
fn decode_stream(
    mut reader: impl Read + Seek,
    work: u64,
    threads: usize,
    on_archive: &mut dyn FnMut(&[u8]) -> Result<(), Error>,
    on_output: &mut Sink<'_>,
) -> io::Result<Totals> {
    use std::collections::VecDeque;
    use std::sync::mpsc::{channel, Receiver, TryRecvError};

    type Outputs = Receiver<Result<(Vec<u64>, Vec<u8>), Error>>;

    /// Hand on the oldest block's outputs that are ready. With `wait`, block
    /// until one more is handed on or the oldest block is done.
    fn drain(
        blocks: &mut VecDeque<(Outputs, usize)>,
        on_output: &mut Sink<'_>,
        totals: &mut Totals,
        in_flight: &mut usize,
        wait: bool,
    ) -> io::Result<()> {
        while let Some((outputs, sent)) = blocks.front_mut() {
            let next = if wait {
                outputs.recv().map_err(|_| TryRecvError::Disconnected)
            } else {
                outputs.try_recv()
            };
            match next {
                Ok(output) => {
                    let (lengths, output) = output?;
                    totals.written += output.len() as u64;
                    totals.frames += 1;
                    on_output(&lengths, output)?;
                    *sent -= 1;
                    *in_flight -= 1;
                    if wait {
                        return Ok(());
                    }
                }
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    if *sent != 0 {
                        return Err(Error::Decode.into());
                    }
                    blocks.pop_front();
                    if wait {
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    let store = store::Shared::new(scan(&mut reader)?);
    let store = &store;
    let threads = threads.max(1);
    // Frames sent to the decoders and not yet handed on.
    let limit = threads * BLOCK;
    let mut totals = Totals::default();
    let mut carry = Vec::new();
    std::thread::scope(|scope| {
        let mut run = || -> io::Result<()> {
            let mut blocks: VecDeque<(Outputs, usize)> = VecDeque::new();
            let mut current = None;
            let mut in_flight = 0usize;
            // The next frame and the frames after it that decide whether it
            // starts a block.
            let mut ahead: VecDeque<(Vec<u8>, Option<usize>)> = VecDeque::new();
            let mut index = 0usize;
            let mut end = false;
            loop {
                while !end && ahead.len() < archive::WINDOW {
                    match next_archive(&mut reader, &mut carry)? {
                        Some(archive) => {
                            totals.read += archive.len() as u64;
                            on_archive(&archive)?;
                            let reads = archive::window_len(&archive);
                            ahead.push_back((archive, reads));
                        }
                        None => end = true,
                    }
                }
                let Some((archive, reads)) = ahead.pop_front() else {
                    break;
                };
                if index == 0 || starts_block(reads, ahead.iter().map(|(_, r)| *r)) {
                    drop(current.take());
                    while blocks.len() >= threads {
                        drain(&mut blocks, on_output, &mut totals, &mut in_flight, true)?;
                    }
                    let (frames, inbox) = channel::<(usize, Vec<u8>)>();
                    let (send, outputs) = channel();
                    scope.spawn(move || {
                        let mut window = Vec::new();
                        for (at, archive) in inbox {
                            let decoded = object_lengths(&archive).and_then(|lengths| {
                                Ok((
                                    lengths,
                                    decode_into(&mut window, &archive, at, store, work)?,
                                ))
                            });
                            let failed = decoded.is_err();
                            if send.send(decoded).is_err() || failed {
                                // Frames after this one will not be offered.
                                store.fail();
                                return;
                            }
                        }
                    });
                    blocks.push_back((outputs, 0));
                    current = Some(frames);
                }
                while in_flight >= limit {
                    drain(&mut blocks, on_output, &mut totals, &mut in_flight, true)?;
                }
                // A decoder that failed has sent its error, which drain reports.
                let _ = current.as_ref().map(|frames| frames.send((index, archive)));
                if let Some((_, sent)) = blocks.back_mut() {
                    *sent += 1;
                }
                in_flight += 1;
                index += 1;
                drain(&mut blocks, on_output, &mut totals, &mut in_flight, false)?;
            }
            drop(current);
            while !blocks.is_empty() {
                drain(&mut blocks, on_output, &mut totals, &mut in_flight, true)?;
            }
            if totals.frames == 0 {
                return Err(Error::Decode.into());
            }
            Ok(())
        };
        // A reference waiting for a frame that will now never come gives up,
        // so every decoder returns before the scope ends.
        run().inspect_err(|_| store.fail())
    })?;
    Ok(totals)
}

/// Decompress the uorc stream `reader` yields onto `writer` (see
/// [`decode_stream`] for the schedule and memory bound). A stream of several
/// objects decompresses to their concatenation.
pub fn decompress_stream(
    reader: impl Read + Seek,
    mut writer: impl Write,
    work: u64,
    threads: usize,
) -> io::Result<Totals> {
    let totals = decode_stream(reader, work, threads, &mut |_| Ok(()), &mut |_, output| {
        writer.write_all(&output)
    })?;
    writer.flush()?;
    Ok(totals)
}

/// Decompress a stream of ordered objects (see [`compress_objects_stream`]),
/// handing every piece of output to `piece` in order: `piece(true, bytes)`
/// starts the next object, `piece(false, bytes)` continues the last one. Each
/// frame's output must split exactly into the object lengths its header
/// declares.
pub fn decompress_objects_stream(
    reader: impl Read + Seek,
    work: u64,
    threads: usize,
    mut piece: impl FnMut(bool, &[u8]) -> io::Result<()>,
) -> io::Result<Totals> {
    let (mut frame, mut objects) = (0usize, 0u64);
    decode_stream(
        reader,
        work,
        threads,
        &mut |_| Ok(()),
        &mut |lengths, output| {
            let total = lengths.iter().try_fold(0u64, |a, &n| a.checked_add(n));
            if total != Some(output.len() as u64) || (frame > 0 && lengths.is_empty()) {
                return Err(Error::Decode.into());
            }
            let mut rest = output.as_slice();
            for (i, &n) in lengths.iter().enumerate() {
                let (bytes, tail) = rest.split_at(n as usize);
                rest = tail;
                let continues = frame > 0 && i == 0;
                if continues && objects == 0 {
                    return Err(Error::Decode.into());
                }
                objects += u64::from(!continues);
                piece(!continues, bytes)?;
            }
            frame += 1;
            Ok(())
        },
    )
}

/// Check a stream end to end without keeping its output: every archive's
/// frame and integrity seal must be intact, it must decode, its output must
/// have the lengths its header declares and, when `original` is given, the
/// concatenated output must equal everything `original` yields. Returns what
/// each archive's header says, in order. Memory stays bounded as in
/// [`decompress_stream`].
pub fn verify_stream(
    reader: impl Read + Seek,
    mut original: Option<impl Read>,
    work: u64,
    threads: usize,
) -> io::Result<Vec<Info>> {
    let mut infos = Vec::new();
    let mut expected = Vec::new();
    decode_stream(
        reader,
        work,
        threads,
        &mut |archive| {
            let info = inspect(archive)?;
            if !info.integrity_ok {
                return Err(Error::Decode);
            }
            infos.push(info);
            Ok(())
        },
        &mut |lengths, output| {
            let total = lengths.iter().try_fold(0u64, |a, &n| a.checked_add(n));
            if total != Some(output.len() as u64) {
                return Err(Error::Decode.into());
            }
            if let Some(original) = original.as_mut() {
                expected.clear();
                read_full(original, &mut expected, output.len())?;
                if expected != output {
                    return Err(Error::Mismatch.into());
                }
            }
            Ok(())
        },
    )?;
    if let Some(original) = original.as_mut() {
        expected.clear();
        read_full(original, &mut expected, 1)?;
        if !expected.is_empty() {
            return Err(Error::Mismatch.into());
        }
    }
    Ok(infos)
}

/// A stream compressed one frame at a time on the caller's thread, for a
/// caller that holds one frame at a time (the WebAssembly client). The
/// archives it returns, written back to back, are the stream
/// [`compress_stream`] writes for the same input.
#[derive(Default)]
pub struct FrameEncoder {
    /// The frames before the next one that it may read.
    before: Vec<Vec<u8>>,
    seen: HashSet<[u8; 32]>,
    frames: usize,
}

impl FrameEncoder {
    /// A new stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// The archive of the stream's next frame. Every frame but the last must
    /// be exactly [`FRAME`] bytes; an empty input is one empty frame.
    pub fn push(&mut self, frame: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
        let lengths = [frame.len() as u64];
        let archive = match reference_for(&mut self.seen, archive::frame_label(frame), &lengths) {
            Some(reference) => reference,
            None => {
                let reach = (self.frames % BLOCK).min(self.before.len());
                let window: Vec<&[u8]> = self.before[self.before.len() - reach..]
                    .iter()
                    .map(Vec::as_slice)
                    .collect();
                compress_frame_with(frame, &window, steps)?
            }
        };
        self.before.push(frame.to_vec());
        if self.before.len() > archive::WINDOW {
            self.before.remove(0);
        }
        self.frames += 1;
        Ok(archive)
    }
}

/// A stream decompressed one archive at a time on the caller's thread (see
/// [`FrameEncoder`]). A stream with reference frames needs them noted first
/// ([`FrameDecoder::note`]), so that the decoder keeps the frames they name.
pub struct FrameDecoder {
    window: Vec<Vec<u8>>,
    store: store::Shared,
    frames: usize,
    work: u64,
}

impl FrameDecoder {
    /// A new stream whose archives may each spend `work` in the reference decoder.
    pub fn new(work: u64) -> Self {
        FrameDecoder {
            window: Vec::new(),
            store: store::Shared::new(store::Store::default()),
            frames: 0,
            work,
        }
    }

    /// Note an archive of the stream before decoding any: a reference frame
    /// names a frame to keep until it is read. Other archives need no note.
    pub fn note(&mut self, archive: &[u8]) {
        self.store.want(archive);
    }

    /// The output of the stream's next archive.
    pub fn push(&mut self, archive: &[u8]) -> Result<Vec<u8>, Error> {
        let output = decode_into(
            &mut self.window,
            archive,
            self.frames,
            &self.store,
            self.work,
        )?;
        self.frames += 1;
        Ok(output)
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

    /// The portfolio's choices depend on each frame and its window only (the
    /// weights are the committed table), so a stream of frames from every
    /// family is the same bytes for any number of threads.
    #[test]
    fn portfolio_streams_do_not_depend_on_threads() {
        let calls: Vec<u8> = (0..20_000u32)
            .flat_map(|k| {
                let mut c = vec![0x48, 0x89, 0xC7, 0xE8];
                c.extend_from_slice(&(0x1000u32.wrapping_sub(k * 9) % 0x300).to_le_bytes());
                c
            })
            .collect();
        let mut x = 0x9E37_79B9u32;
        let noise: Vec<u8> = (0..70_000)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x >> 24) as u8
            })
            .collect();
        let frames = [
            text(3_000, 1),
            text(40_000, 2),
            text(90_000, 3),
            calls,
            noise,
            vec![0; 70_000],
            text(5_000, 4),
        ];
        let families: HashSet<usize> = frames.iter().map(|f| portfolio::family(f)).collect();
        assert_eq!(families.len(), portfolio::FAMILIES);
        let run = |threads: usize| {
            let mut queue = frames.iter();
            let mut next = || Ok(queue.next().map(|f| (f.clone(), vec![f.len() as u64])));
            let mut out = Vec::new();
            compress_frames(&mut next, &mut out, 0, threads).expect("compresses");
            out
        };
        let one = run(1);
        for threads in [2, 3, 4] {
            assert!(run(threads) == one, "{threads} threads");
        }
        let mut back = Vec::new();
        decompress_stream(io::Cursor::new(&one[..]), &mut back, DEFAULT_DECODE_WORK, 2)
            .expect("decodes");
        assert!(back == frames.concat());
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
        // x86 code: dense near calls, so the frame is coded over the filter.
        let calls: Vec<u8> = (0..400u32)
            .flat_map(|k| {
                let mut c = vec![0x48, 0x89, 0xC7, 0xE8];
                c.extend_from_slice(&(0x1000u32.wrapping_sub(k * 9) % 0x300).to_le_bytes());
                c
            })
            .collect();
        assert!(x86::dense(&calls));
        inputs.push(calls);
        // The exported model is slow unoptimized; release runs add a larger input.
        if !cfg!(debug_assertions) {
            inputs.push(text(20_000, 4));
        }
        for input in inputs {
            let native = compress_frame(&input, 0).expect("native");
            let model = on_reference_stack(|| reference_compress(&input, &[], 0)).expect("model");
            assert_eq!(native, model, "{} bytes", input.len());
            assert_eq!(archive::fast_decode(&native).as_ref(), Some(&input));
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
        let a = archive::linked_archive(archive::Codec::Ulz, &[2], &payload).expect("codec");
        assert_eq!(
            decompress(&a, DEFAULT_DECODE_WORK).as_deref(),
            Ok(&b"ab"[..])
        );
        payload.push(0);
        let padded = archive::linked_archive(archive::Codec::Ulz, &[2], &payload).expect("codec");
        assert_eq!(archive::fast_decode(&padded), None);
        assert_eq!(
            decompress(&padded, DEFAULT_DECODE_WORK).as_deref(),
            Ok(&b"ab"[..])
        );
        let mut payload = ncm::encode(b"ab").expect("fits");
        payload.push(255);
        let padded = archive::linked_archive(archive::Codec::Ncm, &[2], &payload).expect("codec");
        assert_eq!(archive::fast_decode(&padded), None);
    }

    #[test]
    fn ncm_beats_ulz_on_text() {
        let input = text(20_000, 9);
        let ulz = archive::linked_archive(archive::Codec::Ulz, &[20_000], &ulz::encode(&input))
            .expect("codec");
        let best = compress(&input, 0).expect("best");
        assert!(best.len() < ulz.len(), "{} vs {}", best.len(), ulz.len());
        assert_eq!(
            decompress(&best, DEFAULT_DECODE_WORK).expect("decodes"),
            input
        );
        let objects = [&input[..7_000], &input[7_000..]];
        let many = compress_objects(&objects, 0).expect("objects");
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
        let totals = compress_stream(&input[..], &mut stream, 0, 3).expect("compress");
        assert_eq!(totals.frames, 3);
        assert_eq!(totals.read, input.len() as u64);
        assert_eq!(frames(&stream).expect("frames").len(), 3);
        assert_eq!(stream, compress(&input, 0).expect("in memory"));
        let mut back = Vec::new();
        let totals = decompress_stream(
            io::Cursor::new(&stream[..]),
            &mut back,
            DEFAULT_DECODE_WORK,
            2,
        )
        .expect("decompress");
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

    /// A window frame is exactly the archive the exported model writes for
    /// that frame and window, and the native decoder accepts it.
    #[test]
    fn window_frames_are_the_model_archives() {
        let a = text(3000, 31);
        let b = text(2000, 32);
        let mut c = a[500..1500].to_vec();
        c.extend(text(700, 33));
        for window in [vec![&a[..]], vec![&a[..], &b[..]], vec![&b[..], &[][..]]] {
            let native = compress_frame_with(&c, &window, 0).expect("native");
            let model = on_reference_stack(|| reference_compress(&c, &window, 0)).expect("model");
            assert_eq!(native, model, "window of {}", window.len());
            assert!(reads_window(&native));
            let info = inspect(&native).expect("inspect");
            assert_eq!(info.representation, Representation::Ncm);
            assert_eq!(info.window, window.len());
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
        let alone = compress_frame(&c, 0).expect("alone");
        let with = compress_frame_with(&c, &[&a], 0).expect("window");
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
        let archive = archive::window_archive(
            archive::Codec::Ulz,
            1,
            &identity,
            &[c.len() as u64],
            &payload,
        )
        .expect("archive");
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
        let alone = compress_frame(&block, 0).expect("one frame").len();
        // Not identical frames (those become reference frames): one byte differs.
        let mut input = block.repeat(3);
        input[FRAME + 7] ^= 1;
        input[2 * FRAME + 7] ^= 2;
        let stream = compress(&input, 0).expect("stream");
        assert_eq!(frames(&stream).expect("frames").len(), 3);
        assert!(
            stream.len() < alone + alone / 16,
            "{} vs {alone}",
            stream.len()
        );
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("decodes") == input);
        let mut back = Vec::new();
        decompress_stream(
            io::Cursor::new(&stream[..]),
            &mut back,
            DEFAULT_DECODE_WORK,
            1,
        )
        .expect("streams");
        assert!(back == input);
        verify(&stream, Some(&input), DEFAULT_DECODE_WORK).expect("verifies");
    }

    /// The encoder's window: up to two frames before, never before the block.
    #[test]
    fn encoder_windows_stay_inside_their_block() {
        let frames: Vec<Vec<u8>> = (0u8..10).map(|i| vec![i]).collect();
        let at = |first: usize, k: usize| -> Vec<u8> {
            window_of(&frames, first, k).iter().map(|f| f[0]).collect()
        };
        assert_eq!(at(0, 0), Vec::<u8>::new());
        assert_eq!(at(0, 1), vec![0]);
        assert_eq!(at(0, 3), vec![1, 2]);
        assert_eq!(at(0, 4), Vec::<u8>::new());
        assert_eq!(at(0, 5), vec![4]);
        assert_eq!(at(0, 7), vec![5, 6]);
        // A batch whose first frame is frame 3 of the stream.
        assert_eq!(at(3, 1), Vec::<u8>::new());
        assert_eq!(at(3, 2), vec![1]);
        assert_eq!(at(3, 4), vec![2, 3]);
    }

    /// The decoder finds blocks from the frame headers alone, whatever block
    /// lengths the encoder chose, and every decode path agrees.
    #[test]
    fn blocks_follow_the_frame_headers() {
        let t = text(4_000, 77);
        let frame = |window: usize| {
            let window = vec![t.as_slice(); window];
            compress_frame_with(&t, &window, 0).expect("frame")
        };
        // Reads 0,1,2 | 0,1,2,0,2: frame 6 reads no window, but frame 7 reads
        // frames 5 and 6, so 6 does not start a block.
        let archives: Vec<Vec<u8>> = [0, 1, 2, 0, 1, 2, 0, 2].into_iter().map(frame).collect();
        let reads: Vec<Option<usize>> = archives.iter().map(|a| archive::window_len(a)).collect();
        assert_eq!(reads, [0, 1, 2, 0, 1, 2, 0, 2].map(Some));
        let starts: Vec<usize> = (0..reads.len())
            .filter(|&i| i == 0 || starts_block(reads[i], reads[i + 1..].iter().copied()))
            .collect();
        assert_eq!(starts, vec![0, 3]);
        let stream = archives.concat();
        let input = t.repeat(archives.len());
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("decodes") == input);
        for threads in [1, 2, 4] {
            let mut back = Vec::new();
            let totals = decompress_stream(
                io::Cursor::new(&stream[..]),
                &mut back,
                DEFAULT_DECODE_WORK,
                threads,
            )
            .expect("streams");
            assert!(back == input, "{threads} threads");
            assert_eq!(totals.frames, archives.len() as u64);
            assert_eq!(totals.read, stream.len() as u64);
        }
        verify(&stream, Some(&input), DEFAULT_DECODE_WORK).expect("verifies");
        // A stream whose block reads a window it was not given fails cleanly.
        let broken = [archives[0].clone(), archives[2].clone()].concat();
        assert_eq!(decompress(&broken, DEFAULT_DECODE_WORK), Err(Error::Decode));
        assert!(decompress_stream(
            io::Cursor::new(&broken[..]),
            &mut Vec::new(),
            DEFAULT_DECODE_WORK,
            3
        )
        .is_err());
    }

    /// An identical frame is a reference frame naming its first occurrence by
    /// κ-label, and every decoder path reproduces it, natively and through the
    /// exported reference decoder.
    #[test]
    fn identical_frames_are_stored_once() {
        let (a, b) = (text(FRAME, 71), text(FRAME, 72));
        let tail = text(5_000, 73);
        let input = [&a[..], &b, &a, &a, &tail].concat();
        let stream = compress(&input, 0).expect("stream");
        let archives = frames(&stream).expect("frames");
        let kinds: Vec<Representation> = archives
            .iter()
            .map(|f| inspect(f).expect("inspect").representation)
            .collect();
        assert_eq!(kinds[2], Representation::Reference);
        assert_eq!(kinds[3], Representation::Reference);
        assert_ne!(kinds[4], Representation::Reference);
        assert_eq!(
            archive::reference_of(archives[2]),
            Some(archive::frame_label(&a))
        );
        assert_eq!(archive::window_len(archives[2]), Some(0));
        assert_eq!(
            archives[2].len() as u64,
            archive::reference_length(&[FRAME as u64])
        );
        assert!(archives[2].len() < 100, "{}", archives[2].len());
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("decodes") == input);
        let mut back = Vec::new();
        decompress_stream(
            io::Cursor::new(&stream[..]),
            &mut back,
            DEFAULT_DECODE_WORK,
            3,
        )
        .expect("streams");
        assert!(back == input);
        verify(&stream, Some(&input), DEFAULT_DECODE_WORK).expect("verifies");
        let mut streamed = Vec::new();
        compress_stream(&input[..], &mut streamed, 0, 2).expect("compress_stream");
        assert!(streamed == stream);
        // Frame at a time, as the WebAssembly client codes: the same stream.
        let mut encoder = FrameEncoder::new();
        let one_by_one: Vec<u8> = input
            .chunks(FRAME)
            .flat_map(|frame| encoder.push(frame, 0).expect("frame"))
            .collect();
        assert!(one_by_one == stream);
        // The exported model decodes a reference frame given the frame it names
        // (a small frame: the graph evaluator copies it byte by byte).
        let small = text(4_000, 74);
        let reference = archive::reference_archive(&archive::frame_label(&small), &[4_000]);
        let model =
            on_reference_stack(|| reference_decompress(&reference, &[&small], DEFAULT_DECODE_WORK))
                .expect("the model decodes it");
        assert!(model == small);
    }

    /// References across blocks decoding on separate cores wait for the
    /// frame they name; a reference whose frame never comes fails cleanly.
    #[test]
    fn references_cross_blocks() {
        let t: Vec<Vec<u8>> = (0..5).map(|i| text(3_000 + i, 90 + i as u64)).collect();
        let mut archives: Vec<Vec<u8>> = t.iter().map(|f| archive::raw_archive(f)).collect();
        let label = |f: &[u8]| archive::frame_label(f);
        for i in [0, 3, 0] {
            archives.push(archive::reference_archive(
                &label(&t[i]),
                &[t[i].len() as u64],
            ));
        }
        let input = [&t[..], &[t[0].clone(), t[3].clone(), t[0].clone()]]
            .concat()
            .concat();
        let stream = archives.concat();
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("decodes") == input);
        verify(&stream, Some(&input), DEFAULT_DECODE_WORK).expect("verifies");
        for threads in [1, 2, 4] {
            let mut back = Vec::new();
            decompress_stream(
                io::Cursor::new(&stream[..]),
                &mut back,
                DEFAULT_DECODE_WORK,
                threads,
            )
            .expect("streams");
            assert!(back == input, "{threads} threads");
        }
        // Archive at a time, as the WebAssembly client decodes: the references
        // are noted first; unnoted, the frames they name are not kept.
        let mut decoder = FrameDecoder::new(DEFAULT_DECODE_WORK);
        archives.iter().for_each(|a| decoder.note(a));
        let back: Vec<u8> = archives
            .iter()
            .flat_map(|a| decoder.push(a).expect("decodes"))
            .collect();
        assert!(back == input);
        let mut unnoted = FrameDecoder::new(DEFAULT_DECODE_WORK);
        for a in &archives[..5] {
            unnoted.push(a).expect("decodes");
        }
        assert_eq!(unnoted.push(&archives[5]), Err(Error::Decode));
        let orphan = [archives[1].clone(), archives[5].clone()].concat();
        assert_eq!(decompress(&orphan, DEFAULT_DECODE_WORK), Err(Error::Decode));
        for threads in [1, 4] {
            let mut sink = Vec::new();
            let streamed = decompress_stream(
                io::Cursor::new(&orphan[..]),
                &mut sink,
                DEFAULT_DECODE_WORK,
                threads,
            );
            assert!(streamed.is_err());
        }
    }

    /// Objects that repeat across frames (two copies of one file in a tree) keep
    /// their piece lengths through a reference frame.
    #[test]
    fn repeated_objects_round_trip() {
        let a = text(FRAME, 81);
        let objects = vec![a.clone(), a.clone(), b"end".to_vec()];
        let stream = compress_objects(&objects, 0).expect("stream");
        let kinds: Vec<Representation> = frames(&stream)
            .expect("frames")
            .iter()
            .map(|f| inspect(f).expect("inspect").representation)
            .collect();
        assert_eq!(kinds[1], Representation::Reference, "{kinds:?}");
        assert_eq!(
            decompress_objects(&stream, DEFAULT_DECODE_WORK).expect("objects"),
            objects
        );
    }
}
