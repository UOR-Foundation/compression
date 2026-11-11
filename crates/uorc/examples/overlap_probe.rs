//! scratch probe: assign chunks to streams by content overlap instead of position.
use std::collections::HashSet;
use uorc::ncm::{self, streams};
fn shingles(c: &[u8]) -> HashSet<u64> {
    let mut s = HashSet::new();
    for w in c.windows(8) {
        let mut h = 0xcbf29ce484222325u64;
        for &b in w { h = (h ^ u64::from(b)).wrapping_mul(0x100000001b3); }
        if h % 8 == 0 { s.insert(h); }
    }
    s
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let n: usize = a[2].parse().unwrap();
    let k: usize = a[3].parse().unwrap();
    let c: usize = a[4].parse().unwrap();
    let e: usize = a[5].parse().unwrap();
    let data = &data[..n];
    let nc = n / c;
    let m = nc / k;
    let sh: Vec<HashSet<u64>> = (0..nc).map(|i| shingles(&data[i * c..(i + 1) * c])).collect();
    let mut sets: Vec<HashSet<u64>> = vec![HashSet::new(); k];
    let mut lists: Vec<Vec<usize>> = vec![Vec::new(); k];
    for s in 0..k { lists[s].push(s * m); sets[s].extend(sh[s * m].iter().copied()); }
    for i in 0..nc {
        if i % m == 0 { continue; }
        let mut best = (0usize, 0usize);
        let mut found = false;
        for s in 0..k {
            if lists[s].len() >= m { continue; }
            let ov = sh[i].iter().filter(|h| sets[s].contains(h)).count();
            if !found || ov > best.0 { best = (ov, s); found = true; }
        }
        lists[best.1].push(i);
        sets[best.1].extend(sh[i].iter().copied());
    }
    let mut moved = 0;
    let mut re = Vec::with_capacity(n);
    for (s, l) in lists.iter().enumerate() {
        for &i in l { if i / m != s { moved += 1; } re.extend_from_slice(&data[i * c..(i + 1) * c]); }
    }
    let plan = streams::Plan { streams: k, epoch: e, weights: streams::Weights::Active };
    let size = |d: &[u8]| streams::encode(&[], d, ncm::FULL, &plan).unwrap().iter().map(Vec::len).sum::<usize>() + 4 + 4 * k;
    let pos = size(&data[..nc * c]);
    let ov = size(&re);
    let serial_re = ncm::encode_profile(&[], &re, ncm::FULL).unwrap().len();
    println!("{} n={} streams={} chunk={} epoch={} moved={}/{} by-position={} by-overlap={} ({:+.2}%) serial-of-reordered={}",
        a[1].rsplit('/').next().unwrap(), n, k, c, e, moved, nc, pos, ov, (ov as f64 / pos as f64 - 1.0) * 100.0, serial_re);
}
