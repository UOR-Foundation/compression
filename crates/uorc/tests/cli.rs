//! End-to-end tests of the `uorc` binary: commands, exit codes and safe output.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn uorc(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uorc"))
        .args(args)
        .output()
        .expect("run uorc")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("exit code")
}

/// A fresh scratch directory per test under the target directory.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn s(path: &Path) -> &str {
    path.to_str().expect("utf-8 path")
}

/// Every entry in `dir` (temporary files would show up here).
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("read dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

const TEXT: &[u8] = b"the quick brown fox jumps over the lazy dog. the quick brown fox. ";

#[test]
fn existing_output_is_refused_and_force_replaces_it() {
    let dir = scratch("refuse");
    let input = dir.join("a.txt");
    let archive = dir.join("a.txt.uorc");
    fs::write(&input, TEXT).unwrap();
    fs::write(&archive, b"keep me").unwrap();

    let refused = uorc(&["compress", s(&input)]);
    assert_eq!(code(&refused), 3, "{refused:?}");
    assert_eq!(fs::read(&archive).unwrap(), b"keep me");

    let forced = uorc(&["compress", "--force", s(&input)]);
    assert_eq!(code(&forced), 0, "{forced:?}");
    assert_ne!(fs::read(&archive).unwrap(), b"keep me");
    assert_eq!(entries(&dir), ["a.txt", "a.txt.uorc"]);

    let restored = dir.join("b.txt");
    assert_eq!(
        code(&uorc(&["decompress", s(&archive), "-o", s(&restored)])),
        0
    );
    assert_eq!(fs::read(&restored).unwrap(), TEXT);
}

#[test]
fn failed_run_leaves_no_output() {
    let dir = scratch("failed");
    let bad = dir.join("bad.uorc");
    fs::write(&bad, b"UORC not really").unwrap();
    let failed = uorc(&["decompress", s(&bad)]);
    assert_eq!(code(&failed), 4, "{failed:?}");
    assert_eq!(entries(&dir), ["bad.uorc"]);
}

#[test]
fn inspect_reads_the_header_without_decoding() {
    let dir = scratch("inspect");
    let input = dir.join("t.txt");
    let text: Vec<u8> = TEXT.iter().copied().cycle().take(20_000).collect();
    fs::write(&input, &text).unwrap();
    assert_eq!(code(&uorc(&["compress", s(&input)])), 0);
    let archive = dir.join("t.txt.uorc");

    let shown = uorc(&["inspect", "--json", s(&archive)]);
    assert_eq!(code(&shown), 0, "{shown:?}");
    let json = String::from_utf8(shown.stdout).unwrap();
    assert!(json.contains("\"representation\":\"ulz-kernel\""), "{json}");
    assert!(json.contains("\"object_lengths\":[20000]"), "{json}");
    assert!(json.contains("\"integrity_ok\":true"), "{json}");

    let mut bytes = fs::read(&archive).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    let corrupt = dir.join("corrupt.uorc");
    fs::write(&corrupt, &bytes).unwrap();
    assert_eq!(code(&uorc(&["inspect", s(&corrupt)])), 4);
    assert_eq!(code(&uorc(&["inspect", s(&input)])), 4);
}

#[test]
fn verify_checks_integrity_and_the_original() {
    let dir = scratch("verify");
    let input = dir.join("v.bin");
    let data: Vec<u8> = (0..=255u8).cycle().take(3000).collect();
    fs::write(&input, &data).unwrap();
    assert_eq!(code(&uorc(&["compress", s(&input)])), 0);
    let archive = dir.join("v.bin.uorc");

    assert_eq!(code(&uorc(&["verify", s(&archive)])), 0);
    assert_eq!(code(&uorc(&["verify", s(&archive), s(&input)])), 0);

    let other = dir.join("other.bin");
    fs::write(&other, &data[1..]).unwrap();
    assert_eq!(code(&uorc(&["verify", s(&archive), s(&other)])), 7);
    assert_eq!(code(&uorc(&["verify", s(&input)])), 4);
    assert_eq!(entries(&dir), ["other.bin", "v.bin", "v.bin.uorc"]);
}

#[test]
fn usage_and_io_errors_have_their_own_codes() {
    assert_eq!(code(&uorc(&["frobnicate", "x"])), 2);
    assert_eq!(code(&uorc(&["compress", "--steps", "many", "x"])), 2);
    assert_eq!(code(&uorc(&["compress", "/nonexistent/uorc/input"])), 3);
}

/// Every file under `dir`, as (relative path, bytes), sorted.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let rel = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                out.push((format!("{rel}/"), Vec::new()));
                walk(base, &path, out);
            } else {
                out.push((rel, fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn directories_round_trip_with_empty_files_binary_and_nesting() {
    let dir = scratch("tree");
    let src = dir.join("src");
    fs::create_dir_all(src.join("a/b/c")).unwrap();
    fs::create_dir_all(src.join("empty-dir")).unwrap();
    fs::write(src.join("a/b/c/deep.txt"), TEXT.repeat(50)).unwrap();
    fs::write(src.join("a/empty"), b"").unwrap();
    fs::write(
        src.join("bin"),
        (0..=255u8).cycle().take(5000).collect::<Vec<_>>(),
    )
    .unwrap();
    fs::write(src.join("a/copy.txt"), TEXT.repeat(50)).unwrap();

    let archive = dir.join("src.uorc");
    let packed = uorc(&["compress", s(&src)]);
    assert_eq!(code(&packed), 0, "{packed:?}");
    let shown = String::from_utf8(uorc(&["inspect", "--json", s(&archive)]).stdout).unwrap();
    assert!(shown.contains("\"objects\":5"), "{shown}"); // manifest + 4 files

    let out = dir.join("out");
    let unpacked = uorc(&["decompress", s(&archive), "-o", s(&out)]);
    assert_eq!(code(&unpacked), 0, "{unpacked:?}");
    assert_eq!(tree(&out.join("src")), tree(&src));

    // Restoring again is refused; --resume finishes a partial restore.
    assert_eq!(code(&uorc(&["decompress", s(&archive), "-o", s(&out)])), 3);
    fs::remove_file(out.join("src/bin")).unwrap();
    fs::write(out.join("src/a/empty"), b"stale").unwrap();
    let resumed = uorc(&["decompress", "--resume", s(&archive), "-o", s(&out)]);
    assert_eq!(code(&resumed), 0, "{resumed:?}");
    assert_eq!(tree(&out.join("src")), tree(&src));
}

#[test]
fn several_files_and_an_empty_directory() {
    let dir = scratch("several");
    fs::write(dir.join("one"), b"1").unwrap();
    fs::write(dir.join("two"), b"").unwrap();
    fs::create_dir(dir.join("nothing")).unwrap();
    let archive = dir.join("all.uorc");
    let packed = uorc(&[
        "compress",
        s(&dir.join("one")),
        s(&dir.join("two")),
        s(&dir.join("nothing")),
        "-o",
        s(&archive),
    ]);
    assert_eq!(code(&packed), 0, "{packed:?}");
    let out = dir.join("out");
    assert_eq!(code(&uorc(&["decompress", s(&archive), "-o", s(&out)])), 0);
    assert_eq!(fs::read(out.join("one")).unwrap(), b"1");
    assert_eq!(fs::read(out.join("two")).unwrap(), b"");
    assert!(out.join("nothing").is_dir());
    assert_eq!(
        code(&uorc(&[
            "compress",
            s(&dir.join("one")),
            s(&dir.join("two"))
        ])),
        2
    );
}
