//! probe: sites one stream vs several streams changed per epoch, merge share,
//! and encode/decode wall time of a streams frame against the serial coder.
//! usage: probe_sites <file> <bytes> [serial] <streams:epoch:threads>...
use std::time::Instant;
use uorc::ncm::{self, streams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse::<usize>().unwrap().min(data.len());
    let data = &data[..n];
    let name = a[1].rsplit('/').next().unwrap();
    let mut specs = &a[3..];
    if specs.first().map(String::as_str) == Some("serial") {
        specs = &specs[1..];
        let t = Instant::now();
        let s = ncm::encode_profile(&[], data, ncm::FULL).unwrap();
        let enc = t.elapsed().as_secs_f64();
        let t = Instant::now();
        let back = ncm::decode_profile(&[], &s, u64::MAX, ncm::FULL).unwrap();
        let dec = t.elapsed().as_secs_f64();
        assert_eq!(back, data);
        println!(
            "{name} serial bytes={} enc={enc:.2}s dec={dec:.2}s",
            s.len()
        );
    }
    for spec in specs {
        let mut it = spec.split(':');
        let k: usize = it.next().unwrap().parse().unwrap();
        let e: usize = it.next().unwrap().parse().unwrap();
        let th: usize = it.next().unwrap_or("4").parse().unwrap();
        let plan = streams::Plan::of(k, e);
        let t = Instant::now();
        let (out, s) = streams::encode_sites(&[], data, ncm::FULL, plan, th).unwrap();
        let enc = t.elapsed().as_secs_f64();
        let t = Instant::now();
        let back = streams::decode(&[], &out, u64::MAX, ncm::FULL, plan, th, true).unwrap();
        let dec = t.elapsed().as_secs_f64();
        assert_eq!(back, data);
        let merge: f64 = s.secs.iter().sum();
        let steps = s.steps;
        let changes: u64 = (0..5).map(|i| s.one[i] + s.versions[i]).sum();
        let shared: u64 = s.versions.iter().sum();
        println!(
            "{name} k={k} e={e} threads={th} bytes={} enc={enc:.2}s dec={dec:.2}s merge={merge:.2}s ({:.1}% of enc) phases b/m/h/w={:.2}/{:.2}/{:.2}/{:.2} epochs={} steps={steps} changes={changes} shared-versions={shared}",
            out.len(),
            100.0 * merge / enc,
            s.secs[0],
            s.secs[1],
            s.secs[2],
            s.secs[3],
            s.epochs
        );
        for (i, kind) in streams::KINDS.iter().enumerate() {
            let tot = s.one[i] + s.shared[i];
            println!(
                "    {kind:14} one={:>11} shared={:>10} ({:5.2}% of sites) versions={:>11} ({:5.2}% of changes)",
                s.one[i],
                s.shared[i],
                100.0 * s.shared[i] as f64 / tot.max(1) as f64,
                s.versions[i],
                100.0 * s.versions[i] as f64 / (s.one[i] + s.versions[i]).max(1) as f64
            );
        }
    }
}
