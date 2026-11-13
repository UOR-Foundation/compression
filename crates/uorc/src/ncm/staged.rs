//! The staged encoder: the input side of the model on its own core, ahead of
//! the mixers.
//!
//! In the encoder the coded bit is the data bit, so everything
//! [`Model::sense`] reads (the context slots, their buckets, tags and counts,
//! the match, short match and repeat models) is a function of the input
//! bytes alone: the mixers, the APMs and the coder never write back into it.
//! The input side therefore runs ahead on one core and hands the mixing side
//! each bit's inputs (the mixer input row and the match state), and the
//! mixing side runs the same arithmetic in the same order as the one-core
//! encoder. The payload is the same bytes whichever path codes it.

use super::{
    adapt_slot, bucket_in, h, prefetch, tick, Bucket, Coder, Kernel, Model, Table, Tables, NC,
};
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{mpsc, Arc};

/// The cores a compression may lend its encoders: a pool of spare cores (the
/// host's threads less those already coding), shared by every thread the
/// compression runs. An encoder on such a thread takes what is spare when it
/// starts and gives it back when it ends; the payload never depends on how
/// many it took.
pub struct Cores {
    free: AtomicIsize,
}

thread_local! {
    static POOL: RefCell<Option<Arc<Cores>>> = const { RefCell::new(None) };
}

/// A core held from a [`Cores`] pool (by a thread the compression starts),
/// given back when dropped.
pub struct Held(Arc<Cores>);

impl Drop for Held {
    fn drop(&mut self) {
        self.0.free.fetch_add(1, Ordering::AcqRel);
    }
}

impl Held {
    /// Run `f` on this thread as a holder of the core: encoders it calls
    /// borrow spare cores from the same pool.
    pub fn run<T>(self, f: impl FnOnce() -> T) -> T {
        let pool = Arc::clone(&self.0);
        let t = pool.enter(f);
        drop(self);
        t
    }
}

/// A thread's own core, spare while the thread only waits (a compression's
/// thread joining the strategies it started), taken back when dropped.
pub struct Idle(Arc<Cores>);

impl Drop for Idle {
    fn drop(&mut self) {
        self.0.free.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Spare cores lent to one encoder, given back when dropped.
pub(super) struct Lent(Arc<Cores>, usize);

impl Drop for Lent {
    fn drop(&mut self) {
        self.0.free.fetch_add(self.1 as isize, Ordering::AcqRel);
    }
}

impl Cores {
    /// A pool of `spare` cores.
    pub fn new(spare: usize) -> Arc<Cores> {
        Arc::new(Cores {
            free: AtomicIsize::new(isize::try_from(spare).unwrap_or(isize::MAX)),
        })
    }

    /// The pool of the compression this thread runs for, if any.
    pub fn current() -> Option<Arc<Cores>> {
        POOL.with(|p| p.borrow().clone())
    }

    /// Run `f` on this thread for the pool's compression.
    pub fn enter<T>(self: &Arc<Self>, f: impl FnOnce() -> T) -> T {
        let before = POOL.with(|p| p.borrow_mut().replace(Arc::clone(self)));
        let t = f();
        POOL.with(|p| *p.borrow_mut() = before);
        t
    }

    /// One core for a thread the compression starts beside its own, taken
    /// whether or not one is spare (the thread runs either way).
    pub fn hold(self: &Arc<Self>) -> Held {
        self.free.fetch_sub(1, Ordering::AcqRel);
        Held(Arc::clone(self))
    }

    /// This thread's core, spare until the guard drops.
    pub fn idle(self: &Arc<Self>) -> Idle {
        self.free.fetch_add(1, Ordering::AcqRel);
        Idle(Arc::clone(self))
    }

    /// The cores the pool is short of (negative) or has spare (positive).
    pub fn balance(&self) -> isize {
        self.free.load(Ordering::Acquire)
    }
}

/// The cores an encoder on this thread borrows for its next stretch: what it
/// holds in `lent`, plus every core now spare, less what the pool is short
/// of. `0` when it codes on its own core: no pool or no core to lend.
pub(super) fn lend(lent: &mut Option<Lent>) -> usize {
    if cfg!(target_family = "wasm") {
        *lent = None;
        return 0;
    }
    let Some(pool) = Cores::current() else {
        return 0;
    };
    let held = lent.as_ref().map_or(0, |l| l.1);
    let mut free = pool.free.load(Ordering::Acquire);
    loop {
        // take every spare core, or give back up to what the pool is short of
        let n = (held as isize + free).max(0) as usize;
        let after = free - (n as isize - held as isize);
        match pool
            .free
            .compare_exchange_weak(free, after, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {
                if let Some(l) = lent.as_mut() {
                    l.1 = n;
                } else {
                    *lent = Some(Lent(pool, n));
                }
                if n == 0 {
                    *lent = None;
                }
                return n;
            }
            Err(now) => free = now,
        }
    }
}

/// The input bytes the stages hand on at a time with `lent` cores, for a
/// model of `na` contexts and mixer rows of `w` inputs: as many as keep the
/// batches in flight between two cores (one written, one queued, one read)
/// inside one core's share of the host's outermost cache level, at the bytes
/// each input byte hands on (the input rows, or the hashes, ranges, rows and
/// site probabilities).
pub(super) fn batch(na: usize, w: usize, lent: usize) -> usize {
    let host = crate::region::Host::current();
    let share = host
        .caches
        .last()
        .map_or(host.page, |c| c.bytes / host.threads.max(1));
    let per_byte = if lent == 1 {
        8 * (2 * w + 1)
    } else {
        2 * na * (4 + 2) + 8 * (2 * (w - na) + 1) + 8 * 2 * na
    };
    (usize::try_from(share).unwrap_or(usize::MAX) / (3 * per_byte)).max(1)
}

impl Model {
    /// This model as its input side (the slots and the byte-level models) and
    /// its mixing side (the mixers, the APMs, the twin, the learned gains):
    /// each half keeps the scalar state both read, and neither holds the
    /// other's tables.
    fn split(mut self) -> (Model, Model) {
        let sl = std::mem::replace(&mut self.sl, Table::new(0, Bucket([0; 16])));
        let mtab = std::mem::replace(&mut self.mtab, Table::new(0, 0));
        let m2tab = std::mem::replace(&mut self.m2tab, Table::new(0, 0));
        let hist = std::mem::take(&mut self.hist);
        let rcnt = std::mem::take(&mut self.rcnt);
        let mut mix = self.clone();
        let mut inputs = self;
        inputs.sl = sl;
        inputs.mtab = mtab;
        inputs.m2tab = m2tab;
        inputs.hist = hist;
        inputs.rcnt = rcnt;
        inputs.w1 = Vec::new();
        inputs.w2 = Vec::new();
        inputs.w3 = Vec::new();
        inputs.a1 = super::Apm::new(0);
        inputs.a2 = super::Apm::new(0);
        inputs.twin = None;
        inputs.gain = None;
        mix.mtab = Table::new(0, 0);
        (inputs, mix)
    }

    /// The model the two sides of [`Model::split`] make again.
    fn join(mut self, mix: Model) -> Model {
        self.w1 = mix.w1;
        self.w2 = mix.w2;
        self.w3 = mix.w3;
        self.a1 = mix.a1;
        self.a2 = mix.a2;
        self.twin = mix.twin;
        self.gain = mix.gain;
        self
    }

    /// The mixing side's [`Model::advance`]: only the partial byte, the last
    /// four bytes, the position and the byte count, which the mixers, APMs
    /// and plasticity read.
    #[inline(always)]
    fn advance_mix(&mut self, y: bool) {
        self.c0 = (self.c0 << 1) | u32::from(y);
        self.bit += 1;
        if self.bit == 8 {
            self.pos += 1;
            self.seen += 1;
            self.c4 = (self.c4 << 8) | (self.c0 & 255);
            self.c0 = 1;
            self.bit = 0;
        }
    }
}

/// One coded bit on the mixing side, from the inputs `m.st` and match state
/// `ms` the input side read: the same arithmetic in the same order as the
/// one-core loop ([`super::encode_bits`]).
#[inline(always)]
fn mix_bit<K: Kernel, const W: usize>(
    m: &mut Model,
    coders: &mut [Coder; 2],
    learning: bool,
    y: bool,
    ms: u32,
) {
    let p = m.mix::<K, W>(ms);
    let c0 = (m.c0 << 1) | u32::from(y);
    if m.bit == 7 {
        m.prefetch_mixers((m.c4 << 8) | (c0 & 255), 1);
    } else {
        m.prefetch_mixers(m.c4, c0);
    }
    let [main, other] = coders;
    if main.live {
        main.code(y, p);
    }
    if other.live {
        let q = m.twin.as_deref().map_or(2048, |t| t.pr);
        other.code(y, q);
    }
    if learning {
        m.learn::<K, W>(y);
    }
    m.mix_update::<K, W>(y);
    m.advance_mix(y);
}

/// What a stretch on lent cores ends with: the model, joined again, and where
/// it stopped, or `None` when both coders were abandoned.
pub(super) type Stretch = Option<(Model, usize)>;

/// Codes `data[at..]` on `lent` cores besides this one, `batch` bytes at a
/// time ([`batch`]), until the end or a batch boundary at which the pool has cores spare
/// or is short of some (the caller lends again and goes on): the same coder
/// output, learned gains and model as the one-core loop over those bytes.
#[inline(always)]
pub(super) fn stretch<K: Kernel, const W: usize>(
    m: Model,
    data: &[u8],
    at: usize,
    coders: &mut [Coder; 2],
    learning: bool,
    lent: usize,
    batch: usize,
) -> Stretch {
    if lent == 1 {
        two_sides::<K, W>(m, data, at, coders, learning, batch)
    } else {
        // the byte-level models on one core, a site range on each other
        sites::<K, W>(m, data, at, coders, learning, lent - 1, batch)
    }
}

/// Whether the mixing side, at a batch boundary, should end the stretch: the
/// pool's balance moved.
fn rebalance() -> bool {
    Cores::current().is_some_and(|p| p.balance() != 0)
}

/// [`stretch`] with the whole input side on one lent core.
#[inline(always)]
fn two_sides<K: Kernel, const W: usize>(
    m: Model,
    data: &[u8],
    at: usize,
    coders: &mut [Coder; 2],
    learning: bool,
    batch: usize,
) -> Stretch {
    let (inputs, mut m) = m.split();
    let stop = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let (full, filled) = mpsc::sync_channel::<Batch>(1);
        let stop = &stop;
        let side = scope.spawn(move || {
            let mut m = inputs;
            for chunk in data[at..].chunks(batch) {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                let mut b = Batch {
                    rows: Vec::with_capacity(chunk.len() * 8 * W),
                    ms: Vec::with_capacity(chunk.len() * 8),
                };
                for &byte in chunk {
                    for k in (0..8).rev() {
                        let y = (byte >> k) & 1;
                        let ms = m.sense();
                        // every input is a stretched probability (±2047),
                        // a match vote (±1024) or the bias (256)
                        b.rows.extend(m.st[..W].iter().map(|&v| v as i16));
                        b.ms.push(ms as u8);
                        m.prefetch_inputs(u32::from(y));
                        m.sense_update(y == 1);
                    }
                }
                if full.send(b).is_err() {
                    break;
                }
            }
            m
        });
        let mut pos = at;
        while let Ok(b) = filled.recv() {
            let chunk = &data[pos..(pos + batch).min(data.len())];
            tick(pos, data.len());
            for (i, &byte) in chunk.iter().enumerate() {
                if !coders[0].live && !coders[1].live {
                    stop.store(true, Ordering::Release);
                    return None;
                }
                for k in (0..8).rev() {
                    let q = i * 8 + 7 - k;
                    for (s, &v) in m.st[..W].iter_mut().zip(&b.rows[q * W..q * W + W]) {
                        *s = i32::from(v);
                    }
                    mix_bit::<K, W>(
                        &mut m,
                        coders,
                        learning,
                        (byte >> k) & 1 == 1,
                        u32::from(b.ms[q]),
                    );
                }
            }
            pos += chunk.len();
            if rebalance() {
                stop.store(true, Ordering::Release);
            }
        }
        drop(filled);
        let inputs = side.join().expect("input side");
        Some((inputs.join(m), pos))
    })
}

/// [`stretch`] with the slot table cut into `ranges` site ranges, each walked
/// on its own lent core, and the byte-level models on another.
#[inline(always)]
fn sites<K: Kernel, const W: usize>(
    m: Model,
    data: &[u8],
    at: usize,
    coders: &mut [Coder; 2],
    learning: bool,
    ranges: usize,
    batch: usize,
) -> Stretch {
    let (mut inputs, mut m) = m.split();
    let mut sl = std::mem::replace(&mut inputs.sl, Table::new(0, Bucket([0; 16])));
    let (na, mask, t) = (m.na, m.mask, m.t);
    let cut = Cut::of(mask, ranges);
    let stop = AtomicBool::new(false);
    let (inputs, pos) = std::thread::scope(|scope| {
        let mut to_sites = Vec::new();
        let mut from_sites = Vec::new();
        let mut rest: &mut [Bucket] = &mut sl;
        for r in 0..cut.ranges {
            let (a, b) = (cut.first_pair(r), cut.first_pair(r + 1));
            let (part, tail) = std::mem::take(&mut rest).split_at_mut((b - a) * 2);
            rest = tail;
            let (tx, rx) = mpsc::sync_channel::<Arc<Bytes>>(1);
            let (stx, srx) = mpsc::sync_channel::<Sites>(1);
            to_sites.push(tx);
            from_sites.push(srx);
            let lo = (a * 32) as u32;
            let r = r as u16;
            scope
                .spawn(move || walk_sites(part, r, lo, mask, na, t, &data[at..], batch, &rx, &stx));
        }
        let (full, filled) = mpsc::sync_channel::<Arc<Bytes>>(1);
        let stop = &stop;
        let side = scope.spawn(move || {
            let mut m = inputs;
            'batches: for chunk in data[at..].chunks(batch) {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                let b = Arc::new(read_bytes::<W>(&mut m, chunk, cut));
                for tx in &to_sites {
                    if tx.send(Arc::clone(&b)).is_err() {
                        break 'batches;
                    }
                }
                if full.send(b).is_err() {
                    break;
                }
            }
            m
        });
        let mut sites: Vec<Sites> = Vec::with_capacity(from_sites.len());
        let mut cur = vec![0usize; from_sites.len()];
        let tail = W - na;
        let mut pos = at;
        while let Ok(b) = filled.recv() {
            sites.clear();
            for rx in &from_sites {
                let Ok(s) = rx.recv() else {
                    stop.store(true, Ordering::Release);
                    return None;
                };
                sites.push(s);
            }
            cur.fill(0);
            let chunk = &data[pos..(pos + batch).min(data.len())];
            tick(pos, data.len());
            for (i, &byte) in chunk.iter().enumerate() {
                if !coders[0].live && !coders[1].live {
                    stop.store(true, Ordering::Release);
                    return None;
                }
                for k in (0..8).rev() {
                    let bit = 7 - k;
                    let q = i * 8 + bit;
                    let o = (i * 2 + bit / 4) * na;
                    for (j, &r) in b.own[o..o + na].iter().enumerate() {
                        let r = usize::from(r);
                        m.st[j] = i32::from(sites[r].0[cur[r]]);
                        cur[r] += 1;
                    }
                    for (s, &v) in m.st[na..W]
                        .iter_mut()
                        .zip(&b.rows[q * tail..q * tail + tail])
                    {
                        *s = i32::from(v);
                    }
                    mix_bit::<K, W>(
                        &mut m,
                        coders,
                        learning,
                        (byte >> k) & 1 == 1,
                        u32::from(b.ms[q]),
                    );
                }
            }
            pos += chunk.len();
            if rebalance() {
                stop.store(true, Ordering::Release);
            }
        }
        drop(filled);
        drop(from_sites);
        Some((side.join().expect("byte side"), pos))
    })?;
    let mut inputs = inputs;
    inputs.sl = sl;
    Some((inputs.join(m), pos))
}

/// One batch of coded bits as the whole input side read them: each bit's
/// mixer input row (`W` inputs, all within ±2047) and its match state.
struct Batch {
    rows: Vec<i16>,
    ms: Vec<u8>,
}

/// The cut of the slot table into site ranges: whole bucket pairs (a bucket
/// and its neighbour, 32 slots, the most one lookup touches), pair `p` of
/// `2^k` in range `(p * ranges) >> k`, so each range is one run of pairs.
#[derive(Clone, Copy)]
struct Cut {
    ranges: u32,
    k: u32,
}

impl Cut {
    fn of(mask: u32, ranges: usize) -> Cut {
        let k = (mask + 1).trailing_zeros() - 5;
        Cut {
            ranges: u32::try_from(ranges).unwrap_or(u32::MAX).clamp(1, 1 << k),
            k,
        }
    }

    /// The range holding the pair a context hash looks up.
    #[inline(always)]
    fn range(self, hh: u32, mask: u32) -> u16 {
        ((u64::from((hh & mask) >> 5) * u64::from(self.ranges)) >> self.k) as u16
    }

    /// The first pair of range `r` (the end of the table for `r = ranges`).
    fn first_pair(self, r: u32) -> usize {
        (u64::from(r) << self.k).div_ceil(u64::from(self.ranges)) as usize
    }
}

/// What the byte-level side hands the site ranges and the mixing side for one
/// batch: for each byte and nibble the `na` context hashes and the range
/// each one's pair lies in, and for each bit the inputs from `na` on and the
/// match state.
struct Bytes {
    hs: Vec<u32>,
    own: Vec<u16>,
    rows: Vec<i16>,
    ms: Vec<u8>,
}

/// One site range's slot inputs for a batch: for each bit, in context order,
/// the stretched probability of each context whose pair lies in the range.
struct Sites(Vec<i16>);

/// Site range `r`'s walk: the buckets `sl` holding the table's slots from
/// `lo`, for every coded bit choosing, reading and adapting the slots of the
/// contexts whose pair lies here, in the model's order (every read of a bit
/// before its adapts, the adapts in context order). A bucket pair and every
/// slot in it live in one range, so the ranges never share a slot and each
/// range's slots evolve exactly as in the whole table.
#[allow(clippy::too_many_arguments)]
fn walk_sites(
    sl: &mut [Bucket],
    r: u16,
    lo: u32,
    mask: u32,
    na: usize,
    t: &Tables,
    data: &[u8],
    batch: usize,
    from: &mpsc::Receiver<Arc<Bytes>>,
    to: &mpsc::SyncSender<Sites>,
) {
    // the contexts whose pair lies here (`own[..owned]`) and their buckets
    let mut bb = [0u32; NC];
    let mut own = [0usize; NC];
    let mut owned = 0usize;
    // a pair elsewhere is not loaded: its prefetch reads this range's first
    let pair = |hh: u32, here: bool| {
        if here {
            (((hh & mask) & !31) - lo) as usize >> 4
        } else {
            0
        }
    };
    for chunk in data.chunks(batch) {
        let Ok(bytes) = from.recv() else {
            return;
        };
        let mut out = Vec::with_capacity(chunk.len() * 8 * na);
        for (i, &byte) in chunk.iter().enumerate() {
            let mut c0 = 1u32;
            for bit in 0..8u32 {
                if bit == 0 || bit == 4 {
                    let at = (i * 2 + bit as usize / 4) * na;
                    let (hs, ow) = (&bytes.hs[at..at + na], &bytes.own[at..at + na]);
                    owned = 0;
                    for (j, &o) in ow.iter().enumerate() {
                        own[owned] = j;
                        owned += usize::from(o == r);
                    }
                    for k in 0..owned {
                        bb[k] = bucket_in(sl, lo, mask, hs[own[k]]);
                    }
                    if bit == 0 {
                        // the coded bytes are known: load this byte's second
                        // nibble and the next byte's first
                        let ahead = (i * 2 + 1) * na;
                        let end = (ahead + 2 * na).min(bytes.hs.len());
                        for (&hh, &o) in bytes.hs[ahead..end].iter().zip(&bytes.own[ahead..end]) {
                            prefetch(&sl[pair(hh, o == r)]);
                        }
                    }
                }
                let part = if bit < 4 {
                    c0
                } else {
                    (c0 & ((1 << (bit - 4)) - 1)) | (1 << (bit - 4))
                };
                for &b in &bb[..owned] {
                    let i = b + part - lo;
                    let p = sl[i as usize >> 4].0[i as usize & 15] & 0xffff;
                    out.push(t.stretch[(p >> 4) as usize]);
                }
                let y = (byte >> (7 - bit)) & 1 == 1;
                for &b in &bb[..owned] {
                    let i = b + part - lo;
                    adapt_slot(&mut sl[i as usize >> 4].0[i as usize & 15], y, &t.rate);
                }
                c0 = (c0 << 1) | u32::from(y);
            }
        }
        if to.send(Sites(out)).is_err() {
            return;
        }
    }
}

/// The byte-level side's batch for `chunk`: what [`walk_sites`] and the
/// mixing side read, from the input side `m` with its slot table taken out.
#[inline(always)]
fn read_bytes<const W: usize>(m: &mut Model, chunk: &[u8], cut: Cut) -> Bytes {
    let (na, mask) = (m.na, m.mask);
    let mut b = Bytes {
        hs: Vec::with_capacity(chunk.len() * 2 * na),
        own: Vec::with_capacity(chunk.len() * 2 * na),
        rows: Vec::with_capacity(chunk.len() * 8 * (W - na)),
        ms: Vec::with_capacity(chunk.len() * 8),
    };
    for &byte in chunk {
        // the two nibbles' hashes, as Model::sense_slots forms them
        for nc in [0, u32::from(byte >> 4) | 16] {
            for &k in &m.act[..na] {
                let hh = h(m.hb[k].wrapping_add(nc.wrapping_mul(0x2F0B_4C27)));
                b.hs.push(hh);
                b.own.push(cut.range(hh, mask));
            }
        }
        for k in (0..8).rev() {
            let ms = m.sense_models();
            // every input is a stretched probability (±2047), a match vote
            // (±1024) or the bias (256)
            b.rows.extend(m.st[na..W].iter().map(|&v| v as i16));
            b.ms.push(ms as u8);
            m.models_update((byte >> k) & 1 == 1);
        }
    }
    b
}
