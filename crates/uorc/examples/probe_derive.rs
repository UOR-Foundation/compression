//! probe: one-pass site counts and the counted derivation against trial encodes.
//! usage: probe_derive <file> <bytes> <host> <threads> [epochs to encode...] [trial]
use std::time::Instant;
use uorc::ncm::{self, streams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse::<usize>().unwrap().min(data.len());
    let data = &data[..n];
    let host: usize = a[3].parse().unwrap();
    let th: usize = a[4].parse().unwrap();
    let p = ncm::FULL;
    let coding =
        uorc::performance::BITS_PER_BYTE * u64::from(uorc::performance::inputs(p)) * n as u64;
    let t = Instant::now();
    let c = streams::count(&[], data, p, host).unwrap();
    println!("count k={host} secs={:.2}", t.elapsed().as_secs_f64());
    for (l, kinds) in c.levels.iter().enumerate() {
        let shares: Vec<String> = kinds
            .iter()
            .map(|k| format!("{:5.1}%", 100.0 * k[2] as f64 / k[0].max(1) as f64))
            .collect();
        println!(
            "  e=2^{l:<2} steps={:>11} chain={:>12} shared-versions/changes b,m,w,a,s={}",
            c.steps(l),
            (coding + c.steps(l)) / host as u64,
            shares.join(" ")
        );
    }
    let t = Instant::now();
    let d = streams::derive_counted(&[], data, p, host, coding, th);
    let secs = t.elapsed().as_secs_f64();
    println!("derive_counted host={host} -> {d:?} secs={secs:.2}");
    let mut epochs: Vec<usize> = Vec::new();
    let mut trial = false;
    for s in &a[5..] {
        if s == "trial" {
            trial = true;
        } else {
            epochs.push(s.parse().unwrap());
        }
    }
    if let Some((plan, _)) = d {
        epochs.insert(0, plan.epoch);
    }
    for e in epochs {
        let plan = streams::Plan {
            streams: host,
            epoch: e,
        };
        let t = Instant::now();
        let (out, s) = streams::encode_sites(&[], data, p, plan, th).unwrap();
        println!(
            "  encode k={host} e={e} bytes={} steps={} secs={:.2}",
            out.len(),
            s.steps,
            t.elapsed().as_secs_f64()
        );
    }
    if trial {
        let t = Instant::now();
        let d = streams::derive(&[], data, p, host, coding, th);
        let secs = t.elapsed().as_secs_f64();
        match d {
            None => println!("trial derive: one stream {secs:.1}s"),
            Some(d) => println!(
                "trial derive: streams={} epoch={} bytes={} steps={} secs={secs:.1}",
                d.plan.streams,
                d.plan.epoch,
                d.payload.len(),
                d.merge
            ),
        }
    }
}
