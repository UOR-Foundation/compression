//! Streams per frame over one shared state.
//!
//! A frame of `n` bytes is cut into `k` contiguous streams. Each stream is
//! coded by its own arithmetic coder, so the streams of a frame decode at the
//! same time on `k` threads, or one after another on fewer; the bytes do not
//! depend on how many threads run them, and neither does the work: every
//! stream codes the same bits against the same state either way. The streams
//! advance in epochs of `e` bytes each. Within an epoch every stream reads the
//! shared state as it was when the epoch began, plus what it changed itself
//! since (its overlay); at the epoch's end the streams' changes are merged
//! into the shared state, and every stream continues from the merge.
//!
//! The merge is a sum of each stream's contribution, so it does not depend on
//! the order the streams are taken in (permutation invariance, the UOR
//! accumulation identity PA_1):
//! - a context slot below the count limit is a running mean: after `n`
//!   observations, `p (2n + 1)` is the prior plus twice the sum of the
//!   observed bits, so the merged slot is the base's sum plus every stream's
//!   added sum, over the added counts; a slot at the count limit (an
//!   exponential average) and every mixer weight and probability map point
//!   move by the mean change of the streams that changed them (adding the
//!   changes overshoots: four streams' summed weight changes code 41% larger);
//! - a bucket a stream re-tagged for another context competes with the
//!   base's context by use (the merged count of its first slot), as the
//!   bucket replacement rule itself decides;
//! - a match table entry keeps the furthest position any stream wrote (the
//!   join of positions, idempotent and order-free);
//! - the history bytes each stream coded are written in place, so every
//!   stream reads them from the next epoch on (bytes another stream has not
//!   finished read as 0).
//!
//! What a stream keeps to itself is its own recent past: the bytes and words
//! before its position, its match pointers and its record model.
//!
//! With one stream the merge is the identity and the payload is the serial
//! coder's, byte for byte, after the 4-byte length.

use super::{Bucket, Kernel, Model, Table, NINP, NINP_NARROW, SLOT_FRESH, WEIGHT_LIMIT};
use std::sync::Arc;

/// How a frame is cut into streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    /// Streams in the frame (at least 1).
    pub streams: usize,
    /// Bytes each stream codes per epoch (at least 1).
    pub epoch: usize,
}

impl Plan {
    /// The stream length for `n` bytes: `n / streams` rounded up to a
    /// multiple of 64 (so every stream starts at the same byte alignment).
    #[must_use]
    pub fn stride(&self, n: usize) -> usize {
        n.div_ceil(self.streams.max(1)).div_ceil(64) * 64
    }

    /// Stream `i`'s bytes of an `n`-byte frame.
    #[must_use]
    pub fn range(&self, n: usize, i: usize) -> std::ops::Range<usize> {
        let r = self.stride(n);
        (i * r).min(n)..((i + 1) * r).min(n)
    }

    fn valid(&self) -> bool {
        self.streams > 0 && self.epoch > 0 && self.streams <= 1 << 16
    }
}

/// The state every stream of a frame reads: the context buckets, the match
/// tables and the history, as merged at the last epoch's end.
pub(super) struct Shared {
    pub(super) sl: Table<Bucket>,
    pub(super) mtab: Table<u32>,
    pub(super) m2tab: Table<u32>,
    hist: Vec<u8>,
}

/// A stream's view: the shared state, and what the stream changed since the
/// epoch began.
#[derive(Clone)]
pub(super) struct Lane {
    shared: Option<Arc<Shared>>,
    /// The stream's bytes coded this epoch, from absolute position `lo`.
    pub(super) buf: Vec<u8>,
    lo: u32,
    /// The buckets the stream copied into its own table this epoch (its
    /// table holds 0 in every other bucket, and the stream reads those from
    /// the shared table).
    btouched: Vec<u32>,
    /// The match table entries the stream wrote this epoch (likewise).
    pub(super) mt: Vec<u32>,
    pub(super) m2t: Vec<u32>,
}

impl Lane {
    #[inline(always)]
    pub(super) fn shared(&self) -> &Shared {
        self.shared
            .as_deref()
            .expect("a stream reads the shared state")
    }

    /// The byte at absolute position `i` as this stream sees it.
    #[inline(always)]
    pub(super) fn abs(&self, i: u32) -> u32 {
        let d = i.wrapping_sub(self.lo) as usize;
        if d < self.buf.len() {
            u32::from(self.buf[d])
        } else {
            u32::from(self.shared().hist[i as usize])
        }
    }

    /// The stream's copy of shared bucket `b`, made on first use (a
    /// bucket's slot 0 is never 0, so 0 marks a bucket not copied).
    #[inline]
    fn own(&mut self, b: u32, sl: &mut Table<Bucket>) {
        if sl[b as usize].0[0] == 0 {
            sl[b as usize] = self.shared().sl[b as usize];
            self.btouched.push(b);
        }
    }

    /// [`Model::bucket`] over the stream's own table and the shared one:
    /// the bucket's first slot index (in the stream's own table).
    #[inline]
    pub(super) fn bucket(&mut self, hh: u32, mask: u32, sl: &mut Table<Bucket>) -> u32 {
        let ck = (hh >> 24) | 1;
        let b0 = ((hh & mask) & !15) >> 4;
        let alt = b0 ^ 1;
        let read = |l: &Lane, sl: &Table<Bucket>, b: u32| {
            let x = &sl[b as usize].0;
            if x[0] == 0 {
                l.shared().sl[b as usize].0
            } else {
                *x
            }
        };
        let x0 = read(self, sl, b0);
        if tag(&x0) == ck {
            self.own(b0, sl);
            return b0 << 4;
        }
        let xa = read(self, sl, alt);
        if tag(&xa) == ck {
            self.own(alt, sl);
            return alt << 4;
        }
        let b = if (x0[1] >> 16) as u8 > (xa[1] >> 16) as u8 {
            alt
        } else {
            b0
        };
        self.own(b, sl);
        let mut fresh = [SLOT_FRESH; 16];
        fresh[0] |= ck << 16;
        sl[b as usize].0 = fresh;
        b << 4
    }
}

#[repr(align(128))]
struct Coder {
    x1: u32,
    x2: u32,
    /// decoding: the code value and the next payload byte
    x: u32,
    ip: usize,
    out: Vec<u8>,
    emitted: usize,
    ok: bool,
}

impl Coder {
    fn new() -> Coder {
        Coder {
            x1: 0,
            x2: u32::MAX,
            x: 0,
            ip: 0,
            out: Vec::new(),
            emitted: 0,
            ok: true,
        }
    }
}

type Part = std::ops::Range<usize>;

/// Merge sections (probe).
pub static SECT: [std::sync::atomic::AtomicU64; 4] = [const { std::sync::atomic::AtomicU64::new(0) }; 4];
/// Time spent merging (probe).
pub static MERGE_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A frame's streams and their shared state.
struct Frame {
    base: Model,
    shared: Arc<Shared>,
    lanes: Vec<Model>,
    plan: Plan,
    before: usize,
    n: usize,
    /// bytes of every stream merged so far
    coded: usize,
    /// per shared bucket: how many streams copied it this epoch
    mark: Vec<u32>,
}

impl Frame {
    fn new(history: &[&[u8]], n: usize, profile: u32, plan: Plan) -> Frame {
        let t0 = std::time::Instant::now();
        let f = Frame::build(history, n, profile, plan);
        SECT[3].fetch_add(t0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        f
    }

    fn build(history: &[&[u8]], n: usize, profile: u32, plan: Plan) -> Frame {
        let mut base = Model::new(n as u32, history, profile);
        let before = base.hoff as usize;
        let mut hist = std::mem::take(&mut base.hist);
        hist.resize(before + n, 0);
        let shared = Arc::new(Shared {
            sl: std::mem::replace(&mut base.sl, Table(Vec::new())),
            mtab: std::mem::replace(&mut base.mtab, Table(Vec::new())),
            m2tab: std::mem::replace(&mut base.m2tab, Table(Vec::new())),
            hist,
        });
        let mut lanes = Vec::with_capacity(plan.streams);
        for i in 0..plan.streams {
            let mut m = base.clone();
            m.sl = Table::new(shared.sl.len(), Bucket([0; 16]));
            m.mtab = Table::new(shared.mtab.len(), 0);
            m.m2tab = Table::new(shared.m2tab.len(), 0);
            let start = before + plan.range(n, i).start;
            m.lane = Some(Box::new(Lane {
                shared: Some(Arc::clone(&shared)),
                buf: Vec::new(),
                lo: start as u32,
                btouched: Vec::new(),
                mt: Vec::new(),
                m2t: Vec::new(),
            }));
            if i > 0 {
                m.hoff = start as u32;
                m.new_byte();
            }
            lanes.push(m);
        }
        let nb = shared.sl.len();
        Frame {
            base,
            shared,
            lanes,
            plan,
            before,
            n,
            coded: 0,
            mark: vec![0; nb],
        }
    }

    /// Stream `i`'s bytes in epoch `[at, end)` (offsets within a stream).
    fn part(&self, i: usize, at: usize, end: usize) -> Part {
        let r = self.plan.range(self.n, i);
        (r.start + at).min(r.end)..(r.start + end).min(r.end)
    }

    /// Run every stream over its part of epoch `[at, end)` on up to
    /// `threads` threads, `work(stream, model, coder, part)`, then merge.
    fn epoch<F>(&mut self, coders: &mut [Coder], at: usize, end: usize, threads: usize, work: F)
    where
        F: Fn(usize, &mut Model, &mut Coder, Part) + Sync,
    {
        let parts: Vec<Part> = (0..self.lanes.len())
            .map(|i| self.part(i, at, end))
            .collect();
        for (m, p) in self.lanes.iter_mut().zip(&parts) {
            let l = m.lane.as_mut().expect("a stream");
            l.lo = (self.before + p.start) as u32;
            l.buf.clear();
        }
        let threads = threads.clamp(1, self.lanes.len());
        let jobs: Vec<(usize, &mut Model, &mut Coder)> = self
            .lanes
            .iter_mut()
            .zip(coders.iter_mut())
            .enumerate()
            .map(|(i, (m, c))| (i, m, c))
            .collect();
        if threads == 1 || cfg!(target_family = "wasm") {
            for (i, m, c) in jobs {
                work(i, m, c, parts[i].clone());
            }
        } else {
            // stream i runs on thread i % threads: the same work either way
            let mut per: Vec<Vec<(usize, &mut Model, &mut Coder)>> =
                (0..threads).map(|_| Vec::new()).collect();
            for j in jobs {
                per[j.0 % threads].push(j);
            }
            let (work, parts) = (&work, &parts);
            std::thread::scope(|s| {
                for batch in per {
                    s.spawn(move || {
                        for (i, m, c) in batch {
                            work(i, m, c, parts[i].clone());
                        }
                    });
                }
            });
        }
        self.coded += parts.iter().map(|p| p.end - p.start).sum::<usize>();
        let t = std::time::Instant::now();
        self.merge(&parts, threads);
        MERGE_NS.fetch_add(t.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    }

    fn merge(&mut self, parts: &[Part], threads: usize) {
        for m in &mut self.lanes {
            m.lane.as_mut().expect("a stream").shared = None;
        }
        let shared = Arc::get_mut(&mut self.shared).expect("the streams released the state");
        let t0 = std::time::Instant::now();
        // context buckets: every bucket a stream copied, merged from all its
        // versions, the buckets split among the threads by range
        let nb = shared.sl.len();
        if std::env::var("STREAMS_COUNT").is_ok() {
            let t: usize = self.lanes.iter().map(|m| m.lane.as_ref().expect("s").btouched.len()).sum();
            let mt: usize = self.lanes.iter().map(|m| m.lane.as_ref().expect("s").mt.len()).sum();
            eprintln!("epoch bytes={} touched buckets={} per byte={:.1} mt={}", parts.iter().map(|p| p.len()).sum::<usize>(), t, t as f64 / parts.iter().map(|p| p.len()).sum::<usize>() as f64, mt);
        }
        let threads = threads.clamp(1, nb);
        let span = nb.div_ceil(threads);
        // mark[b]: 0 untouched, 1 + the one stream that copied it, or MANY
        const MANY: u32 = u32::MAX;
        let mut work: Vec<Vec<u32>> = vec![Vec::new(); threads];
        for (j, m) in self.lanes.iter().enumerate() {
            let l = m.lane.as_ref().expect("a stream");
            for &b in &l.btouched {
                let seen = &mut self.mark[b as usize];
                if *seen == 0 {
                    *seen = j as u32 + 1;
                    work[b as usize / span].push(b);
                } else {
                    *seen = MANY;
                }
            }
        }
        // every table split at the same bucket ranges, one range per thread
        let mut jobs: Vec<(&mut [Bucket], &mut [u32], Vec<&mut [Bucket]>, &[u32])> = shared
            .sl
            .chunks_mut(span)
            .zip(self.mark.chunks_mut(span))
            .zip(&work)
            .map(|((sl, mark), w)| (sl, mark, Vec::new(), &w[..]))
            .collect();
        for m in &mut self.lanes {
            for (t, c) in m.sl.chunks_mut(span).enumerate() {
                jobs[t].2.push(c);
            }
        }
        let job = |t: usize, (sl, mark, lanes, work): (&mut [Bucket], &mut [u32], Vec<&mut [Bucket]>, &[u32])| {
            let mut lanes = lanes;
            let mut versions: Vec<[u32; 16]> = Vec::with_capacity(lanes.len());
            let ahead = 8;
            for (i, &b) in work.iter().enumerate() {
                if let Some(&n) = work.get(i + ahead) {
                    let n = n as usize - t * span;
                    super::prefetch(&sl[n]);
                    match mark[n] {
                        MANY => {}
                        j => super::prefetch(&lanes[j as usize - 1][n]),
                    }
                }
                let i = b as usize - t * span;
                let x = sl[i].0;
                let v = match mark[i] {
                    MANY => {
                        versions.clear();
                        for l in lanes.iter_mut() {
                            let v = l[i].0;
                            versions.push(if v[0] == 0 { x } else { v });
                            l[i] = Bucket([0; 16]);
                        }
                        merge_bucket(&x, &versions)
                    }
                    j => {
                        // one stream changed the bucket: if it kept the
                        // context, its copy is the merge (a sum with one term)
                        let l = &mut lanes[j as usize - 1][i];
                        let v = l.0;
                        *l = Bucket([0; 16]);
                        if same_context(&x, &v) {
                            v
                        } else {
                            versions.clear();
                            versions.resize(lanes.len(), x);
                            versions[j as usize - 1] = v;
                            merge_bucket(&x, &versions)
                        }
                    }
                };
                sl[i] = Bucket(v);
                mark[i] = 0;
            }
        };
        if threads == 1 || cfg!(target_family = "wasm") {
            for (t, j) in jobs.into_iter().enumerate() {
                job(t, j);
            }
        } else {
            let job = &job;
            std::thread::scope(|sc| {
                for (t, j) in jobs.into_iter().enumerate() {
                    sc.spawn(move || job(t, j));
                }
            });
        }
        for m in &mut self.lanes {
            m.lane.as_mut().expect("a stream").btouched.clear();
        }
        SECT[0].fetch_add(t0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        let t0 = std::time::Instant::now();
        // match tables: the furthest position a stream wrote. Streams lie in
        // frame order, so every position a later stream wrote is further:
        // writing the streams' entries in stream order leaves the furthest.
        for m in &self.lanes {
            let l = m.lane.as_ref().expect("a stream");
            for &key in &l.mt {
                shared.mtab[key as usize] = m.mtab[key as usize];
            }
            for &key in &l.m2t {
                shared.m2tab[key as usize] = m.m2tab[key as usize];
            }
        }
        // the epoch's bytes, in place
        for (m, p) in self.lanes.iter().zip(parts) {
            let l = m.lane.as_ref().expect("a stream");
            let lo = self.before + p.start;
            shared.hist[lo..lo + l.buf.len()].copy_from_slice(&l.buf);
        }
        SECT[1].fetch_add(t0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        let t0 = std::time::Instant::now();
        // mixer weights, probability maps and the match models' own slots
        let base = &mut self.base;
        let lanes = &mut self.lanes;
        let wl = (-WEIGHT_LIMIT, WEIGHT_LIMIT);
        merge_words(&mut base.w1, lanes, |m| &mut m.w1, wl);
        merge_words(&mut base.w2, lanes, |m| &mut m.w2, wl);
        merge_words(&mut base.w3, lanes, |m| &mut m.w3, wl);
        merge_words(&mut base.a1.t, lanes, |m| &mut m.a1.t, (0, 65535));
        merge_words(&mut base.a2.t, lanes, |m| &mut m.a2.t, (0, 65535));
        merge_small(&mut base.msp, &mut base.msn, lanes, |m| {
            (&mut m.msp, &mut m.msn)
        });
        merge_small(&mut base.m2sp, &mut base.m2sn, lanes, |m| {
            (&mut m.m2sp, &mut m.m2sn)
        });
        merge_small(&mut base.rsp, &mut base.rsn, lanes, |m| {
            (&mut m.rsp, &mut m.rsn)
        });
        SECT[2].fetch_add(t0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        for m in lanes.iter_mut() {
            m.seen = self.coded as u32;
            let l = m.lane.as_mut().expect("a stream");
            for &key in &l.mt {
                m.mtab[key as usize] = 0;
            }
            for &key in &l.m2t {
                m.m2tab[key as usize] = 0;
            }
            l.mt.clear();
            l.m2t.clear();
            l.buf.clear();
            l.shared = Some(Arc::clone(&self.shared));
        }
    }
}

/// Every stream's copy of a word table merged into `base` and back: each
/// entry moves by the mean change of the streams that changed it.
fn merge_words<T, F>(base: &mut [T], lanes: &mut [Model], get: F, (lo, hi): (i32, i32))
where
    T: Copy + PartialEq + Into<i32> + TryFrom<i32>,
    F: Fn(&mut Model) -> &mut Vec<T>,
{
    const CHUNK: usize = 64;
    for c in (0..base.len()).step_by(CHUNK) {
        let e = (c + CHUNK).min(base.len());
        if lanes.iter_mut().all(|m| get(m)[c..e] == base[c..e]) {
            continue;
        }
        for j in c..e {
            merge_word(base, lanes, &get, j, (lo, hi));
        }
    }
}

fn merge_word<T, F>(base: &mut [T], lanes: &mut [Model], get: &F, j: usize, (lo, hi): (i32, i32))
where
    T: Copy + PartialEq + Into<i32> + TryFrom<i32>,
    F: Fn(&mut Model) -> &mut Vec<T>,
{
    {
        let x = base[j];
        let (mut changed, mut d) = (0i64, 0i64);
        for m in lanes.iter_mut() {
            let v = get(m)[j];
            if v != x {
                changed += 1;
                d += i64::from(v.into()) - i64::from(x.into());
            }
        }
        if changed == 0 {
            return;
        }
        let v = (i64::from(x.into()) + d / changed).clamp(i64::from(lo), i64::from(hi)) as i32;
        let Ok(v) = T::try_from(v) else {
            unreachable!("clamped into the table's range")
        };
        base[j] = v;
        for m in lanes.iter_mut() {
            get(m)[j] = v;
        }
    }
}

/// The match models' slots (probability and count apart) merged as context
/// slots.
fn merge_small<const N: usize, F>(bp: &mut [u16; N], bn: &mut [u8; N], lanes: &mut [Model], get: F)
where
    F: Fn(&mut Model) -> (&mut [u16; N], &mut [u8; N]),
{
    let pack = |p: u16, n: u8| u32::from(p) | (u32::from(n) << 16);
    let mut v = Vec::with_capacity(lanes.len());
    for j in 0..N {
        v.clear();
        for m in lanes.iter_mut() {
            let (p, n) = get(m);
            v.push(pack(p[j], n[j]));
        }
        let x = merge_slot(pack(bp[j], bn[j]), &v);
        bp[j] = (x & 0xffff) as u16;
        bn[j] = (x >> 16) as u8;
        for m in lanes.iter_mut() {
            let (p, n) = get(m);
            p[j] = bp[j];
            n[j] = bn[j];
        }
    }
}

/// One slot (probability in the low 16 bits, count above) merged from the
/// base and the streams' versions of it.
fn merge_slot(b: u32, ls: &[u32]) -> u32 {
    let (pb, nb) = (i64::from(b & 0xffff), i64::from(b >> 16));
    let (mut sum, mut dn, mut dp, mut sat, mut changed) = (0i64, 0i64, 0i64, nb >= 127, 0i64);
    for &l in ls {
        if l == b {
            continue;
        }
        changed += 1;
        let (p, n) = (i64::from(l & 0xffff), i64::from(l >> 16));
        // a stream whose count fell below the base's cleared the slot (its
        // bucket was recycled and came back): it adds what it learned since
        let (op, on) = if n < nb {
            (i64::from(SLOT_FRESH), 0)
        } else {
            (pb, nb)
        };
        sat |= n >= 127;
        sum += p * (2 * n + 1) - op * (2 * on + 1);
        dn += n - on;
        dp += p - op;
    }
    if changed == 0 {
        return b;
    }
    let tot = nb + dn;
    let p = if sat {
        pb + dp / changed
    } else {
        (pb * (2 * nb + 1) + sum) / (2 * tot.max(0) + 1)
    };
    (p.clamp(0, 65535) as u32) | ((tot.clamp(0, 127) as u32) << 16)
}

/// Whether `l` holds the context `base` holds, learned further (its tag,
/// and no count below the base's).
fn same_context(base: &[u32; 16], l: &[u32; 16]) -> bool {
    tag(l) == tag(base) && (1..16).all(|s| l[s] >> 16 >= base[s] >> 16)
}

fn tag(x: &[u32; 16]) -> u32 {
    (x[0] >> 16) & 0xff
}

/// A bucket merged from the base and the streams' versions.
fn merge_bucket(base: &[u32; 16], ls: &[[u32; 16]]) -> [u32; 16] {
    // groups by tag: the base's context stays a candidate only while some
    // stream holds it; every other tag a stream gave the bucket starts fresh
    // (a stream that recycled the bucket and came back to the base's tag
    // holds a fresh context, not the base's: one of its counts fell)
    let mut groups: Vec<(bool, [u32; 16], Vec<&[u32; 16]>)> = Vec::new();
    if ls.iter().any(|l| same_context(base, l)) {
        groups.push((false, *base, Vec::new()));
    }
    for l in ls {
        if l == base {
            continue;
        }
        if same_context(base, l) {
            groups[0].2.push(l);
        } else if let Some(g) = groups.iter_mut().find(|g| g.0 && tag(&g.1) == tag(l)) {
            g.2.push(l);
        } else {
            let mut fresh = [SLOT_FRESH; 16];
            fresh[0] |= tag(l) << 16;
            groups.push((true, fresh, vec![l]));
        }
    }
    let mut best: Option<((u32, u32), [u32; 16])> = None;
    let mut v = Vec::with_capacity(ls.len());
    for (_, origin, members) in &groups {
        let mut m = *origin;
        for s in 1..16 {
            v.clear();
            v.extend(members.iter().map(|x| x[s]));
            m[s] = merge_slot(origin[s], &v);
        }
        // the more used context keeps the bucket (by its first slot's count,
        // then the smaller tag), as the replacement rule decides
        let key = (m[1] >> 16, u32::MAX - tag(&m));
        best = match best {
            Some((bk, b)) if bk >= key => Some((bk, b)),
            _ => Some((key, m)),
        };
    }
    best.map_or(*base, |(_, b)| b)
}

/// Encode `data` (after `history`) as a streams payload: the length (4
/// bytes, big-endian), the length of every stream's coder output but the
/// last (4 bytes each, big-endian), then the outputs, on up to `threads`
/// threads (the payload does not depend on them). `None` for an invalid plan
/// or profile, or an input that does not fit the 4-byte length.
#[must_use]
pub fn encode(
    history: &[u8],
    data: &[u8],
    profile: u32,
    plan: Plan,
    threads: usize,
) -> Option<Vec<u8>> {
    if !super::valid_profile(profile) || !plan.valid() {
        return None;
    }
    u32::try_from(data.len()).ok()?;
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 was detected on this processor just above.
            return Some(unsafe { encode_avx2(history, data, profile, plan, threads) });
        }
        Some(encode_k::<super::Scalar>(
            history, data, profile, plan, threads,
        ))
    }
    #[cfg(target_arch = "aarch64")]
    {
        Some(encode_k::<super::arm::Neon>(
            history, data, profile, plan, threads,
        ))
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        Some(encode_k::<super::wasm::Simd128>(
            history, data, profile, plan, threads,
        ))
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )))]
    {
        Some(encode_k::<super::Scalar>(
            history, data, profile, plan, threads,
        ))
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
fn encode_avx2(history: &[u8], data: &[u8], profile: u32, plan: Plan, threads: usize) -> Vec<u8> {
    encode_k::<super::x86::Avx2>(history, data, profile, plan, threads)
}

#[inline(always)]
fn encode_k<K: Kernel>(
    history: &[u8],
    data: &[u8],
    profile: u32,
    plan: Plan,
    threads: usize,
) -> Vec<u8> {
    let mut f = Frame::new(&[history], data.len(), profile, plan);
    if f.base.stride == NINP_NARROW {
        encode_frame::<K, NINP_NARROW>(&mut f, data, threads)
    } else {
        encode_frame::<K, NINP>(&mut f, data, threads)
    }
}

#[inline(always)]
fn encode_frame<K: Kernel, const W: usize>(f: &mut Frame, data: &[u8], threads: usize) -> Vec<u8> {
    let k = f.plan.streams;
    let r = f.plan.stride(f.n);
    let mut coders: Vec<Coder> = (0..k).map(|_| Coder::new()).collect();
    let mut at = 0;
    while at < r {
        let end = (at + f.plan.epoch).min(r);
        f.epoch(&mut coders, at, end, threads, |_, m, c, part| {
            code::<K, W>(m, c, &data[part]);
        });
        at = end;
    }
    let mut out = Vec::new();
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    for c in &mut coders {
        c.out.push((c.x1 >> 24) as u8);
    }
    for c in &coders[..k - 1] {
        out.extend_from_slice(&(c.out.len() as u32).to_be_bytes());
    }
    for c in &coders {
        out.extend_from_slice(&c.out);
    }
    out
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

/// Decode a streams payload [`encode`] wrote under `profile` and `plan`
/// after `history`, on up to `threads` threads (the output and the work do
/// not depend on them). `canonical` accepts only the payload [`encode`]
/// writes for the output (every stream's coder bytes exactly), so a payload
/// has one meaning. `None` past `limit` bytes or for a malformed payload.
#[must_use]
pub fn decode(
    history: &[&[u8]],
    payload: &[u8],
    limit: u64,
    profile: u32,
    plan: Plan,
    threads: usize,
    canonical: bool,
) -> Option<Vec<u8>> {
    if !super::valid_profile(profile) || !plan.valid() {
        return None;
    }
    let n = super::declared_length(payload)?;
    if u64::from(n) > limit {
        return None;
    }
    let a = (history, payload, n as usize, profile, plan, threads, canonical);
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 was detected on this processor just above.
            return unsafe { decode_avx2(a) };
        }
        decode_k::<super::Scalar>(a)
    }
    #[cfg(target_arch = "aarch64")]
    {
        decode_k::<super::arm::Neon>(a)
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        decode_k::<super::wasm::Simd128>(a)
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )))]
    {
        decode_k::<super::Scalar>(a)
    }
}

type DecodeArgs<'a> = (&'a [&'a [u8]], &'a [u8], usize, u32, Plan, usize, bool);

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
fn decode_avx2(a: DecodeArgs<'_>) -> Option<Vec<u8>> {
    decode_k::<super::x86::Avx2>(a)
}

#[inline(always)]
fn decode_k<K: Kernel>(
    (history, payload, n, profile, plan, threads, canonical): DecodeArgs<'_>,
) -> Option<Vec<u8>> {
    let k = plan.streams;
    // the stream outputs' bounds
    let mut bounds = Vec::with_capacity(k + 1);
    bounds.push(4 + 4 * (k - 1));
    for i in 0..k - 1 {
        let l = u32::from_be_bytes(payload.get(4 + 4 * i..8 + 4 * i)?.try_into().ok()?);
        bounds.push(bounds[i].checked_add(l as usize)?);
    }
    if bounds[k - 1] > payload.len() {
        return None;
    }
    bounds.push(payload.len());
    let streams: Vec<&[u8]> = (0..k).map(|i| &payload[bounds[i]..bounds[i + 1]]).collect();
    let mut coders: Vec<Coder> = streams
        .iter()
        .map(|s| {
            let mut c = Coder::new();
            for j in 0..4 {
                c.x = (c.x << 8) | u32::from(s.get(j).copied().unwrap_or(255));
            }
            c.ip = 4;
            c
        })
        .collect();
    let mut f = Frame::new(history, n, profile, plan);
    let narrow = f.base.stride == NINP_NARROW;
    let r = plan.stride(n);
    let mut at = 0;
    while at < r {
        let end = (at + plan.epoch).min(r);
        let streams = &streams;
        f.epoch(&mut coders, at, end, threads, |i, m, c, part| {
            if narrow {
                uncode::<K, NINP_NARROW>(m, c, streams[i], part.len(), canonical);
            } else {
                uncode::<K, NINP>(m, c, streams[i], part.len(), canonical);
            }
        });
        at = end;
    }
    for (c, s) in coders.iter().zip(&streams) {
        if !c.ok
            || (canonical
                && (s.len() != c.emitted + 1 || s[c.emitted] != (c.x1 >> 24) as u8))
        {
            return None;
        }
    }
    drop(f.lanes);
    let shared = Arc::try_unwrap(f.shared).ok()?;
    let mut out = shared.hist;
    out.drain(..f.before);
    Some(out)
}

#[inline(always)]
fn uncode<K: Kernel, const W: usize>(
    m: &mut Model,
    c: &mut Coder,
    s: &[u8],
    count: usize,
    canonical: bool,
) {
    if !c.ok {
        return;
    }
    for _ in 0..count {
        for _ in 0..8 {
            let p = m.predict::<K, W>();
            let xm = c.x1 + ((u64::from(c.x2 - c.x1) * u64::from(p)) >> 12) as u32;
            let y = c.x <= xm;
            if y {
                c.x2 = xm;
            } else {
                c.x1 = xm + 1;
            }
            while (c.x1 ^ c.x2) >> 24 == 0 {
                if canonical {
                    if s.get(c.emitted) != Some(&((c.x2 >> 24) as u8)) {
                        c.ok = false;
                        return;
                    }
                    c.emitted += 1;
                }
                c.x1 <<= 8;
                c.x2 = (c.x2 << 8) | 255;
                c.x = (c.x << 8) | u32::from(s.get(c.ip).copied().unwrap_or(255));
                c.ip += 1;
            }
            m.update::<K, W>(y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ncm::{encode_profile, FULL};

    fn sample(n: usize, seed: u32) -> Vec<u8> {
        let mut x = seed;
        let words = [
            "stream ", "shared ", "state ", "merge ", "epoch ", "the ", "of ", "fn ", "\n",
        ];
        let mut v = Vec::new();
        while v.len() < n {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            v.extend_from_slice(words[(x % words.len() as u32) as usize].as_bytes());
            if x % 7 == 0 {
                v.push((x >> 24) as u8);
            }
        }
        v.truncate(n);
        v
    }

    /// One stream is the serial coder, byte for byte.
    #[test]
    fn one_stream_is_serial() {
        let d = sample(40_000, 0x2545_F491);
        let w = sample(5_000, 7);
        let serial = encode_profile(&w, &d, FULL).expect("fits");
        let plan = Plan {
            streams: 1,
            epoch: 4096,
        };
        assert_eq!(encode(&w, &d, FULL, plan, 1), Some(serial));
    }

    /// Streams round-trip, and neither the payload nor the output depends on
    /// the thread count.
    #[test]
    fn streams_round_trip_at_any_thread_count() {
        let d = sample(70_001, 0x2545_F491);
        let w = sample(3000, 9);
        for (k, e) in [(2, 1000), (3, 4096), (4, 64), (5, 100_000)] {
            let plan = Plan { streams: k, epoch: e };
            let p1 = encode(&w, &d, FULL, plan, 1).expect("fits");
            assert_eq!(encode(&w, &d, FULL, plan, 4).as_ref(), Some(&p1));
            for t in [1, 2, 7] {
                let out = decode(&[&w], &p1, u64::MAX, FULL, plan, t, true);
                assert_eq!(out.as_deref(), Some(&d[..]), "k {k} e {e} threads {t}");
            }
            let mut bad = p1.clone();
            let last = bad.len() - 1;
            bad[last] ^= 1;
            assert_ne!(
                decode(&[&w], &bad, u64::MAX, FULL, plan, 2, true).as_deref(),
                Some(&d[..])
            );
        }
    }
}
