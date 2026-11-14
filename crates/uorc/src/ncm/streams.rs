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
    /// Bits of the probability each bit is coded at: 12 (a word's
    /// probability field zero) or 16 (the field 16).
    pub bits: u32,
    /// How the model learns ([`super::Learning`]; the word's learning
    /// fields, [`COUNT_AT`]).
    pub learning: super::Learning,
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
        self.streams > 0
            && self.epoch > 0
            && self.streams <= 1 << 16
            && (self.bits == 12 || self.bits == 16)
            && self.learning.valid()
    }

    /// One stream: the serial coder.
    pub const ONE: Plan = Plan {
        streams: 1,
        epoch: 1,
        bits: 12,
        learning: super::Learning::STANDARD,
    };

    /// Whether the frame's coder is the v0.4 coder: one stream at 12-bit
    /// probabilities (the field zero), learning as the standard model.
    #[must_use]
    pub fn serial(&self) -> bool {
        self.streams == 1 && self.bits == 12 && self.learning.standard()
    }

    /// `streams` streams of `epoch` bytes at 12-bit probabilities.
    #[must_use]
    pub const fn of(streams: usize, epoch: usize) -> Plan {
        Plan {
            streams,
            epoch,
            bits: 12,
            learning: super::Learning::STANDARD,
        }
    }
}

/// Bit of a frame word where the stream count less one starts (16 bits).
pub const STREAMS_AT: u32 = 37;
/// Bit of a frame word where the epoch's base-2 logarithm starts (5 bits).
pub const EPOCH_AT: u32 = 53;
/// Bit of a frame word where the probability field starts (5 bits): zero
/// for 12-bit probabilities, otherwise the bits (16).
pub const PROB_AT: u32 = 32;
/// Bits of a frame word below the learning fields: the profile and the plan
/// (bits 58-63 are free for the frame's own fields).
pub const PLAN_BITS: u32 = u64::BITS;
/// Bit of a frame word where the consolidation field starts (4 bits): a
/// context slot's count bits ([`super::Learning::count_bits`]), 0 for the
/// standard count.
pub const COUNT_AT: u32 = PLAN_BITS;
/// Width of the consolidation field: it holds 0 to
/// [`super::Learning::COUNT_BITS_MAX`].
pub const COUNT_FIELD: u32 = u32::BITS - super::Learning::COUNT_BITS_MAX.leading_zeros();
/// Bit of a frame word past its last field.
pub const WORD_END: u32 = COUNT_AT + COUNT_FIELD;

/// A frame word: the profile, the plan and the learning fields.
pub type Word = u128;

/// The frame word an entry names: the model `profile` in the low 32 bits,
/// the plan above ([`STREAMS_AT`], [`EPOCH_AT`]). One stream is the profile
/// itself, so every frame coded before streams keeps its word. `None` for a
/// plan the word cannot carry (an epoch not a power of two).
#[must_use]
pub fn word(profile: u32, plan: Plan) -> Option<Word> {
    if !plan.valid() || !plan.epoch.is_power_of_two() || plan.epoch.trailing_zeros() >= 32 {
        return None;
    }
    let p = if plan.bits == 12 {
        0
    } else {
        Word::from(plan.bits)
    };
    let learning = Word::from(plan.learning.count_bits) << COUNT_AT;
    let low = Word::from(profile) | p << PROB_AT | learning;
    if plan.streams == 1 {
        return Some(low);
    }
    let k = (plan.streams - 1) as Word;
    let e = Word::from(plan.epoch.trailing_zeros());
    Some(low | k << STREAMS_AT | e << EPOCH_AT)
}

/// The model profile and plan of frame `word`, when it is one [`word`]
/// writes (a single meaning per word: one stream carries no epoch).
#[must_use]
pub fn split(word: Word) -> Option<(u32, Plan)> {
    // nothing between the plan and the learning fields, nor past them
    if (word >> (EPOCH_AT + 5)) & ((1 << (COUNT_AT - EPOCH_AT - 5)) - 1) != 0
        || word >> WORD_END != 0
    {
        return None;
    }
    let learning = super::Learning {
        count_bits: ((word >> COUNT_AT) & ((1 << COUNT_FIELD) - 1)) as u32,
    };
    let bits = match (word >> PROB_AT) & 31 {
        0 => 12,
        16 => 16,
        _ => return None,
    };
    let profile = (word & Word::from(u32::MAX)) as u32;
    let streams = ((word >> STREAMS_AT) & 0xffff) as usize + 1;
    let e = ((word >> EPOCH_AT) & 31) as u32;
    if !super::valid_profile(profile) || (streams == 1 && e != 0) {
        return None;
    }
    let plan = Plan {
        streams,
        epoch: 1 << e,
        bits,
        learning,
    };
    plan.valid().then_some((profile, plan))
}

/// A frame cut into streams for a decode host: its plan, payload and merge
/// steps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Derived {
    /// The plan the payload is coded under.
    pub plan: Plan,
    /// The streams payload ([`encode`]).
    pub payload: Vec<u8>,
    /// The merge's steps ([`merge_steps`]).
    pub merge: u64,
}

/// The streams of `data` after `history` under `profile` for a decode host
/// of `host` threads, derived from the requirement that the longest chain of
/// dependent steps is the total over the host's threads: the stream count
/// steps up from one while the counted chain, coding steps `coding` plus
/// merge steps over `min(k, host)`, falls, and a stream holds at least one
/// [`Plan::stride`] unit and the host holds the decoder (`holds(k)`); once the count reaches the host's threads the
/// chain's divisor is fixed, and a further stream is kept only when it codes
/// fewer bytes (then fewer steps). For each count the epoch is decided by the least
/// archive, then steps: it starts at one epoch per stream and halves while
/// the payload shrinks. `None` when one stream is the derivation (the serial
/// coder): always on a one-thread host, where the chain of `k > 1` streams
/// is `coding` plus their merge steps, more than `coding` once a second
/// stream changes anything.
#[must_use]
pub fn derive(
    history: &[u8],
    data: &[u8],
    profile: u32,
    host: usize,
    coding: u64,
    threads: usize,
    holds: impl Fn(usize) -> bool,
) -> Option<Derived> {
    if host <= 1 || !super::valid_profile(profile) {
        return None;
    }
    let n = data.len();
    // the chain of the best count so far, as (steps, threads) over which they run
    let (mut steps, mut over) = (u128::from(coding), 1u128);
    let mut best: Option<Derived> = None;
    for k in 2..=(1 << 16) {
        let plan = Plan::of(k, 1);
        // every stream holds at least one stride unit, and the host holds
        // the decoder's working tables
        if plan.stride(n) * (k - 1) >= n || !holds(k) {
            break;
        }
        let trial = epochs(history, data, profile, k, threads)?;
        let (s, o) = (
            u128::from(coding) + u128::from(trial.merge),
            k.min(host) as u128,
        );
        // Past the host's threads the divisor no longer changes, so the
        // chain does not order the counts: there the product's own order
        // decides, fewer bytes, then fewer steps.
        let keep = if o == over {
            let b = best.as_ref().map_or(usize::MAX, |b| b.payload.len());
            trial.payload.len() < b || (trial.payload.len() == b && s < steps)
        } else {
            s * over < steps * o
        };
        if !keep {
            break;
        }
        (steps, over) = (s, o);
        best = Some(trial);
    }
    best
}

/// The epoch of `k` streams by the least payload, then steps: one epoch per
/// stream first, then halved while the payload shrinks.
fn epochs(history: &[u8], data: &[u8], profile: u32, k: usize, threads: usize) -> Option<Derived> {
    let stride = Plan::of(k, 1).stride(data.len());
    let mut epoch = stride.next_power_of_two();
    let trial = |epoch, stop| {
        let plan = Plan::of(k, epoch);
        encode_below(history, data, profile, plan, threads, stop).map(|(payload, merge)| Derived {
            plan,
            payload,
            merge,
        })
    };
    let mut best = trial(epoch, usize::MAX)?;
    while epoch > 1 {
        epoch /= 2;
        // a trial whose coders reach the best payload cannot be kept
        let Some(t) = trial(epoch, best.payload.len()) else {
            break;
        };
        if t.payload.len() >= best.payload.len() {
            break;
        }
        best = t;
    }
    Some(best)
}

/// Threads a frame's streams run on: one per stream, at most the host's.
#[must_use]
pub fn threads(plan: Plan) -> usize {
    #[cfg(target_family = "wasm")]
    {
        let _ = plan;
        1
    }
    #[cfg(not(target_family = "wasm"))]
    {
        std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(plan.streams)
    }
}

/// The merge's steps for `data` after `history` under `profile` and `plan`:
/// per epoch and stream, the buckets, match table entries, and mixer weight
/// and probability map entries the stream changed (zero for one stream).
#[must_use]
pub fn merge_steps(history: &[u8], data: &[u8], profile: u32, plan: Plan) -> Option<u64> {
    if plan.streams == 1 {
        return Some(0);
    }
    encode_counted(history, data, profile, plan, threads(plan)).map(|(_, s)| s)
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
    /// The stream's bytes coded this epoch, from absolute position `lo`
    /// (moved out of the thread's history when its run ends).
    pub(super) buf: Vec<u8>,
    lo: u32,
    /// The buckets the stream copied into its own table this epoch (its
    /// table holds 0 in every other bucket, and the stream reads those from
    /// the shared table).
    btouched: Vec<u32>,
    /// The match table entries the stream wrote this epoch (likewise).
    pub(super) mt: Vec<u32>,
    pub(super) m2t: Vec<u32>,
    /// What the stream changed this epoch, compact, once its run ends: its
    /// buckets and match table entries (the thread's tables are then clean
    /// for the next stream it runs).
    delta: Vec<(u32, Bucket)>,
    mtd: Vec<(u32, u32)>,
    m2d: Vec<(u32, u32)>,
}

impl Lane {
    #[inline(always)]
    pub(super) fn shared(&self) -> &Shared {
        self.shared
            .as_deref()
            .expect("a stream reads the shared state")
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

/// A thread's working tables: the copies of shared buckets and match table
/// entries made by the stream it is running (0 everywhere else). A stream
/// runs on them and leaves them clean, so a thread needs one set however
/// many streams it runs.
struct Own {
    sl: Table<Bucket>,
    mtab: Table<u32>,
    m2tab: Table<u32>,
    /// the history as merged (bytes no stream has finished read 0)
    hist: Vec<u8>,
}

impl Own {
    fn new(s: &Shared) -> Own {
        Own {
            sl: Table::new(s.sl.len(), Bucket([0; 16])),
            mtab: Table::new(s.mtab.len(), 0),
            m2tab: Table::new(s.m2tab.len(), 0),
            hist: s.hist.clone(),
        }
    }
}

/// Run `work` for stream `m` on the thread's tables `own`, then move what the
/// stream changed into its compact delta and leave the tables clean.
fn run_on<F>(own: &mut Own, m: &mut Model, work: F)
where
    F: FnOnce(&mut Model),
{
    std::mem::swap(&mut own.sl, &mut m.sl);
    std::mem::swap(&mut own.mtab, &mut m.mtab);
    std::mem::swap(&mut own.m2tab, &mut m.m2tab);
    std::mem::swap(&mut own.hist, &mut m.hist);
    work(m);
    let l = m.lane.as_mut().expect("a stream");
    let (lo, hi) = (l.lo as usize, (m.hoff + m.pos) as usize);
    l.buf.clear();
    l.buf.extend_from_slice(&m.hist[lo..hi]);
    m.hist[lo..hi].fill(0);
    l.btouched.sort_unstable();
    for &b in &l.btouched {
        let v = &mut m.sl[b as usize];
        l.delta.push((b, *v));
        *v = Bucket([0; 16]);
    }
    for &k in &l.mt {
        l.mtd.push((k, std::mem::take(&mut m.mtab[k as usize])));
    }
    for &k in &l.m2t {
        l.m2d.push((k, std::mem::take(&mut m.m2tab[k as usize])));
    }
    l.btouched.clear();
    l.mt.clear();
    l.m2t.clear();
    std::mem::swap(&mut own.sl, &mut m.sl);
    std::mem::swap(&mut own.mtab, &mut m.mtab);
    std::mem::swap(&mut own.m2tab, &mut m.m2tab);
    std::mem::swap(&mut own.hist, &mut m.hist);
}

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
    /// one set of working tables per thread
    own: Vec<Own>,
    /// the merge's steps so far ([`Frame::merge`])
    steps: u64,
}

impl Frame {
    fn new(history: &[&[u8]], n: usize, profile: u32, plan: Plan) -> Frame {
        let mut base = Model::new(n as u32, history, plan.learning.point(profile));
        base.pbits = plan.bits;
        let before = base.hoff as usize;
        let mut hist = std::mem::take(&mut base.hist);
        hist.resize(before + n, 0);
        let shared = Arc::new(Shared {
            sl: std::mem::replace(&mut base.sl, Table::empty()),
            mtab: std::mem::replace(&mut base.mtab, Table::empty()),
            m2tab: std::mem::replace(&mut base.m2tab, Table::empty()),
            hist,
        });
        let mut own = Own::new(&shared);
        let mut lanes = Vec::with_capacity(plan.streams);
        for i in 0..plan.streams {
            let mut m = base.clone();
            m.sl = Table::empty();
            m.hist = Vec::new();
            m.mtab = Table::empty();
            m.m2tab = Table::empty();
            let start = before + plan.range(n, i).start;
            m.lane = Some(Box::new(Lane {
                shared: Some(Arc::clone(&shared)),
                buf: Vec::new(),
                lo: start as u32,
                btouched: Vec::new(),
                mt: Vec::new(),
                m2t: Vec::new(),
                delta: Vec::new(),
                mtd: Vec::new(),
                m2d: Vec::new(),
            }));
            if i > 0 {
                m.hoff = start as u32;
                run_on(&mut own, &mut m, Model::new_byte);
            }
            lanes.push(m);
        }
        Frame {
            base,
            shared,
            lanes,
            plan,
            before,
            n,
            coded: 0,
            own: vec![own],
            steps: 0,
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
        let threads = if cfg!(target_family = "wasm") {
            1
        } else {
            threads
        };
        while self.own.len() < threads {
            self.own.push(Own::new(&self.shared));
        }
        let jobs: Vec<(usize, &mut Model, &mut Coder)> = self
            .lanes
            .iter_mut()
            .zip(coders.iter_mut())
            .enumerate()
            .map(|(i, (m, c))| (i, m, c))
            .collect();
        // stream i runs on thread i % threads: the same work either way
        let mut per: Vec<Vec<(usize, &mut Model, &mut Coder)>> =
            (0..threads).map(|_| Vec::new()).collect();
        for j in jobs {
            per[j.0 % threads].push(j);
        }
        let (work, parts) = (&work, &parts);
        let run = |own: &mut Own, batch: Vec<(usize, &mut Model, &mut Coder)>| {
            for (i, m, c) in batch {
                run_on(own, m, |m| work(i, m, c, parts[i].clone()));
            }
        };
        if threads == 1 {
            for (own, batch) in self.own.iter_mut().zip(per) {
                run(own, batch);
            }
        } else {
            let run = &run;
            std::thread::scope(|s| {
                for (own, batch) in self.own.iter_mut().zip(per) {
                    s.spawn(move || run(own, batch));
                }
            });
        }
        self.coded += parts.iter().map(|p| p.end - p.start).sum::<usize>();
        self.merge(parts, threads);
    }

    /// Merge every stream's changes into the shared state and back. Its
    /// steps: one per bucket, match table entry, and mixer weight or map
    /// entry that a stream changed this epoch, summed over the streams (none
    /// with one stream, whose merge is the identity).
    fn merge(&mut self, parts: &[Part], threads: usize) {
        if self.lanes.len() > 1 {
            self.steps += self
                .lanes
                .iter()
                .map(|m| {
                    let l = m.lane.as_ref().expect("a stream");
                    (l.delta.len() + l.mtd.len() + l.m2d.len()) as u64
                })
                .sum::<u64>();
        }
        for m in &mut self.lanes {
            m.lane.as_mut().expect("a stream").shared = None;
        }
        let shared = Arc::get_mut(&mut self.shared).expect("the streams released the state");
        // context buckets: every bucket a stream copied, merged from all its
        // versions, the buckets split among the threads by range
        let nb = shared.sl.len();
        let threads = threads.clamp(1, nb);
        let span = nb.div_ceil(threads);
        let deltas: Vec<&[(u32, Bucket)]> = self
            .lanes
            .iter()
            .map(|m| &m.lane.as_ref().expect("a stream").delta[..])
            .collect();
        let k = deltas.len();
        // every stream's delta is in bucket order: each thread merges the
        // buckets of its range, walking the deltas side by side
        let job = |t: usize, sl: &mut [Bucket]| {
            let lo = (t * span) as u32;
            let hi = lo + sl.len() as u32;
            let mut at: Vec<usize> = deltas
                .iter()
                .map(|d| d.partition_point(|&(b, _)| b < lo))
                .collect();
            let mut versions: Vec<[u32; 16]> = Vec::with_capacity(k);
            let mut groups: Vec<Group> = Vec::new();
            loop {
                let mut b = hi;
                let (mut first, mut count) = (0, 0);
                for (j, d) in deltas.iter().enumerate() {
                    if let Some(&(x, _)) = d.get(at[j]) {
                        if x < b {
                            (b, first, count) = (x, j, 1);
                        } else if x == b {
                            count += 1;
                        }
                    }
                }
                if b >= hi {
                    break;
                }
                let i = (b - lo) as usize;
                let x = sl[i].0;
                let v = if count == 1 {
                    let v = deltas[first][at[first]].1 .0;
                    at[first] += 1;
                    single(&x, &v, k)
                } else {
                    versions.clear();
                    for (j, d) in deltas.iter().enumerate() {
                        match d.get(at[j]) {
                            Some(&(y, v)) if y == b => {
                                versions.push(v.0);
                                at[j] += 1;
                            }
                            _ => versions.push(x),
                        }
                    }
                    merge_bucket(&x, &versions, &mut groups)
                };
                sl[i] = Bucket(v);
            }
        };
        if threads == 1 || cfg!(target_family = "wasm") {
            job(0, &mut shared.sl);
        } else {
            let job = &job;
            std::thread::scope(|sc| {
                for (t, sl) in shared.sl.chunks_mut(span).enumerate() {
                    sc.spawn(move || job(t, sl));
                }
            });
        }
        // match tables: the furthest position a stream wrote. Streams lie in
        // frame order, so every position a later stream wrote is further:
        // writing the streams' entries in stream order leaves the furthest.
        for m in &self.lanes {
            let l = m.lane.as_ref().expect("a stream");
            for &(key, v) in &l.mtd {
                shared.mtab[key as usize] = v;
            }
            for &(key, v) in &l.m2d {
                shared.m2tab[key as usize] = v;
            }
        }
        // the epoch's bytes, in place
        for (m, p) in self.lanes.iter().zip(parts) {
            let l = m.lane.as_ref().expect("a stream");
            let lo = self.before + p.start;
            shared.hist[lo..lo + l.buf.len()].copy_from_slice(&l.buf);
            for o in &mut self.own {
                o.hist[lo..lo + l.buf.len()].copy_from_slice(&l.buf);
            }
        }
        // mixer weights, probability maps and the match models' own slots
        let base = &mut self.base;
        let lanes = &mut self.lanes;
        let wl = (-WEIGHT_LIMIT, WEIGHT_LIMIT);
        let changed = merge_words(&mut base.w1, lanes, |m| &mut m.w1, wl)
            + merge_words(&mut base.w2, lanes, |m| &mut m.w2, wl)
            + merge_words(&mut base.w3, lanes, |m| &mut m.w3, wl)
            + merge_words(&mut base.a1.t, lanes, |m| &mut m.a1.t, (0, 65535))
            + merge_words(&mut base.a2.t, lanes, |m| &mut m.a2.t, (0, 65535))
            + merge_small(&mut base.msp, &mut base.msn, lanes, |m| {
                (&mut m.msp, &mut m.msn)
            })
            + merge_small(&mut base.m2sp, &mut base.m2sn, lanes, |m| {
                (&mut m.m2sp, &mut m.m2sn)
            })
            + merge_small(&mut base.rsp, &mut base.rsn, lanes, |m| {
                (&mut m.rsp, &mut m.rsn)
            });
        if lanes.len() > 1 {
            self.steps += changed;
        }
        for m in lanes.iter_mut() {
            m.seen = self.coded as u32;
            let l = m.lane.as_mut().expect("a stream");
            l.delta.clear();
            l.mtd.clear();
            l.m2d.clear();
            l.buf.clear();
            l.shared = Some(Arc::clone(&self.shared));
        }
    }
}

/// Every stream's copy of a word table merged into `base` and back: each
/// entry moves by the mean change of the streams that changed it. The
/// number of (stream, entry) changes.
fn merge_words<T, F>(base: &mut [T], lanes: &mut [Model], get: F, (lo, hi): (i32, i32)) -> u64
where
    T: Copy + PartialEq + Into<i32> + TryFrom<i32>,
    F: Fn(&mut Model) -> &mut [T],
{
    const CHUNK: usize = 64;
    let mut steps = 0;
    for c in (0..base.len()).step_by(CHUNK) {
        let e = (c + CHUNK).min(base.len());
        if lanes.iter_mut().all(|m| get(m)[c..e] == base[c..e]) {
            continue;
        }
        for j in c..e {
            steps += merge_word(base, lanes, &get, j, (lo, hi));
        }
    }
    steps
}

fn merge_word<T, F>(
    base: &mut [T],
    lanes: &mut [Model],
    get: &F,
    j: usize,
    (lo, hi): (i32, i32),
) -> u64
where
    T: Copy + PartialEq + Into<i32> + TryFrom<i32>,
    F: Fn(&mut Model) -> &mut [T],
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
            return 0;
        }
        let v = (i64::from(x.into()) + d / changed).clamp(i64::from(lo), i64::from(hi)) as i32;
        let Ok(v) = T::try_from(v) else {
            unreachable!("clamped into the table's range")
        };
        base[j] = v;
        for m in lanes.iter_mut() {
            get(m)[j] = v;
        }
        changed as u64
    }
}

/// The match models' slots (probability and count apart) merged as context
/// slots. The number of (stream, slot) changes.
fn merge_small<const N: usize, F>(
    bp: &mut [u16; N],
    bn: &mut [u8; N],
    lanes: &mut [Model],
    get: F,
) -> u64
where
    F: Fn(&mut Model) -> (&mut [u16; N], &mut [u8; N]),
{
    let pack = |p: u16, n: u8| u32::from(p) | (u32::from(n) << 16);
    let mut v = Vec::with_capacity(lanes.len());
    let mut steps = 0;
    for j in 0..N {
        v.clear();
        let b = pack(bp[j], bn[j]);
        for m in lanes.iter_mut() {
            let (p, n) = get(m);
            v.push(pack(p[j], n[j]));
        }
        steps += v.iter().filter(|&&x| x != b).count() as u64;
        let x = merge_slot(pack(bp[j], bn[j]), &v);
        bp[j] = (x & 0xffff) as u16;
        bn[j] = (x >> 16) as u8;
        for m in lanes.iter_mut() {
            let (p, n) = get(m);
            p[j] = bp[j];
            n[j] = bn[j];
        }
    }
    steps
}

/// One slot (probability in the low 16 bits, count above) merged from the
/// base and the streams' versions of it.
fn merge_slot(b: u32, ls: &[u32]) -> u32 {
    let mut acc = Acc::default();
    for &l in ls {
        acc.add(b, l);
    }
    acc.done(b)
}

/// A slot's merge in progress: the streams' added sums and counts.
#[derive(Clone, Copy, Default)]
struct Acc {
    sum: i64,
    dn: i64,
    dp: i64,
    sat: bool,
    changed: i64,
}

impl Acc {
    /// Add a stream's version `l` of the slot whose base is `b`.
    #[inline]
    fn add(&mut self, b: u32, l: u32) {
        if l == b {
            return;
        }
        let (pb, nb) = (i64::from(b & 0xffff), i64::from(b >> 16));
        self.changed += 1;
        let (p, n) = (i64::from(l & 0xffff), i64::from(l >> 16));
        // a stream whose count fell below the base's cleared the slot (its
        // bucket was recycled and came back): it adds what it learned since
        let (op, on) = if n < nb {
            (i64::from(SLOT_FRESH), 0)
        } else {
            (pb, nb)
        };
        self.sat |= n >= 127 || nb >= 127;
        self.sum += p * (2 * n + 1) - op * (2 * on + 1);
        self.dn += n - on;
        self.dp += p - op;
    }

    /// The merged slot over base `b`.
    #[inline]
    fn done(&self, b: u32) -> u32 {
        if self.changed == 0 {
            return b;
        }
        let (pb, nb) = (i64::from(b & 0xffff), i64::from(b >> 16));
        let tot = nb + self.dn;
        let p = if self.sat {
            pb + self.dp / self.changed
        } else {
            (pb * (2 * nb + 1) + self.sum) / (2 * tot.max(0) + 1)
        };
        (p.clamp(0, 65535) as u32) | ((tot.clamp(0, 127) as u32) << 16)
    }
}

/// One candidate context of a bucket being merged: its origin (the base's
/// bucket, or a fresh one with the tag a stream gave it) and the streams'
/// added sums per slot.
#[derive(Clone)]
struct Group {
    fresh: bool,
    origin: [u32; 16],
    acc: [Acc; 16],
}

/// [`merge_bucket`] when one stream of `k` changed the bucket to `v`: a sum
/// with one term is that term, and a context the stream put in the bucket
/// competes with the base's (which the other streams still hold) by use.
fn single(x: &[u32; 16], v: &[u32; 16], k: usize) -> [u32; 16] {
    if k == 1 || same_context(x, v) {
        return *v;
    }
    let key = |m: &[u32; 16]| (m[1] >> 16, u32::MAX - tag(m));
    if key(v) > key(x) {
        *v
    } else {
        *x
    }
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
fn merge_bucket(base: &[u32; 16], ls: &[[u32; 16]], groups: &mut Vec<Group>) -> [u32; 16] {
    // groups by tag: the base's context stays a candidate only while some
    // stream holds it; every other tag a stream gave the bucket starts fresh
    // (a stream that recycled the bucket and came back to the base's tag
    // holds a fresh context, not the base's: one of its counts fell)
    groups.clear();
    if ls.iter().any(|l| same_context(base, l)) {
        groups.push(Group {
            fresh: false,
            origin: *base,
            acc: [Acc::default(); 16],
        });
    }
    for l in ls {
        if l == base {
            continue;
        }
        let g = if same_context(base, l) {
            0
        } else if let Some(g) = groups
            .iter()
            .position(|g| g.fresh && tag(&g.origin) == tag(l))
        {
            g
        } else {
            let mut origin = [SLOT_FRESH; 16];
            origin[0] |= tag(l) << 16;
            groups.push(Group {
                fresh: true,
                origin,
                acc: [Acc::default(); 16],
            });
            groups.len() - 1
        };
        let g = &mut groups[g];
        for ((a, &o), &v) in g.acc.iter_mut().zip(&g.origin).zip(l).skip(1) {
            a.add(o, v);
        }
    }
    let mut best: Option<((u32, u32), [u32; 16])> = None;
    for g in groups.iter() {
        let mut m = g.origin;
        for ((m, a), &o) in m.iter_mut().zip(&g.acc).zip(&g.origin).skip(1) {
            *m = a.done(o);
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
    encode_counted(history, data, profile, plan, threads).map(|(p, _)| p)
}

/// [`encode`], with the merge's steps (see [`merge_steps`]).
#[must_use]
pub fn encode_counted(
    history: &[u8],
    data: &[u8],
    profile: u32,
    plan: Plan,
    threads: usize,
) -> Option<(Vec<u8>, u64)> {
    encode_below(history, data, profile, plan, threads, usize::MAX)
}

/// [`encode_counted`], or `None` once the coders' output reaches `stop`
/// bytes (a payload that long cannot be kept).
fn encode_below(
    history: &[u8],
    data: &[u8],
    profile: u32,
    plan: Plan,
    threads: usize,
    stop: usize,
) -> Option<(Vec<u8>, u64)> {
    if !super::valid_profile(profile) || !plan.valid() {
        return None;
    }
    u32::try_from(data.len()).ok()?;
    let a = (history, data, profile, plan, threads, stop);
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 was detected on this processor just above.
            return unsafe { encode_avx2(a) };
        }
        encode_k::<super::Scalar>(a)
    }
    #[cfg(target_arch = "aarch64")]
    {
        encode_k::<super::arm::Neon>(a)
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        encode_k::<super::wasm::Simd128>(a)
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )))]
    {
        encode_k::<super::Scalar>(a)
    }
}

type EncodeArgs<'a> = (&'a [u8], &'a [u8], u32, Plan, usize, usize);

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
fn encode_avx2(a: EncodeArgs<'_>) -> Option<(Vec<u8>, u64)> {
    encode_k::<super::x86::Avx2>(a)
}

#[inline(always)]
fn encode_k<K: Kernel>(
    (history, data, profile, plan, threads, stop): EncodeArgs<'_>,
) -> Option<(Vec<u8>, u64)> {
    let mut f = Frame::new(&[history], data.len(), profile, plan);
    let out = if f.base.stride == NINP_NARROW {
        encode_frame::<K, NINP_NARROW>(&mut f, data, threads, stop)
    } else {
        encode_frame::<K, NINP>(&mut f, data, threads, stop)
    };
    Some((out?, f.steps))
}

#[inline(always)]
fn encode_frame<K: Kernel, const W: usize>(
    f: &mut Frame,
    data: &[u8],
    threads: usize,
    stop: usize,
) -> Option<Vec<u8>> {
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
        if coders.iter().map(|c| c.out.len()).sum::<usize>() >= stop {
            return None;
        }
    }
    let mut out = Vec::new();
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    for (j, c) in coders.iter_mut().enumerate() {
        c.out.push((c.x1 >> 24) as u8);
        // a stream's coder abandons the encoding past twice its length and
        // 64 bytes, as a frame's coder does (`ncm::encode`)
        let len = ((j + 1) * r).min(f.n) - (j * r).min(f.n);
        if c.out.len() > 2 * len + 64 {
            return None;
        }
    }
    for c in &coders[..k - 1] {
        out.extend_from_slice(&(c.out.len() as u32).to_be_bytes());
    }
    for c in &coders {
        out.extend_from_slice(&c.out);
    }
    Some(out)
}

#[inline(always)]
fn code<K: Kernel, const W: usize>(m: &mut Model, c: &mut Coder, bytes: &[u8]) {
    for &byte in bytes {
        for k in (0..8).rev() {
            let y = (byte >> k) & 1 == 1;
            let p = m.predict::<K, W>();
            let xm = c.x1 + ((u64::from(c.x2 - c.x1) * u64::from(p)) >> m.pbits) as u32;
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
    let a = (
        history, payload, n as usize, profile, plan, threads, canonical,
    );
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
        if !c.ok || (canonical && (s.len() != c.emitted + 1 || s[c.emitted] != (c.x1 >> 24) as u8))
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
            let xm = c.x1 + ((u64::from(c.x2 - c.x1) * u64::from(p)) >> m.pbits) as u32;
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
            if x.is_multiple_of(7) {
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
        let plan = Plan::of(1, 4096);
        assert_eq!(encode(&w, &d, FULL, plan, 1), Some(serial));
    }

    /// Streams round-trip, and neither the payload nor the output depends on
    /// the thread count.
    #[test]
    fn streams_round_trip_at_any_thread_count() {
        let d = sample(70_001, 0x2545_F491);
        let w = sample(3000, 9);
        for (k, e) in [(2, 1000), (3, 4096), (4, 64), (5, 100_000)] {
            let plan = Plan::of(k, e);
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
