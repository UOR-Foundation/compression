//! scratch: 4 independent serial encodes in threads of one process
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = std::fs::read(&a[1]).unwrap();
    let k: usize = a[2].parse().unwrap();
    let r = 524288;
    let t = std::time::Instant::now();
    std::thread::scope(|s| {
        for i in 0..k {
            let d = &d[i * r..(i + 1) * r];
            s.spawn(move || {
                uorc::ncm::encode_profile(&[], d, uorc::ncm::FULL)
                    .unwrap()
                    .len()
            });
        }
    });
    println!("threads={k} secs={:.2}", t.elapsed().as_secs_f64());
}
