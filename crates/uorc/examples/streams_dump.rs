//! scratch: native streams payload of a file's prefix (history = first h bytes), for the IR check.
use uorc::ncm::{self, streams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let h: usize = a[2].parse().unwrap();
    let n: usize = a[3].parse().unwrap();
    let k: usize = a[4].parse().unwrap();
    let e: usize = a[5].parse().unwrap();
    let plan = streams::Plan {
        streams: k,
        epoch: e,
    };
    let (out, steps) = streams::encode_counted(&data[..h], &data[h..h + n], ncm::FULL, plan, 1).unwrap();
    println!("{steps}");
    std::fs::write(&a[6], out).unwrap();
}
