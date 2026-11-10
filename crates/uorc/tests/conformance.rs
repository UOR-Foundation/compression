//! Conformance evidence for the shipped client (`src/Uorc/Registry.lex.tex`).
//!
//! Each test name ends in the row it discharges. Every check runs the client
//! that ships: the library, the `uorc` binary built from the same crate, and
//! the exported compress/decompress underneath both.

use std::io::Write;
use std::process::{Command, Stdio};

use uorc::archive::{chain, BASIS_ID};
use uorc::{
    compress_objects as compress_objects_at, decompress, decompress_objects, inspect, verify,
    Error, Representation, DEFAULT_DECODE_WORK, DEFAULT_STEPS,
};

fn compress(input: &[u8], steps: u64) -> Result<Vec<u8>, Error> {
    uorc::compress(input, steps)
}

fn compress_objects(objects: &[Vec<u8>], steps: u64) -> Result<Vec<u8>, Error> {
    compress_objects_at(objects, steps)
}

/// 200 zero bytes as a 64-byte program archive found by the exported search.
/// Its program is not the ulz kernel, so it decodes by the reference semantics.
const ZEROS_200: &str = "554f52430201000101c80114020100010101000100030200c8012401010001010b1420bec29aa29cd5b2136b604ed97ba986b96e842955bf7feb8e396389b2b5";

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
        .collect()
}

fn kernel_fixture() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/kernel-empty-padded.uorc"
    ))
    .expect("fixture")
}

/// Deterministic bytes with no structure (xorshift64*).
fn noise(len: usize, mut state: u64) -> Vec<u8> {
    (0..len)
        .map(|_| {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            (state.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 56) as u8
        })
        .collect()
}

/// Text, a structured binary record stream, noise, and edge cases.
fn corpus() -> Vec<(&'static str, Vec<u8>)> {
    let text: Vec<u8> = b"It was the best of times, it was the worst of times; "
        .iter()
        .copied()
        .cycle()
        .take(30_000)
        .collect();
    let records: Vec<u8> = (0u32..4_000)
        .flat_map(|i| {
            let mut r = i.to_le_bytes().to_vec();
            r.extend_from_slice(&(i * 7 % 251).to_be_bytes());
            r.extend_from_slice(b"\0\xff\xfe");
            r
        })
        .collect();
    vec![
        ("empty", Vec::new()),
        ("one byte", vec![0]),
        ("text", text),
        ("records", records),
        ("noise", noise(20_000, 0x9e37_79b9_7f4a_7c15)),
        (
            "invalid utf-8",
            vec![0xc3, 0x28, 0xa0, 0xa1, 0xe2, 0x28, 0xa1, 0xff, 0],
        ),
    ]
}

fn cli(args: &[&str], stdin: &[u8]) -> (bool, Vec<u8>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_uorc"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("run the uorc binary");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin)
        .expect("write stdin");
    let out = child.wait_with_output().expect("uorc exits");
    (out.status.success(), out.stdout)
}

/// The size of the raw archive of `len` bytes: what compress writes for noise
/// it cannot shrink.
fn raw_size(len: usize) -> usize {
    compress(&noise(len, 7), 0).expect("archive").len()
}

#[test]
fn real_archives_decode_to_every_original_byte_uc_real_01() {
    for (name, input) in corpus() {
        let archive = compress(&input, DEFAULT_STEPS).expect(name);
        let back = decompress(&archive, DEFAULT_DECODE_WORK).expect(name);
        assert_eq!(back, input, "{name}");
    }
    // Text takes a coded archive: a real program over the basis, not a copy.
    let (_, text) = &corpus()[2];
    assert!(compress(text, DEFAULT_STEPS).expect("text").len() < text.len() / 4);
}

#[test]
fn library_and_cli_share_one_semantics_uc_api_01() {
    for (name, input) in corpus() {
        let archive = compress(&input, DEFAULT_STEPS).expect(name);
        let (ok, cli_archive) = cli(&["compress", "-", "-o", "-"], &input);
        assert!(ok, "{name}: cli compress");
        assert_eq!(
            cli_archive, archive,
            "{name}: cli and library archives differ"
        );
        let (ok, cli_back) = cli(&["decompress", "-", "-o", "-"], &archive);
        assert!(ok, "{name}: cli decompress");
        assert_eq!(cli_back, input, "{name}");
    }
    let (ok, _) = cli(&["decompress", "-", "-o", "-"], b"not an archive");
    assert!(!ok, "a decode failure is a failed exit");

    // inspect reads the header without decoding; verify decodes and compares.
    let (_, text) = &corpus()[2];
    let archive = compress(text, DEFAULT_STEPS).expect("text");
    let info = inspect(&archive).expect("inspect");
    assert_eq!(info.output_len, text.len() as u64);
    assert_ne!(info.representation, Representation::Raw);
    assert!(info.integrity_ok);
    assert_eq!(
        verify(&archive, Some(text), DEFAULT_DECODE_WORK),
        Ok(vec![info])
    );
    let mut other = text.clone();
    other[0] ^= 1;
    assert_eq!(
        verify(&archive, Some(&other), DEFAULT_DECODE_WORK),
        Err(Error::Mismatch)
    );
    let (ok, json) = cli(&["inspect", "-"], &archive);
    assert!(ok, "cli inspect");
    assert!(String::from_utf8_lossy(&json).contains(info_name(&archive)));
}

fn info_name(archive: &[u8]) -> &'static str {
    inspect(archive).expect("inspect").representation.name()
}

#[test]
fn ordered_objects_keep_every_boundary_uc_obj_01() {
    let vectors: Vec<Vec<Vec<u8>>> = vec![
        vec![],
        vec![Vec::new()],
        vec![Vec::new(), Vec::new(), Vec::new()],
        vec![b"alpha".to_vec(), Vec::new(), vec![0, 0xff, 0], Vec::new()],
        vec![
            noise(3_000, 11),
            noise(3_000, 11),
            Vec::new(),
            b"tail".to_vec(),
        ],
    ];
    for objects in vectors {
        let archive = compress_objects(&objects, DEFAULT_STEPS).expect("archive");
        let lengths: Vec<u64> = objects.iter().map(|o| o.len() as u64).collect();
        if objects.len() != 1 {
            assert_eq!(inspect(&archive).expect("header").object_lengths, lengths);
        }
        assert_eq!(
            decompress_objects(&archive, DEFAULT_DECODE_WORK),
            Ok(objects.clone())
        );
        let mut longer = archive.clone();
        longer.push(0);
        assert!(decompress_objects(&longer, DEFAULT_DECODE_WORK).is_err());
        assert!(decompress_objects(&archive[..archive.len() - 1], DEFAULT_DECODE_WORK).is_err());
    }
}

#[test]
fn only_complete_canonical_archives_decode_uc_wire_01() {
    let (_, text) = &corpus()[2];
    for archive in [
        hex(ZEROS_200),
        compress(text, DEFAULT_STEPS).expect("coded archive"),
    ] {
        assert!(decompress(&archive, DEFAULT_DECODE_WORK).is_ok());
        for cut in [0, 1, 4, archive.len() / 2, archive.len() - 1] {
            assert_eq!(
                decompress(&archive[..cut], DEFAULT_DECODE_WORK),
                Err(Error::Decode),
                "truncated at {cut}"
            );
        }
        for extra in [&[0u8][..], b"\x01\x02\x03"] {
            let mut longer = archive.clone();
            longer.extend_from_slice(extra);
            assert_eq!(
                decompress(&longer, DEFAULT_DECODE_WORK),
                Err(Error::Decode),
                "trailing bytes"
            );
        }
        let mut version = archive.clone();
        version[4] ^= 0x80;
        assert_eq!(
            decompress(&version, DEFAULT_DECODE_WORK),
            Err(Error::Decode),
            "unsupported identity"
        );
    }
}

#[test]
fn every_flipped_bit_fails_the_integrity_trailer_uc_int_01() {
    for archive in [hex(ZEROS_200), compress(b"abc", 0).expect("raw archive")] {
        for index in 0..archive.len() {
            for bit in [0x01u8, 0x80] {
                let mut bad = archive.clone();
                bad[index] ^= bit;
                assert_eq!(
                    decompress(&bad, DEFAULT_DECODE_WORK),
                    Err(Error::Decode),
                    "byte {index} bit {bit:#x}"
                );
            }
        }
    }
}

#[test]
fn reference_semantics_decode_program_archives_uc_mach_01() {
    assert_eq!(
        decompress(&hex(ZEROS_200), DEFAULT_DECODE_WORK),
        Ok(vec![0u8; 200])
    );
    assert_eq!(
        decompress(&kernel_fixture(), DEFAULT_DECODE_WORK),
        Ok(Vec::new())
    );
}

#[test]
fn decode_work_is_charged_and_refused_whole_uc_mach_02() {
    let archive = hex(ZEROS_200);
    assert_eq!(decompress(&archive, 1), Err(Error::Decode));
    assert_eq!(
        decompress(&archive, DEFAULT_DECODE_WORK),
        Ok(vec![0u8; 200])
    );
}

#[test]
fn search_keeps_only_checked_candidates_uc_syn_01() {
    let input = vec![0u8; 200];
    let mut previous = usize::MAX;
    for steps in [0, 10_000, 200_000] {
        let archive = compress(&input, steps).expect("archive");
        assert_eq!(decompress(&archive, DEFAULT_DECODE_WORK), Ok(input.clone()));
        assert!(archive.len() <= previous, "{steps} steps grew the archive");
        previous = archive.len();
    }
}

#[test]
fn compress_returns_the_smallest_checked_candidate_uc_opt_01() {
    for (name, input) in corpus() {
        let archive = compress(&input, DEFAULT_STEPS).expect(name);
        assert!(
            archive.len() <= raw_size(input.len()),
            "{name}: larger than the raw archive"
        );
    }
    let (_, text) = &corpus()[2];
    let archive = compress(text, DEFAULT_STEPS).expect("text");
    assert!(archive.len() < raw_size(text.len()));
}

/// Offset of the basis identity inside a conditional archive's header.
fn basis_at(archive: &[u8]) -> usize {
    archive
        .windows(BASIS_ID.len())
        .position(|w| w == BASIS_ID)
        .expect("the archive names the standard basis")
}

#[test]
fn conditional_archives_bind_the_actual_basis_uc_bas_01() {
    let (_, text) = &corpus()[2];
    {
        let archive = uorc::compress(text, DEFAULT_STEPS).expect("archive");
        let info = inspect(&archive).expect("header");
        assert_eq!(info.basis, Some(BASIS_ID));
        assert_eq!(decompress(&archive, DEFAULT_DECODE_WORK).as_ref(), Ok(text));
        // Another basis identity, resealed so only the identity differs: the
        // archive is not accepted because a tag happens to be well formed.
        let at = basis_at(&archive);
        let mut other = archive.clone();
        other[at] ^= 1;
        let end = other.len() - 32;
        let seal = chain(&other[..end]);
        other[end..].copy_from_slice(&seal);
        assert!(
            inspect(&other)
                .expect("still a well-formed header")
                .integrity_ok
        );
        assert_eq!(decompress(&other, DEFAULT_DECODE_WORK), Err(Error::Decode));
    }
}
