//! The uorc client library.
//!
//! Every operation here is the certified Rust that `lexlean production build`
//! exports from `src/Uorc/Client.lex.tex` (see `scripts/client-export.mjs`).
//! This crate only moves bytes across that boundary and names its failures.

use std::fmt;

/// Search steps spent by [`compress`] when the caller names no budget.
pub const DEFAULT_STEPS: u64 = 10_000_000;

/// Decoder work allowed by [`decompress`]; decoding stops on its own when done.
pub const DEFAULT_DECODE_WORK: u64 = 1 << 40;

/// A failed client operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The search ended its step budget before any checked archive existed.
    Budget,
    /// A 64-bit natural-number operation in the exported code overflowed.
    Overflow,
    /// The bytes are not a self-contained UORC archive this client decodes.
    Decode,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::Budget => "no checked archive was found within the step budget",
            Error::Overflow => "a natural-number operation overflowed 64 bits",
            Error::Decode => "input is not a decodable self-contained UORC archive",
        })
    }
}

impl std::error::Error for Error {}

/// Stack reserved for the exported code on native targets. It recurses
/// structurally over archives and byte ropes, so depth grows with the input;
/// the reservation is virtual and only the pages actually touched are committed.
#[cfg(not(target_family = "wasm"))]
const STACK_BYTES: usize = 1 << 30;

#[cfg(not(target_family = "wasm"))]
fn on_stack<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("uorc".to_owned())
            .stack_size(STACK_BYTES)
            .spawn_scoped(scope, work)
            .expect("spawn the uorc worker thread")
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

#[cfg(target_family = "wasm")]
fn on_stack<T>(work: impl FnOnce() -> T) -> T {
    work()
}

/// Compress `input` into the smallest checked archive found within `steps`
/// search steps. The archive is verified against `input` before it is returned.
pub fn compress(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    on_stack(|| compress_here(input, steps))
}

fn compress_here(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    match uorc_compress::run(uorc_compress::Bytes::lit(input), steps) {
        Ok(Some(archive)) => Ok(archive.octets().to_vec()),
        Ok(None) => Err(Error::Budget),
        Err(_) => Err(Error::Overflow),
    }
}

/// Reconstruct the original bytes of a self-contained archive.
pub fn decompress(archive: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    on_stack(|| decompress_here(archive, work))
}

fn decompress_here(archive: &[u8], work: u64) -> Result<Vec<u8>, Error> {
    match uorc_decompress::run(uorc_decompress::Bytes::lit(archive), work) {
        Ok(Some(output)) => Ok(output.octets().to_vec()),
        Ok(None) => Err(Error::Decode),
        Err(_) => Err(Error::Overflow),
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
    fn zero_budget_is_a_budget_error() {
        assert_eq!(compress(b"abc", 0), Err(Error::Budget));
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
}
