//! Streams per frame over one shared state.
//!
//! A frame of `n` bytes is cut into `k` contiguous streams. Each stream is
//! coded by its own arithmetic coder, so the streams of a frame decode at the
//! same time on `k` threads, or one after another on fewer; the bytes do not
//! depend on how many threads run them. The streams advance in epochs of `e`
//! bytes each. Within an epoch every stream reads the shared state as it was
//! when the epoch began, plus what it learned itself since; at the epoch's
//! end the streams' changes are merged into the shared state, and every
//! stream continues from the merge.
//!
//! The merge is a sum of each stream's contribution, so it does not depend on
//! the order the streams are taken in (permutation invariance, the UOR
//! accumulation identity PA_1):
//! - a context slot is a running mean: after `n` observations at count `n`,
//!   `p (2n + 1)` is the prior plus twice the sum of the observed bits, so the
//!   merged slot is the base's sum plus every stream's added sum, over the
//!   added counts (a slot past the count limit adds its change in `p`);
//! - a bucket a stream re-tagged for another context competes with the base's
//!   context by use (the merged count of its first slot), as the bucket
//!   replacement rule itself decides;
//! - mixer weights and probability map points add every stream's change
//!   ([`Weights::Sum`]) or their mean ([`Weights::Mean`]);
//! - a match table entry keeps the furthest position any stream wrote (the
//!   join of positions, idempotent and order-free);
//! - the history bytes each stream coded are written in place, so every
//!   stream reads them from the next epoch on.
//!
//! What a stream keeps to itself is its own recent past: the bytes and words
//! before its position, its match pointers and its record model.

use super::{Bucket, Kernel, Model, NINP, NINP_NARROW, SLOT_FRESH};

/// How mixer weights and probability map points merge.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weights {
    /// The base plus every stream's change.
    Sum,
    /// The base plus the mean of the streams' changes.
    Mean,
    /// The base plus the mean change of the streams that changed it.
    Active,
}

/// How a frame is cut into streams.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    /// Streams in the frame.
    pub streams: usize,
    /// Bytes each stream codes per epoch.
    pub epoch: usize,
    /// The weight merge.
    pub weights: Weights,
}

impl Plan {
    /// The stream length for `n` bytes: `n / streams` rounded up to a
    /// multiple of 64 (so every stream starts at the same byte alignment).
    #[must_use]
    pub fn stride(&self, n: usize) -> usize {
        n.div_ceil(self.streams.max(1)).div_ceil(64) * 64
    }
}

struct Coder {
    x1: u32,
    x2: u32,
    out: Vec<u8>,
}

/// Encode `data` (after `window`) as `plan.streams` coder outputs, one per
/// stream (each ending with its flush byte; no length header).
#[must_use]
pub fn encode(window: &[u8], data: &[u8], profile: u32, plan: &Plan) -> Option<Vec<Vec<u8>>> {
    if !super::valid_profile(profile) || plan.streams == 0 || plan.epoch == 0 {
        return None;
    }
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 was detected on this processor just above.
            return unsafe { encode_avx2(window, data, profile, plan) };
        }
        encode_k::<super::Scalar>(window, data, profile, plan)
    }
    #[cfg(target_arch = "aarch64")]
    {
        encode_k::<super::arm::Neon>(window, data, profile, plan)
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        encode_k::<super::Scalar>(window, data, profile, plan)
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
fn encode_avx2(window: &[u8], data: &[u8], profile: u32, plan: &Plan) -> Option<Vec<Vec<u8>>> {
    encode_k::<super::x86::Avx2>(window, data, profile, plan)
}

#[inline(always)]
fn encode_k<K: Kernel>(
    window: &[u8],
    data: &[u8],
    profile: u32,
    plan: &Plan,
) -> Option<Vec<Vec<u8>>> {
    let n = u32::try_from(data.len()).ok()?;
    let base = Model::new(n, &[window], profile);
    if base.stride == NINP_NARROW {
        run::<K, NINP_NARROW>(base, window.len(), data, plan)
    } else {
        run::<K, NINP>(base, window.len(), data, plan)
    }
}

/// The streams' models: lane `i` codes `data[i * r..]` for `r` bytes.
fn lanes(mut base: Model, before: usize, n: usize, plan: &Plan) -> (Model, Vec<Model>) {
    base.hist.resize(before + n, 0);
    base.laid = true;
    let r = plan.stride(n);
    let mut lanes = Vec::with_capacity(plan.streams);
    for i in 0..plan.streams {
        let mut m = base.clone();
        if i > 0 {
            m.hoff = (before + (i * r).min(n)) as u32;
            m.new_byte();
        }
        lanes.push(m);
    }
    (base, lanes)
}

fn run<K: Kernel, const W: usize>(
    base: Model,
    before: usize,
    data: &[u8],
    plan: &Plan,
) -> Option<Vec<Vec<u8>>> {
    let n = data.len();
    let r = plan.stride(n);
    let (mut base, mut lanes) = lanes(base, before, n, plan);
    let mut coders: Vec<Coder> = (0..plan.streams)
        .map(|_| Coder {
            x1: 0,
            x2: u32::MAX,
            out: Vec::new(),
        })
        .collect();
    let mut at = 0;
    while at < r {
        let end = (at + plan.epoch).min(r);
        std::thread::scope(|s| {
            for (i, (m, c)) in lanes.iter_mut().zip(coders.iter_mut()).enumerate() {
                let lo = (i * r + at).min(n);
                let hi = (i * r + end).min(n).min((i + 1) * r);
                let bytes = &data[lo..hi];
                if bytes.is_empty() {
                    continue;
                }
                s.spawn(move || code::<K, W>(m, c, bytes));
            }
        });
        let coded: usize = (0..plan.streams)
            .map(|i| (i * r + end).min(n).min((i + 1) * r).saturating_sub(i * r))
            .sum();
        merge(&mut base, &mut lanes, plan, before, r, n, at, end);
        for m in lanes.iter_mut() {
            m.seen = coded as u32;
        }
        base.seen = coded as u32;
        at = end;
    }
    Some(
        coders
            .into_iter()
            .map(|mut c| {
                c.out.push((c.x1 >> 24) as u8);
                c.out
            })
            .collect(),
    )
}

#[inline(always)]
fn code<K: Kernel, const W: usize>(m: &mut Model, c: &mut Coder, bytes: &[u8]) {
    for &byte in bytes {
        for k in (0..8).rev() {
            let y = (byte >> k) & 1 == 1;
            let p = m.predict::<K, W>();
            let xm = c.x1 + ((u64::from(c.x2 - c.x1) * u64::from(p)) >> 12) as u32;
            if y {
                c.x2 = xm;
            } else {
                c.x1 = xm + 1;
            }
            while (c.x1 ^ c.x2) >> 24 == 0 {
                c.out.push((c.x2 >> 24) as u8);
                c.x1 <<= 8;
                c.x2 = (c.x2 << 8) | 255;
            }
            m.update::<K, W>(y);
        }
    }
}

/// One slot (probability in the low 16 bits, count above) merged from the
/// base and the streams' versions of it.
fn merge_slot(b: u32, ls: &[u32]) -> u32 {
    let (pb, nb) = (i64::from(b & 0xffff), i64::from(b >> 16));
    let (mut sum, mut dn, mut dp, mut sat, mut changed) = (0i64, 0i64, 0i64, nb >= 127, false);
    for &l in ls {
        if l == b {
            continue;
        }
        changed = true;
        let (p, n) = (i64::from(l & 0xffff), i64::from(l >> 16));
        // a stream whose count fell below the base's cleared the slot (its
        // bucket was recycled and came back): it adds what it learned since
        let (op, on) = if n < nb { (i64::from(SLOT_FRESH), 0) } else { (pb, nb) };
        sat |= n >= 127;
        sum += p * (2 * n + 1) - op * (2 * on + 1);
        dn += n - on;
        dp += p - op;
    }
    if !changed {
        return b;
    }
    let tot = nb + dn;
    let p = if sat {
        pb + dp / ls.iter().filter(|&&l| l != b).count() as i64
    } else {
        (pb * (2 * nb + 1) + sum) / (2 * tot.max(0) + 1)
    };
    (p.clamp(0, 65535) as u32) | ((tot.clamp(0, 127) as u32) << 16)
}

/// probe counter
pub static CONFLICTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// probe counter
pub static CHANGED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn tag(x: &[u32; 16]) -> u32 {
    (x[0] >> 16) & 0xff
}

/// A bucket merged from the base and the streams' versions.
fn merge_bucket(base: &[u32; 16], ls: &[&[u32; 16]]) -> [u32; 16] {
    // groups by tag: the base's tag first, then every tag a stream gave it
    // the base's context stays a candidate only while some stream holds it
    let mut groups: Vec<([u32; 16], Vec<&[u32; 16]>)> = Vec::new();
    if ls.iter().any(|l| tag(l) == tag(base)) {
        groups.push((*base, Vec::new()));
    }
    for &l in ls {
        if l == base {
            continue;
        }
        if tag(l) == tag(base) {
            groups[0].1.push(l);
        } else if let Some(g) = groups.iter_mut().find(|g| tag(&g.0) == tag(l)) {
            g.1.push(l);
        } else {
            let mut fresh = [SLOT_FRESH; 16];
            fresh[0] |= tag(l) << 16;
            groups.push((fresh, vec![l]));
        }
    }
    if groups.len() > 1 {
        CONFLICTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    let mut best: Option<((u32, u32, u32), [u32; 16])> = None;
    for (origin, members) in &groups {
        let mut m = *origin;
        for s in 1..16 {
            let v: Vec<u32> = members.iter().map(|x| x[s]).collect();
            m[s] = merge_slot(origin[s], &v);
        }
        // the more used context keeps the bucket (by its first slot's count,
        // then by tag), as the replacement rule decides between neighbours
        let recent = std::env::var("STREAMS_RECENT").is_ok();
        let key = (
            u32::from(recent && !members.is_empty()),
            m[1] >> 16,
            u32::MAX - tag(&m),
        );
        best = match best {
            Some((bk, b)) if bk >= key => Some((bk, b)),
            _ => Some((key, m)),
        };
    }
    best.map_or(*base, |(_, b)| b)
}

fn merge_word(b: i32, ls: &[i32], w: Weights, lo: i32, hi: i32) -> i32 {
    let d: i64 = ls.iter().map(|&l| i64::from(l) - i64::from(b)).sum();
    let d = match w {
        Weights::Sum => d,
        Weights::Mean => d / ls.len() as i64,
        Weights::Active => d / (ls.iter().filter(|&&l| l != b).count().max(1) as i64),
    };
    (i64::from(b) + d).clamp(i64::from(lo), i64::from(hi)) as i32
}

#[allow(clippy::too_many_arguments)]
fn merge(
    base: &mut Model,
    lanes: &mut [Model],
    plan: &Plan,
    before: usize,
    r: usize,
    n: usize,
    at: usize,
    end: usize,
) {
    let k = lanes.len();
    let keep = std::env::var("STREAMS_KEEP").unwrap_or_default();
    let keep = |c: char| keep.contains(c);
    // context buckets
    for b in 0..if keep('s') { 0 } else { base.sl.len() } {
        let x = base.sl[b].0;
        if lanes.iter().all(|m| m.sl[b].0 == x) {
            continue;
        }
        CHANGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let ls: Vec<&[u32; 16]> = lanes.iter().map(|m| &m.sl[b].0).collect();
        let merged = Bucket(merge_bucket(&x, &ls));
        base.sl[b] = merged;
        for m in lanes.iter_mut() {
            m.sl[b] = merged;
        }
    }
    // match tables: the furthest position a stream wrote
    for b in 0..if keep('m') { 0 } else { base.mtab.len() } {
        let x = base.mtab[b];
        let v = lanes.iter().map(|m| m.mtab[b]).filter(|&v| v != x).max();
        if let Some(v) = v {
            base.mtab[b] = v;
            for m in lanes.iter_mut() {
                m.mtab[b] = v;
            }
        }
    }
    for b in 0..if keep('m') { 0 } else { base.m2tab.len() } {
        let x = base.m2tab[b];
        let v = lanes.iter().map(|m| m.m2tab[b]).filter(|&v| v != x).max();
        if let Some(v) = v {
            base.m2tab[b] = v;
            for m in lanes.iter_mut() {
                m.m2tab[b] = v;
            }
        }
    }
    // mixer weights
    let lim = super::WEIGHT_LIMIT;
    let mut ls = vec![0i32; k];
    for which in 0..if keep('w') { 0 } else { 3 } {
        let len = match which {
            0 => base.w1.len(),
            1 => base.w2.len(),
            _ => base.w3.len(),
        };
        for j in 0..len {
            let get = |m: &Model| match which {
                0 => m.w1[j],
                1 => m.w2[j],
                _ => m.w3[j],
            };
            let x = get(base);
            if lanes.iter().all(|m| get(m) == x) {
                continue;
            }
            for (l, m) in ls.iter_mut().zip(lanes.iter()) {
                *l = get(m);
            }
            let v = merge_word(x, &ls, plan.weights, -lim, lim);
            let set = |m: &mut Model| match which {
                0 => m.w1[j] = v,
                1 => m.w2[j] = v,
                _ => m.w3[j] = v,
            };
            set(base);
            for m in lanes.iter_mut() {
                set(m);
            }
        }
    }
    // probability maps
    for which in 0..if keep('a') { 0 } else { 2 } {
        let len = if which == 0 { base.a1.t.len() } else { base.a2.t.len() };
        for j in 0..len {
            let get = |m: &Model| i32::from(if which == 0 { m.a1.t[j] } else { m.a2.t[j] });
            let x = get(base);
            if lanes.iter().all(|m| get(m) == x) {
                continue;
            }
            for (l, m) in ls.iter_mut().zip(lanes.iter()) {
                *l = get(m);
            }
            let v = merge_word(x, &ls, plan.weights, 0, 65535) as u16;
            if which == 0 {
                base.a1.t[j] = v;
                lanes.iter_mut().for_each(|m| m.a1.t[j] = v);
            } else {
                base.a2.t[j] = v;
                lanes.iter_mut().for_each(|m| m.a2.t[j] = v);
            }
        }
    }
    // the match models' own slots (probability and count apart)
    let small = |b: (u16, u8), ls: &[(u16, u8)]| {
        let pack = |(p, n): (u16, u8)| u32::from(p) | (u32::from(n) << 16);
        let v: Vec<u32> = ls.iter().map(|&x| pack(x)).collect();
        let m = merge_slot(pack(b), &v);
        ((m & 0xffff) as u16, (m >> 16) as u8)
    };
    macro_rules! merge_small {
        ($p:ident, $n:ident) => {
            for j in 0..base.$p.len() {
                let x = (base.$p[j], base.$n[j]);
                let v: Vec<(u16, u8)> = lanes.iter().map(|m| (m.$p[j], m.$n[j])).collect();
                let (p, c) = small(x, &v);
                base.$p[j] = p;
                base.$n[j] = c;
                for m in lanes.iter_mut() {
                    m.$p[j] = p;
                    m.$n[j] = c;
                }
            }
        };
    }
    merge_small!(msp, msn);
    merge_small!(m2sp, m2sn);
    merge_small!(rsp, rsn);
    // the epoch's bytes, in place for every stream
    for i in 0..k {
        let lo = before + (i * r + at).min(n);
        let hi = before + (i * r + end).min(n).min((i + 1) * r);
        if lo >= hi {
            continue;
        }
        let bytes = lanes[i].hist[lo..hi].to_vec();
        base.hist[lo..hi].copy_from_slice(&bytes);
        for (j, m) in lanes.iter_mut().enumerate() {
            if j != i {
                m.hist[lo..hi].copy_from_slice(&bytes);
            }
        }
    }
}
