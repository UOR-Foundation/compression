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

use std::fmt;
use std::io::{self, Read, Write};

pub mod archive;
pub mod ulz;

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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::Budget => "no checked archive was found within the step budget",
            Error::Overflow => "a natural-number operation overflowed 64 bits",
            Error::Decode => "input is not a decodable stream of self-contained UORC archives",
        })
    }
}

impl std::error::Error for Error {}

impl From<Error> for io::Error {
    fn from(e: Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, e)
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
    if steps > 0 {
        return on_reference_stack(|| reference_compress(frame, steps));
    }
    let n = frame.len() as u64;
    let payload = ulz::encode(frame);
    // The model writes the coder output into a table of the next power of two
    // at or above 2n + 64 bytes and falls back to raw if it would not fit.
    let fits = (payload.len() as u64 - 8) <= (2 * n + 64).next_power_of_two();
    if fits && ulz::decode(&payload, n).as_deref() == Some(frame) {
        let kernel = archive::program_archive(n, &payload);
        if (kernel.len() as u64) < archive::raw_length(n) {
            return Ok(kernel);
        }
    }
    Ok(archive::raw_archive(frame))
}

fn reference_compress(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    match uorc_compress::run(uorc_compress::Bytes::lit(input), steps) {
        Ok(Some(archive)) => Ok(archive.octets().to_vec()),
        Ok(None) => Err(Error::Budget),
        Err(_) => Err(Error::Overflow),
    }
}

/// Reconstruct the bytes of one self-contained archive. Archives the native
/// path vouches for (raw, and ulz kernel archives this client writes) decode
/// natively; every other archive is reconstructed by the exported reference
/// decoder within `work`.
pub fn decompress_frame(archive: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    if let Some(output) = archive::fast_decode(archive) {
        return Ok(output);
    }
    on_reference_stack(|| reference_decompress(archive, work))
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

/// Compress `input` into a uorc stream (one archive per [`FRAME`] bytes),
/// using every available core.
pub fn compress(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    let frames: Vec<&[u8]> = if input.is_empty() {
        vec![input]
    } else {
        input.chunks(FRAME).collect()
    };
    let mut out = Vec::new();
    for archive in parallel(&frames, default_threads(), |f| compress_frame(f, steps)) {
        out.extend_from_slice(&archive?);
    }
    Ok(out)
}

/// Split a uorc stream into its archives.
pub fn frames(stream: &[u8]) -> Result<Vec<&[u8]>, Error> {
    let mut rest = stream;
    let mut out = Vec::new();
    while !rest.is_empty() {
        let len = archive::frame_length(rest)
            .filter(|&l| l <= rest.len())
            .ok_or(Error::Decode)?;
        let (frame, tail) = rest.split_at(len);
        out.push(frame);
        rest = tail;
    }
    if out.is_empty() {
        return Err(Error::Decode);
    }
    Ok(out)
}

/// Reconstruct the original bytes of a uorc stream, using every available core.
pub fn decompress(stream: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    let frames = frames(stream)?;
    let mut out = Vec::new();
    for part in parallel(&frames, default_threads(), |a| decompress_frame(a, work)) {
        out.extend_from_slice(&part?);
    }
    Ok(out)
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
fn next_archive(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut frame = Vec::new();
    read_full(reader, &mut frame, archive::HEADER_MAX)?;
    if frame.is_empty() {
        return Ok(None);
    }
    let len = archive::frame_length(&frame).ok_or(Error::Decode)?;
    if len < frame.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "archive shorter than its header",
        ));
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
    loop {
        let mut batch = Vec::with_capacity(threads);
        while batch.len() < threads {
            match next_archive(&mut reader)? {
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
        inputs.extend([(64, 1), (777, 2), (4096, 3), (20_000, 4)].map(|(n, s)| text(n, s)));
        inputs.push((0..=255u8).cycle().take(5000).collect());
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
            let archive = archive::program_archive(input.len() as u64, &p);
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
            if accepted >= 2 && declined >= 2 {
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
}
