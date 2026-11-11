//! scratch probe: serial encode and decode time
fn main() {
    for a in std::env::args().skip(1) {
        let d = std::fs::read(&a).unwrap();
        let t = std::time::Instant::now();
        let v = uorc::ncm::encode_profile(&[], &d, uorc::ncm::FULL).unwrap();
        let e = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        let back = uorc::ncm::decode_profile(&[], &v, u64::MAX, uorc::ncm::FULL).unwrap();
        assert_eq!(back, d);
        println!("{} n={} bytes={} enc={:.1} dec={:.1}", a.rsplit('/').next().unwrap(), d.len(), v.len(), e, t.elapsed().as_secs_f64());
    }
}
