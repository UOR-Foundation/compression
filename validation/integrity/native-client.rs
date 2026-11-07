// Validation client for the exact diagnostic package recorded in evidence.json.
// f5 is Sha256.hash in that certified program; this is not a production adapter.
// Input TSV is emitted by the repository importer and its source-owned chain fixtures.
use std::io::{self, BufRead};
use uorc_integrity_probe as g;
fn octets(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn good<E, T>(value: g::R<Result<T, E>>) -> T {
    match value {
        Ok(Ok(value)) => value,
        _ => panic!("generated model failed"),
    }
}
fn finish(state: g::Adt5) -> g::Bytes {
    good(g::f17(300, good(g::f16(state))))
}
fn main() {
    let mut cases = 0;
    let mut partitions = 0;
    let mut chain_cases = 0;
    let mut seen = std::collections::BTreeSet::new();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let fields: Vec<_> = line.split('\t').collect();
        let id = fields[0];
        assert!(seen.insert(id.to_string()), "duplicate case ID");
        let input = octets(fields[1]);
        let expected = octets(fields[2]);
        if id.starts_with("chain_") {
            assert_eq!(
                good(g::run(g::Bytes::lit(&input))).octets(),
                expected.as_slice(),
                "{id}: native chain mismatch"
            );
            let mut state = good(g::f1());
            for byte in &input {
                state = good(g::f2(state, g::Bytes::lit(&[*byte])));
            }
            let closing = good(g::f3(state));
            assert_eq!(
                good(g::f4(600, closing)).octets(),
                expected.as_slice(),
                "{id}: native chain partition mismatch"
            );
            chain_cases += 1;
            continue;
        }
        assert_eq!(
            good(g::f5(g::Bytes::lit(&input))).octets(),
            expected.as_slice(),
            "{id}: native known-answer mismatch"
        );
        for width in [1, 2, 7, 31, 64, 257] {
            let mut state = g::f15();
            for chunk in input.chunks(width) {
                state = good(g::f14(state, g::Bytes::lit(chunk)));
            }
            assert_eq!(
                finish(state).octets(),
                expected.as_slice(),
                "{id}: native partition mismatch"
            );
            partitions += 1;
        }
        if input.len() <= 64 {
            for split in 0..=input.len() {
                let state = good(g::f14(g::f15(), g::Bytes::lit(&input[..split])));
                let state = good(g::f14(state, g::Bytes::lit(&input[split..])));
                assert_eq!(
                    finish(state).octets(),
                    expected.as_slice(),
                    "{id}: native split mismatch"
                );
                partitions += 1;
            }
        }
        cases += 1;
    }
    assert!(matches!(g::f25(g::f15()), Ok(Err(g::Adt1::C4))));
    let g::Adt5::C0(words, _, _, _, _) = g::f15();
    let at_limit = g::Adt5::C0(
        words.clone(),
        g::Bytes::lit(&[0; 63]),
        2_305_843_009_213_693_951,
        None,
        g::Adt8::C0,
    );
    assert!(matches!(g::f35(at_limit, 0), Ok(Err(g::Adt1::C1))));
    let invalid_count = g::Adt5::C0(
        words,
        g::Bytes::lit(&[]),
        2_305_843_009_213_693_952,
        None,
        g::Adt8::C0,
    );
    assert!(matches!(g::f25(invalid_count), Ok(Err(g::Adt1::C0))));
    let begun = good(g::f16(g::f15()));
    assert!(matches!(g::f17(0, begun.clone()), Ok(Err(g::Adt1::C5))));
    assert!(matches!(
        g::f21(begun, g::Bytes::lit(&[])),
        Ok(Err(g::Adt1::C3))
    ));
    let mut state = good(g::f16(g::f15()));
    for _ in 0..300 {
        if matches!(&state, g::Adt5::C0(_, _, _, _, g::Adt8::C2)) {
            break;
        }
        state = good(g::f24(state));
    }
    let first = good(g::f25(state.clone()));
    let again = good(g::f25(good(g::f16(state.clone()))));
    assert_eq!(first.octets(), again.octets());
    let stepped = good(g::f25(good(g::f24(state))));
    assert_eq!(first.octets(), stepped.octets());
    assert_eq!(cases, 129);
    assert_eq!(partitions, 2919);
    assert_eq!(chain_cases, 7);
    println!("{cases} native CAVP cases; {partitions} native partitions; {chain_cases} native chain fixtures; native finalization/error cases passed");
}
