//! scratch probe: streams-per-frame coder against the serial coder, bytes and time.
use uorc::ncm::{self, streams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse::<usize>().unwrap().min(data.len());
    let data = &data[..n];
    let serial: usize = a[3].parse().unwrap();
    let serial = if serial == 0 {
        let t = std::time::Instant::now();
        let s = ncm::encode_profile(&[], data, ncm::FULL).unwrap().len();
        println!("serial encode secs={:.2}", t.elapsed().as_secs_f64());
        s
    } else {
        serial
    };
    for spec in &a[4..] {
        let mut it = spec.split(':');
        let k: usize = it.next().unwrap().parse().unwrap();
        let e: usize = it.next().unwrap().parse().unwrap();
        let th: usize = it.next().unwrap_or("4").parse().unwrap();
        let plan = streams::Plan { streams: k, epoch: e };
        let t = std::time::Instant::now();
        let out = streams::encode(&[], data, ncm::FULL, plan, th).unwrap();
        let enc = t.elapsed().as_secs_f64();
        let me = streams::MERGE_NS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 1e9;
        let sec: Vec<f64> = streams::SECT.iter().map(|x| x.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 1e9).collect();
        println!("  encode merge={me:.2}s buckets={:.2} tables+hist={:.2} words={:.2} setup={:.2}", sec[0], sec[1], sec[2], sec[3]);
        let t = std::time::Instant::now();
        let back = streams::decode(&[], &out, u64::MAX, ncm::FULL, plan, th, true).unwrap();
        let dec = t.elapsed().as_secs_f64();
        streams::MERGE_NS.swap(0, std::sync::atomic::Ordering::Relaxed);
        for x in &streams::SECT { x.swap(0, std::sync::atomic::Ordering::Relaxed); }
        assert_eq!(back, data);
        println!(
            "{} n={} streams={} epoch={} threads={} serial={} streams={} ({:+.2}%) enc={:.2}s dec={:.2}s",
            a[1].rsplit('/').next().unwrap(), n, k, e, th, serial, out.len(),
            (out.len() as f64 / serial as f64 - 1.0) * 100.0, enc, dec
        );
    }
}
