//! probe: native streams payload size and merge steps of a file prefix (history h, n bytes, k streams, epoch e).
use uorc::ncm::{self, streams};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let p = |i: usize| a[i].parse::<usize>().unwrap();
    let (h, n, k, e) = (p(2), p(3), p(4), p(5));
    let plan = streams::Plan { streams: k, epoch: e };
    let (out, steps) = streams::encode_counted(&data[..h], &data[h..h + n], ncm::FULL, plan, 1).unwrap();
    println!("{} {}", out.len(), steps);
}
