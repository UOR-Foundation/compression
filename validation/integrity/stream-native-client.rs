// Nonproduction client for the exact diagnostic artifact in stream-evidence.json.
// Function/ADT numbers bind that artifact only; this file implements no hash.
use std::io::{self, BufRead};
use uorc_stream_max_probe as g;
fn bytes(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn good<T, E>(r: g::R<Result<T, E>>) -> T {
    match r {
        Ok(Ok(v)) => v,
        _ => panic!("generated source error"),
    }
}
fn sha_pending(h: &g::Adt5) -> bool {
    matches!(h, g::Adt5::C0(_, _, _, Some(g::Adt7::C0(_, _, _, 64)), _))
}
fn pending(s: &g::Adt2) -> bool {
    let g::Adt2::C0(_, stage, _) = s;
    match stage {
        g::Adt3::C0(g::Adt4::C0(h, _, _)) | g::Adt3::C2(g::Adt4::C0(h, _, _), _) => sha_pending(h),
        g::Adt3::C1(h) | g::Adt3::C3(h, _) => sha_pending(h),
        _ => false,
    }
}
fn drive(limit: u64, s: g::Adt2, input: &[u8]) -> (g::Adt2, usize) {
    let mut job = good(g::f9(limit, s, g::Bytes::lit(input)));
    let mut calls = 0;
    for _ in 0..600 * input.len() + 1200 {
        let g::Adt9::C0(ref state, _, _) = job;
        if pending(state) {
            calls += 1;
        }
        match good(g::f16(limit, job)) {
            g::Adt10::C0(next) => job = next,
            g::Adt10::C1(done) => return (done, calls),
        }
    }
    panic!("source step allowance");
}
fn finish(limit: u64, s: g::Adt2) -> (g::Bytes, usize) {
    let mut state = good(g::f5(limit, s));
    let mut calls = 0;
    for _ in 0..1200 {
        if let g::Adt2::C0(_, g::Adt3::C4(bytes), true) = &state {
            return (bytes.clone(), calls);
        }
        if pending(&state) {
            calls += 1;
        }
        state = good(g::f12(limit, state));
    }
    panic!("source finalization allowance");
}
fn protocols() {
    assert!(matches!(g::f3(59), Ok(Err(g::Adt0::C2))));
    assert!(matches!(g::f3(g::f2() + 1), Ok(Err(g::Adt0::C2))));
    let canceled = g::Adt2::C0(true, g::Adt3::C5, false);
    assert!(matches!(g::f5(96, canceled.clone()), Ok(Err(g::Adt0::C8))));
    assert!(matches!(g::f12(96, canceled.clone()), Ok(Err(g::Adt0::C8))));
    assert!(matches!(
        g::f9(96, canceled, g::Bytes::lit(&[])),
        Ok(Err(g::Adt0::C8))
    ));
    let initial = good(g::f3(96));
    assert!(matches!(
        g::f6(0, 96, initial.clone()),
        Ok(Err(g::Adt0::C7))
    ));
    let g::Adt9::C0(s, b, _) = good(g::f9(96, initial.clone(), g::Bytes::lit(&[])));
    assert!(matches!(
        g::f16(96, g::Adt9::C0(s, b, 1)),
        Ok(Err(g::Adt0::C3))
    ));
    let (ready, _) = drive(96, initial, &[]);
    assert!(matches!(g::f12(96, ready.clone()), Ok(Err(g::Adt0::C4))));
    assert!(matches!(
        g::f6(1200, 96, ready.clone()),
        Ok(Err(g::Adt0::C6))
    ));
    let closing = good(g::f5(96, ready));
    assert!(matches!(
        g::f9(96, closing, g::Bytes::lit(&[])),
        Ok(Err(g::Adt0::C5))
    ));
    assert_eq!(
        g::f38(true, g::f2(), false).unwrap().octets(),
        bytes("001fffffffffffffdb")
    );
    assert_eq!(
        g::f38(false, g::f2(), false).unwrap().octets(),
        bytes("001fffffffffffffbb")
    );
    assert_eq!(
        g::f53(2305843009213693951).unwrap().octets(),
        bytes("1fffffffffffffff")
    );
}
fn main() {
    protocols();
    let mut seen = std::collections::BTreeSet::new();
    let mut cases = 0;
    let mut partitions = 0;
    let mut frames = 0;
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let f: Vec<_> = line.split('\t').collect();
        assert!(seen.insert(f[0].to_string()), "duplicate ID");
        if f[0].starts_with("frame_") {
            let first = f[1] == "true";
            let count = f[2].parse::<u64>().unwrap();
            let terminal = f[3] == "true";
            assert_eq!(
                g::f38(first, count, terminal).unwrap().octets(),
                bytes(f[4]),
                "{}",
                f[0]
            );
            frames += 1;
            continue;
        }
        let limit = f[1].parse::<u64>().unwrap();
        let n = f[2].parse::<usize>().unwrap();
        let expected = bytes(f[3]);
        let count = f[4].parse::<usize>().unwrap();
        let input: Vec<_> = (0..n).map(|i| i as u8).collect();
        let (updated, used) = drive(limit, good(g::f3(limit)), &input);
        let (observed, last) = finish(limit, updated);
        assert_eq!(observed.octets(), expected, "{}", f[0]);
        assert_eq!(used + last, count, "{} actual compression calls", f[0]);
        if limit == g::f2() {
            assert_eq!(
                good(g::run(g::Bytes::lit(&input))).octets(),
                expected,
                "{} production root",
                f[0]
            );
        }
        for width in [1.max(n / 257), 7.max(n / 17), 64.max(n / 3)] {
            let mut state = good(g::f3(limit));
            let mut used = 0;
            for chunk in input.chunks(width) {
                let (next, c) = drive(limit, state, chunk);
                state = next;
                used += c;
            }
            let (out, last) = finish(limit, state);
            assert_eq!(out.octets(), expected, "{} partition", f[0]);
            assert_eq!(used + last, count, "{} partition calls", f[0]);
            partitions += 1;
        }
        cases += 1;
    }
    assert_eq!((cases, partitions, frames), (94, 282, 189));
    println!("94 native stream fixtures; 282 partitions; 189 frame controls; actual compression counts and typed errors passed");
}
