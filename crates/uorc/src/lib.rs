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
//! over the standard basis plus the outputs of the frames before it in the
//! block (up to [`REACH`]), which its payload reads as history, so the
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
//! Repeats are stored once at any offset. Every frame's output is cut into
//! content-defined units ([`chunk`]), and any run of consecutive units of one
//! frame has a κ-label, the identity of the basis `[library, run]`
//! ([`archive::frame_label`], a SHA-256 chain over the run's content). Where
//! the input repeats at least [`chunk::MIN_RUN`] bytes of units of an earlier
//! frame, the frame before ends there and the repeat is written as a
//! reference frame ([`archive::reference_archive`]): a conditional archive
//! over that basis whose 10-byte body returns the earlier run, so the repeat
//! is coded, stored and decoded once, wherever the copies sit. A whole frame
//! is one such run (the identical aligned frames of v0.3 streams). A
//! reference frame reads no window, so it may start a block. The decoder
//! reads the headers first, then cuts each frame's output into units and
//! keeps only the runs a reference names, until their last reference; a
//! reference whose run another block is still decoding waits for it. Input
//! without repeated runs is framed and coded exactly as before.

use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub mod archive;
pub mod chunk;
mod frame;
pub mod ncm;
pub mod performance;
pub mod portfolio;
pub mod region;
pub mod states;
mod store;
pub mod tree;
pub mod ulz;
pub mod weights;
pub mod x86;

pub use frame::{frame_length, frames, inspect, object_lengths, Info, Representation, HEADER_HINT};

/// Input bytes per frame. Each frame becomes one self-contained archive.
///
/// Unit tests cut frames at 1 MiB: every framing rule (cuts, references,
/// windows, blocks) reads the frame length only through this constant and
/// holds the same at any length well above [`chunk::MAX_UNIT`], while coding
/// full 8 MiB frames through the whole portfolio cost most of the acceptance
/// leg's time.
///
/// Facet: frameBytes
pub const FRAME: usize = if cfg!(test) { 1 << 20 } else { 8 << 20 };

/// Coded frames per block, the encoder's schedule: the first frame of every
/// block reads no window, and the others read up to every frame before them
/// in the same block ([`REACH`]). Reference frames inside a block do
/// not count, so they do not shorten the stretch one core decodes. The stream does not store this number. The
/// decoder finds blocks from the frame headers alone (a frame that reads no
/// window starts one) and decodes them on separate cores, so any block
/// length decodes, including a v0.2 stream, which is one block.
///
/// Facet: blockFrames
pub const BLOCK: usize = 4;

/// Most frames a window frame reads: every earlier frame of its block.
pub const REACH: usize = BLOCK - 1;

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
    compress_pieces(frame, &[frame.len() as u64], &[], steps, true)
}

/// [`compress_frame`] for a frame that follows the frames `window` (the
/// frames before it in its block, oldest first), exactly as the model's
/// `compress input window steps`: the ncm candidate is the window archive
/// whose payload reads those frames as history.
pub fn compress_frame_with(frame: &[u8], window: &[&[u8]], steps: u64) -> Result<Vec<u8>, Error> {
    compress_pieces(frame, &[frame.len() as u64], window, steps, true)
}

/// [`compress_frame`] with the output split into objects of `lengths`, which
/// sum to the frame's length. Only the header differs from the one-object
/// archive, so the choice between linked and raw is the same.
fn compress_pieces(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    steps: u64,
    side_by_side: bool,
) -> Result<Vec<u8>, Error> {
    let host = performance::declared_host();
    compress_pieces_on(frame, lengths, window, steps, side_by_side, host)
}

/// The archive a frame gets when it is written for a decode host whose
/// frame decoder runs on `threads` threads (otherwise the declared host,
/// [`performance::declared_host`]): [`compress_frame_with`] at another point
/// of the host facet (with `steps == 0`).
pub fn compress_frame_for(frame: &[u8], window: &[&[u8]], threads: u64) -> Vec<u8> {
    let host = performance::Host {
        threads,
        ..performance::declared_host()
    };
    compress_pieces_on(frame, &[frame.len() as u64], window, 0, true, host)
        .unwrap_or_else(|_| archive::raw_objects(frame, &[frame.len() as u64]))
}

fn compress_pieces_on(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    steps: u64,
    side_by_side: bool,
    host: performance::Host,
) -> Result<Vec<u8>, Error> {
    if steps > 0 {
        // `Client.compress` with steps: the smaller of the portfolio's archive
        // (native, the model's bytes) and the program search's (the exported
        // search, `Client.search`), the search kept only when strictly shorter
        // (`Client.smaller`); as one object, then relabelled
        let n = [frame.len() as u64];
        let quick = compress_pieces(frame, &n, window, 0, side_by_side)?;
        let found = on_reference_stack(|| reference_search(frame, steps))?;
        let archive = match found {
            Some(f) if f.len() < quick.len() => f,
            _ => quick,
        };
        return if lengths.len() == 1 {
            Ok(archive)
        } else {
            frame::relabel(&archive, lengths)
        };
    }
    let raw = archive::raw_length(lengths);
    // the portfolio codes over the fixed entries' window; the window step
    // then reaches every frame given
    let reach = window;
    let window = &window[window.len().saturating_sub(archive::WINDOW)..];
    let found = portfolio_trials(
        frame,
        lengths,
        window,
        &portfolio::plan(frame),
        side_by_side,
    );
    let best = found
        .into_iter()
        .rev()
        .fold(None, |kept, head| pick_trial(head, kept));
    match best {
        Some((linked, _)) if (linked.len() as u64) < raw => {
            let (stepped, k) = step_window(frame, lengths, reach, linked, host);
            let window = &reach[reach.len() - k..];
            let stepped = step_table(frame, lengths, window, stepped, host);
            Ok(step_streams(frame, lengths, window, stepped, host))
        }
        _ => Ok(archive::raw_objects(frame, lengths)),
    }
}

/// The window facets stepped on the frame's chosen archive
/// (`Client.stepWindow`), before the table facet ([`step_table`]). The
/// chosen coding (x86 or not, and its profile word) is recoded over one more
/// earlier frame of `reach` at a time (all the frames before it in its block,
/// oldest first), then with one more match table bit at a time, and each
/// step is kept only while the archive gets strictly shorter and the decoder
/// still fits `host` ([`performance::Host::holds`]); each facet stops at the first
/// step that does not pay or does not fit. The frame records its window by
/// its entry and its match bits in its profile word, so a frame where no
/// step pays is the portfolio's archive byte for byte. Returns the archive
/// and the number of frames it reads (the last ones of `reach`). State
/// frames read no window and are returned as they are, as are raw frames.
pub fn step_window(
    frame: &[u8],
    lengths: &[u64],
    reach: &[&[u8]],
    chosen: Vec<u8>,
    host: performance::Host,
) -> (Vec<u8>, usize) {
    let mut k = reach.len().min(archive::WINDOW);
    let one = |w: u64| {
        ncm::streams::split(w)
            .filter(|(_, p)| p.serial())
            .map(|(m, _)| m)
    };
    let (x86, mut word) = match archive::frame_codec(&chosen) {
        Some(archive::Codec::Profile(w)) if one(w).is_some() => (false, one(w).unwrap_or(0)),
        Some(archive::Codec::ProfileX86(w)) if one(w).is_some() => (true, one(w).unwrap_or(0)),
        // the v0.3 entries code the V03 profile
        Some(archive::Codec::Ncm) => (false, ncm::V03),
        Some(archive::Codec::NcmX86) => (true, ncm::V03),
        _ => return (chosen, k),
    };
    if archive::state_frame(&chosen).is_some() {
        return (chosen, k);
    }
    let n = frame.len() as u64;
    let codec = if x86 {
        archive::Codec::NcmX86
    } else {
        archive::Codec::Ncm
    };
    // the frame over the last `depth` frames of `reach` under word `w`, when
    // it fits the host and is strictly shorter than `best`
    let trial = |depth: usize, w: u32, best: &[u8]| -> Option<Vec<u8>> {
        let window = &reach[reach.len() - depth..];
        let history = window.concat();
        if !host.holds(n, history.len() as u64, false, w) {
            return None;
        }
        let filtered;
        let (data, past) = if x86 {
            filtered = (
                x86::filter(frame, false, history.len() as u64).0,
                x86::filter(&history, false, 0).0,
            );
            (&filtered.0[..], &filtered.1[..])
        } else {
            (frame, &history[..])
        };
        let identity = (!window.is_empty()).then(|| archive::window_identity(window));
        let frame_of = |payload: &[u8]| match &identity {
            None => archive::linked_archive_profile(codec, u64::from(w), lengths, payload),
            Some(id) => {
                archive::window_archive_profile(codec, u64::from(w), window, id, lengths, payload)
            }
        };
        let stop = best.len().saturating_sub(frame_of(&[])?.len());
        let a = frame_of(&ncm::encode_below(past, data, w, stop)?)?;
        (a.len() < best.len()).then_some(a)
    };
    let mut best = chosen;
    // window depth: one more earlier frame while it pays
    while k < reach.len() {
        match trial(k + 1, word, &best) {
            Some(a) => {
                best = a;
                k += 1;
            }
            None => break,
        }
    }
    // match table bits: one more while it pays
    let mut mb = ncm::match_bits(n, word);
    while mb < ncm::MATCH_FIELD {
        let w = ncm::with_match_bits(word, mb + 1);
        match trial(k, w, &best) {
            Some(a) => {
                best = a;
                word = w;
                mb += 1;
            }
            None => break,
        }
    }
    (best, k)
}

/// The context table facet stepped on the frame's chosen archive
/// (`Client.stepTable`): the chosen profile is recoded with one more table
/// bit at a time, kept while the archive gets strictly shorter (the steps
/// per byte do not depend on the table), and the stepping ends at the first
/// table that does not pay or that `host` cannot hold
/// ([`performance::Host::footprint`]). The frame records the table bits it was coded
/// with in its profile word ([`ncm::TABLE_SHIFT`]), so a frame where no step
/// pays is the archive the portfolio chose, byte for byte. Raw frames and
/// frames of the fixed v0.3 entries are returned as they are.
pub fn step_table(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    chosen: Vec<u8>,
    host: performance::Host,
) -> Vec<u8> {
    let n = frame.len() as u64;
    // (state, x86, profile word) of the chosen frame
    let (state, x86, word) = if let Some(f) = archive::state_frame(&chosen) {
        (f.state, f.x86, f.profile)
    } else {
        let one = |w: u64| {
            ncm::streams::split(w)
                .filter(|(_, p)| p.serial())
                .map(|(m, _)| m)
        };
        match archive::frame_codec(&chosen) {
            Some(archive::Codec::Profile(w)) if one(w).is_some() => (0, false, one(w).unwrap_or(0)),
            Some(archive::Codec::ProfileX86(w)) if one(w).is_some() => {
                (0, true, one(w).unwrap_or(0))
            }
            _ => return chosen,
        }
    };
    let joined;
    let history: &[u8] = if state != 0 {
        match states::state(state) {
            Some(s) => s,
            None => return chosen,
        }
    } else {
        joined = window.concat();
        &joined
    };
    let filtered;
    let (data, past) = if x86 {
        filtered = (
            x86::filter(frame, false, history.len() as u64).0,
            x86::filter(history, false, 0).0,
        );
        (&filtered.0[..], &filtered.1[..])
    } else {
        (frame, history)
    };
    let identity = if state != 0 {
        states::identity(state)
    } else {
        (!window.is_empty()).then(|| archive::window_identity(window))
    };
    let frame_of = |w: u32, payload: &[u8]| -> Option<Vec<u8>> {
        if state != 0 {
            return archive::state_archive(x86, w, identity.as_ref()?, lengths, payload);
        }
        let codec = if x86 {
            archive::Codec::NcmX86
        } else {
            archive::Codec::Ncm
        };
        match &identity {
            None => archive::linked_archive_profile(codec, u64::from(w), lengths, payload),
            Some(id) => {
                archive::window_archive_profile(codec, u64::from(w), window, id, lengths, payload)
            }
        }
    };
    let warm = state != 0;
    let modelled = if warm { n + history.len() as u64 } else { n };
    let mut best = chosen;
    let mut tb = ncm::table_bits(modelled, word);
    while tb < ncm::TABLE_MAX {
        tb += 1;
        let w = ncm::with_table_bits(word, tb);
        if !host.holds(n, history.len() as u64, warm, w) {
            break;
        }
        // only a strictly shorter archive is kept, so the encoder stops once
        // its payload reaches the chosen archive's length less the form
        let Some(form) = frame_of(w, &[]).map(|a| a.len()) else {
            break;
        };
        let stop = best.len().saturating_sub(form);
        let payload = if warm {
            ncm::encode_warm_below(past, data, w, stop)
        } else {
            ncm::encode_below(past, data, w, stop)
        };
        match payload.and_then(|p| frame_of(w, &p)) {
            Some(a) if a.len() < best.len() => best = a,
            _ => break,
        }
    }
    best
}

/// The probability and streams facets stepped last on the frame's chosen
/// archive (`Client.stepStreams`). The probability field first: the chosen
/// coding is tried at 16-bit probabilities as one stream (the frame's model,
/// coded at 16 bits, [`ncm::streams::Plan::bits`]), and kept only when its
/// archive is shorter (the decode steps are the same). Then, for a decode
/// host whose frame decoder runs on more than one thread, the coding is cut
/// into streams over one shared state ([`ncm::streams::derive`]): the stream
/// count steps while the counted longest chain of decode steps on the host
/// falls, and the epoch is the least archive, then steps. On a one-thread
/// host (the page's worker, the default) one stream is the derivation. Raw
/// frames, state frames and the fixed v0.3 entries are kept byte for byte.
pub fn step_streams(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    chosen: Vec<u8>,
    host: performance::Host,
) -> Vec<u8> {
    if archive::state_frame(&chosen).is_some() {
        return chosen;
    }
    let (x86, word) = match archive::frame_codec(&chosen) {
        Some(archive::Codec::Profile(w)) => (false, w),
        Some(archive::Codec::ProfileX86(w)) => (true, w),
        _ => return chosen,
    };
    let Some((profile, plan)) = ncm::streams::split(word) else {
        return chosen;
    };
    if !plan.serial() {
        return chosen;
    }
    let history = window.concat();
    let filtered;
    let (data, past) = if x86 {
        filtered = (
            x86::filter(frame, false, history.len() as u64).0,
            x86::filter(&history, false, 0).0,
        );
        (&filtered.0[..], &filtered.1[..])
    } else {
        (frame, &history[..])
    };
    let codec = if x86 {
        archive::Codec::NcmX86
    } else {
        archive::Codec::Ncm
    };
    let identity = (!window.is_empty()).then(|| archive::window_identity(window));
    let archive_of = |plan: ncm::streams::Plan, payload: &[u8]| {
        let w = ncm::streams::word(profile, plan)?;
        match &identity {
            None => archive::linked_archive_profile(codec, w, lengths, payload),
            Some(id) => archive::window_archive_profile(codec, w, window, id, lengths, payload),
        }
    };
    let host_threads = usize::try_from(host.threads).unwrap_or(usize::MAX).max(1);
    let threads = ncm::streams::threads(ncm::streams::Plan::of(host_threads, 1));
    // the probability field, by bytes (the steps are equal)
    let wide = ncm::streams::Plan {
        bits: 16,
        ..ncm::streams::Plan::ONE
    };
    let mut best = chosen;
    let mut bits = 12;
    if let Some((payload, _)) = ncm::streams::encode_below(past, data, profile, wide, 1, best.len())
    {
        if let Some(a) = archive_of(wide, &payload).filter(|a| a.len() < best.len()) {
            best = a;
            bits = 16;
        }
    }
    if host_threads <= 1 {
        return best;
    }
    let n = frame.len() as u64;
    let h = history.len() as u64;
    let coding = performance::trial_steps(profile, n);
    let holds = |k: usize| host.holds_streams(n, h, profile, k as u64);
    let Some(d) =
        ncm::streams::derive(past, data, profile, bits, host_threads, coding, threads, holds)
    else {
        return best;
    };
    archive_of(d.plan, &d.payload).unwrap_or(best)
}

/// The trials of `plan` on `frame`, in plan order: each strategy's archive
/// and decode steps, or `None` when it was excluded, failed, or could not
/// be shorter than the raw archive (or, coded one after the other, one byte
/// past the least archive so far).
fn portfolio_trials(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    plan: &[portfolio::Strategy],
    side_by_side: bool,
) -> Vec<Option<(Vec<u8>, u64)>> {
    let history = window.concat();
    let identity = (!window.is_empty()).then(|| archive::window_identity(window));
    // The strategy portfolio (`Client.portfolio`): code the frame by the
    // first strategies its family's schedule orders, as trials of archive
    // bytes and decode steps, and keep the least in bytes, then in steps
    // (`performance::chosen`). A filtered strategy codes the filtered frame
    // after the filtered window; it is excluded, never coded, when the
    // filter rewrites nothing (`Client.excluded`).
    let raw = archive::raw_length(lengths);
    let n = frame.len() as u64;
    // `code(strategy, learn, bound)`: the strategy's archive and its decode
    // steps, or `None` when it cannot be shorter than `bound` bytes. An
    // archive is at least its empty-payload form plus the payload, so the
    // encoder stops once the payload passes `bound` less that form: an
    // archive it would have finished could not have been chosen, and every
    // archive it does finish is the one it always wrote.
    let prepared = |x86: bool| -> Option<Prepared<'_>> {
        let (data, past) = if x86 {
            let (data, hits) = x86::filter(frame, false, history.len() as u64);
            if hits == 0 {
                return None;
            }
            (
                std::borrow::Cow::Owned(data),
                std::borrow::Cow::Owned(x86::filter(&history, false, 0).0),
            )
        } else {
            (
                std::borrow::Cow::Borrowed(frame),
                std::borrow::Cow::Borrowed(&history[..]),
            )
        };
        Some(Prepared {
            data,
            past,
            codec: if x86 {
                archive::Codec::NcmX86
            } else {
                archive::Codec::Ncm
            },
            lengths,
            window,
            identity: identity.as_ref(),
            n,
        })
    };
    // A state strategy (`Archive.stateCandidate`): the frame coded as the
    // continuation of the κ-named model state, its own basis root; its steps
    // are the reference measure, n + |S| modelled bytes. `warm_input` gives
    // the bytes it codes and replays (through the x86 filter or not).
    let warm_input = |strategy: portfolio::Strategy| -> Option<WarmInput> {
        let state = states::state(strategy.state)?;
        let identity = states::identity(strategy.state)?;
        let (data, from) = if strategy.x86 {
            let (data, hits) = x86::filter(frame, false, state.len() as u64);
            if hits == 0 {
                return None;
            }
            (
                std::borrow::Cow::Owned(data),
                std::borrow::Cow::Owned(x86::filter(state, false, 0).0),
            )
        } else {
            (
                std::borrow::Cow::Borrowed(frame),
                std::borrow::Cow::Borrowed(state),
            )
        };
        Some(WarmInput {
            data,
            from,
            identity,
        })
    };
    let warm_stop = |strategy: portfolio::Strategy, w: &WarmInput, bound: u64| {
        let form =
            archive::state_archive(strategy.x86, strategy.profile, &w.identity, lengths, &[])
                .map_or(0, |a| a.len() as u64);
        usize::try_from(bound.saturating_sub(form)).unwrap_or(usize::MAX)
    };
    let warm_trial = |strategy: portfolio::Strategy, w: &WarmInput, payload: Option<Vec<u8>>| {
        let steps = performance::trial_steps(strategy.profile, n + w.from.len() as u64);
        payload
            .and_then(|payload| {
                archive::state_archive(
                    strategy.x86,
                    strategy.profile,
                    &w.identity,
                    lengths,
                    &payload,
                )
            })
            .map(|a| (a, steps))
    };
    let code = |strategy: portfolio::Strategy, learn: bool, bound: u64| -> Option<(Vec<u8>, u64)> {
        if strategy.state != 0 {
            let w = warm_input(strategy)?;
            let payload = ncm::encode_warm_below(
                &w.from,
                &w.data,
                strategy.profile,
                warm_stop(strategy, &w, bound),
            );
            return warm_trial(strategy, &w, payload);
        }
        let at = prepared(strategy.x86)?;
        if !learn {
            return ncm::encode_below(
                &at.past,
                &at.data,
                strategy.profile,
                at.stop(strategy.profile, bound),
            )
            .and_then(|payload| at.frame_of(strategy.profile, &payload));
        }
        // The learned profile (`Client.learnedTrial`): the learning pass is
        // bounded by the raw archive only: the learned profile can be shorter
        // than a bound its own pass exceeds, and stopping that pass would lose
        // it (the model always codes both).
        let coded = ncm::encode_learn_below(
            &at.past,
            &at.data,
            strategy.profile,
            at.stop(strategy.profile, raw),
        );
        at.settle(strategy.profile, coded, bound)
    };
    // Twins without a state (the same transform, profiles that differ only in
    // mixer selection, `ncm::F_SEL`) learn in one shared pass: the same two
    // learning passes, each bounded by the raw archive, for one pass of the
    // work they share (`ncm::encode_pairs_learn_below`).
    // Twins continuing the same state share one replay of it and one pass
    // over the frame (`ncm::encode_pair_warm_below`), each bounded as alone.
    let twins = |f: portfolio::Strategy, s: portfolio::Strategy| {
        f.state == s.state && f.x86 == s.x86 && f.profile ^ s.profile == ncm::F_SEL
    };
    let code_twins = |f: portfolio::Strategy, s: portfolio::Strategy| {
        let at = prepared(f.x86)?;
        let (a, b) = ncm::encode_pairs_learn_below(
            &at.past,
            &at.data,
            f.profile,
            [at.stop(f.profile, raw), at.stop(s.profile, raw)],
        );
        Some((at, a, b))
    };
    // The scheduled strategies: each one without a state also learns its
    // context mask (a state strategy codes its profile as given). With cores
    // to spare they code side by side, each bounded by the raw archive;
    // otherwise one after the other, each later one bounded by one byte past
    // the least archive so far (an archive of as many bytes may still have
    // fewer steps). Either way the kept archive is the model's right fold of
    // `pick` (`Client.portfolio`, `performance::chosen`).
    let learns = |k: usize| plan[k].state == 0;
    // Each strategy codes with the first later unpaired twin of the plan in
    // one pass (`mate`), side by side or one group after the other.
    let mut mate: Vec<Option<usize>> = vec![None; plan.len()];
    let mut paired = vec![false; plan.len()];
    for k in 0..plan.len() {
        if paired[k] {
            continue;
        }
        if let Some(j) = (k + 1..plan.len()).find(|&j| !paired[j] && twins(plan[k], plan[j])) {
            mate[k] = Some(j);
            paired[k] = true;
            paired[j] = true;
        }
    }
    // `group(k, bound)`: the trials of strategy k and of its mate, if any.
    let group = |k: usize, bound: u64| -> (Trial, Trial) {
        let Some(j) = mate[k] else {
            return (code(plan[k], learns(k), bound), None);
        };
        let (s, t) = (plan[k], plan[j]);
        if s.state != 0 {
            let Some(w) = warm_input(s) else {
                return (None, None);
            };
            let (a, b) = ncm::encode_pair_warm_below(
                &w.from,
                &w.data,
                s.profile,
                [warm_stop(s, &w, bound), warm_stop(t, &w, bound)],
            );
            return (warm_trial(s, &w, a), warm_trial(t, &w, b));
        }
        match code_twins(s, t) {
            None => (None, None),
            Some((at, ca, cb)) => at.settle_pair([s.profile, t.profile], [ca, cb], bound),
        }
    };
    let leads: Vec<usize> = (0..plan.len())
        .filter(|&k| !(0..k).any(|i| mate[i] == Some(k)))
        .collect();
    let mut found: Vec<Option<Trial>> = vec![None; plan.len()];
    let mut place = |k: usize, (a, b): (Trial, Trial)| {
        if let Some(j) = mate[k] {
            found[j] = Some(b);
        }
        found[k] = Some(a);
    };
    if side_by_side && cfg!(not(target_family = "wasm")) {
        // each group on its own core, bounded by the raw archive
        let coded: Vec<(Trial, Trial)> = std::thread::scope(|s| {
            // each group's thread holds a core of the compression's pool
            // (taken before any starts), so its encoders borrow only cores
            // no group runs on
            let pool = ncm::Cores::current();
            let held: Vec<_> = leads[1..]
                .iter()
                .map(|_| pool.as_ref().map(ncm::Cores::hold))
                .collect();
            let others: Vec<_> = leads[1..]
                .iter()
                .zip(held)
                .map(|(&k, held)| {
                    s.spawn(move || match held {
                        Some(h) => h.run(|| group(k, raw)),
                        None => group(k, raw),
                    })
                })
                .collect();
            let first = group(leads[0], raw);
            // this thread only waits from here: its core is spare
            let _idle = pool.as_ref().map(ncm::Cores::idle);
            std::iter::once(first)
                .chain(
                    others
                        .into_iter()
                        .map(|h| h.join().expect("strategy thread")),
                )
                .collect()
        });
        for (&k, c) in leads.iter().zip(coded) {
            place(k, c);
        }
    } else {
        // one group after the other, each bounded by one byte past the least
        // archive so far; a twin is coded under the bound of its mate's
        // place, which can only let a trial finish that the fold of `pick`
        // would not keep (a bound never changes an archive)
        let mut bound = raw;
        for &k in &leads {
            let c = group(k, bound);
            for t in [&c.0, &c.1].into_iter().flatten() {
                bound = bound.min(t.0.len() as u64 + 1);
            }
            place(k, c);
        }
    }
    found.into_iter().map(Option::flatten).collect()
}

/// A coded trial: the archive and its decode steps.
type Trial = Option<(Vec<u8>, u64)>;

/// What a state strategy codes: the frame and the state it continues, both
/// through the x86 filter or not, and the state's identity.
struct WarmInput<'a> {
    data: std::borrow::Cow<'a, [u8]>,
    from: std::borrow::Cow<'a, [u8]>,
    identity: [u8; 32],
}

/// A frame prepared for one transform: the bytes the coder reads (the frame
/// and its window, through the x86 filter or not) and how a payload becomes
/// the frame's archive.
struct Prepared<'a> {
    data: std::borrow::Cow<'a, [u8]>,
    past: std::borrow::Cow<'a, [u8]>,
    codec: archive::Codec,
    lengths: &'a [u64],
    window: &'a [&'a [u8]],
    identity: Option<&'a [u8; 32]>,
    n: u64,
}

impl Prepared<'_> {
    // The native ncm encoder is byte-identical to the model
    // (`native_archives_are_the_model_archives`) and every encoder output
    // decodes to its input, so a self-decode would only repeat its cost.
    fn frame_of(&self, profile: u32, payload: &[u8]) -> Option<(Vec<u8>, u64)> {
        match self.identity {
            None => archive::linked_archive_profile(
                self.codec,
                u64::from(profile),
                self.lengths,
                payload,
            ),
            Some(identity) => archive::window_archive_profile(
                self.codec,
                u64::from(profile),
                self.window,
                identity,
                self.lengths,
                payload,
            ),
        }
        .map(|a| (a, performance::trial_steps(profile, self.n)))
    }

    /// The payload length at which a frame of `profile` reaches `bound`.
    fn stop(&self, profile: u32, bound: u64) -> usize {
        let form = self.frame_of(profile, &[]).map_or(0, |a| a.0.len() as u64);
        usize::try_from(bound.saturating_sub(form)).unwrap_or(usize::MAX)
    }

    /// [`Prepared::settle`] for twins coded in one learning pass: when both
    /// learned profiles are themselves twins (the same contexts pruned), both
    /// pruned codings share one pass too ([`ncm::encode_pair_below`]); each
    /// payload is the one its own pass writes, so each trial is `settle`'s.
    fn settle_pair(
        &self,
        profiles: [u32; 2],
        coded: [Option<(Vec<u8>, u32)>; 2],
        bound: u64,
    ) -> (Trial, Trial) {
        let [a, b] = coded;
        let learned = |c: &Option<(Vec<u8>, u32)>, p: u32| c.as_ref().map_or(p, |c| c.1);
        let (la, lb) = (learned(&a, profiles[0]), learned(&b, profiles[1]));
        let shared = la != profiles[0] && lb != profiles[1] && la ^ lb == ncm::F_SEL;
        let (Some(a), Some(b), true) = (&a, &b, shared) else {
            return (
                self.settle(profiles[0], a, bound),
                self.settle(profiles[1], b, bound),
            );
        };
        let plain_a = self.frame_of(profiles[0], &a.0);
        let plain_b = self.frame_of(profiles[1], &b.0);
        let keep = |plain: &Trial| {
            plain
                .as_ref()
                .map_or(bound, |p| bound.min(p.0.len() as u64 + 1))
        };
        let (pa, pb) = ncm::encode_pair_below(
            &self.past,
            &self.data,
            la,
            false,
            [self.stop(la, keep(&plain_a)), self.stop(lb, keep(&plain_b))],
        );
        let pruned_a = pa.and_then(|(p, _)| self.frame_of(la, &p));
        let pruned_b = pb.and_then(|p| self.frame_of(lb, &p));
        (pick_trial(pruned_a, plain_a), pick_trial(pruned_b, plain_b))
    }

    /// The learned trial (`Client.learnedTrial`) from a learning pass: the
    /// strategy's profile without the contexts that saved nothing on this
    /// frame, coded too and kept when fewer bytes, or as many bytes and fewer
    /// steps.
    fn settle(
        &self,
        profile: u32,
        coded: Option<(Vec<u8>, u32)>,
        bound: u64,
    ) -> Option<(Vec<u8>, u64)> {
        let (payload, learned) = coded?;
        let plain = self.frame_of(profile, &payload);
        if learned == profile {
            return plain;
        }
        let keep = plain
            .as_ref()
            .map_or(bound, |p| bound.min(p.0.len() as u64 + 1));
        let pruned = ncm::encode_below(&self.past, &self.data, learned, self.stop(learned, keep))
            .and_then(|p| self.frame_of(learned, &p));
        pick_trial(pruned, plain)
    }
}

/// The archive of scheduled strategy `which` (an index into
/// [`portfolio::plan`], fewer than [`portfolio::MAX_PLAN`]) for a frame
/// with objects of `lengths` after the frames `window`, bounded by the raw
/// archive: `None` when there is no such strategy, it is excluded, or it
/// cannot be shorter. Coding every scheduled strategy and passing the
/// archives to [`choose`] gives the archive [`compress_frame_with`] writes
/// (with `steps == 0`), so callers on separate cores (the page's workers)
/// write the library's bytes.
pub fn compress_strategy(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    which: usize,
) -> Option<Vec<u8>> {
    let strategy = *portfolio::plan(frame).get(which)?;
    // the portfolio codes over the fixed entries' window (see compress_pieces)
    let window = &window[window.len().saturating_sub(archive::WINDOW)..];
    portfolio_trials(frame, lengths, window, &[strategy], true)
        .pop()
        .flatten()
        .map(|(a, _)| a)
}

/// The archive a frame gets when the encoder codes `strategies` (in order)
/// instead of the ones [`portfolio::plan`] schedules, each bounded by the
/// raw archive and each without a state learning its context mask, keeping
/// the least in bytes, then in decode steps. With `portfolio::SHIPPED.plan(frame)`
/// it is the archive [`compress_frame_with`] writes (with `steps == 0`); with
/// other strategies it is the frame at another point of the codec space (the
/// claim decider, `crates/frontier`). `None` entries (a strategy the coder
/// cannot apply) yield no candidate.
pub fn compress_with_strategies(
    frame: &[u8],
    lengths: &[u64],
    window: &[&[u8]],
    strategies: &[Option<portfolio::Strategy>],
) -> Vec<u8> {
    let plan: Vec<portfolio::Strategy> = strategies.iter().flatten().copied().collect();
    let best = portfolio_trials(frame, lengths, window, &plan, true)
        .into_iter()
        .rev()
        .fold(None, |kept, head| pick_trial(head, kept));
    match best {
        Some((linked, _)) if (linked.len() as u64) < archive::raw_length(lengths) => linked,
        _ => archive::raw_objects(frame, lengths),
    }
}

/// The frame's archive from its scheduled strategies' archives, in plan
/// order (see [`compress_strategy`]): the least in bytes, then in decode
/// steps (`Client.pick`), then the raw archive unless the kept one is
/// strictly shorter.
pub fn choose(frame: &[u8], lengths: &[u64], archives: Vec<Option<Vec<u8>>>) -> Vec<u8> {
    let n = frame.len() as u64;
    let best = archives
        .into_iter()
        .map(|a| {
            a.and_then(|a| Some((trial_steps(&a, n)?, a)))
                .map(|(s, a)| (a, s))
        })
        .rev()
        .fold(None, |kept, head| pick_trial(head, kept));
    match best {
        Some((linked, _)) if (linked.len() as u64) < archive::raw_length(lengths) => linked,
        _ => archive::raw_objects(frame, lengths),
    }
}

/// The decode steps of a coded ncm archive of `n` output bytes, as the
/// portfolio measures them (`Client.trialSteps`): a state frame models
/// n + |S| bytes.
fn trial_steps(archive: &[u8], n: u64) -> Option<u64> {
    let info = frame::inspect(archive).ok()?;
    if !matches!(
        info.representation,
        frame::Representation::Ncm | frame::Representation::NcmX86
    ) {
        return None;
    }
    let profile = info.ncm_profile.unwrap_or(ncm::V03);
    let modelled = match archive::state_frame(archive) {
        Some(st) => n + states::state(st.state)?.len() as u64,
        None => n,
    };
    Some(performance::trial_steps(profile, modelled))
}

/// `Client.pick` over coded archives and their steps: `head` replaces `kept`
/// when it has fewer bytes, or as many bytes and fewer steps
/// (`performance::pick`).
fn pick_trial(
    head: Option<(Vec<u8>, u64)>,
    kept: Option<(Vec<u8>, u64)>,
) -> Option<(Vec<u8>, u64)> {
    match (head, kept) {
        (None, kept) => kept,
        (head, None) => head,
        (Some(h), Some(k)) => {
            let t = |a: &(Vec<u8>, u64)| performance::Trial {
                bytes: a.0.len() as u64,
                steps: a.1,
            };
            // `pick` returns the kept trial unless the head is strictly better
            if performance::pick(Some(t(&h)), t(&k)) == t(&k) {
                Some(k)
            } else {
                Some(h)
            }
        }
    }
}

/// The client's model data as the exported model reads it: the strategy
/// table, the weight table and the κ-named states, in order.
fn model_data() -> (
    uorc_compress::Bytes,
    uorc_compress::Bytes,
    uorc_compress::List<uorc_compress::Bytes>,
) {
    use uorc_compress::{Bytes, List};
    (
        Bytes::lit(portfolio::SHIPPED.strategies),
        Bytes::lit(portfolio::SHIPPED.weights),
        states::ALL
            .iter()
            .rev()
            .fold(List::nil(), |list, s| List::cons(Bytes::lit(s), list)),
    )
}

/// The exported model's archive of one frame with no window and no search
/// (`Client.compress input [] 0`), for checking the native client against it.
#[doc(hidden)]
pub fn model_compress_frame(frame: &[u8]) -> Result<Vec<u8>, Error> {
    on_reference_stack(|| reference_compress(frame, &[], 0))
}

/// The exported model's decode of one archive with no window
/// (`Client.decompress`), for checking the native client against it.
#[doc(hidden)]
pub fn model_decompress_frame(archive: &[u8]) -> Result<Vec<u8>, Error> {
    on_reference_stack(|| reference_decompress(archive, &[], DEFAULT_DECODE_WORK))
}

/// The exported program search (`Client.search input steps`).
fn reference_search(input: &[u8], steps: u64) -> Result<Option<Vec<u8>>, Error> {
    match uorc_search::run(uorc_search::Bytes::lit(input), steps) {
        Ok(found) => Ok(found.map(|a| a.octets().to_vec())),
        Err(_) => Err(Error::Overflow),
    }
}

fn reference_compress(input: &[u8], window: &[&[u8]], steps: u64) -> Result<Vec<u8>, Error> {
    use uorc_compress::{Bytes, List};
    let (strategies, weights, states) = model_data();
    match uorc_compress::run(
        strategies,
        weights,
        states,
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
        archive::fast_decode(archive).or_else(|| archive::state_decode(archive))
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
        states::ALL
            .iter()
            .rev()
            .fold(List::nil(), |list, s| List::cons(Bytes::lit(s), list)),
        work,
    ) {
        Ok(Some(output)) => Ok(output.octets().to_vec()),
        Ok(None) => Err(Error::Decode),
        Err(_) => Err(Error::Overflow),
    }
}

/// Whether frame `i` starts a block: it reads no window, and none of the
/// frames after it (`next`, those whose windows could reach before it) reads
/// a frame before it. A window frame of unknown layout reads up to
/// [`archive::WINDOW`] frames.
fn starts_block(reads: Option<usize>, next: impl Iterator<Item = Option<usize>>) -> bool {
    reads == Some(0)
        && next
            .take(REACH - 1)
            .enumerate()
            .all(|(d, r)| r.unwrap_or(archive::WINDOW) <= d + 1)
}

/// Decode `archive`, frame `index` of a stream, against `window` (the outputs
/// of the frames before it in its block, at most [`REACH`]), then
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
    if window.len() > REACH {
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

/// What the decoder's schedule of a stream costs (`Uorc.Performance::wholeHost`,
/// `Uorc.Performance::leastArchive`), from [`decode_schedule`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    /// Decode steps of every frame ([`performance::frame_steps`]).
    pub total: u64,
    /// The longest chain of dependent decode steps: frames of a block decode
    /// in order on one core, a reference frame waits for the frame that holds
    /// its run, and blocks are otherwise free to run at once (the schedule
    /// [`decompress_stream`] keeps on any number of threads).
    pub chain: u64,
    /// The information the frames' models assign to their output, in 2^-24
    /// bit ([`ncm::INFORMATION_BITS`]): the ncm decoder's sum
    /// ([`ncm::measure_information`]) for coded frames, 8 bits per byte for
    /// raw frames, none for a reference frame.
    pub information: u64,
    /// Frames of the stream.
    pub frames: u64,
}

/// Decode `stream` on this thread in stream order, and measure the schedule
/// the decoder keeps on more threads ([`Schedule`]). Exact: it depends on
/// the stream's bytes alone. Fails as [`decompress`] does, and for a frame
/// the step measure has no value for (ulz and program frames).
pub fn decode_schedule(stream: &[u8], work: u64) -> Result<(Vec<u8>, Schedule), Error> {
    let archives = frames(stream)?;
    let mut wanted: std::collections::HashMap<[u8; 32], u64> = Default::default();
    for a in &archives {
        if let Some((label, n)) = archive::reference_target(a) {
            wanted.insert(label, n);
        }
    }
    let reads: Vec<Option<usize>> = archives.iter().map(|a| archive::window_len(a)).collect();
    // the frame each wanted run is first found in, and the run's bytes
    let mut held: std::collections::HashMap<[u8; 32], (usize, Vec<u8>)> = Default::default();
    let mut finish: Vec<u64> = Vec::with_capacity(archives.len());
    let mut window: Vec<Vec<u8>> = Vec::new();
    let mut out = Vec::new();
    let (mut total, mut information) = (0u64, 0u64);
    for (i, a) in archives.iter().enumerate() {
        let info = inspect(a)?;
        let steps = performance::frame_steps(&info).ok_or(Error::Decode)?;
        total = total.checked_add(steps).ok_or(Error::Decode)?;
        let starts = i == 0 || starts_block(reads[i], reads[i + 1..].iter().copied());
        let mut ready = if starts { 0 } else { finish[i - 1] };
        let output = match archive::reference_of(a) {
            Some(label) => {
                let (at, run) = held.get(&label).ok_or(Error::Decode)?;
                ready = ready.max(finish[*at]);
                match archive::reference_decode(a, run) {
                    Some(output) => output,
                    None => decompress_frame_with(a, &[run], work)?,
                }
            }
            None => {
                let view: Vec<&[u8]> = window.iter().map(Vec::as_slice).collect();
                let (output, bits, payloads) =
                    ncm::measure_information(|| decompress_frame_with(a, &view, work));
                let output = output?;
                match info.representation {
                    Representation::Raw => {
                        information += (8 << ncm::INFORMATION_BITS) * output.len() as u64
                    }
                    Representation::Ncm | Representation::NcmX86 => {
                        // every coded frame is decoded by the native ncm decoder
                        // (its single payload), so its information is counted
                        if payloads != 1 {
                            return Err(Error::Decode);
                        }
                        information += bits;
                    }
                    _ => return Err(Error::Decode),
                }
                output
            }
        };
        finish.push(ready + steps);
        let mut missing: std::collections::HashMap<u64, std::collections::HashSet<[u8; 32]>> =
            Default::default();
        for (label, n) in &wanted {
            if !held.contains_key(label) {
                missing.entry(*n).or_default().insert(*label);
            }
        }
        if !missing.is_empty() {
            for (label, run) in chunk::runs(&output, missing) {
                held.insert(label, (i, output[run].to_vec()));
            }
        }
        out.extend_from_slice(&output);
        window.push(output);
        if window.len() > REACH {
            window.remove(0);
        }
    }
    Ok((
        out,
        Schedule {
            total,
            chain: finish.iter().copied().max().unwrap_or(0),
            information,
            frames: archives.len() as u64,
        },
    ))
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
/// objects it holds when no run repeats (see the crate documentation): the
/// layout [`chunk::Framer`] keeps for such input.
#[cfg(test)]
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

/// The frames of everything `reader` yields, cut by `framer`, handed to `code`
/// about `batch` frames' worth of bytes at a time. `limit` is how many bytes `reader` must yield (objects of
/// known lengths) or `None` (a plain stream).
fn framed(
    mut framer: chunk::Framer,
    reader: impl Read,
    limit: Option<u64>,
    batch: usize,
    code: &mut dyn FnMut(Vec<chunk::Frame>) -> io::Result<()>,
) -> io::Result<()> {
    let mut reader = reader.take(limit.unwrap_or(u64::MAX));
    let mut read = 0u64;
    let mut ready: Vec<chunk::Frame> = Vec::new();
    let mut buf = Vec::new();
    loop {
        buf.clear();
        read_full(&mut reader, &mut buf, FRAME)?;
        read += buf.len() as u64;
        let end = buf.len() < FRAME;
        if end {
            if limit.is_some_and(|n| read < n) {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            ready.extend(framer.push(&buf));
            ready.extend(framer.finish());
        } else {
            ready.extend(framer.push(&buf));
        }
        // A batch holds `batch` frames' worth of bytes, so that short frames
        // (cut where a repeated run begins) do not leave cores idle.
        loop {
            let mut bytes = 0;
            let full = ready.iter().position(|f| {
                bytes += f.bytes.len();
                bytes >= batch * FRAME
            });
            let take = match full {
                Some(k) => k + 1,
                None if end && !ready.is_empty() => ready.len(),
                None => break,
            };
            let rest = ready.split_off(take);
            code(std::mem::replace(&mut ready, rest))?;
        }
        if end {
            return Ok(());
        }
    }
}

/// Code the frames `framer` cuts from `reader` onto `writer`, `threads` frames
/// at a time. A repeated run is a reference frame; any other frame reads the
/// frames before it in its [`BLOCK`] as its window.
fn compress_frames(
    framer: chunk::Framer,
    reader: impl Read,
    limit: Option<u64>,
    writer: &mut dyn Write,
    steps: u64,
    threads: usize,
) -> io::Result<Totals> {
    let threads = threads.max(1);
    let mut totals = Totals::default();
    // The frames before the batch that its first frames read.
    let mut before: Vec<Vec<u8>> = Vec::new();
    framed(framer, reader, limit, threads, &mut |batch| {
        let all: Vec<&[u8]> = before
            .iter()
            .map(Vec::as_slice)
            .chain(batch.iter().map(|f| f.bytes.as_slice()))
            .collect();
        let jobs: Vec<usize> = (before.len()..all.len())
            .filter(|&k| batch[k - before.len()].reference.is_none())
            .collect();
        // a frame codes its two strategies side by side when the batch leaves a core free
        let spare = 2 * jobs.len() <= threads;
        // the cores no frame's worker holds, lent to the encoders as they start
        let cores = ncm::Cores::new(threads.saturating_sub(jobs.len().max(1)));
        let mut coded = parallel(&jobs, threads, |&k| {
            let frame = &batch[k - before.len()];
            cores.enter(|| {
                compress_pieces(
                    all[k],
                    &frame.pieces,
                    &all[k - frame.window..k],
                    steps,
                    spare,
                )
            })
        })
        .into_iter();
        drop(all);
        for frame in &batch {
            let archive = match frame.reference {
                Some(label) => reference_or_raw(&frame.bytes, &label, &frame.pieces),
                None => coded.next().expect("one archive per coded frame")?,
            };
            writer.write_all(&archive)?;
            totals.read += frame.bytes.len() as u64;
            totals.written += archive.len() as u64;
            totals.frames += 1;
        }
        before.extend(batch.into_iter().map(|f| f.bytes));
        let keep = before.len().saturating_sub(REACH);
        before.drain(..keep);
        Ok(())
    })?;
    writer.flush()?;
    Ok(totals)
}

/// The reference frame for a repeated run, or its raw archive when that is
/// not larger (a run is never that short, but the choice stays exact).
fn reference_or_raw(bytes: &[u8], label: &[u8; 32], pieces: &[u64]) -> Vec<u8> {
    if archive::reference_length(pieces) < archive::raw_length(pieces) {
        archive::reference_archive(label, pieces)
    } else {
        archive::raw_objects(bytes, pieces)
    }
}

/// Compress everything `reader` yields into a uorc stream on `writer`, one
/// [`FRAME`] at a time with `threads` frames in flight. Memory stays bounded by
/// `threads` frames plus the window and the run index ([`chunk::INDEX_UNITS`])
/// whatever the input size. Every archive is written exactly as the model
/// writes it for its frame and window. The stream depends on the input alone,
/// not on `threads`.
pub fn compress_stream(
    reader: impl Read,
    mut writer: impl Write,
    steps: u64,
    threads: usize,
) -> io::Result<Totals> {
    compress_frames(
        chunk::Framer::plain(),
        reader,
        None,
        &mut writer,
        steps,
        threads,
    )
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
    let total = lengths
        .iter()
        .try_fold(0u64, |a, &n| a.checked_add(n))
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    compress_frames(
        chunk::Framer::objects(lengths),
        reader,
        Some(total),
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

/// A count of cores free for decoding coded frames (see [`decode_stream`]).
struct Cores {
    free: std::sync::Mutex<usize>,
    freed: std::sync::Condvar,
}

/// A core taken from [`Cores`], given back when dropped.
struct Core<'a>(&'a Cores);

impl Cores {
    fn new(n: usize) -> Self {
        Cores {
            free: std::sync::Mutex::new(n),
            freed: std::sync::Condvar::new(),
        }
    }

    fn take(&self) -> Core<'_> {
        let mut free = self
            .free
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while *free == 0 {
            free = self
                .freed
                .wait(free)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        *free -= 1;
        Core(self)
    }
}

impl Drop for Core<'_> {
    fn drop(&mut self) {
        *self
            .0
            .free
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
        self.0.freed.notify_one();
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
                    let len = output.len();
                    on_output(&lengths, output)?;
                    *sent -= 1;
                    *in_flight -= len;
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
    if threads <= 1 {
        // One core decodes in stream order with no worker thread, which is
        // also the only schedule a host without threads (WebAssembly) can run.
        let (mut totals, mut carry, mut window) = (Totals::default(), Vec::new(), Vec::new());
        while let Some(archive) = next_archive(&mut reader, &mut carry)? {
            totals.read += archive.len() as u64;
            on_archive(&archive)?;
            let lengths = object_lengths(&archive)?;
            let output = decode_into(&mut window, &archive, totals.frames as usize, store, work)?;
            totals.written += output.len() as u64;
            totals.frames += 1;
            on_output(&lengths, output)?;
        }
        if totals.frames == 0 {
            return Err(Error::Decode.into());
        }
        return Ok(totals);
    }
    // Output bytes of frames sent to the decoders and not yet handed on.
    let limit = threads * BLOCK * FRAME;
    // Cores decoding coded frames: a block whose reference frame waits for a
    // frame of another block holds none, so other blocks run meanwhile.
    let cores = &Cores::new(threads);
    // The same cores as the encoder's pool: a decoder holds one, and the
    // model states it warms (an encode without a coder) borrow the rest.
    let lend = &ncm::Cores::new(threads);
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
                while !end && ahead.len() < REACH {
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
                    while blocks.len() >= threads * BLOCK {
                        drain(&mut blocks, on_output, &mut totals, &mut in_flight, true)?;
                    }
                    let (frames, inbox) = channel::<(usize, Vec<u8>)>();
                    let (send, outputs) = channel();
                    scope.spawn(move || {
                        let mut window = Vec::new();
                        for (at, archive) in inbox {
                            let core = archive::reference_of(&archive)
                                .is_none()
                                .then(|| (cores.take(), lend.hold()));
                            let mut decode = || {
                                object_lengths(&archive).and_then(|lengths| {
                                    Ok((
                                        lengths,
                                        decode_into(&mut window, &archive, at, store, work)?,
                                    ))
                                })
                            };
                            let decoded = match core {
                                Some((core, held)) => {
                                    let decoded = held.run(decode);
                                    drop(core);
                                    decoded
                                }
                                None => decode(),
                            };
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
                let len = object_lengths(&archive).map_or(0, |l| l.iter().sum::<u64>() as usize);
                while in_flight > 0 && in_flight + len > limit {
                    drain(&mut blocks, on_output, &mut totals, &mut in_flight, true)?;
                }
                // A decoder that failed has sent its error, which drain reports.
                let _ = current.as_ref().map(|frames| frames.send((index, archive)));
                if let Some((_, sent)) = blocks.back_mut() {
                    *sent += 1;
                }
                in_flight += len;
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

/// A stream compressed on the caller's thread as its input arrives, for a
/// caller that holds a piece of input at a time (the WebAssembly client). The
/// archives it returns, written back to back, are the stream
/// [`compress_stream`] writes for the same input, however it is pieced.
pub struct FrameEncoder {
    framer: chunk::Framer,
    /// The frames before the next one that it may read.
    before: Vec<Vec<u8>>,
}

impl Default for FrameEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameEncoder {
    /// A new stream.
    pub fn new() -> Self {
        FrameEncoder {
            framer: chunk::Framer::plain(),
            before: Vec::new(),
        }
    }

    /// A new stream of ordered objects of `lengths`, whose bytes are pushed
    /// back to back (a file tree's [`tree::Layout`]): the archives are the
    /// stream [`compress_objects_stream`] writes for the same objects.
    pub fn objects(lengths: &[u64]) -> Self {
        FrameEncoder {
            framer: chunk::Framer::objects(lengths),
            before: Vec::new(),
        }
    }

    /// The next piece of input: returns the archives of the frames it
    /// completes, back to back (none until more than a frame is pending).
    pub fn push(&mut self, input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
        let planned = self.plan(input);
        code_all(planned, steps)
    }

    /// The end of input: the archives of the remaining frames.
    pub fn finish(&mut self, steps: u64) -> Result<Vec<u8>, Error> {
        let planned = self.plan_end();
        code_all(planned, steps)
    }

    /// [`FrameEncoder::push`] without the coding: the frames it completes,
    /// each a finished repeat or a frame with the window it reads, so that a
    /// caller can code them on other cores ([`Planned::code`], or
    /// [`compress_strategy`] and [`choose`]) and write the archives in order.
    pub fn plan(&mut self, input: &[u8]) -> Vec<Planned> {
        let frames = self.framer.push(input);
        self.planned(frames)
    }

    /// [`FrameEncoder::finish`] without the coding (see [`FrameEncoder::plan`]).
    pub fn plan_end(&mut self) -> Vec<Planned> {
        let frames = self.framer.finish();
        self.planned(frames)
    }

    fn planned(&mut self, frames: Vec<chunk::Frame>) -> Vec<Planned> {
        let mut out = Vec::new();
        for frame in frames {
            out.push(match frame.reference {
                Some(label) => Planned::Done {
                    length: frame.bytes.len() as u64,
                    archive: reference_or_raw(&frame.bytes, &label, &frame.pieces),
                },
                None => Planned::Code {
                    bytes: frame.bytes.clone(),
                    pieces: frame.pieces,
                    window: self.before[self.before.len() - frame.window..].to_vec(),
                },
            });
            self.before.push(frame.bytes);
            if self.before.len() > REACH {
                self.before.remove(0);
            }
        }
        out
    }
}

fn code_all(planned: Vec<Planned>, steps: u64) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    for frame in planned {
        out.extend_from_slice(&frame.code(steps)?);
    }
    Ok(out)
}

/// A frame [`FrameEncoder::plan`] has cut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Planned {
    /// A repeat of an earlier run, already its archive.
    Done {
        /// The frame's input length.
        length: u64,
        /// Its archive.
        archive: Vec<u8>,
    },
    /// A frame still to code after the frames of its window (oldest first).
    Code {
        /// The frame's input bytes.
        bytes: Vec<u8>,
        /// The pieces of objects it holds.
        pieces: Vec<u64>,
        /// The frames it reads as history.
        window: Vec<Vec<u8>>,
    },
}

impl Planned {
    /// The frame's archive, the one [`FrameEncoder::push`] writes for it.
    pub fn code(&self, steps: u64) -> Result<Vec<u8>, Error> {
        match self {
            Planned::Done { archive, .. } => Ok(archive.clone()),
            Planned::Code {
                bytes,
                pieces,
                window,
            } => {
                let window: Vec<&[u8]> = window.iter().map(Vec::as_slice).collect();
                compress_pieces(bytes, pieces, &window, steps, true)
            }
        }
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

    /// With steps, native compress is the model's: its own portfolio archive
    /// and the exported search, the smaller kept.
    #[test]
    fn searched_archives_are_the_model_archives() {
        for (input, steps) in [(vec![0u8; 200], 10_000), (text(300, 4), 20_000)] {
            let model = on_reference_stack(|| reference_compress(&input, &[], steps));
            assert_eq!(compress_frame(&input, steps), model);
        }
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

    /// Streams the released v0.2.0, v0.3.0 and v0.4.0 binaries wrote for one input
    /// (an 8 MiB frame, the same frame again, then a 4 KiB frame) decode
    /// under the appended library: v0.2.0's ulz frame and ulz window frames;
    /// v0.3.0's ncm frame, its reference frame (labelled over the v0.3
    /// library) and an ncm window frame over the two frames before.
    #[test]
    fn released_streams_decode() {
        let frame: Vec<u8> = (0..8 << 20)
            .map(|i| ((((i >> 4) * 7 + (i >> 12)) & 0x3f) + 0x20) as u8)
            .collect();
        let input = [&frame[..], &frame[..], &frame[1000..5096]].concat();
        for stream in [
            &include_bytes!("../tests/compat/stream-v0.2.0.uorc")[..],
            include_bytes!("../tests/compat/stream-v0.3.0.uorc"),
            include_bytes!("../tests/compat/stream-v0.4.0.uorc"),
        ] {
            let mut out = Vec::new();
            decompress_stream(io::Cursor::new(stream), &mut out, DEFAULT_DECODE_WORK, 2)
                .expect("decodes");
            assert!(out == input);
        }
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
        let families: std::collections::HashSet<usize> =
            frames.iter().map(|f| portfolio::family(f)).collect();
        assert_eq!(families.len(), portfolio::FAMILIES);
        // each frame as its own stream, so every family is a frame of its own
        let run = |threads: usize| -> Vec<Vec<u8>> {
            frames
                .iter()
                .map(|f| {
                    let mut out = Vec::new();
                    compress_stream(&f[..], &mut out, 0, threads).expect("compresses");
                    out
                })
                .collect()
        };
        let one = run(1);
        for threads in [2, 3, 4] {
            assert!(run(threads) == one, "{threads} threads");
        }
        for (stream, frame) in one.iter().zip(&frames) {
            let mut back = Vec::new();
            decompress_stream(
                io::Cursor::new(&stream[..]),
                &mut back,
                DEFAULT_DECODE_WORK,
                2,
            )
            .expect("decodes");
            assert!(&back == frame);
        }
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
            assert_eq!(decompress_frame(&native, 0).as_ref(), Ok(&input));
        }
    }

    fn noise(n: usize, seed: u64) -> Vec<u8> {
        let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                (x >> 24) as u8
            })
            .collect()
    }

    /// UC-PERF-16: the window facets step on the chosen archive. A frame that
    /// repeats the oldest frame of its block is coded over every earlier
    /// frame (its entry names three) and is strictly smaller than the
    /// portfolio's archive over the last two; any frame is the portfolio's
    /// archive byte for byte or a strictly shorter one. Both decode.
    #[test]
    fn window_step_keeps_only_a_shorter_archive_uc_perf_16() {
        let a = include_bytes!("../../../README.md").to_vec();
        let (b, c) = (noise(30_000, 2), noise(30_000, 3));
        let window: [&[u8]; 3] = [&a, &b, &c];
        let deep = compress_frame_with(&a, &window, 0).expect("deep");
        let shallow = compress_frame_with(&a, &window[1..], 0).expect("shallow");
        assert_eq!(archive::window_len(&deep), Some(3));
        assert!(
            deep.len() < shallow.len(),
            "{} {}",
            deep.len(),
            shallow.len()
        );
        assert_eq!(decompress_frame_with(&deep, &window, 0).as_ref(), Ok(&a));
        // any other frame: the portfolio's archive, or a strictly shorter one
        let d: Vec<u8> = a.iter().rev().copied().collect();
        let kept = compress_frame_with(&d, &window, 0).expect("kept");
        let portfolio = compress_frame_with(&d, &window[1..], 0).expect("portfolio");
        assert!(kept == portfolio || kept.len() < portfolio.len());
        assert_eq!(decompress_frame_with(&kept, &window, 0).as_ref(), Ok(&d));
    }

    /// UC-PERF-17: the profiled window entry over k frames is one form for
    /// every k: for one and two frames it is the fixed entry byte for byte,
    /// and for more the decoder reads k from the entry, with or without the
    /// x86 filter.
    #[test]
    fn window_entry_is_one_form_for_every_depth_uc_perf_17() {
        for x86 in [false, true] {
            for k in 1..=2 {
                let fixed =
                    |c: archive::Codec| archive::window_archive(c, k, &[0; 32], &[5], &[1, 2, 3]);
                let p = ncm::FULL;
                let generic = archive::window_profile_template(k, x86);
                let built = fixed(archive::Codec::profiled(u64::from(p), x86)).expect("entry");
                let body_at = built.len() - archive::SEAL;
                let body = &built[..body_at];
                assert!(body.windows(generic.0.len()).any(|w| w == &generic.0[..]));
                assert!(body.ends_with(&generic.1), "x86 {x86} k {k}");
            }
        }
        let text = include_bytes!("../../../README.md");
        for k in 3..=6usize {
            let frames: Vec<Vec<u8>> = (0..k)
                .map(|i| text[i * 500..i * 500 + 3000].to_vec())
                .collect();
            let window: Vec<&[u8]> = frames.iter().map(Vec::as_slice).collect();
            let frame = &text[100..2600];
            for x86 in [false, true] {
                let codec = if x86 {
                    archive::Codec::NcmX86
                } else {
                    archive::Codec::Ncm
                };
                let history = window.concat();
                let (data, past) = if x86 {
                    (
                        x86::filter(frame, false, history.len() as u64).0,
                        x86::filter(&history, false, 0).0,
                    )
                } else {
                    (frame.to_vec(), history.clone())
                };
                let payload = ncm::encode_profile(&past, &data, ncm::FULL).expect("payload");
                let id = archive::window_identity(&window);
                let a = archive::window_archive_profile(
                    codec,
                    u64::from(ncm::FULL),
                    &window,
                    &id,
                    &[frame.len() as u64],
                    &payload,
                )
                .expect("archive");
                assert_eq!(archive::window_len(&a), Some(k));
                assert_eq!(archive::window_decode(&a, &window).as_deref(), Some(frame));
            }
        }
    }

    /// The exported model decodes every frame word the native client
    /// writes: recorded context table bits, recorded match table bits, and
    /// the 16-bit probability field, alone and together (v0.5.0's model
    /// refused any profile word of 2^20 or more, so every frame with recorded
    /// table bits, which the table facet writes from about 8 KiB on), and the
    /// client's own frame of the README.
    #[test]
    fn model_decodes_every_frame_word_native_writes() {
        let text: Vec<u8> = (0..3000u32)
            .flat_map(|i| format!("row {} of {}\n", i % 97, i / 7).into_bytes())
            .take(12_000)
            .collect();
        let n = text.len() as u64;
        let tables = ncm::with_table_bits(ncm::FULL, 22);
        let both = ncm::with_match_bits(tables, 20);
        let mut words = Vec::new();
        for profile in [ncm::FULL, tables, both] {
            for bits in [12, 16] {
                words.push((profile, bits));
            }
        }
        for (profile, bits) in words {
            let plan = ncm::streams::Plan {
                bits,
                ..ncm::streams::Plan::ONE
            };
            let word = ncm::streams::word(profile, plan).expect("word");
            let payload =
                ncm::encode_bits_below(&[], &text, profile, bits, usize::MAX).expect("payload");
            let a = archive::linked_archive_profile(archive::Codec::Ncm, word, &[n], &payload)
                .expect("archive");
            assert_eq!(
                decompress_frame(&a, DEFAULT_DECODE_WORK).expect("native"),
                text
            );
            assert_eq!(
                model_decompress_frame(&a).expect("model"),
                text,
                "model decode of word {word:#x}"
            );
        }
        // and the client's own frame
        let readme = include_bytes!("../../../README.md");
        let own = compress_frame(readme, 0).expect("compress");
        assert_eq!(model_decompress_frame(&own).expect("model"), readme);
    }

    /// UC-PERF-08: the shipped compress keeps exactly `performance::chosen`
    /// over the raw archive and the trials it coded: least in bytes, then in
    /// decode steps.
    #[test]
    fn compress_is_the_chosen_trial_uc_perf_08() {
        let text = include_bytes!("../../../README.md");
        let mut code = Vec::new();
        for k in 0..3000u32 {
            code.extend_from_slice(&[0xE8, (k % 251) as u8, 0, 0, 0, 0x48, 0x89, (k % 7) as u8]);
        }
        let frames: [&[u8]; 4] = [&text[..2000], &text[..], &code, &[7; 5000]];
        for frame in frames {
            let n = frame.len() as u64;
            let lengths = [n];
            let raw = performance::Trial {
                bytes: archive::raw_length(&lengths),
                steps: n,
            };
            let plan = portfolio::plan(frame);
            let trials: Vec<Option<performance::Trial>> =
                portfolio_trials(frame, &lengths, &[], &plan, true)
                    .iter()
                    .map(|t| {
                        t.as_ref().map(|(a, s)| performance::Trial {
                            bytes: a.len() as u64,
                            steps: *s,
                        })
                    })
                    .collect();
            let want = performance::chosen(raw, &trials);
            let got = compress_frame(frame, 0).expect("compress");
            // the chosen trial, or the same coding at 16-bit probabilities
            // when that is shorter (`step_streams`; the steps are the same)
            assert!(got.len() as u64 <= want.bytes);
            let info = frame::inspect(&got).expect("inspect");
            let steps = match info.state {
                Some(_) => performance::trial_steps(
                    info.ncm_profile.expect("profile"),
                    n + states::state(archive::state_frame(&got).expect("state frame").state)
                        .expect("named")
                        .len() as u64,
                ),
                None => performance::frame_steps(&info).expect("steps"),
            };
            assert_eq!(steps, want.steps);
        }
    }

    /// UC-PERF-07: a frame the x86 filter leaves unchanged (zero hits) codes
    /// no x86 candidate, with or without a state; every other strategy codes.
    #[test]
    fn zero_hits_code_no_x86_candidate_uc_perf_07() {
        let frame = &include_bytes!("../../../README.md")[..3000];
        assert_eq!(x86::filter(frame, false, 0).1, 0);
        let plan: Vec<portfolio::Strategy> = portfolio::STRATEGIES
            .iter()
            .flat_map(|&s| [s, portfolio::Strategy { x86: true, ..s }])
            .collect();
        let trials = portfolio_trials(frame, &[frame.len() as u64], &[], &plan, true);
        for (s, t) in plan.iter().zip(&trials) {
            assert_eq!(t.is_none(), s.x86, "{s:?}");
        }
    }

    /// Every strategy of the portfolio, κ-named state or not, codes a frame
    /// whose archive decodes to the frame, natively and by the exported
    /// model: a state changes the size of an archive, never its meaning. A
    /// state frame names its state by the identity of the basis [library,
    /// S], reads no window, and any change to it is refused.
    #[test]
    fn states_change_size_never_meaning() {
        let input = text(3000, 7);
        let lengths = [input.len() as u64];
        let mut least_state = usize::MAX;
        for (k, st) in portfolio::STRATEGIES.iter().enumerate() {
            let Some(s) = states::state(st.state) else {
                continue;
            };
            let id = states::identity(st.state).expect("named");
            let payload = ncm::encode_warm(s, &input, st.profile).expect("codes");
            let a =
                archive::state_archive(st.x86, st.profile, &id, &lengths, &payload).expect("frame");
            assert_eq!(archive::basis_of(&a), Some(id), "strategy {k}");
            least_state = least_state.min(a.len());
            assert_eq!(archive::window_len(&a), Some(0));
            assert_eq!(decompress_frame(&a, 0).as_ref(), Ok(&input), "strategy {k}");
            let model = on_reference_stack(|| reference_decompress(&a, &[], DEFAULT_DECODE_WORK));
            assert_eq!(model.as_ref(), Ok(&input), "strategy {k} by the model");
            let mut bad = a.clone();
            let at = bad.len() - archive::SEAL - 1;
            bad[at] ^= 1;
            assert!(decompress_frame(&bad, 0).is_err());
        }
        // the kept archive is no larger than any state frame (that a state
        // makes small text smaller is checked on the shipped states, in
        // tests/shipped_states.rs)
        let a = compress_frame(&input, 0).expect("codes");
        assert!(a.len() <= least_state, "{} vs {least_state}", a.len());
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
        // The reference semantics runs the kernel graph at seconds per output
        // byte (64 bytes: 265 s, 16 bytes: 97 s); 16 bytes keeps both an
        // accepted and a declined mutation within the rounds.
        let input = text(16, 11);
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
        // That the stream is the in-memory one for any thread count is
        // identical_frames_are_stored_once's check (window and reference
        // frames alike): coding these 16 MiB again checks nothing more.
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
        // Input without repeated runs keeps this layout under the framer.
        let noise = text(2 * FRAME + 10, 9)
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).to_le_bytes()[7])
            .collect::<Vec<u8>>();
        for lengths in [
            vec![],
            vec![0],
            vec![f],
            vec![2 * f + 1],
            vec![f, 0, 3],
            vec![5, f],
            vec![f, 0],
            vec![3, 0, 0],
            vec![f - 1, 1, 0, f, 2],
        ] {
            let total: u64 = lengths.iter().sum();
            let mut framer = chunk::Framer::objects(&lengths);
            let mut cut = framer.push(&noise[..total as usize]);
            cut.extend(framer.finish());
            let pieces: Vec<Vec<u64>> = cut.into_iter().map(|c| c.pieces).collect();
            assert_eq!(pieces, plan(&lengths), "{lengths:?}");
        }
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
        // One object is a plain stream: a header property, so a frame-sized
        // object checks nothing a small one does not (frame plans for one
        // object are frame_plans_continue_objects_across_frames).
        let one = text(5_000, 8);
        assert_eq!(
            compress_objects(&[&one], 0).expect("one"),
            compress(&one, 0).expect("plain")
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
        // Not repeated runs (those become reference frames): a byte differs
        // every few hundred.
        let mut input = block.repeat(3);
        for at in (FRAME..3 * FRAME).step_by(997) {
            input[at] ^= 1;
        }
        let stream = compress(&input, 0).expect("stream");
        let archives = frames(&stream).expect("frames");
        assert_eq!(archives.len(), 3);
        // The first frame reads no window: it is the block's archive alone.
        assert_eq!(archive::window_len(archives[0]), Some(0));
        let alone = archives[0].len();
        assert!(
            stream.len() < alone + alone / 16,
            "{} vs {alone}",
            stream.len()
        );
        // Every decode path over window frames is identical_frames_are_stored_once's
        // check; one decode here shows these matches resolve.
        assert!(decompress(&stream, DEFAULT_DECODE_WORK).expect("decodes") == input);
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
        let mut one_by_one: Vec<u8> = input
            .chunks(FRAME)
            .flat_map(|frame| encoder.push(frame, 0).expect("frame"))
            .collect();
        one_by_one.extend(encoder.finish(0).expect("end"));
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

    /// Content repeated at an unaligned offset beyond the window is a
    /// reference frame naming the run of units of the earlier frame it
    /// repeats, and the stream decodes.
    #[test]
    fn repeats_at_any_offset_are_stored_once() {
        let a = text(3 << 20, 101);
        let gap = text(3 * FRAME + 777, 102);
        let input = [&a[..], &gap, &a, b"end"].concat();
        let stream = compress(&input, 0).expect("stream");
        let infos: Vec<Info> = frames(&stream)
            .expect("frames")
            .iter()
            .map(|f| inspect(f).expect("inspect"))
            .collect();
        let repeated: u64 = infos
            .iter()
            .filter(|i| i.representation == Representation::Reference)
            .map(|i| i.output_len)
            .sum();
        assert!(
            repeated + 2 * chunk::MAX_UNIT as u64 >= a.len() as u64,
            "{repeated} of {}",
            a.len()
        );
        let mut back = Vec::new();
        decompress_stream(
            io::Cursor::new(&stream[..]),
            &mut back,
            DEFAULT_DECODE_WORK,
            default_threads(),
        )
        .expect("streams");
        assert!(back == input);
    }

    /// The decoder finds a run a reference names anywhere inside an earlier
    /// frame, on every decode path, including a v0.3 reference to a whole
    /// frame in the same stream.
    #[test]
    fn references_name_runs_inside_frames() {
        let t: Vec<Vec<u8>> = (0..3).map(|i| text(600_000, 110 + i)).collect();
        let ends = chunk::units(&t[1]);
        let run = &t[1][ends[3]..ends[20]];
        let mut archives: Vec<Vec<u8>> = t.iter().map(|f| archive::raw_archive(f)).collect();
        for named in [run, &t[0][..], run] {
            archives.push(archive::reference_archive(
                &archive::frame_label(named),
                &[named.len() as u64],
            ));
        }
        let input = [&t.concat()[..], run, &t[0], run].concat();
        let stream = archives.concat();
        for threads in [1, 4] {
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
        verify(&stream, Some(&input), DEFAULT_DECODE_WORK).expect("verifies");
        let mut decoder = FrameDecoder::new(DEFAULT_DECODE_WORK);
        archives.iter().for_each(|a| decoder.note(a));
        let back: Vec<u8> = archives
            .iter()
            .flat_map(|a| decoder.push(a).expect("decodes"))
            .collect();
        assert!(back == input);
        // A run that is not a run of units of one earlier frame is not found.
        let across = [&t[0][t[0].len() - 5_000..], &t[1][..5_000]].concat();
        let orphan = [
            archives[0].clone(),
            archives[1].clone(),
            archive::reference_archive(&archive::frame_label(&across), &[10_000]),
        ]
        .concat();
        assert_eq!(decompress(&orphan, DEFAULT_DECODE_WORK), Err(Error::Decode));
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

    #[test]
    fn serial_bytes_are_unchanged_by_the_past_rule() {
        // The match models reach only positions before the one coded
        // (`Uorc.Performance::matchFromPast`): a stream may find a later
        // stream's position in the shared match table and ignores it. With one
        // stream every position a match table holds is in the past, so the
        // frame's coder writes the exported model's bytes, whose frame coder
        // has no such test, alone and after a window.
        let base = text(1500, 41);
        let mut input = base.clone();
        for k in 0..6 {
            input.extend_from_slice(&base[k * 100..k * 100 + 400]);
        }
        let native = compress_frame(&input, 0).expect("native");
        let model = on_reference_stack(|| reference_compress(&input, &[], 0)).expect("model");
        assert_eq!(native, model);
        let (a, b) = input.split_at(1500);
        let native = compress_frame_with(b, &[a], 0).expect("native");
        let model = on_reference_stack(|| reference_compress(b, &[a], 0)).expect("model");
        assert_eq!(native, model);
    }

    #[test]
    fn stream_frames_round_trip_natively() {
        let text: Vec<u8> = (0..40_000u32)
            .flat_map(|i| format!("line {} of {}\n", i % 977, i / 13).into_bytes())
            .take(48_000)
            .collect();
        let one = compress_frame(&text, 0).expect("frame");
        let coded = compress_frame_for(&text, &[], 4);
        assert_ne!(coded, one, "streams name another frame word");
        assert_eq!(
            decompress_frame(&coded, DEFAULT_DECODE_WORK).expect("decodes"),
            text
        );
        // the default host runs one thread: the frame compress_frame writes
        assert_eq!(compress_frame_for(&text, &[], 1), one);
        // a window frame of streams reads its window
        let (a, b) = text.split_at(20_000);
        let w = compress_frame_for(b, &[a], 4);
        assert_eq!(
            decompress_frame_with(&w, &[a], DEFAULT_DECODE_WORK).expect("decodes"),
            b
        );
    }

    /// A coded payload is the information its model assigns, rounded up to
    /// whole bytes, plus the coder's 4-byte length and up to its final byte
    /// or two; a one-frame stream's chain is all its steps.
    #[test]
    fn the_schedule_counts_information_and_the_chain() {
        let data = text(60_000, 3);
        for profile in [ncm::FULL, ncm::V03] {
            let payload = ncm::encode_profile(&[], &data, profile).expect("payload");
            let (back, bits, payloads) =
                ncm::measure_information(|| ncm::decode_profile(&[], &payload, u64::MAX, profile));
            assert_eq!((back.as_deref(), payloads), (Some(&data[..]), 1));
            let whole = bits.div_ceil(8 << ncm::INFORMATION_BITS) as usize;
            assert!(
                (4..=6).contains(&(payload.len() as i64 - whole as i64)),
                "{profile:#x} {} {whole}",
                payload.len()
            );
        }
        // outside a measurement nothing is counted
        assert_eq!(ncm::measure_information(|| ()).1, 0);
        let one = compress(&data, 0).expect("compress");
        let (back, s) = decode_schedule(&one, u64::MAX).expect("schedule");
        assert_eq!((back, s.frames, s.chain), (data.clone(), 1, s.total));
        assert!(s.information.div_ceil(8 << ncm::INFORMATION_BITS) < one.len() as u64);
    }
}
