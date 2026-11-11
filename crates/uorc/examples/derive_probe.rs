//! scratch probe: derived streams for a declared host, with compress time.
use uorc::ncm::{self, streams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let host: usize = a[2].parse().unwrap();
    let th: usize = a.get(3).map_or(4, |s| s.parse().unwrap());
    let p = ncm::FULL;
    let coding = uorc::performance::BITS_PER_BYTE
        * u64::from(uorc::performance::inputs(p))
        * data.len() as u64;
    let t = std::time::Instant::now();
    let d = streams::derive(&[], &data, p, host, coding, th);
    let secs = t.elapsed().as_secs_f64();
    match d {
        None => println!("host={host} one stream, {secs:.1}s"),
        Some(d) => println!(
            "host={host} streams={} epoch={} payload={} merge={} ({:.4} of coding) derive={secs:.1}s",
            d.plan.streams, d.plan.epoch, d.payload.len(), d.merge, d.merge as f64 / coding as f64
        ),
    }
}
