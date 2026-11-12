//! The ncm coder, natively: uorc's neuromorphic context-mixing kernel,
//! byte-identical to the LexLean model (`Uorc.Codec.NcmEncode`/`NcmDecode`,
//! generated from `client/gen/ncm_model.py`) and to the kernel graph that
//! carries its meaning in every ncm archive.
//!
//! Per bit, 17 hashed contexts each hold an adaptive probability: orders 0-4,
//! 6 and 8, the current and previous word, sparse and 4-byte-aligned
//! contexts, two contexts from a record model that learns the input's row
//! length, and the byte a long match predicts. Each context's slots live in a
//! 16-slot bucket tagged with a check byte; a bucket that does not carry the
//! context's tag is looked for in its neighbour and otherwise recycled (the
//! less used of the two), so colliding contexts do not share statistics. A
//! match model adds its own probability and a vote, and a bias completes 20
//! inputs. Three integer neural mixers (selected by the partial byte, and by
//! the previous byte or the byte before it with the bit position) combine them
//! in the logistic domain, and every weight learns by a bounded Hebbian rule,
//! `w += x * err * rate`, clamped to +-2^22, with a plasticity that decays
//! from 1/512 to 3/8192 as the input is seen. Two adaptive probability maps
//! (selected by the partial byte and by an order-3 hash) refine the mixed
//! probability, which drives a 32-bit binary arithmetic coder.
//!
//! A payload is the input length (4 bytes, big-endian) and the coder output.
//!
//! The plasticity of every learning part (mixer weights, context slots and
//! the probability maps) is the one the Prism model `Uorc.Neuromorphic`
//! (`src/Uorc/Neuromorphic.lex.tex`) states, with its bounds kernel-checked
//! there; [`standard`] holds its constants.

/// ncm's plasticity as `Uorc.Neuromorphic` states it. A test holds every
/// constant here equal to the model's, and the coder reads them from here.
pub mod standard {
    /// The mixers.
    pub const MIXERS: usize = 3;
    /// Inputs per mixer: the hashed contexts, the match model's two, a bias,
    /// the short match model's two and the repeat model's two (a profile
    /// word may leave any of them out).
    pub const MIXER_INPUTS: usize = super::NC + 7;
    /// Hashed contexts, each a slot that adapts toward the coded bit.
    pub const CONTEXTS: usize = super::NC;
    /// Every mixer weight stays within this bound (`hebbianBoundsHold`).
    pub const WEIGHT_LIMIT: i32 = 1 << 22;
    /// A fresh weight on the contexts, the match model's inputs and the bias:
    /// 1/4 in 16-bit fixed point. The short match and repeat inputs start at
    /// 0 (`weightInitialAdded`).
    pub const WEIGHT_INITIAL: i32 = 1 << 14;
    /// A mixer input's magnitude bound (stretched probabilities, vote, bias).
    pub const INPUT_LIMIT: i32 = 2047;
    /// A mixer error's magnitude bound (12-bit probabilities).
    pub const ERROR_LIMIT: i32 = 4096;
    /// A weight learns `x * err * plasticity / 2^PLASTICITY_SHIFT` per bit.
    pub const PLASTICITY_SHIFT: u32 = 16;
    /// The plasticity's floor, approached as the input is seen.
    pub const PLASTICITY_FLOOR: i64 = 24;
    /// The plasticity above the floor at the first byte.
    pub const PLASTICITY_GAIN: i64 = 104;
    /// Bytes after which the plasticity above the floor has halved.
    pub const PLASTICITY_HALF_LIFE: i64 = 4096;
    /// A weight's largest change in one bit (`laneFitHolds`).
    pub const STEP_LIMIT: i32 = 16376;
    /// A slot's count saturates here: the slot is consolidated.
    pub const COUNT_LIMIT: u8 = 127;
    /// A slot adapts by `RATE_NUMERATOR / (2 * count + 3)` / 65536 of its gap.
    pub const RATE_NUMERATOR: i32 = 131_072;
    /// A probability map point moves 1/2^REFINEMENT_SHIFT of its gap.
    pub const REFINEMENT_SHIFT: i32 = 6;

    /// The mixers' plasticity after `pos` coded bytes, in 1/65536 per unit
    /// of `x * err`: 128 at the start, decaying toward 24 (`plasticity`).
    #[must_use]
    pub fn plasticity(pos: u32) -> i32 {
        (PLASTICITY_FLOOR
            + PLASTICITY_GAIN * PLASTICITY_HALF_LIFE / (PLASTICITY_HALF_LIFE + i64::from(pos)))
            as i32
    }
}

const NC: usize = 17;
/// Profile bit: the short match model (keyed on 4 bytes, found from 4).
pub const F_SHORT: u32 = 1 << 17;
/// Profile bit: the repeat-distance model (its inputs and context 13).
pub const F_REP: u32 = 1 << 18;
/// Profile bit: mixer 1 and the first APM selected by the two match models'
/// states as well as the partial byte (pays on long inputs, costs on short).
pub const F_SEL: u32 = 1 << 19;
/// The v0.3 model: every context, no short match or repeat model.
pub const V03: u32 = (1 << NC) - 1;
/// Every context and every match feature.
pub const FULL: u32 = (1 << 20) - 1;
/// A mixer's row stride: its at most `NC + 7` inputs padded with zero inputs to three
/// 8-lane vectors, so the vector kernels need no tail.
const NINP: usize = 24;
/// The narrow row stride, one 64-byte line per mixer row, for a profile with
/// at most 16 inputs.
const NINP_NARROW: usize = 16;
/// The largest table [`Model::sized`] gives by length alone: index bits plus
/// the 8-bit check byte span the 32-bit context hash, so a larger table is
/// a facet the frame records ([`TABLE_SHIFT`]) once stepping it pays.
const TB_MAX: u32 = 24;
/// Where a profile word's table field starts: bits 0-19 name the contexts and
/// features, bits 20-25 the context table's bits when the frame records them
/// (0: the length's own size, [`default_table_bits`]), and bits 26-30 the
/// match tables' bits when the frame records them ([`MATCH_SHIFT`]).
pub const TABLE_SHIFT: u32 = 20;
/// Where a profile word's match table field starts (0: the match tables
/// follow the context table's default size, `Model::sized`).
pub const MATCH_SHIFT: u32 = 26;
const TABLE_FIELD: u32 = 63;
/// The largest value the match tables' field holds (five bits).
pub const MATCH_FIELD: u32 = 31;
/// Fewest table bits a frame may record: one bucket pair (two 16-slot buckets).
pub const TABLE_MIN: u32 = 5;
/// Most table bits a frame may record: a slot index is a 32-bit hash's low bits.
pub const TABLE_MAX: u32 = 32;
const SQT: [i32; 33] = [
    1, 2, 3, 6, 10, 16, 27, 45, 73, 120, 194, 310, 488, 747, 1101, 1546, 2047, 2549, 2994, 3348,
    3607, 3785, 3901, 3975, 4022, 4050, 4068, 4079, 4085, 4089, 4092, 4093, 4094,
];
const WEIGHT_LIMIT: i32 = standard::WEIGHT_LIMIT;
/// Adaptive probability map rows: 33 interpolation points over the stretch domain.
const APM_ROW: usize = 33;
const APM_RATE: i32 = standard::REFINEMENT_SHIFT;
const APM2_BITS: u32 = 16;

fn squash(d: i32) -> i32 {
    if d > 2047 {
        return 4095;
    }
    if d < -2047 {
        return 0;
    }
    let w = d & 127;
    let i = ((d >> 7) + 16) as usize;
    (SQT[i] * (128 - w) + SQT[i + 1] * w + 64) >> 7
}

struct Tables {
    stretch: [i16; 4096],
    /// `squash(d)` at `d + 2048`, for `d` in -2048..=2047: the integer LUT the
    /// coder reads in place of interpolating, with the same values.
    squash: [i16; 4096],
    rate: [i32; 128],
    /// `cost[p]` = 256 log2(4096 / p), the cost in 1/256 bit of coding a bit
    /// at probability p / 4096 (`cost_table`).
    cost: [u16; 4096],
}

/// The integer log the encoder's context gains are counted in
/// (`ncm_model.py` `encodeLearn` fills its COST table the same way):
/// 256 log2(4096 / p) by normalising p to 2^15 <= m < 2^16 and squaring once
/// for each of 8 fraction bits.
fn cost_table() -> [u16; 4096] {
    let mut cost = [0u16; 4096];
    for (p, c) in cost.iter_mut().enumerate().skip(1) {
        let (mut m, mut e) = (p as u64, 0u32);
        while m < 32768 {
            m *= 2;
            e += 1;
        }
        let mut frac = 0u32;
        for j in 0..8 {
            m = m * m / 32768;
            if m >= 65536 {
                m /= 2;
                frac += 1 << (7 - j);
            }
        }
        *c = (3072 - (256 * (15 - e) + frac)) as u16;
    }
    cost
}

fn tables() -> &'static Tables {
    static T: std::sync::OnceLock<Tables> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        let mut stretch = [2047i16; 4096];
        let mut pi = 0usize;
        for x in -2047..=2047 {
            let v = squash(x) as usize;
            for s in stretch.iter_mut().take(v + 1).skip(pi) {
                *s = x as i16;
            }
            pi = v + 1;
        }
        let mut rate = [0i32; 128];
        for (c, r) in rate.iter_mut().enumerate() {
            *r = standard::RATE_NUMERATOR / (2 * c as i32 + 3);
        }
        let mut sq = [0i16; 4096];
        for (i, v) in sq.iter_mut().enumerate() {
            *v = squash(i as i32 - 2048) as i16;
        }
        Tables {
            stretch,
            squash: sq,
            rate,
            cost: cost_table(),
        }
    })
}

/// An adaptive probability map: a probability, refined by interpolating
/// between learned points of its row for the context.
#[derive(Clone)]
struct Apm {
    t: Vec<u16>,
    idx: usize,
}

impl Apm {
    fn new(rows: usize) -> Apm {
        let row: Vec<u16> = (0..APM_ROW as i32)
            .map(|j| (squash((j - 16) * 128) * 16) as u16)
            .collect();
        Apm {
            t: row.repeat(rows),
            idx: 0,
        }
    }

    #[inline]
    fn refine(&mut self, stretch: &[i16; 4096], p: u32, cx: usize) -> i32 {
        let s = (i32::from(stretch[p as usize]) + 2048) as usize;
        let (lo, w) = (s >> 7, (s & 127) as i32);
        let base = cx * APM_ROW + lo;
        self.idx = base + (w >> 6) as usize;
        (i32::from(self.t[base]) * (128 - w) + i32::from(self.t[base + 1]) * w) >> 11
    }

    #[inline]
    fn learn(&mut self, y: bool) {
        let yi = i32::from(y);
        let g = (yi << 16) + (yi << APM_RATE) - 2 * yi;
        let t = &mut self.t[self.idx];
        *t = (i32::from(*t) + ((g - i32::from(*t)) >> APM_RATE)) as u16;
    }
}

#[inline]
fn h(mut a: u32) -> u32 {
    a = a.wrapping_mul(0x9E37_79B1);
    a = a.wrapping_add(a >> 15);
    a = a.wrapping_mul(0x2C1B_3C6D);
    a = a.wrapping_add(a >> 12);
    a = a.wrapping_mul(0x297A_2D39);
    a.wrapping_add(a >> 15)
}

/// The match length bucket: exact below 16, then coarser.
#[inline]
fn len_bucket(l: u32) -> u32 {
    if l < 16 {
        l
    } else if l < 32 {
        16 + ((l - 16) >> 2)
    } else if l < 64 {
        20 + ((l - 32) >> 3)
    } else if l < 512 {
        24 + ((l - 64) >> 6)
    } else {
        31
    }
}

/// What a byte position's context hashes read: the bytes and words before
/// it, the record model's row length `r` (with `at` reading the history), and
/// the match model's expected byte (`0x100 + byte`, or 0) and length bucket.
struct Ctx<F: Fn(u32) -> u32> {
    pos: u32,
    c4: u32,
    c8: u32,
    wordh: u32,
    word1: u32,
    r: u32,
    at: F,
    mexp: u32,
    mbucket: u32,
    /// Context 13's value: the sparse context, or under [`F_REP`] the
    /// repeat model's expected byte and run with the previous byte.
    c13: u32,
}

/// The 17 context hashes of a byte position.
#[inline(always)]
fn context_hashes<F: Fn(u32) -> u32>(x: &Ctx<F>) -> [u32; NC] {
    let (pos, c4, c8, r) = (x.pos, x.c4, x.c8, x.r);
    let b1 = c4 & 255;
    let p3 = pos & 3;
    let ab = if r > 0 && pos >= r {
        (x.at)(pos - r)
    } else {
        0
    };
    let ab1 = if r > 0 && pos + 1 >= r {
        (x.at)(pos + 1 - r)
    } else {
        0
    };
    let ab2 = if r > 0 && pos >= 2 * r {
        (x.at)(pos - 2 * r)
    } else {
        0
    };
    let hc4 = h(c4);
    let v: [u32; NC] = [
        0,
        b1 + 0x100,
        c4 & 0xffff,
        (c4 & 0xff_ffff) + 0x100_0000,
        c4.wrapping_add(0x300_0000),
        hc4.wrapping_add(c8 & 0xffff),
        hc4.wrapping_add(c8),
        x.wordh.wrapping_add(h(x.word1)),
        x.wordh,
        (c4 & 0xff00) + (p3 << 16),
        (c4 & 0xffff_0000) + p3,
        b1 + (p3 << 8) + 0x80_0000,
        (c4 & 0xffff) + (p3 << 16) + 0x400_0000,
        x.c13,
        ab.wrapping_add(b1 << 8)
            .wrapping_add(r << 16)
            .wrapping_add(0x1000_0000),
        ab.wrapping_add(ab1 << 8)
            .wrapping_add(ab2 << 16)
            .wrapping_add(r << 24)
            .wrapping_add(0x8000_0000),
        x.mexp
            .wrapping_add(x.mbucket << 9)
            .wrapping_add(0x7700_0000),
    ];
    let mut hb = [0; NC];
    for (k, (hb, v)) in hb.iter_mut().zip(v).enumerate() {
        *hb = h(v.wrapping_add((k as u32).wrapping_mul(0x0100_0193)));
    }
    hb
}

/// Asks the processor to start loading the cache line at `p`. A prefetch
/// never faults and changes no value, whatever the address.
#[inline(always)]
fn prefetch<T>(p: &T) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch hint reads nothing architecturally and cannot fault.
    unsafe {
        use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
        _mm_prefetch::<_MM_HINT_T0>(std::ptr::from_ref(p).cast());
    }
    #[cfg(target_arch = "aarch64")]
    // SAFETY: `prfm` is a hint: it reads nothing architecturally and cannot
    // fault.
    unsafe {
        std::arch::asm!("prfm pldl1keep, [{0}]", in(reg) std::ptr::from_ref(p), options(nostack, preserves_flags, readonly));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let _ = p;
}

#[inline]
fn adapt(p: &mut u16, n: &mut u8, y: bool, rate: &[i32; 128]) {
    let r = rate[usize::from(*n)];
    let pv = i32::from(*p);
    let t = if y { 65535 } else { 0 };
    *p = (pv + (((i64::from(t - pv)) * i64::from(r)) >> 16) as i32) as u16;
    if *n < standard::COUNT_LIMIT {
        *n += 1;
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(64))]
struct Bucket([u32; 16]);

/// A model table. In WebAssembly, whose memory never shrinks, a dropped
/// table is kept for the next frame's model of the same size: allocating
/// ~100 MiB of fresh tables per frame left the allocator's free space in
/// pieces, and the memory grew with the number of frames.
struct Table<T: Spare>(Vec<T>);

impl<T: Spare> Clone for Table<T> {
    fn clone(&self) -> Self {
        Table(self.0.clone())
    }
}

trait Spare: Copy + 'static {
    fn spares() -> &'static std::thread::LocalKey<std::cell::RefCell<Vec<Vec<Self>>>>;
}

thread_local! {
    static SPARE_BUCKETS: std::cell::RefCell<Vec<Vec<Bucket>>> = const { std::cell::RefCell::new(Vec::new()) };
    static SPARE_WORDS: std::cell::RefCell<Vec<Vec<u32>>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Spare for Bucket {
    fn spares() -> &'static std::thread::LocalKey<std::cell::RefCell<Vec<Vec<Self>>>> {
        &SPARE_BUCKETS
    }
}

impl Spare for u32 {
    fn spares() -> &'static std::thread::LocalKey<std::cell::RefCell<Vec<Vec<Self>>>> {
        &SPARE_WORDS
    }
}

impl<T: Spare> Table<T> {
    /// `len` entries of `value`.
    fn new(len: usize, value: T) -> Table<T> {
        if cfg!(target_family = "wasm") && len > 0 {
            let kept = T::spares().with(|s| {
                let mut s = s.borrow_mut();
                let at = s.iter().position(|t| t.len() == len)?;
                Some(s.swap_remove(at))
            });
            if let Some(mut t) = kept {
                t.fill(value);
                return Table(t);
            }
        }
        Table(vec![value; len])
    }
}

impl<T: Spare> Drop for Table<T> {
    fn drop(&mut self) {
        if cfg!(target_family = "wasm") && !self.0.is_empty() {
            let t = std::mem::take(&mut self.0);
            T::spares().with(|s| {
                let mut s = s.borrow_mut();
                // a model holds one bucket table and two word tables
                if s.len() < 2 {
                    s.push(t);
                }
            });
        }
    }
}

impl<T: Spare> std::ops::Deref for Table<T> {
    type Target = [T];
    #[inline(always)]
    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Spare> std::ops::DerefMut for Table<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

const SLOT_FRESH: u32 = 32768;

/// [`adapt`] on a packed slot (probability low, count above it).
#[inline]
fn adapt_slot(s: &mut u32, y: bool, rate: &[i32; 128]) {
    let n = *s >> 16;
    let pv = (*s & 0xffff) as i32;
    let t = if y { 65535 } else { 0 };
    let p = (pv + ((i64::from(t - pv) * i64::from(rate[n as usize])) >> 16) as i32) as u32;
    *s = (p & 0xffff) | ((n + u32::from(n < 127)) << 16);
}

/// Warmed models kept for reuse (state bytes, profile, table mask, model):
/// the most recently used last. How many the process keeps is the declared
/// decode host's rule ([`crate::performance::Host::warmed_kept`]): every one
/// the frame's decoder leaves room for, so a later frame continuing the same
/// state replays nothing and decodes the same bytes.
type Warmed = Vec<(Box<[u8]>, u32, u32, Model)>;
static WARMED: std::sync::Mutex<Warmed> = std::sync::Mutex::new(Vec::new());

/// Bytes a warmed model kept for reuse holds: the tables sized for the state
/// and the frame, and the state's bytes in its history.
pub fn warmed_bytes(n: u64, state: u64, profile: u32) -> u64 {
    table_bytes(n, state, true, profile).saturating_add(state)
}

#[derive(Clone)]
// streams of a frame keep their models side by side: no cache line spans two
#[repr(align(128))]
struct Model {
    t: &'static Tables,
    /// The profile's contexts, in order (`act[..na]`): the mixers read them as
    /// inputs `0..na`, then the match model's two inputs and the bias, then
    /// the short match model's two and the repeat model's two when the
    /// profile has them. A context or feature outside the profile is never
    /// looked up or adapted and would feed every mixer 0, which neither moves
    /// a dot product nor trains a weight, so leaving its input out is exactly
    /// the model with that input 0.
    act: [usize; NC],
    na: usize,
    /// The mixer row stride: [`NINP_NARROW`] when the inputs fit, else [`NINP`].
    stride: usize,
    mask: u32,
    mb: u32,
    /// The context slots, 16 to a bucket: a slot holds its probability in
    /// the low 16 bits and its count in the next 8, so a bucket is one 64-byte
    /// line and a lookup touches one line (slot 0's count is the bucket's
    /// check byte).
    sl: Table<Bucket>,
    hist: Vec<u8>,
    /// History bytes ahead of the coded ones in `hist` (a window frame's
    /// preceding outputs): the match model reaches into them.
    hoff: u32,
    pos: u32,
    /// Bytes the model has learned from (its plasticity's clock): `pos` for
    /// one stream; for a stream of a laned frame, every stream's bytes merged
    /// into the shared state plus its own since.
    seen: u32,
    /// A stream of a laned frame: the shared state it reads and its own
    /// changes since the epoch began (`None` for one stream).
    lane: Option<Box<streams::Lane>>,
    c4: u32,
    c8: u32,
    wordh: u32,
    word1: u32,
    c0: u32,
    bit: u32,
    mtab: Table<u32>,
    profile: u32,
    m2tab: Table<u32>,
    m2ptr: u32,
    m2len: u32,
    m2exp: u32,
    m2b: u32,
    m2sp: [u16; 64],
    m2sn: [u8; 64],
    m2i: usize,
    rd: u32,
    rrun: u32,
    rexp: u32,
    rok: bool,
    ra: bool,
    rsp: [u16; 128],
    rsn: [u8; 128],
    ri: usize,
    mptr: u32,
    mlen: u32,
    mexp: u32,
    mbucket: u32,
    msp: [u16; 64],
    msn: [u8; 64],
    lastp: [u32; 256],
    lastd: [u32; 256],
    rcnt: Vec<u16>,
    r: u32,
    hb: [u32; NC],
    /// The inputs and context hashes prefetch_next computed for the byte it
    /// expected next.
    spec_key: [u32; 9],
    spec_hb: [u32; NC],
    bb: [u32; NC],
    ix: [u32; NC],
    st: [i32; NINP],
    mi: usize,
    w1: Vec<i32>,
    w2: Vec<i32>,
    w3: Vec<i32>,
    s1: usize,
    s2: usize,
    s3: usize,
    pm1: i32,
    pm2: i32,
    pm3: i32,
    /// the three mixers' dot products for the bit being coded
    dots: [i64; 3],
    /// With the encoder learning: what each context saved, in 1/256 bit, over
    /// the bits coded so far (see [`Model::learn`]).
    gain: Option<Box<[i64; NC]>>,
    a1: Apm,
    a2: Apm,
    /// The twin of a shared pass: the profile with [`F_SEL`] flipped reads
    /// the same contexts, slots, match and repeat state and mixers 2 and 3,
    /// so only its mixer 1, its two APMs and its coder are its own.
    twin: Option<Box<Twin>>,
}

impl Model {
    /// The model for `n` bytes coded after `history` (a window frame's
    /// preceding outputs): the match model starts holding every history
    /// position, and every other model starts fresh.
    /// A model for `n` bytes coded after `history`, the concatenation of its
    /// pieces (a window decodes without first joining its frames).
    fn new(n: u32, history: &[&[u8]], profile: u32) -> Model {
        let before: usize = history.iter().map(|h| h.len()).sum();
        let (mut mask, mut mb) = Model::sized(n);
        if let Some(tb) = recorded_table_bits(profile) {
            mask = ((1u64 << tb) - 1) as u32;
        }
        if let Some(bits) = recorded_match_bits(profile) {
            mb = bits;
        }
        let profile = profile & FULL;
        let mut act = [0; NC];
        let mut na = 0;
        for k in 0..NC {
            if profile >> k & 1 == 1 {
                act[na] = k;
                na += 1;
            }
        }
        let short = profile & F_SHORT != 0;
        let inputs = na + 3 + 2 * usize::from(short) + 2 * usize::from(profile & F_REP != 0);
        let stride = if inputs <= NINP_NARROW {
            NINP_NARROW
        } else {
            NINP
        };
        let mut msp = [0u16; 64];
        for (i, p) in msp.iter_mut().enumerate() {
            *p = if i % 2 == 1 { 60000 } else { 5536 };
        }
        let mut m = Model {
            t: tables(),
            act,
            na,
            stride,
            mask,
            mb,
            sl: Table::new((mask as usize + 1) / 16, Bucket([SLOT_FRESH; 16])),
            hist: vec![0; before + n as usize],
            hoff: before as u32,
            pos: 0,
            seen: 0,
            lane: None,
            c4: 0,
            c8: 0,
            wordh: 0,
            word1: 0,
            c0: 1,
            bit: 0,
            mtab: Table::new(1 << mb, 0),
            profile,
            m2tab: Table::new(if short { 1 << mb } else { 0 }, 0),
            m2ptr: 0,
            m2len: 0,
            m2exp: 0,
            m2b: 0,
            m2sp: msp,
            m2sn: [0; 64],
            m2i: 0,
            rd: 0,
            rrun: 0,
            rexp: 0,
            rok: false,
            ra: false,
            rsp: [32768; 128],
            rsn: [0; 128],
            ri: 0,
            mptr: 0,
            mlen: 0,
            mexp: 0,
            mbucket: 0,
            msp,
            msn: [0; 64],
            lastp: [0; 256],
            lastd: [0; 256],
            rcnt: vec![0; 65536],
            r: 0,
            hb: [0; NC],
            spec_key: [u32::MAX; 9],
            spec_hb: [0; NC],
            bb: [0; NC],
            ix: [0; NC],
            st: [0; NINP],
            mi: 0,
            w1: weights(256 * 9, na + 3, stride),
            w2: weights(2048, na + 3, stride),
            w3: weights(2048, na + 3, stride),
            s1: 0,
            dots: [0; 3],
            gain: None,
            s2: 0,
            s3: 0,
            pm1: 0,
            pm2: 0,
            pm3: 0,
            a1: Apm::new(256 * 9),
            a2: Apm::new(1 << APM2_BITS),
            twin: None,
        };
        let mut at = 0;
        for piece in history {
            m.hist[at..at + piece.len()].copy_from_slice(piece);
            at += piece.len();
        }
        for p in 7..m.hoff {
            let k = m.match_key(p);
            m.mtab[k] = p;
        }
        if short {
            for p in 4..m.hoff {
                let k = m.short_key(p);
                m.m2tab[k] = p;
            }
        }
        m.new_byte();
        m
    }

    /// The model for `n` bytes coded after the κ-named model state `state`
    /// (`Ncm.decodeWarm`): the whole model codes the state's bytes first, as
    /// though they opened the frame, and emits nothing for them, so the frame
    /// is coded as their continuation. The tables are sized for `w + n` bytes,
    /// which for a state of 16 KiB or more is the largest size whatever `n`,
    /// so the warmed model is computed once per state and profile and cloned.
    fn warmed<K: Kernel>(n: u32, state: &[u8], profile: u32) -> Option<Model> {
        let total = n.checked_add(u32::try_from(state.len()).ok()?)?;
        let (mut mask, _) = Model::sized(total);
        if let Some(tb) = recorded_table_bits(profile) {
            mask = ((1u64 << tb) - 1) as u32;
        }
        let mut cache = WARMED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(i) = cache
            .iter()
            .position(|(s, p, k, _)| *p == profile && *k == mask && s[..] == *state)
        {
            let hit = cache.remove(i);
            let m = hit.3.clone();
            cache.push(hit);
            return Some(m);
        }
        let mut m = Model::new(total, &[], profile);
        if m.stride == NINP_NARROW {
            m.warm::<K, NINP_NARROW>(state);
        } else {
            m.warm::<K, NINP>(state);
        }
        let keep = crate::performance::declared_host().warmed_kept(
            u64::from(n),
            state.len() as u64,
            profile,
        );
        let keep = usize::try_from(keep).unwrap_or(usize::MAX);
        // least recently used first out, until this one fits (or, keeping
        // none, until none is held)
        let held = cache.len().min(keep.saturating_sub(1));
        let over = cache.len() - held;
        cache.drain(..over);
        if keep > 0 {
            cache.push((state.into(), profile, mask, m.clone()));
        }
        Some(m)
    }

    /// Codes `state`'s bits through the whole model without a coder: a pass
    /// over the state that reports progress like any other.
    fn warm<K: Kernel, const W: usize>(&mut self, state: &[u8]) {
        for (i, &byte) in state.iter().enumerate() {
            if i % TICK == 0 {
                tick(i, state.len());
            }
            for k in (0..8).rev() {
                self.predict::<K, W>();
                self.update::<K, W>((byte >> k) & 1 == 1);
            }
        }
    }

    /// (table mask, match table bits) for `n` coded bytes.
    fn sized(n: u32) -> (u32, u32) {
        let mut tb = 16;
        while tb < TB_MAX && (1u64 << tb) < u64::from(n) * 1024 {
            tb += 1;
        }
        ((1u32 << tb) - 1, if tb > 22 { 22 } else { tb - 2 })
    }

    /// The byte at coded position `i`.
    #[inline]
    fn at(&self, i: u32) -> u32 {
        self.abs(self.hoff + i)
    }

    /// The byte at absolute position `i` (history first).
    #[inline]
    fn abs(&self, i: u32) -> u32 {
        u32::from(self.hist[i as usize])
    }

    /// The match table entry `k`.
    #[inline]
    fn mt(&self, k: usize) -> u32 {
        match (&self.lane, self.mtab[k]) {
            (Some(l), 0) => l.shared().mtab[k],
            (_, v) => v,
        }
    }

    #[inline]
    fn set_mt(&mut self, k: usize, v: u32) {
        if let Some(l) = &mut self.lane {
            if self.mtab[k] == 0 {
                l.mt.push(k as u32);
            }
        }
        self.mtab[k] = v;
    }

    /// The short match table entry `k`.
    #[inline]
    fn mt2(&self, k: usize) -> u32 {
        match (&self.lane, self.m2tab[k]) {
            (Some(l), 0) => l.shared().m2tab[k],
            (_, v) => v,
        }
    }

    #[inline]
    fn set_mt2(&mut self, k: usize, v: u32) {
        if let Some(l) = &mut self.lane {
            if self.m2tab[k] == 0 {
                l.m2t.push(k as u32);
            }
        }
        self.m2tab[k] = v;
    }

    /// The match-model key of absolute position `p`: the hash of the 7 bytes before it.
    fn match_key(&self, p: u32) -> usize {
        self.match_key_by(p, |i| self.abs(i))
    }

    /// The short match model's key of absolute position `p`: the hash of the 4 bytes before it.
    fn short_key(&self, p: u32) -> usize {
        let mut k = 0x55u32;
        for i in 1..=4 {
            k = k
                .wrapping_mul(0x2F0B_4C27)
                .wrapping_add(self.abs(p - i))
                .wrapping_add(1);
        }
        (h(k) >> (32 - self.mb)) as usize
    }

    /// How many bytes before absolute positions `cand` and `pa` agree (at most 1024).
    #[inline]
    fn agree(&self, cand: u32, pa: u32) -> u32 {
        let mut l = 0;
        while l < 1024 && l < cand && self.abs(cand - 1 - l) == self.abs(pa - 1 - l) {
            l += 1;
        }
        l
    }

    /// [`Model::match_key`], reading absolute positions through `abs`.
    #[inline(always)]
    fn match_key_by(&self, p: u32, abs: impl Fn(u32) -> u32) -> usize {
        let mut k = 0u32;
        for i in 1..=7 {
            k = k
                .wrapping_mul(0x2F0B_4C27)
                .wrapping_add(abs(p - i))
                .wrapping_add(1);
        }
        (h(k) >> (32 - self.mb)) as usize
    }

    fn new_byte(&mut self) {
        let pos = self.pos;
        let pa = self.hoff + pos;
        // the repeat model's run: how many bytes in a row its prediction held
        if self.rok {
            self.rrun = if self.rexp == self.abs(pa - 1) {
                (self.rrun + 1).min(63)
            } else {
                0
            };
        }
        let rep = self.profile & F_REP != 0;
        if self.mlen > 0 {
            if self.mlen < 65535 {
                self.mlen += 1;
            }
            self.mptr += 1;
        }
        if pa >= 7 {
            let k = self.match_key(pa);
            if self.mlen == 0 {
                let cand = self.mt(k);
                // a stream of a laned frame may read a later stream's
                // position: a match points only into the past
                if cand > 0 && cand < pa {
                    let l = self.agree(cand, pa);
                    if l >= 7 {
                        self.mptr = cand;
                        self.mlen = l;
                        if rep {
                            self.rd = pa - cand;
                            self.rrun = 0;
                        }
                    }
                }
            }
            self.set_mt(k, pa);
        }
        if self.profile & F_SHORT != 0 {
            if self.m2len > 0 {
                if self.m2len < 65535 {
                    self.m2len += 1;
                }
                self.m2ptr += 1;
            }
            if pa >= 4 {
                let k = self.short_key(pa);
                if self.m2len == 0 {
                    let cand = self.mt2(k);
                    if cand > 0 && cand < pa {
                        let l = self.agree(cand, pa);
                        if l >= 4 {
                            self.m2ptr = cand;
                            self.m2len = l;
                            if rep {
                                self.rd = pa - cand;
                                self.rrun = 0;
                            }
                        }
                    }
                }
                self.set_mt2(k, pa);
            }
            if self.m2len > 0 {
                self.m2exp = self.abs(self.m2ptr);
                self.m2b = len_bucket(self.m2len);
            } else {
                self.m2exp = 0;
                self.m2b = 0;
            }
        }
        // the repeat model: the byte at the distance of the last match found
        self.rok = rep && self.rd > 0;
        if self.rok {
            self.rexp = self.abs(pa - self.rd);
        }
        if self.mlen > 0 {
            self.mexp = self.abs(self.mptr);
            self.mbucket = len_bucket(self.mlen);
        } else {
            self.mexp = 0;
            self.mbucket = 0;
        }
        let key = [
            pos,
            self.c4,
            self.c8,
            self.wordh,
            self.word1,
            self.r,
            if self.mlen > 0 { self.mexp + 0x100 } else { 0 },
            self.mbucket,
            self.c13(self.c4),
        ];
        // The hashes prefetch_next computed for the byte it expected are
        // these when every input matches (the bytes `at` reads behind `pos`
        // are the same history either way).
        self.hb = if self.spec_key == key {
            self.spec_hb
        } else {
            context_hashes(&Ctx {
                pos,
                c4: key[1],
                c8: key[2],
                wordh: key[3],
                word1: key[4],
                r: key[5],
                at: |i| self.at(i),
                mexp: key[6],
                mbucket: key[7],
                c13: key[8],
            })
        };
    }

    /// Context 13's value for the last four bytes `c4`.
    #[inline(always)]
    fn c13(&self, c4: u32) -> u32 {
        if self.profile & F_REP == 0 {
            c4 & 0x00ff_00ff
        } else {
            let rx = if self.rok {
                (self.rexp + 256) + (self.rrun.min(15) << 9)
            } else {
                0
            };
            rx.wrapping_add((c4 & 255) << 16).wrapping_add(0x5500_0000)
        }
    }

    /// Starts loading what the next bit will read, for the likelier value of
    /// the bit being coded (`p` is its probability of a 1): the second
    /// nibble's 17 buckets ahead of bit 4, the next byte's first-nibble
    /// buckets (from the contexts that byte would have) ahead of bit 0, and
    /// the next APM row. A prefetch changes no result, only when memory is
    /// read, so a wrong guess costs a few loads and nothing else.
    #[inline(always)]
    fn prefetch_next(&mut self, p: u32) {
        let y = u32::from(p >= 2048);
        let c0 = (self.c0 << 1) | y;
        let (c4, c0) = if self.bit == 7 {
            let b = c0 & 255;
            let (mut wordh, mut word1) = (self.wordh, self.word1);
            if (b as u8).is_ascii_alphabetic() || b >= 128 {
                wordh = h(wordh.wrapping_add(b | 32));
            } else if wordh != 0 {
                word1 = wordh;
                wordh = 0;
            }
            // coded and absolute readers that see `b` at its position
            let (pos, pa) = (self.pos, self.hoff + self.pos);
            let at = |i: u32| if i == pos { b } else { self.at(i) };
            let abs = |i: u32| if i == pa { b } else { self.abs(i) };
            let (mexp, mbucket) = if self.mlen > 0 && self.mexp == b && self.mptr < pa {
                (
                    abs(self.mptr + 1) + 0x100,
                    len_bucket((self.mlen + 1).min(65535)),
                )
            } else {
                (0, 0)
            };
            let key = [
                pos + 1,
                (self.c4 << 8) | b,
                (self.c8 << 8) | (self.c4 >> 24),
                wordh,
                word1,
                self.r,
                mexp,
                mbucket,
                if self.profile & F_REP == 0 {
                    ((self.c4 << 8) | b) & 0x00ff_00ff
                } else {
                    // the repeat state the next byte has if no new match is found
                    let rx = if self.rok && self.rd <= pa + 1 {
                        let run = if self.rexp == b { self.rrun + 1 } else { 0 };
                        abs(pa + 1 - self.rd) + 256 + (run.min(15) << 9)
                    } else {
                        0
                    };
                    rx.wrapping_add(b << 16).wrapping_add(0x5500_0000)
                },
            ];
            let hb = context_hashes(&Ctx {
                pos: key[0],
                c4: key[1],
                c8: key[2],
                wordh,
                word1,
                r: key[5],
                at,
                mexp,
                mbucket,
                c13: key[8],
            });
            self.prefetch_buckets(&hb, 0);
            if pa + 1 >= 7 {
                let k = self.match_key_by(pa + 1, abs);
                prefetch(&self.mtab[k]);
                if let Some(l) = &self.lane {
                    prefetch(&l.shared().mtab[k]);
                }
            }
            self.spec_key = key;
            self.spec_hb = hb;
            ((self.c4 << 8) | b, 1)
        } else {
            if self.bit == 3 {
                self.prefetch_buckets(&self.hb, (c0 & 15) | 16);
            }
            (self.c4, c0)
        };
        let bit = (self.bit + 1) & 7;
        for (w, s) in [
            (&self.w1, c0 as usize),
            (&self.w2, ((c4 & 255) * 8 + bit) as usize),
            (&self.w3, (((c4 >> 8) & 255) * 8 + bit) as usize),
        ] {
            prefetch(&w[s * self.stride]);
            prefetch(&w[s * self.stride + self.stride - 1]);
        }
        let cx2 =
            h((c4 & 0xff_ffff).wrapping_mul(0x2F0B_4C27).wrapping_add(c0)) >> (32 - APM2_BITS);
        let row = cx2 as usize * APM_ROW;
        prefetch(&self.a2.t[row]);
        prefetch(&self.a2.t[row + APM_ROW - 1]);
    }

    #[inline(always)]
    fn prefetch_buckets(&self, hb: &[u32; NC], nc: u32) {
        for &k in &self.act[..self.na] {
            let hb = hb[k];
            let hh = h(hb.wrapping_add(nc.wrapping_mul(0x2F0B_4C27)));
            // a bucket and its neighbour share one 32-slot line pair
            let b = ((hh & self.mask) & !31) as usize;
            prefetch(&self.sl[b >> 4]);
            prefetch(&self.sl[(b >> 4) + 1]);
            if let Some(l) = &self.lane {
                prefetch(&l.shared().sl[b >> 4]);
                prefetch(&l.shared().sl[(b >> 4) + 1]);
            }
        }
    }

    /// The bucket for a context hash: the one of a neighbouring pair that
    /// carries its check byte (in the unused slot 0), else the less used one
    /// (by slot 1's count), cleared and tagged.
    #[inline]
    fn bucket(&mut self, hh: u32) -> u32 {
        if let Some(l) = &mut self.lane {
            return l.bucket(hh, self.mask, &mut self.sl);
        }
        let ck = ((hh >> 24) | 1) as u8;
        let b0 = (hh & self.mask) & !15;
        let alt = b0 ^ 16;
        let count = |s: u32| (s >> 16) as u8;
        let (x0, xa) = (&self.sl[b0 as usize >> 4].0, &self.sl[alt as usize >> 4].0);
        if count(x0[0]) == ck {
            return b0;
        }
        if count(xa[0]) == ck {
            return alt;
        }
        let b = if count(x0[1]) > count(xa[1]) { alt } else { b0 };
        let mut fresh = [SLOT_FRESH; 16];
        fresh[0] |= u32::from(ck) << 16;
        self.sl[b as usize >> 4].0 = fresh;
        b
    }

    #[inline(always)]
    fn predict<K: Kernel, const W: usize>(&mut self) -> u32 {
        let t = self.t;
        if self.bit == 0 || self.bit == 4 {
            let nc = if self.bit == 0 {
                0
            } else {
                (self.c0 & 15) | 16
            };
            for j in 0..self.na {
                let hh = h(self.hb[self.act[j]].wrapping_add(nc.wrapping_mul(0x2F0B_4C27)));
                self.bb[j] = self.bucket(hh);
            }
        }
        let part = if self.bit < 4 {
            self.c0
        } else {
            (self.c0 & ((1 << (self.bit - 4)) - 1)) | (1 << (self.bit - 4))
        };
        let na = self.na;
        for j in 0..na {
            let i = self.bb[j] + part;
            self.ix[j] = i;
            let p = self.sl[i as usize >> 4].0[i as usize & 15] & 0xffff;
            self.st[j] = i32::from(t.stretch[(p >> 4) as usize]);
        }
        let mut eb = 0;
        if self.mlen > 0 {
            let bp = 7 - self.bit;
            if ((self.mexp | 256) >> (bp + 1)) == self.c0 {
                eb = (self.mexp >> bp) & 1;
            } else {
                self.mlen = 0;
                self.mbucket = 0;
            }
        }
        self.mi = (self.mbucket * 2 + eb) as usize;
        self.st[na] = if self.mlen > 0 {
            i32::from(t.stretch[usize::from(self.msp[self.mi] >> 4)])
        } else {
            0
        };
        let vote = if self.mbucket > 15 { 1024 } else { 256 };
        self.st[na + 1] = if self.mlen == 0 {
            0
        } else if eb == 1 {
            vote
        } else {
            -vote
        };
        self.st[na + 2] = 256;
        let bp = 7 - self.bit;
        let mut eb2 = 0;
        if self.m2len > 0 {
            if ((self.m2exp | 256) >> (bp + 1)) == self.c0 {
                eb2 = (self.m2exp >> bp) & 1;
            } else {
                self.m2len = 0;
                self.m2b = 0;
            }
        }
        self.ra = self.rok && ((self.rexp | 256) >> (bp + 1)) == self.c0;
        let ebr = if self.ra { (self.rexp >> bp) & 1 } else { 0 };
        let mut at = na + 3;
        if self.profile & F_SHORT != 0 {
            self.m2i = (self.m2b * 2 + eb2) as usize;
            self.st[at] = if self.m2len > 0 {
                i32::from(t.stretch[usize::from(self.m2sp[self.m2i] >> 4)])
            } else {
                0
            };
            let vote2 = if self.m2b > 15 { 1024 } else { 256 };
            self.st[at + 1] = if self.m2len == 0 {
                0
            } else if eb2 == 1 {
                vote2
            } else {
                -vote2
            };
            at += 2;
        }
        if self.profile & F_REP != 0 {
            self.ri = (self.rrun * 2 + ebr) as usize;
            self.st[at] = if self.ra {
                i32::from(t.stretch[usize::from(self.rsp[self.ri] >> 4)])
            } else {
                0
            };
            self.st[at + 1] = if !self.ra {
                0
            } else if ebr == 1 {
                256
            } else {
                -256
            };
        }
        // mixer 1 and the first APM: the partial byte and the two match states
        let mstate =
            (if self.m2len > 0 { 1 + eb2 } else { 0 }) + 3 * (if self.ra { 1 + ebr } else { 0 });
        let msel = if self.profile & F_SEL == 0 { 0 } else { mstate };
        let c1 = (self.c0 + 256 * msel) as usize;
        self.s1 = c1 * W;
        self.s2 = ((self.c4 & 255) * 8 + self.bit) as usize * W;
        self.s3 = (((self.c4 >> 8) & 255) * 8 + self.bit) as usize * W;
        let [a, b, c] = K::dot::<W>(
            row(&self.st, 0),
            row(&self.w1, self.s1),
            row(&self.w2, self.s2),
            row(&self.w3, self.s3),
        );
        self.dots = [a, b, c];
        let d1 = (a >> 16).clamp(-2047, 2047) as i32;
        let d2 = (b >> 16).clamp(-2047, 2047) as i32;
        let d3 = (c >> 16).clamp(-2047, 2047) as i32;
        let sq = |d: i32| i32::from(t.squash[(d + 2048) as usize]);
        self.pm1 = sq(d1);
        self.pm2 = sq(d2);
        self.pm3 = sq(d3);
        // the mean of the three, in -2048..=2047
        let p = sq((((d1 + d2 + d3 + 6144) * 21846) >> 16) - 2048).clamp(1, 4095) as u32;
        let q1 = self.a1.refine(&t.stretch, p, c1);
        let cx2 = h((self.c4 & 0xff_ffff)
            .wrapping_mul(0x2F0B_4C27)
            .wrapping_add(self.c0))
            >> (32 - APM2_BITS);
        let q2 = self.a2.refine(&t.stretch, p, cx2 as usize);
        let pr = ((p as i32 + q1 + 2 * q2) / 4).clamp(1, 4095) as u32;
        if let Some(tw) = self.twin.as_deref_mut() {
            let c1 = (self.c0 + 256 * if tw.sel { mstate } else { 0 }) as usize;
            tw.s1 = c1 * W;
            let a = K::dot1::<W>(row(&self.st, 0), row(&tw.w1, tw.s1));
            tw.dot1 = a;
            let d1 = (a >> 16).clamp(-2047, 2047) as i32;
            tw.pm1 = sq(d1);
            let p = sq((((d1 + d2 + d3 + 6144) * 21846) >> 16) - 2048).clamp(1, 4095) as u32;
            let q1 = tw.a1.refine(&t.stretch, p, c1);
            let q2 = tw.a2.refine(&t.stretch, p, cx2 as usize);
            tw.pr = ((p as i32 + q1 + 2 * q2) / 4).clamp(1, 4095) as u32;
        }
        self.prefetch_next(pr);
        pr
    }

    /// Charge each kept context the cost of coding bit `y` with its term left
    /// out of the three mixers (the mean of their outputs, as `predict` takes
    /// it), less the cost with it: what the context saved on this bit.
    fn learn<const W: usize>(&mut self, y: bool) {
        let t = self.t;
        let sq = |d: i64| i32::from(t.squash[(d.clamp(-2047, 2047) + 2048) as usize]);
        let mean = |d: [i64; 3]| {
            let c = |d: i64| (d >> 16).clamp(-2047, 2047);
            sq((((c(d[0]) + c(d[1]) + c(d[2]) + 6144) * 21846) >> 16) - 2048).clamp(1, 4095)
        };
        let cost = |p: i32| i64::from(t.cost[if y { p } else { 4096 - p } as usize]);
        if let Some(gain) = self.gain.as_mut() {
            let base = cost(mean(self.dots));
            for j in 0..self.na {
                let x = i64::from(self.st[j]);
                let without = [
                    self.dots[0] - i64::from(self.w1[self.s1 + j]) * x,
                    self.dots[1] - i64::from(self.w2[self.s2 + j]) * x,
                    self.dots[2] - i64::from(self.w3[self.s3 + j]) * x,
                ];
                gain[self.act[j]] += cost(mean(without)) - base;
            }
        }
        // the twin's mixer 1 in place of this profile's, mixers 2 and 3 shared
        if let Some(tw) = self.twin.as_deref_mut() {
            if let Some(gain) = tw.gain.as_mut() {
                let dots = [tw.dot1, self.dots[1], self.dots[2]];
                let base = cost(mean(dots));
                for j in 0..self.na {
                    let x = i64::from(self.st[j]);
                    let without = [
                        dots[0] - i64::from(tw.w1[tw.s1 + j]) * x,
                        dots[1] - i64::from(self.w2[self.s2 + j]) * x,
                        dots[2] - i64::from(self.w3[self.s3 + j]) * x,
                    ];
                    gain[self.act[j]] += cost(mean(without)) - base;
                }
            }
        }
    }

    /// The profile without the contexts that saved nothing (gain <= 0).
    fn learned(&self) -> u32 {
        self.pruned(self.profile, self.gain.as_deref())
    }

    /// The twin's profile without the contexts that saved nothing under it.
    fn twin_learned(&self) -> u32 {
        let twin = self.twin.as_deref();
        self.pruned(self.profile ^ F_SEL, twin.and_then(|t| t.gain.as_deref()))
    }

    fn pruned(&self, mut profile: u32, gain: Option<&[i64; NC]>) -> u32 {
        if let Some(gain) = gain {
            for j in 0..self.na {
                if gain[self.act[j]] <= 0 {
                    profile &= !(1 << self.act[j]);
                }
            }
        }
        profile
    }

    #[inline(always)]
    fn update<K: Kernel, const W: usize>(&mut self, y: bool) {
        self.a1.learn(y);
        self.a2.learn(y);
        let rate = &self.t.rate;
        for j in 0..self.na {
            let i = self.ix[j] as usize;
            adapt_slot(&mut self.sl[i >> 4].0[i & 15], y, rate);
        }
        if self.mlen > 0 {
            adapt(&mut self.msp[self.mi], &mut self.msn[self.mi], y, rate);
        }
        if self.ra {
            adapt(&mut self.rsp[self.ri], &mut self.rsn[self.ri], y, rate);
        }
        if self.m2len > 0 {
            adapt(&mut self.m2sp[self.m2i], &mut self.m2sn[self.m2i], y, rate);
        }
        let lr = standard::plasticity(self.seen);
        let yb = i32::from(y) << 12;
        // |err| <= 4096 and lr <= 128, so err * lr <= 2^19 and x * err * lr
        // (|x| <= 2047) stays inside i32 (`laneFitHolds`): the kernels learn
        // in 32-bit lanes.
        let e = [
            (yb - self.pm1) * lr,
            (yb - self.pm2) * lr,
            (yb - self.pm3) * lr,
        ];
        K::train::<W>(
            row(&self.st, 0),
            row_mut(&mut self.w1, self.s1),
            row_mut(&mut self.w2, self.s2),
            row_mut(&mut self.w3, self.s3),
            e,
        );
        if let Some(tw) = self.twin.as_deref_mut() {
            tw.a1.learn(y);
            tw.a2.learn(y);
            K::train1::<W>(
                row(&self.st, 0),
                row_mut(&mut tw.w1, tw.s1),
                (yb - tw.pm1) * lr,
            );
        }
        self.c0 = (self.c0 << 1) | u32::from(y);
        self.bit += 1;
        if self.bit == 8 {
            self.end_byte();
        }
    }

    /// The byte in `c0` is coded: the history, the order and word contexts,
    /// the record model, then the next byte's contexts.
    #[inline(always)]
    fn end_byte(&mut self) {
        {
            let b = self.c0 & 255;
            // a laned stream writes its range of the frame in place; a model
            // that codes past the length it was sized for (a warm state's
            // continuation) grows its history
            match self.hist.get_mut((self.hoff + self.pos) as usize) {
                Some(at) => *at = b as u8,
                None => self.hist.push(b as u8),
            }
            self.pos += 1;
            self.seen += 1;
            self.c8 = (self.c8 << 8) | (self.c4 >> 24);
            self.c4 = (self.c4 << 8) | b;
            if (b as u8).is_ascii_alphabetic() || b >= 128 {
                self.wordh = h(self.wordh.wrapping_add(b | 32));
            } else if self.wordh != 0 {
                self.word1 = self.wordh;
                self.wordh = 0;
            }
            let bi = b as usize;
            let d = self.pos - self.lastp[bi];
            if self.lastp[bi] > 0 && d > 1 && d < 65536 && d == self.lastd[bi] {
                let c = &mut self.rcnt[d as usize];
                *c = c.saturating_add(1);
                if self.rcnt[d as usize] > self.rcnt[self.r as usize] {
                    self.r = d;
                }
            }
            self.lastd[bi] = d;
            self.lastp[bi] = self.pos;
            self.c0 = 1;
            self.bit = 0;
            self.new_byte();
        }
    }

    /// The sites coding `byte` touches, without coding it: `f(table, index)`
    /// for every context bucket looked up (table 0), match and short match
    /// table entry written (1, 2), mixer and map row trained (3: `w1` and
    /// `a1` by `c1`, 6: `a2`; 4: `w2` and 5: `w3` by their group of eight
    /// rows, one per bit of the byte) and match-model slot adapted
    /// (7: match, 8: short match, 9: repeat). The buckets a lookup chooses,
    /// their tags, counts and clears and the match tables are what coding
    /// leaves (none depends on a probability); the probabilities are not
    /// kept. On a model without a lane.
    pub(super) fn touch_byte(&mut self, byte: u32, f: &mut impl FnMut(usize, &[u32])) {
        // a byte trains all eight rows of mixer 2's and mixer 3's row group
        f(4, &[self.c4 & 255]);
        f(5, &[(self.c4 >> 8) & 255]);
        for k in (0..8).rev() {
            if self.bit == 0 || self.bit == 4 {
                let nc = if self.bit == 0 {
                    0
                } else {
                    (self.c0 & 15) | 16
                };
                // every lookup's pair asked for first, then looked up
                let mut hh = [0u32; NC];
                for j in 0..self.na {
                    hh[j] = h(self.hb[self.act[j]].wrapping_add(nc.wrapping_mul(0x2F0B_4C27)));
                    let b = ((hh[j] & self.mask) & !31) as usize >> 4;
                    prefetch(&self.sl[b]);
                    prefetch(&self.sl[b + 1]);
                }
                let mut bs = [0u32; NC];
                for j in 0..self.na {
                    let b = self.bucket(hh[j]);
                    // the nibble's first bit adapts slot 1, whose count the
                    // replacement rule reads
                    let s = &mut self.sl[b as usize >> 4].0[1];
                    if *s >> 16 < u32::from(standard::COUNT_LIMIT) {
                        *s += 1 << 16;
                    }
                    bs[j] = b >> 4;
                }
                f(0, &bs[..self.na]);
            }
            let bp = 7 - self.bit;
            let mut eb = 0;
            if self.mlen > 0 {
                if ((self.mexp | 256) >> (bp + 1)) == self.c0 {
                    eb = (self.mexp >> bp) & 1;
                } else {
                    self.mlen = 0;
                    self.mbucket = 0;
                }
            }
            let mut eb2 = 0;
            if self.m2len > 0 {
                if ((self.m2exp | 256) >> (bp + 1)) == self.c0 {
                    eb2 = (self.m2exp >> bp) & 1;
                } else {
                    self.m2len = 0;
                    self.m2b = 0;
                }
            }
            self.ra = self.rok && ((self.rexp | 256) >> (bp + 1)) == self.c0;
            let ebr = if self.ra { (self.rexp >> bp) & 1 } else { 0 };
            let msel = if self.profile & F_SEL == 0 {
                0
            } else {
                (if self.m2len > 0 { 1 + eb2 } else { 0 }) + 3 * (if self.ra { 1 + ebr } else { 0 })
            };
            f(3, &[self.c0 + 256 * msel]);
            f(
                6,
                &[h((self.c4 & 0xff_ffff)
                    .wrapping_mul(0x2F0B_4C27)
                    .wrapping_add(self.c0))
                    >> (32 - APM2_BITS)],
            );
            if self.mlen > 0 {
                f(7, &[self.mbucket * 2 + eb]);
            }
            if self.profile & F_SHORT != 0 && self.m2len > 0 {
                f(8, &[self.m2b * 2 + eb2]);
            }
            if self.profile & F_REP != 0 && self.ra {
                f(9, &[self.rrun * 2 + ebr]);
            }
            self.c0 = (self.c0 << 1) | ((byte >> k) & 1);
            self.bit += 1;
        }
        self.end_byte();
        let pa = self.hoff + self.pos;
        if pa >= 7 {
            f(1, &[self.match_key(pa) as u32]);
        }
        if self.profile & F_SHORT != 0 && pa >= 4 {
            f(2, &[self.short_key(pa) as u32]);
        }
    }
}

/// The parts of a profile that differ from its [`F_SEL`] twin's: mixer 1's
/// weights and selection, the two APMs, and the probability they give.
#[derive(Clone)]
struct Twin {
    sel: bool,
    w1: Vec<i32>,
    a1: Apm,
    a2: Apm,
    s1: usize,
    pm1: i32,
    pr: u32,
    /// mixer 1's dot product for the bit being coded
    dot1: i64,
    /// With the twin learning: what each context saved under the twin's
    /// mixer 1 (see [`Model::learn`]).
    gain: Option<Box<[i64; NC]>>,
}

/// Fresh mixer weights for `rows` contexts in rows of `stride`: 1/4 on the
/// first `v03` inputs (the contexts, the match model's and the bias), 0 on the
/// short match and repeat inputs (they earn their weight) and on padding.
fn weights(rows: usize, v03: usize, stride: usize) -> Vec<i32> {
    let mut w = vec![0; rows * stride];
    for r in w.chunks_exact_mut(stride) {
        r[..v03].fill(standard::WEIGHT_INITIAL);
    }
    w
}

#[inline(always)]
fn row<const W: usize>(w: &[i32], s: usize) -> &[i32; W] {
    w[s..s + W].try_into().expect("row")
}

#[inline(always)]
fn row_mut<const W: usize>(w: &mut [i32], s: usize) -> &mut [i32; W] {
    (&mut w[s..s + W]).try_into().expect("row")
}

/// The three mixers' arithmetic: the exact dot products of the inputs with
/// three weight rows, and the bounded Hebbian step on those rows. Every kernel
/// computes the same integers; only the instructions differ.
trait Kernel {
    fn dot<const W: usize>(x: &[i32; W], w1: &[i32; W], w2: &[i32; W], w3: &[i32; W]) -> [i64; 3];
    fn train<const W: usize>(
        x: &[i32; W],
        w1: &mut [i32; W],
        w2: &mut [i32; W],
        w3: &mut [i32; W],
        e: [i32; 3],
    );

    /// The exact dot product of the inputs with one weight row (a shared
    /// pass's twin mixer 1): the same integers as the first of [`Kernel::dot`].
    #[inline(always)]
    fn dot1<const W: usize>(x: &[i32; W], w: &[i32; W]) -> i64 {
        let mut a = 0i64;
        for i in 0..W {
            a += i64::from(x[i]) * i64::from(w[i]);
        }
        a
    }

    /// The bounded Hebbian step on one weight row, as [`Kernel::train`].
    #[inline(always)]
    fn train1<const W: usize>(x: &[i32; W], w: &mut [i32; W], e: i32) {
        for i in 0..W {
            w[i] = (w[i] + ((x[i] * e) >> 16)).clamp(-WEIGHT_LIMIT, WEIGHT_LIMIT);
        }
    }
}

/// The portable kernel (x86_64 without AVX2, and wasm without simd128).
#[cfg_attr(
    any(
        target_arch = "aarch64",
        all(target_arch = "wasm32", target_feature = "simd128")
    ),
    allow(dead_code)
)]
struct Scalar;

impl Kernel for Scalar {
    #[inline(always)]
    fn dot<const W: usize>(x: &[i32; W], w1: &[i32; W], w2: &[i32; W], w3: &[i32; W]) -> [i64; 3] {
        let (mut a, mut b, mut c) = (0i64, 0i64, 0i64);
        for i in 0..W {
            let x = i64::from(x[i]);
            a += x * i64::from(w1[i]);
            b += x * i64::from(w2[i]);
            c += x * i64::from(w3[i]);
        }
        [a, b, c]
    }

    #[inline(always)]
    fn train<const W: usize>(
        x: &[i32; W],
        w1: &mut [i32; W],
        w2: &mut [i32; W],
        w3: &mut [i32; W],
        e: [i32; 3],
    ) {
        for i in 0..W {
            let learn = |w: &mut i32, e: i32| {
                *w = (*w + ((x[i] * e) >> 16)).clamp(-WEIGHT_LIMIT, WEIGHT_LIMIT);
            };
            learn(&mut w1[i], e[0]);
            learn(&mut w2[i], e[1]);
            learn(&mut w3[i], e[2]);
        }
    }
}

/// Runs `f::<K>(args)` with the best kernel this processor has: on x86_64,
/// AVX2 when runtime detection finds it (the whole coder loop is then compiled
/// with AVX2), else the portable kernel; NEON on aarch64, which always has it.
macro_rules! dispatch {
    ($f:ident($($a:expr),*)) => {{
        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("avx2") {
                // SAFETY: AVX2 was detected on this processor just above.
                return unsafe { x86::$f($($a),*) };
            }
            $f::<Scalar>($($a),*)
        }
        #[cfg(target_arch = "aarch64")]
        {
            $f::<arm::Neon>($($a),*)
        }
        #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
        {
            $f::<wasm::Simd128>($($a),*)
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            all(target_arch = "wasm32", target_feature = "simd128")
        )))]
        {
            $f::<Scalar>($($a),*)
        }
    }};
}

pub mod streams;

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::{Kernel, WEIGHT_LIMIT};
    use std::arch::x86_64::*;

    pub(super) struct Avx2;

    /// The encoder loop compiled for AVX2.
    #[target_feature(enable = "avx2")]
    pub(super) fn encoder(
        history: &[u8],
        state: &[u8],
        data: &[u8],
        profile: u32,
        learn: bool,
        stop: usize,
    ) -> Option<(Vec<u8>, u32)> {
        super::encoder::<Avx2>(history, state, data, profile, learn, stop)
    }

    /// The shared-pass encoder loop compiled for AVX2.
    #[target_feature(enable = "avx2")]
    pub(super) fn encoder_pair(
        history: &[u8],
        state: &[u8],
        data: &[u8],
        profile: u32,
        learn: [bool; 2],
        stops: [usize; 2],
    ) -> super::Pairs {
        super::encoder_pair::<Avx2>(history, state, data, profile, learn, stops)
    }

    /// The decoder loop compiled for AVX2.
    #[target_feature(enable = "avx2")]
    pub(super) fn decoder(
        history: &[&[u8]],
        state: &[u8],
        payload: &[u8],
        limit: u64,
        canonical: bool,
        profile: u32,
    ) -> Option<Vec<u8>> {
        super::decoder::<Avx2>(history, state, payload, limit, canonical, profile)
    }

    /// Eight lanes of a row.
    ///
    /// # Safety
    /// AVX2 must be available and `i * 8 + 8 <= W`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn lanes<const W: usize>(r: &[i32; W], i: usize) -> __m256i {
        // SAFETY: the caller keeps the eight lanes inside the row.
        unsafe { _mm256_loadu_si256(r.as_ptr().add(i * 8).cast()) }
    }

    /// The exact i64 sums of `x[j] * w[j]` over even and odd lanes.
    #[target_feature(enable = "avx2")]
    #[inline]
    fn madd(acc: __m256i, xe: __m256i, xo: __m256i, w: __m256i) -> __m256i {
        let even = _mm256_mul_epi32(xe, w);
        let odd = _mm256_mul_epi32(xo, _mm256_srli_epi64::<32>(w));
        _mm256_add_epi64(acc, _mm256_add_epi64(even, odd))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    fn hsum(v: __m256i) -> i64 {
        let s = _mm_add_epi64(_mm256_castsi256_si128(v), _mm256_extracti128_si256::<1>(v));
        _mm_cvtsi128_si64(s) + _mm_extract_epi64::<1>(s)
    }

    /// # Safety
    /// AVX2 must be available.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn dot<const W: usize>(
        x: &[i32; W],
        w1: &[i32; W],
        w2: &[i32; W],
        w3: &[i32; W],
    ) -> [i64; 3] {
        let (mut a, mut b, mut c) = (
            _mm256_setzero_si256(),
            _mm256_setzero_si256(),
            _mm256_setzero_si256(),
        );
        for i in 0..W / 8 {
            // SAFETY: AVX2 is available and i < W / 8.
            unsafe {
                let xe = lanes(x, i);
                let xo = _mm256_srli_epi64::<32>(xe);
                a = madd(a, xe, xo, lanes(w1, i));
                b = madd(b, xe, xo, lanes(w2, i));
                c = madd(c, xe, xo, lanes(w3, i));
            }
        }
        [hsum(a), hsum(b), hsum(c)]
    }

    /// # Safety
    /// AVX2 must be available.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn train<const W: usize>(
        x: &[i32; W],
        w1: &mut [i32; W],
        w2: &mut [i32; W],
        w3: &mut [i32; W],
        e: [i32; 3],
    ) {
        let (hi, lo) = (
            _mm256_set1_epi32(WEIGHT_LIMIT),
            _mm256_set1_epi32(-WEIGHT_LIMIT),
        );
        for i in 0..W / 8 {
            // SAFETY: AVX2 is available and i < W / 8.
            unsafe {
                let xv = lanes(x, i);
                step(w1, i, xv, e[0], hi, lo);
                step(w2, i, xv, e[1], hi, lo);
                step(w3, i, xv, e[2], hi, lo);
            }
        }
    }

    /// `w += (x * e) >> 16`, clamped to `lo..=hi`, on eight lanes of a row.
    ///
    /// # Safety
    /// AVX2 must be available and `i * 8 + 8 <= W`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn step<const W: usize>(
        w: &mut [i32; W],
        i: usize,
        x: __m256i,
        e: i32,
        hi: __m256i,
        lo: __m256i,
    ) {
        // SAFETY: the caller keeps the eight lanes inside the row.
        unsafe {
            let p = w.as_mut_ptr().add(i * 8).cast::<__m256i>();
            let d = _mm256_srai_epi32::<16>(_mm256_mullo_epi32(x, _mm256_set1_epi32(e)));
            let v = _mm256_add_epi32(_mm256_loadu_si256(p), d);
            _mm256_storeu_si256(p, _mm256_max_epi32(_mm256_min_epi32(v, hi), lo));
        }
    }

    impl Kernel for Avx2 {
        #[inline(always)]
        fn dot<const W: usize>(
            x: &[i32; W],
            w1: &[i32; W],
            w2: &[i32; W],
            w3: &[i32; W],
        ) -> [i64; 3] {
            // SAFETY: `Avx2` is only used once AVX2 has been detected.
            unsafe { dot(x, w1, w2, w3) }
        }

        #[inline(always)]
        fn train<const W: usize>(
            x: &[i32; W],
            w1: &mut [i32; W],
            w2: &mut [i32; W],
            w3: &mut [i32; W],
            e: [i32; 3],
        ) {
            // SAFETY: `Avx2` is only used once AVX2 has been detected.
            unsafe { train(x, w1, w2, w3, e) }
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod arm {
    use super::{Kernel, WEIGHT_LIMIT};
    use std::arch::aarch64::*;

    pub(super) struct Neon;

    impl Kernel for Neon {
        #[inline(always)]
        fn dot<const W: usize>(
            x: &[i32; W],
            w1: &[i32; W],
            w2: &[i32; W],
            w3: &[i32; W],
        ) -> [i64; 3] {
            // SAFETY: NEON is part of aarch64, and every 4-lane load at
            // i * 4 < W stays inside its row.
            unsafe {
                let (mut a, mut b, mut c) = (vdupq_n_s64(0), vdupq_n_s64(0), vdupq_n_s64(0));
                for i in 0..W / 4 {
                    let xv = vld1q_s32(x.as_ptr().add(i * 4));
                    let xl = vget_low_s32(xv);
                    let acc = |s: int64x2_t, w: &[i32; W]| {
                        let wv = vld1q_s32(w.as_ptr().add(i * 4));
                        vmlal_high_s32(vmlal_s32(s, xl, vget_low_s32(wv)), xv, wv)
                    };
                    a = acc(a, w1);
                    b = acc(b, w2);
                    c = acc(c, w3);
                }
                [vaddvq_s64(a), vaddvq_s64(b), vaddvq_s64(c)]
            }
        }

        #[inline(always)]
        fn train<const W: usize>(
            x: &[i32; W],
            w1: &mut [i32; W],
            w2: &mut [i32; W],
            w3: &mut [i32; W],
            e: [i32; 3],
        ) {
            // SAFETY: NEON is part of aarch64, and every 4-lane access at
            // i * 4 < W stays inside its row.
            unsafe {
                let (hi, lo) = (vdupq_n_s32(WEIGHT_LIMIT), vdupq_n_s32(-WEIGHT_LIMIT));
                for i in 0..W / 4 {
                    let xv = vld1q_s32(x.as_ptr().add(i * 4));
                    for (w, e) in [(&mut *w1, e[0]), (&mut *w2, e[1]), (&mut *w3, e[2])] {
                        let p = w.as_mut_ptr().add(i * 4);
                        let d = vshrq_n_s32::<16>(vmulq_n_s32(xv, e));
                        vst1q_s32(p, vmaxq_s32(vminq_s32(vaddq_s32(vld1q_s32(p), d), hi), lo));
                    }
                }
            }
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
mod wasm {
    use super::{Kernel, WEIGHT_LIMIT};
    use std::arch::wasm32::*;

    /// The WebAssembly SIMD128 kernel: 4 lanes of i32, products widened to
    /// i64 pairs exactly as the scalar kernel computes them.
    pub(super) struct Simd128;

    impl Kernel for Simd128 {
        #[inline(always)]
        fn dot<const W: usize>(
            x: &[i32; W],
            w1: &[i32; W],
            w2: &[i32; W],
            w3: &[i32; W],
        ) -> [i64; 3] {
            // SAFETY: simd128 is enabled at compile time for this module, and
            // every 4-lane load at i * 4 < W (W a multiple of 4) stays inside
            // its row; v128_load takes unaligned addresses.
            unsafe {
                let (mut a, mut b, mut c) = (i64x2_splat(0), i64x2_splat(0), i64x2_splat(0));
                for i in 0..W / 4 {
                    let xv = v128_load(x.as_ptr().add(i * 4).cast());
                    let acc = |s: v128, w: &[i32; W]| {
                        let wv = v128_load(w.as_ptr().add(i * 4).cast());
                        i64x2_add(
                            i64x2_add(s, i64x2_extmul_low_i32x4(xv, wv)),
                            i64x2_extmul_high_i32x4(xv, wv),
                        )
                    };
                    a = acc(a, w1);
                    b = acc(b, w2);
                    c = acc(c, w3);
                }
                let sum = |v: v128| i64x2_extract_lane::<0>(v) + i64x2_extract_lane::<1>(v);
                [sum(a), sum(b), sum(c)]
            }
        }

        #[inline(always)]
        fn train<const W: usize>(
            x: &[i32; W],
            w1: &mut [i32; W],
            w2: &mut [i32; W],
            w3: &mut [i32; W],
            e: [i32; 3],
        ) {
            // SAFETY: as in `dot`; the stores write the rows they loaded.
            unsafe {
                let (hi, lo) = (i32x4_splat(WEIGHT_LIMIT), i32x4_splat(-WEIGHT_LIMIT));
                for i in 0..W / 4 {
                    let xv = v128_load(x.as_ptr().add(i * 4).cast());
                    for (w, e) in [(&mut *w1, e[0]), (&mut *w2, e[1]), (&mut *w3, e[2])] {
                        let p = w.as_mut_ptr().add(i * 4).cast::<v128>();
                        let d = i32x4_shr(i32x4_mul(xv, i32x4_splat(e)), 16);
                        v128_store(p, i32x4_max(i32x4_min(i32x4_add(v128_load(p), d), hi), lo));
                    }
                }
            }
        }
    }
}

/// Input bytes between two calls of the progress hook: 16 KiB, about a
/// tenth of a second of coding in the browser, so the page's bar moves
/// within a frame and soon after a file starts.
pub const TICK: usize = 1 << 14;

/// A progress hook: bytes done, and the length of the pass.
pub type Progress = fn(u64, u64);

std::thread_local! {
    static PROGRESS: std::cell::Cell<Option<Progress>> = const { std::cell::Cell::new(None) };
}

/// Set (or clear) this thread's progress hook: every coding or decoding
/// pass over a frame calls it with the bytes done and the pass's length, at
/// the start and once per [`TICK`] bytes. It never changes a payload.
pub fn set_progress(hook: Option<Progress>) {
    PROGRESS.with(|p| p.set(hook));
}

#[inline(never)]
fn tick(done: usize, total: usize) {
    if let Some(hook) = PROGRESS.with(std::cell::Cell::get) {
        hook(done as u64, total as u64);
    }
}

/// Encode `data` as an ncm payload, or `None` when the coder output would pass
/// `2 * len + 64` bytes (the model abandons the encoding there) or the input
/// does not fit the 4-byte length.
pub fn encode(data: &[u8]) -> Option<Vec<u8>> {
    encode_with(&[], data)
}

/// [`encode`] for a window frame: `data` coded after `history`, which the
/// match model reaches into (`Ncm.encodeWith`).
pub fn encode_with(history: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    encode_profile(history, data, V03)
}

/// Whether `profile` names an ncm profile: a set of the 17 contexts and the
/// [`F_SHORT`], [`F_REP`] and [`F_SEL`] features, and a table field that is 0
/// or records [`TABLE_MIN`] to [`TABLE_MAX`] bits.
pub fn valid_profile(profile: u32) -> bool {
    let tb = (profile >> TABLE_SHIFT) & TABLE_FIELD;
    profile >> 31 == 0 && (tb == 0 || (TABLE_MIN..=TABLE_MAX).contains(&tb))
}

/// The context table bits a profile word records, if it records them.
pub fn recorded_table_bits(profile: u32) -> Option<u32> {
    let tb = (profile >> TABLE_SHIFT) & TABLE_FIELD;
    (tb != 0).then_some(tb)
}

/// The match tables' bits a profile word records, if it records them.
pub fn recorded_match_bits(profile: u32) -> Option<u32> {
    let mb = (profile >> MATCH_SHIFT) & MATCH_FIELD;
    (mb != 0).then_some(mb)
}

/// Whether a profile word records any table field (only the current
/// library's ncm blocks read them).
pub fn records_tables(profile: u32) -> bool {
    profile >> TABLE_SHIFT != 0
}

/// The context table bits of a model coding `modelled` bytes (the frame, or
/// a state and the frame) with no recorded table field: the least `tb` with
/// 2^tb >= 1024 `modelled`, from 16 to 24 (`Model::sized`).
pub fn default_table_bits(modelled: u64) -> u32 {
    let mut tb = 16;
    while tb < TB_MAX && (1u64 << tb) < modelled.saturating_mul(1024) {
        tb += 1;
    }
    tb
}

/// The table bits a model coding `modelled` bytes under `profile` uses.
pub fn table_bits(modelled: u64, profile: u32) -> u32 {
    recorded_table_bits(profile).unwrap_or_else(|| default_table_bits(modelled))
}

/// `profile` with the context table field set to `tb` bits (0 clears it);
/// the other fields are kept.
pub fn with_table_bits(profile: u32, tb: u32) -> u32 {
    (profile & !(TABLE_FIELD << TABLE_SHIFT)) | ((tb & TABLE_FIELD) << TABLE_SHIFT)
}

/// `profile` with the match tables' field set to `mb` bits (0 clears it);
/// the other fields are kept.
pub fn with_match_bits(profile: u32, mb: u32) -> u32 {
    (profile & !(MATCH_FIELD << MATCH_SHIFT)) | ((mb & MATCH_FIELD) << MATCH_SHIFT)
}

/// The match tables' bits a model coding `n` bytes after `history` bytes
/// under `profile` uses: the recorded field, else the size the coded length
/// alone gives (`Model::sized`).
pub fn match_bits(n: u64, profile: u32) -> u32 {
    recorded_match_bits(profile)
        .unwrap_or_else(|| Model::sized(u32::try_from(n).unwrap_or(u32::MAX)).1)
}

/// Bytes a decoder of this coder holds for a frame of `n` bytes after
/// `history` bytes (a window's frames, or with `warm` a model state the
/// model codes first) under `profile`: the context table (4 bytes a slot),
/// the match table and, with [`F_SHORT`], the short match table (4 bytes an
/// entry), and the history and frame bytes themselves. The client compares
/// it with the declared decode host before it allocates.
pub fn decoder_bytes(n: u64, history: u64, warm: bool, profile: u32) -> u64 {
    table_bytes(n, history, warm, profile).saturating_add(n.saturating_add(history))
}

/// The tables' part of [`decoder_bytes`]: the context table and the match
/// tables.
pub fn table_bytes(n: u64, history: u64, warm: bool, profile: u32) -> u64 {
    let modelled = if warm { n.saturating_add(history) } else { n };
    let tb = table_bits(modelled, profile);
    let mb = recorded_match_bits(profile)
        .unwrap_or_else(|| Model::sized(u32::try_from(modelled).unwrap_or(u32::MAX)).1);
    let matches = 1 + u64::from(profile & F_SHORT != 0);
    (4u64 << tb).saturating_add((4u64 << mb) * matches)
}

/// [`encode_with`] under the model `profile` (a [`valid_profile`]; [`V03`] is
/// the v0.3 coder, [`FULL`] every feature): `None` for any other word, or
/// when [`encode`] would give `None`. The payload decodes only under the same
/// profile.
pub fn encode_profile(history: &[u8], data: &[u8], profile: u32) -> Option<Vec<u8>> {
    if !valid_profile(profile) {
        return None;
    }
    encode_below(history, data, profile, usize::MAX)
}

/// [`encode_profile`], abandoned (`None`) as soon as the payload would reach
/// `stop` bytes: the caller's bound past which the payload cannot win. The
/// payload only grows as bytes are coded, so a payload this returns is the
/// one [`encode_profile`] returns.
pub fn encode_below(history: &[u8], data: &[u8], profile: u32, stop: usize) -> Option<Vec<u8>> {
    if !valid_profile(profile) {
        return None;
    }
    encode_any(history, data, profile, false, stop).map(|(payload, _)| payload)
}

fn encode_any(
    history: &[u8],
    data: &[u8],
    profile: u32,
    learn: bool,
    stop: usize,
) -> Option<(Vec<u8>, u32)> {
    dispatch!(encoder(history, &[], data, profile, learn, stop))
}

/// [`encode_profile`], and the profile the encoder learned for this input:
/// `profile` without the contexts whose leave-one-out gain over the coded bits
/// was not positive (`NcmEncode.learnProfile`). The client codes that profile
/// too and keeps the smaller frame.
pub fn encode_learn(history: &[u8], data: &[u8], profile: u32) -> Option<(Vec<u8>, u32)> {
    encode_learn_below(history, data, profile, usize::MAX)
}

/// [`encode_learn`] abandoned at `stop` payload bytes, as [`encode_below`].
pub fn encode_learn_below(
    history: &[u8],
    data: &[u8],
    profile: u32,
    stop: usize,
) -> Option<(Vec<u8>, u32)> {
    if !valid_profile(profile) {
        return None;
    }
    encode_any(history, data, profile, true, stop)
}

/// One pass that codes `data` under `profile` (learning its mask with
/// `learn`, as [`encode_learn_below`]) and under its twin `profile ^ F_SEL`
/// (as [`encode_below`]), each abandoned at its own stop. The two profiles
/// read the same contexts, slots, match and repeat models and mixers 2 and 3;
/// the twin keeps its own mixer 1, APMs and coder, so each payload is the one
/// its own pass writes, for one pass of the shared work.
pub fn encode_pair_below(
    history: &[u8],
    data: &[u8],
    profile: u32,
    learn: bool,
    stops: [usize; 2],
) -> Pair {
    let (a, b) = encode_pairs(history, data, profile, [learn, false], stops);
    (a, b.map(|(payload, _)| payload))
}

/// [`encode_pair_below`] with both profiles learning their masks, as
/// [`encode_learn_below`] does for each: the twin learns from its own mixer 1
/// over the shared mixers 2 and 3, so each learned profile is the one its own
/// learning pass finds.
pub fn encode_pairs_learn_below(
    history: &[u8],
    data: &[u8],
    profile: u32,
    stops: [usize; 2],
) -> Pairs {
    encode_pairs(history, data, profile, [true, true], stops)
}

fn encode_pairs(
    history: &[u8],
    data: &[u8],
    profile: u32,
    learn: [bool; 2],
    stops: [usize; 2],
) -> Pairs {
    if !valid_profile(profile) {
        return (None, None);
    }
    dispatch!(encoder_pair(history, &[], data, profile, learn, stops))
}

/// The payloads of `profile` and of its [`F_SEL`] twin, each coded as the
/// continuation of the model state `state` ([`encode_warm_below`]), from one
/// pass: one replay of the state through the work the twins share, then one
/// pass over `data`, each abandoned at its own stop. Each payload is the one
/// [`encode_warm_below`] writes for its profile.
pub fn encode_pair_warm_below(
    state: &[u8],
    data: &[u8],
    profile: u32,
    stops: [usize; 2],
) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    if !valid_profile(profile) || state.is_empty() {
        return (None, None);
    }
    let (a, b) = warm_pair(state, data, profile, stops);
    (a.map(|(p, _)| p), b.map(|(p, _)| p))
}

fn warm_pair(state: &[u8], data: &[u8], profile: u32, stops: [usize; 2]) -> Pairs {
    dispatch!(encoder_pair(
        &[],
        state,
        data,
        profile,
        [false, false],
        stops
    ))
}

#[inline(always)]
fn encoder<K: Kernel>(
    history: &[u8],
    state: &[u8],
    data: &[u8],
    profile: u32,
    learn: bool,
    stop: usize,
) -> Option<(Vec<u8>, u32)> {
    let n = u32::try_from(data.len()).ok()?;
    let m = if state.is_empty() {
        Model::new(n, &[history], profile)
    } else {
        Model::warmed::<K>(n, state, profile)?
    };
    run_encoder::<K>(m, data, learn, [stop, 0]).0
}

/// The payloads of `profile` (learning its mask with `learn`) and of its
/// [`F_SEL`] twin, from one pass, each abandoned at its own stop.
pub type Pair = (Option<(Vec<u8>, u32)>, Option<Vec<u8>>);

/// The payloads and learned profiles of a profile and of its [`F_SEL`] twin
/// (see [`encode_pairs_learn_below`]).
pub type Pairs = (Option<(Vec<u8>, u32)>, Option<(Vec<u8>, u32)>);

#[inline(always)]
fn encoder_pair<K: Kernel>(
    history: &[u8],
    state: &[u8],
    data: &[u8],
    profile: u32,
    learn: [bool; 2],
    stops: [usize; 2],
) -> Pairs {
    let modelled = u32::try_from(data.len())
        .ok()
        .zip(u32::try_from(state.len()).ok())
        .and_then(|(n, w)| n.checked_add(w).map(|total| (n, total)));
    let Some((n, total)) = modelled else {
        return (None, None);
    };
    // with a state, the model `Model::warmed` builds, sized for the state and
    // the frame; the twin replays the state with it below
    let mut m = if state.is_empty() {
        Model::new(n, &[history], profile)
    } else {
        Model::new(total, &[], profile)
    };
    m.twin = Some(Box::new(Twin {
        sel: profile & F_SEL == 0,
        w1: weights(256 * 9, m.na + 3, m.stride),
        a1: Apm::new(256 * 9),
        a2: Apm::new(1 << APM2_BITS),
        s1: 0,
        pm1: 0,
        pr: 2048,
        dot1: 0,
        gain: learn[1].then(|| Box::new([0; NC])),
    }));
    if !state.is_empty() {
        if m.stride == NINP_NARROW {
            m.warm::<K, NINP_NARROW>(state);
        } else {
            m.warm::<K, NINP>(state);
        }
    }
    run_encoder::<K>(m, data, learn[0], stops)
}

#[inline(always)]
fn run_encoder<K: Kernel>(mut m: Model, data: &[u8], learn: bool, stops: [usize; 2]) -> Pairs {
    if learn {
        m.gain = Some(Box::new([0; NC]));
    }
    if m.stride == NINP_NARROW {
        encode_bits::<K, NINP_NARROW>(m, data, stops)
    } else {
        encode_bits::<K, NINP>(m, data, stops)
    }
}

/// One binary arithmetic coder: its interval, its output and the cap past
/// which its payload is abandoned.
struct Coder {
    x1: u32,
    x2: u32,
    out: Vec<u8>,
    cap: usize,
    live: bool,
}

impl Coder {
    fn new(len: usize, stop: usize, live: bool) -> Coder {
        let mut out = Vec::with_capacity(if live { len / 2 + 16 } else { 0 });
        out.extend_from_slice(&(len as u32).to_be_bytes());
        Coder {
            x1: 0,
            x2: u32::MAX,
            out,
            // the payload ends one byte longer than the bytes emitted so far
            cap: (2 * len + 64).min(stop.saturating_sub(5)),
            live,
        }
    }

    /// Codes bit `y` at probability `p` (of a 1, in 4096ths); a coder whose
    /// output reaches its cap is abandoned.
    #[inline(always)]
    fn code(&mut self, y: bool, p: u32) {
        let (mut x1, mut x2) = (self.x1, self.x2);
        let xm = x1 + ((u64::from(x2 - x1) * u64::from(p)) >> 12) as u32;
        if y {
            x2 = xm;
        } else {
            x1 = xm + 1;
        }
        while (x1 ^ x2) >> 24 == 0 {
            if self.out.len() - 4 >= self.cap {
                self.live = false;
                return;
            }
            self.out.push((x2 >> 24) as u8);
            x1 <<= 8;
            x2 = (x2 << 8) | 255;
        }
        self.x1 = x1;
        self.x2 = x2;
    }

    fn finish(mut self) -> Option<Vec<u8>> {
        if !self.live || self.out.len() - 4 >= self.cap {
            return None;
        }
        self.out.push((self.x1 >> 24) as u8);
        Some(self.out)
    }
}

#[inline(always)]
fn encode_bits<K: Kernel, const W: usize>(mut m: Model, data: &[u8], stops: [usize; 2]) -> Pairs {
    let twin = m.twin.is_some();
    let learning = m.gain.is_some() || m.twin.as_ref().is_some_and(|t| t.gain.is_some());
    let mut main = Coder::new(data.len(), stops[0], true);
    let mut other = Coder::new(data.len(), stops[1], twin);
    for (i, &byte) in data.iter().enumerate() {
        if i % TICK == 0 {
            tick(i, data.len());
        }
        if !main.live && !other.live {
            return (None, None);
        }
        for k in (0..8).rev() {
            let y = (byte >> k) & 1 == 1;
            let p = m.predict::<K, W>();
            if main.live {
                main.code(y, p);
            }
            if other.live {
                let q = m.twin.as_deref().map_or(2048, |t| t.pr);
                other.code(y, q);
            }
            if learning {
                m.learn::<W>(y);
            }
            m.update::<K, W>(y);
        }
    }
    let (learned, twin_learned) = (m.learned(), m.twin_learned());
    (
        main.finish().map(|p| (p, learned)),
        other.finish().map(|p| (p, twin_learned)),
    )
}

/// [`encode_profile`] from the κ-named model state `state` (`Ncm.runWarm`):
/// `data` is coded as the continuation of the state's bytes, which the whole
/// model learns first. The payload decodes only from the same state and profile.
pub fn encode_warm(state: &[u8], data: &[u8], profile: u32) -> Option<Vec<u8>> {
    encode_warm_below(state, data, profile, usize::MAX)
}

/// [`encode_warm`] abandoned at `stop` payload bytes, as [`encode_below`].
pub fn encode_warm_below(state: &[u8], data: &[u8], profile: u32, stop: usize) -> Option<Vec<u8>> {
    if !valid_profile(profile) || state.is_empty() {
        return None;
    }
    warm_encoder(state, data, profile, stop).map(|(payload, _)| payload)
}

fn warm_encoder(state: &[u8], data: &[u8], profile: u32, stop: usize) -> Option<(Vec<u8>, u32)> {
    dispatch!(encoder(&[], state, data, profile, false, stop))
}

/// Decode a payload [`encode_warm`] wrote from `state` under `profile`.
pub fn decode_warm(state: &[u8], payload: &[u8], limit: u64, profile: u32) -> Option<Vec<u8>> {
    if !valid_profile(profile) || state.is_empty() {
        return None;
    }
    dispatch!(decoder(&[], state, payload, limit, false, profile))
}

/// [`decode_warm`] of a payload of exactly `n` bytes that [`encode_warm`]
/// writes for its own output, checked in the same pass ([`decode_canonical`]).
pub fn decode_canonical_warm(
    state: &[u8],
    payload: &[u8],
    n: u64,
    profile: u32,
) -> Option<Vec<u8>> {
    if !valid_profile(profile)
        || state.is_empty()
        || declared_length(payload).map(u64::from) != Some(n)
    {
        return None;
    }
    dispatch!(decoder(&[], state, payload, n, true, profile))
}

/// The output length a payload declares, if it has its 4-byte header.
pub fn declared_length(payload: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(payload.get(..4)?.try_into().ok()?))
}

/// Decode an ncm payload. Every payload of at least 4 bytes has exactly one
/// meaning (bytes past its end read as 255), so this is the kernel graph's
/// meaning; `limit` bounds the declared length the caller will accept.
pub fn decode(payload: &[u8], limit: u64) -> Option<Vec<u8>> {
    run_decoder(&[], payload, limit, false)
}

/// [`decode`] for a window frame coded after `history`.
pub fn decode_with(history: &[u8], payload: &[u8], limit: u64) -> Option<Vec<u8>> {
    run_decoder(history, payload, limit, false)
}

/// Decode a payload of exactly `n` bytes that [`encode`] writes for its own
/// output, as `Archive.linkedNcm` requires (`encode(decode(p)) == p`), in one
/// pass: the encoder and the decoder share their coder state, so the bytes the
/// encoder would emit are checked against the payload as the decoder shifts.
pub fn decode_canonical(payload: &[u8], n: u64) -> Option<Vec<u8>> {
    if declared_length(payload).map(u64::from) != Some(n) {
        return None;
    }
    run_decoder(&[], payload, n, true)
}

/// [`decode_canonical`] for a window frame coded after `history`.
pub fn decode_canonical_with(history: &[u8], payload: &[u8], n: u64) -> Option<Vec<u8>> {
    if declared_length(payload).map(u64::from) != Some(n) {
        return None;
    }
    run_decoder(history, payload, n, true)
}

/// [`decode_canonical_with`] for a payload coded with `profile` (a
/// [`valid_profile`]).
pub fn decode_canonical_profile(
    history: &[u8],
    payload: &[u8],
    n: u64,
    profile: u32,
) -> Option<Vec<u8>> {
    if declared_length(payload).map(u64::from) != Some(n) {
        return None;
    }
    run_profile(history, payload, n, true, profile)
}

/// [`decode_canonical_profile`] after the frames of a window, without
/// joining them first: the model holds the only copy of the history.
pub fn decode_canonical_window(
    window: &[&[u8]],
    payload: &[u8],
    n: u64,
    profile: u32,
) -> Option<Vec<u8>> {
    if declared_length(payload).map(u64::from) != Some(n) {
        return None;
    }
    run_pieces(window, payload, n, true, profile)
}

fn run_decoder(history: &[u8], payload: &[u8], limit: u64, canonical: bool) -> Option<Vec<u8>> {
    run_profile(history, payload, limit, canonical, V03)
}

/// The decoder under the model `profile`: `None` for a word that is not a
/// [`valid_profile`].
pub(crate) fn run_profile(
    history: &[u8],
    payload: &[u8],
    limit: u64,
    canonical: bool,
    profile: u32,
) -> Option<Vec<u8>> {
    run_pieces(&[history], payload, limit, canonical, profile)
}

/// [`run_profile`] after a history given as pieces.
fn run_pieces(
    history: &[&[u8]],
    payload: &[u8],
    limit: u64,
    canonical: bool,
    profile: u32,
) -> Option<Vec<u8>> {
    if !valid_profile(profile) {
        return None;
    }
    dispatch!(decoder(history, &[], payload, limit, canonical, profile))
}

/// Decode a payload [`encode_profile`] wrote under `profile`.
pub fn decode_profile(history: &[u8], payload: &[u8], limit: u64, profile: u32) -> Option<Vec<u8>> {
    run_profile(history, payload, limit, false, profile)
}

#[inline(always)]
fn decoder<K: Kernel>(
    history: &[&[u8]],
    state: &[u8],
    payload: &[u8],
    limit: u64,
    canonical: bool,
    profile: u32,
) -> Option<Vec<u8>> {
    let n = declared_length(payload)?;
    if u64::from(n) > limit {
        return None;
    }
    let (m, ahead) = if state.is_empty() {
        let m = Model::new(n, history, profile);
        let before = m.hoff as usize;
        (m, before)
    } else {
        (Model::warmed::<K>(n, state, profile)?, state.len())
    };
    if m.stride == NINP_NARROW {
        decode_bits::<K, NINP_NARROW>(m, ahead, payload, canonical)
    } else {
        decode_bits::<K, NINP>(m, ahead, payload, canonical)
    }
}

#[inline(always)]
fn decode_bits<K: Kernel, const W: usize>(
    mut m: Model,
    history: usize,
    payload: &[u8],
    canonical: bool,
) -> Option<Vec<u8>> {
    let n = declared_length(payload)?;
    let cap = 2 * n as usize + 64;
    let mut ip = 4usize;
    let mut x = 0u32;
    for _ in 0..4 {
        x = (x << 8) | u32::from(payload.get(ip).copied().unwrap_or(255));
        ip += 1;
    }
    // coder bytes the encoder has emitted so far
    let mut emitted = 0usize;
    let (mut x1, mut x2) = (0u32, u32::MAX);
    for i in 0..n as usize {
        if i % TICK == 0 {
            tick(i, n as usize);
        }
        for _ in 0..8 {
            let p = m.predict::<K, W>();
            let xm = x1 + ((u64::from(x2 - x1) * u64::from(p)) >> 12) as u32;
            let y = x <= xm;
            if y {
                x2 = xm;
            } else {
                x1 = xm + 1;
            }
            while (x1 ^ x2) >> 24 == 0 {
                if canonical {
                    if emitted >= cap || payload.get(4 + emitted) != Some(&((x2 >> 24) as u8)) {
                        return None;
                    }
                    emitted += 1;
                }
                x1 <<= 8;
                x2 = (x2 << 8) | 255;
                x = (x << 8) | u32::from(payload.get(ip).copied().unwrap_or(255));
                ip += 1;
            }
            m.update::<K, W>(y);
        }
    }
    if canonical
        && (emitted >= cap
            || payload.len() != 4 + emitted + 1
            || payload[4 + emitted] != (x1 >> 24) as u8)
    {
        return None;
    }
    let mut out = m.hist;
    out.drain(..history);
    // a warmed model kept for reuse was sized for the frame it first coded
    out.truncate(n as usize);
    // Give back the history's share of the buffer: the output outlives the
    // model (the next frame's window holds it).
    out.shrink_to_fit();
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warm_state_is_the_model_coder() {
        // client/gen/ncm_model.py py_encode_warm: FULL then V03
        let hex = |t: &str| -> Vec<u8> {
            (0..t.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&t[i..i + 2], 16).unwrap())
                .collect()
        };
        let fx: Vec<Vec<u8>> = include_str!("../tests/ncm-warm.fixture")
            .lines()
            .map(hex)
            .collect();
        let (state, x) = (&fx[0], &fx[1]);
        for (payload, profile) in [(&fx[2], FULL), (&fx[3], V03)] {
            assert_eq!(encode_warm(state, x, profile).as_ref(), Some(payload));
            for _ in 0..2 {
                assert_eq!(
                    decode_canonical_warm(state, payload, x.len() as u64, profile).as_ref(),
                    Some(x)
                );
            }
            assert_eq!(
                encoder::<Scalar>(&[], state, x, profile, false, usize::MAX)
                    .map(|r| r.0)
                    .as_ref(),
                Some(payload)
            );
        }
        assert_ne!(
            decode_warm(&state[1..], &fx[2], u64::MAX, FULL).as_ref(),
            Some(x)
        );
    }

    #[test]
    fn round_trips() {
        let text = b"the quick brown fox jumps over the lazy dog; ".repeat(50);
        for data in [&b""[..], b"a", b"abcabcabcabc", &text[..], &[0u8; 5000][..]] {
            let p = encode(data).expect("fits");
            assert_eq!(decode(&p, u64::MAX).as_deref(), Some(data));
            assert_eq!(
                decode_canonical(&p, data.len() as u64).as_deref(),
                Some(data)
            );
        }
    }

    /// Every kernel this processor runs codes the same bytes as the portable
    /// one, so an archive never depends on the machine that wrote it.
    #[test]
    fn kernels_agree() {
        let mut x = 0x2545_F491u32;
        let noise: Vec<u8> = (0..20_000)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x >> 24) as u8
            })
            .collect();
        let rows: Vec<u8> = (0..30_000u32)
            .map(|i| (i % 37 * 7 + i / 999) as u8)
            .collect();
        let text = b"Sing, O goddess, the anger of Achilles son of Peleus. ".repeat(300);
        for data in [&b""[..], b"z", &noise[..], &rows[..], &text[..]] {
            let p = encoder::<Scalar>(&[], &[], data, V03, false, usize::MAX)
                .expect("fits")
                .0;
            assert_eq!(encode(data).as_deref(), Some(&p[..]));
            let n = data.len() as u64;
            assert_eq!(decode(&p, n).as_deref(), Some(data));
            assert_eq!(
                decoder::<Scalar>(&[], &[], &p, n, true, V03).as_deref(),
                Some(data)
            );
            // and after a window history
            let w = encoder::<Scalar>(&text[..999], &[], data, V03, false, usize::MAX)
                .expect("fits")
                .0;
            assert_eq!(encode_with(&text[..999], data).as_deref(), Some(&w[..]));
            assert_eq!(decode_with(&text[..999], &w, n).as_deref(), Some(data));
            // and under narrow and wide profiles
            for profile in [0x1059e, 0x1e79f, 1, FULL, 0x1059e | F_REP, V03 | F_SHORT] {
                let p = encoder::<Scalar>(&text[..999], &[], data, profile, false, usize::MAX)
                    .expect("fits")
                    .0;
                assert_eq!(
                    encode_profile(&text[..999], data, profile).as_deref(),
                    Some(&p[..])
                );
                assert_eq!(
                    decode_canonical_profile(&text[..999], &p, n, profile).as_deref(),
                    Some(data)
                );
            }
        }
    }

    /// A profile is the model with the left-out contexts' inputs held at 0:
    /// The learning encoder writes the same payload as the plain one and
    /// learns a profile inside its own, which codes and decodes; the cost LUT
    /// is 256 log2(4096 / p) to within 1.5 units (truncated fraction bits).
    #[test]
    fn learned_profiles_code_and_stay_inside() {
        let t = tables();
        assert_eq!((t.cost[1], t.cost[2048], t.cost[1024]), (3072, 256, 512));
        for p in 1..4096usize {
            let exact = 256.0 * (4096.0 / p as f64).log2();
            assert!((f64::from(t.cost[p]) - exact).abs() < 1.5, "{p}");
        }
        let text =
            b"learned profiles drop the contexts that saved nothing on this frame; ".repeat(40);
        for profile in [FULL, V03, 0x1059e | F_REP] {
            let (payload, learned) = encode_learn(b"a window", &text, profile).expect("fits");
            assert_eq!(
                Some(&payload),
                encode_profile(b"a window", &text, profile).as_ref()
            );
            assert_eq!(learned & !profile, 0);
            assert_eq!(learned >> NC, profile >> NC);
            let p = encode_profile(b"a window", &text, learned).expect("fits");
            assert_eq!(
                decode_profile(b"a window", &p, text.len() as u64, learned).as_deref(),
                Some(&text[..])
            );
        }
    }

    /// every profile codes, and only its own profile reads its payload.
    #[test]
    fn profiles_round_trip_and_differ() {
        let text = b"profiles leave contexts out; the mixers never see them. ".repeat(80);
        let full = encode(&text).expect("fits");
        assert_eq!(encode_profile(&[], &text, V03), Some(full.clone()));
        for profile in [0, 1, 0x1059e, 0x1e79f, V03 >> 1, FULL, F_SHORT | 0x1059e] {
            let p = encode_profile(&[], &text, profile).expect("fits");
            assert_ne!(p, full);
            let n = text.len() as u64;
            assert_eq!(
                decode_canonical_profile(&[], &p, n, profile).as_deref(),
                Some(&text[..])
            );
            assert_eq!(decode_canonical_profile(&[], &p, n, V03), None);
        }
        assert!(valid_profile(0) && !valid_profile(FULL + 1));
        assert_eq!(encode_profile(&[], &text, FULL + 1), None);
    }

    #[test]
    fn recorded_tables_are_the_tables_the_frame_codes_with() {
        let text: Vec<u8> = (0..40_000u32)
            .flat_map(|i| format!("row {} of {}\n", i % 977, i % 31).into_bytes())
            .take(60_000)
            .collect();
        let n = text.len() as u64;
        assert_eq!(default_table_bits(n), 24);
        assert!(valid_profile(with_table_bits(FULL, TABLE_MIN)));
        assert!(!valid_profile(with_table_bits(FULL, TABLE_MIN - 1)));
        assert!(!valid_profile(with_table_bits(FULL, TABLE_MAX + 1)));
        assert!(!valid_profile(FULL | 1 << 31));
        // recording the default size is the default frame; another size codes differently
        let plain = encode_profile(&[], &text, FULL).expect("fits");
        assert_eq!(
            encode_profile(&[], &text, with_table_bits(FULL, 24)),
            Some(plain.clone())
        );
        for word in [
            with_table_bits(FULL, 12),
            with_table_bits(FULL, 25),
            FULL | 9 << MATCH_SHIFT,
            with_table_bits(V03, 20) | 18 << MATCH_SHIFT,
        ] {
            let p = encode_profile(&[], &text, word).expect("fits");
            assert_ne!(p, plain);
            assert_eq!(
                decode_canonical_profile(&[], &p, n, word).as_deref(),
                Some(&text[..])
            );
            assert_ne!(
                decode_canonical_profile(&[], &p, n, FULL).as_deref(),
                Some(&text[..])
            );
        }
        // the footprint counts the tables the word names
        let base = decoder_bytes(n, 0, false, FULL);
        assert_eq!(
            decoder_bytes(n, 0, false, with_table_bits(FULL, 25)) - base,
            4 << 24
        );
        assert_eq!(base, (4 << 24) + 2 * (4 << 22) + n);
    }

    /// A bounded encode returns exactly the unbounded payload when that is
    /// shorter than the bound, and nothing otherwise.
    #[test]
    fn bounded_encodes_are_the_payload_or_nothing() {
        let text = b"a bound never changes a payload, it only stops early. ".repeat(40);
        for profile in [V03, FULL] {
            let p = encode_profile(&[], &text, profile).expect("fits");
            assert_eq!(
                encode_below(&[], &text, profile, p.len() + 1),
                Some(p.clone())
            );
            assert_eq!(encode_below(&[], &text, profile, p.len()), None);
            assert_eq!(encode_below(&[], &text, profile, 0), None);
        }
    }

    #[test]
    fn canonical_means_the_encoder_wrote_it() {
        let text = b"one pass checks what a re-encode would. ".repeat(20);
        let p = encode(&text).expect("fits");
        let mut cases = vec![
            p.clone(),
            p[..p.len() - 1].to_vec(),
            [&p[..], &[0]].concat(),
        ];
        for i in 4..p.len() {
            for flip in [1u8, 0x80] {
                let mut q = p.clone();
                q[i] ^= flip;
                cases.push(q);
            }
        }
        for q in cases {
            let n = u64::from(declared_length(&q).expect("header"));
            let reencoded = decode(&q, n).and_then(|out| encode(&out)).as_deref() == Some(&q[..]);
            assert_eq!(decode_canonical(&q, n).is_some(), reencoded);
        }
    }
}
