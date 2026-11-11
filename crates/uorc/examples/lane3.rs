//! scratch probe: contiguous lanes; each lane sees its own stream fully and
//! every other lane's finished epochs (ideal state bus with one epoch of lag).
use std::sync::atomic::{AtomicUsize, Ordering};
use uorc::ncm;
fn size(d: &[u8]) -> usize {
    if d.is_empty() {
        return 5;
    }
    ncm::encode_profile(&[], d, ncm::FULL)
        .map(|v| v.len())
        .unwrap()
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse().unwrap();
    let data = &data[..n];
    let t: usize = a[3].parse().unwrap();
    let e: usize = a[4].parse().unwrap();
    let share: bool = a[5] == "bus";
    let r = n / t;
    let mut jobs = Vec::new();
    for k in 0..t {
        let mut o = 0;
        while o < r {
            jobs.push((k, o));
            o += e;
        }
    }
    let next = AtomicUsize::new(0);
    let total = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..4 {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= jobs.len() {
                    break;
                }
                let (k, o) = jobs[i];
                let mut buf = Vec::new();
                if share {
                    for j in 0..t {
                        if j != k {
                            buf.extend_from_slice(&data[j * r..j * r + o.min(r)]);
                        }
                    }
                }
                buf.extend_from_slice(&data[k * r..k * r + o]);
                let base = size(&buf);
                buf.extend_from_slice(&data[k * r + o..k * r + (o + e).min(r)]);
                total.fetch_add(size(&buf) - base, Ordering::Relaxed);
            });
        }
    });
    let serial = size(&data[..r * t]);
    let x = total.load(Ordering::Relaxed);
    println!(
        "n={} lanes={} epoch={} {} serial={} laned={} ({:+.2}%)",
        n,
        t,
        e,
        a[5],
        serial,
        x,
        (x as f64 / serial as f64 - 1.0) * 100.0
    );
}
