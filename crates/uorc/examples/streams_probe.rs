//! scratch probe: real streams-per-frame coder against the serial coder.
use uorc::ncm::{self, streams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse::<usize>().unwrap().min(data.len());
    let data = &data[..n];
    let serial: usize = a[3].parse().unwrap();
    let serial = if serial == 0 {
        ncm::encode_profile(&[], data, ncm::FULL).unwrap().len()
    } else {
        serial
    };
    for spec in &a[4..] {
        let mut it = spec.split(':');
        let k: usize = it.next().unwrap().parse().unwrap();
        let e: usize = it.next().unwrap().parse().unwrap();
        let w = match it.next().unwrap_or("sum") {
            "mean" => streams::Weights::Mean,
            "active" => streams::Weights::Active,
            _ => streams::Weights::Sum,
        };
        let plan = streams::Plan { streams: k, epoch: e, weights: w };
        let t = std::time::Instant::now();
        let out = streams::encode(&[], data, ncm::FULL, &plan).unwrap();
        let total: usize = out.iter().map(Vec::len).sum::<usize>() + 4 + 4 * k;
        println!(
            "{} n={} streams={} epoch={} {:?} serial={} streams={} ({:+.2}%) secs={:.1}",
            a[1].rsplit('/').next().unwrap(), n, k, e, w, serial, total,
            (total as f64 / serial as f64 - 1.0) * 100.0, t.elapsed().as_secs_f64()
        );
        println!("  changed={} conflicts={}", streams::CHANGED.swap(0, std::sync::atomic::Ordering::Relaxed), streams::CONFLICTS.swap(0, std::sync::atomic::Ordering::Relaxed));
    }
}
