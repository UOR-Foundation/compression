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

const NC: usize = 17;
const NIN: usize = NC + 7;
/// Profile bit: the short match model (keyed on 4 bytes, found from 4).
pub const F_SHORT: u32 = 1 << 17;
/// Profile bit: the repeat-distance model (its inputs and context 13).
pub const F_REP: u32 = 1 << 18;
/// Profile bit: mixer 1 and the first APM selected by the two match models'
/// states as well as the partial byte (pays on long inputs, costs on short).
pub const F_SEL: u32 = 1 << 19;
/// The v0.3 model: every context, no short match or repeat model.
pub const V03: u32 = (1 << 17) - 1;
/// Every context and every match feature.
pub const FULL: u32 = (1 << 20) - 1;
/// A mixer's row stride: its `NIN` inputs padded with zero inputs to three
/// 8-lane vectors, so the vector kernels need no tail.
const NINP: usize = 24;
const TB_MAX: u32 = 24;
const SQT: [i32; 33] = [
    1, 2, 3, 6, 10, 16, 27, 45, 73, 120, 194, 310, 488, 747, 1101, 1546, 2047, 2549, 2994, 3348,
    3607, 3785, 3901, 3975, 4022, 4050, 4068, 4079, 4085, 4089, 4092, 4093, 4094,
];
const WEIGHT_LIMIT: i32 = 1 << 22;
/// Adaptive probability map rows: 33 interpolation points over the stretch domain.
const APM_ROW: usize = 33;
const APM_RATE: i32 = 6;
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
    rate: [i32; 128],
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
            *r = 131_072 / (2 * c as i32 + 3);
        }
        Tables { stretch, rate }
    })
}

/// An adaptive probability map: a probability, refined by interpolating
/// between learned points of its row for the context.
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
    if *n < 127 {
        *n += 1;
    }
}

struct Model {
    t: &'static Tables,
    mask: u32,
    mb: u32,
    sp: Vec<u16>,
    sn: Vec<u8>,
    hist: Vec<u8>,
    /// History bytes ahead of the coded ones in `hist` (a window frame's
    /// preceding outputs): the match model reaches into them.
    hoff: u32,
    pos: u32,
    c4: u32,
    c8: u32,
    wordh: u32,
    word1: u32,
    c0: u32,
    bit: u32,
    mtab: Vec<u32>,
    profile: u32,
    m2tab: Vec<u32>,
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
    a1: Apm,
    a2: Apm,
}

impl Model {
    /// The model for `n` bytes coded after `history` (a window frame's
    /// preceding outputs): the match model starts holding every history
    /// position, and every other model starts fresh.
    fn new(n: u32, history: &[u8], profile: u32) -> Model {
        let (mask, mb) = Model::sized(n);
        let mut msp = [0u16; 64];
        for (i, p) in msp.iter_mut().enumerate() {
            *p = if i % 2 == 1 { 60000 } else { 5536 };
        }
        let mut m = Model {
            t: tables(),
            mask,
            mb,
            sp: vec![32768; mask as usize + 1],
            sn: vec![0; mask as usize + 1],
            hist: Vec::with_capacity(history.len() + n as usize),
            hoff: history.len() as u32,
            pos: 0,
            c4: 0,
            c8: 0,
            wordh: 0,
            word1: 0,
            c0: 1,
            bit: 0,
            mtab: vec![0; 1 << mb],
            profile,
            m2tab: if profile & F_SHORT != 0 {
                vec![0; 1 << mb]
            } else {
                Vec::new()
            },
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
            bb: [0; NC],
            ix: [0; NC],
            st: [0; NINP],
            mi: 0,
            w1: weights(256 * 9),
            w2: weights(2048),
            w3: weights(2048),
            s1: 0,
            s2: 0,
            s3: 0,
            pm1: 0,
            pm2: 0,
            pm3: 0,
            a1: Apm::new(256 * 9),
            a2: Apm::new(1 << APM2_BITS),
        };
        m.hist.extend_from_slice(history);
        for p in 7..m.hoff {
            let k = m.match_key(p);
            m.mtab[k] = p;
        }
        if profile & F_SHORT != 0 {
            for p in 4..m.hoff {
                let k = m.short_key(p);
                m.m2tab[k] = p;
            }
        }
        m.new_byte();
        m
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
        u32::from(self.hist[(self.hoff + i) as usize])
    }

    /// The byte at absolute position `i` (history first).
    #[inline]
    fn abs(&self, i: u32) -> u32 {
        u32::from(self.hist[i as usize])
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
                let cand = self.mtab[k];
                if cand > 0 {
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
            self.mtab[k] = pa;
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
                    let cand = self.m2tab[k];
                    if cand > 0 {
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
                self.m2tab[k] = pa;
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
        self.hb = context_hashes(&Ctx {
            pos,
            c4: self.c4,
            c8: self.c8,
            wordh: self.wordh,
            word1: self.word1,
            r: self.r,
            at: |i| self.at(i),
            mexp: if self.mlen > 0 { self.mexp + 0x100 } else { 0 },
            mbucket: self.mbucket,
            c13: self.c13(self.c4),
        });
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
    fn prefetch_next(&self, p: u32) {
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
            let hb = context_hashes(&Ctx {
                pos: pos + 1,
                c4: (self.c4 << 8) | b,
                c8: (self.c8 << 8) | (self.c4 >> 24),
                wordh,
                word1,
                r: self.r,
                at,
                mexp,
                mbucket,
                c13: if self.profile & F_REP == 0 {
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
            });
            self.prefetch_buckets(&hb, 0);
            if pa + 1 >= 7 {
                prefetch(&self.mtab[self.match_key_by(pa + 1, abs)]);
            }
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
            prefetch(&w[s * NINP]);
            prefetch(&w[s * NINP + NINP - 1]);
        }
        let cx2 =
            h((c4 & 0xff_ffff).wrapping_mul(0x2F0B_4C27).wrapping_add(c0)) >> (32 - APM2_BITS);
        let row = cx2 as usize * APM_ROW;
        prefetch(&self.a2.t[row]);
        prefetch(&self.a2.t[row + APM_ROW - 1]);
    }

    #[inline(always)]
    fn prefetch_buckets(&self, hb: &[u32; NC], nc: u32) {
        for &hb in hb {
            let hh = h(hb.wrapping_add(nc.wrapping_mul(0x2F0B_4C27)));
            // a bucket and its neighbour share one 32-slot line pair
            let b = ((hh & self.mask) & !31) as usize;
            prefetch(&self.sp[b]);
            prefetch(&self.sn[b]);
        }
    }

    /// The bucket for a context hash: the one of a neighbouring pair that
    /// carries its check byte (in the unused slot 0), else the less used one
    /// (by slot 1's count), cleared and tagged.
    #[inline]
    fn bucket(&mut self, hh: u32) -> u32 {
        let ck = ((hh >> 24) | 1) as u8;
        let b0 = (hh & self.mask) & !15;
        let alt = b0 ^ 16;
        if self.sn[b0 as usize] == ck {
            return b0;
        }
        if self.sn[alt as usize] == ck {
            return alt;
        }
        let b = if self.sn[b0 as usize + 1] > self.sn[alt as usize + 1] {
            alt
        } else {
            b0
        };
        let b = b as usize;
        self.sp[b..b + 16].fill(32768);
        self.sn[b..b + 16].fill(0);
        self.sn[b] = ck;
        b as u32
    }

    #[inline(always)]
    fn predict<K: Kernel>(&mut self) -> u32 {
        let t = self.t;
        if self.bit == 0 || self.bit == 4 {
            let nc = if self.bit == 0 {
                0
            } else {
                (self.c0 & 15) | 16
            };
            for k in 0..NC {
                let hh = h(self.hb[k].wrapping_add(nc.wrapping_mul(0x2F0B_4C27)));
                self.bb[k] = self.bucket(hh);
            }
        }
        let part = if self.bit < 4 {
            self.c0
        } else {
            (self.c0 & ((1 << (self.bit - 4)) - 1)) | (1 << (self.bit - 4))
        };
        for k in 0..NC {
            let i = self.bb[k] + part;
            self.ix[k] = i;
            self.st[k] = i32::from(t.stretch[usize::from(self.sp[i as usize] >> 4)]);
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
        self.st[NC] = if self.mlen > 0 {
            i32::from(t.stretch[usize::from(self.msp[self.mi] >> 4)])
        } else {
            0
        };
        let vote = if self.mbucket > 15 { 1024 } else { 256 };
        self.st[NC + 1] = if self.mlen == 0 {
            0
        } else if eb == 1 {
            vote
        } else {
            -vote
        };
        self.st[NC + 2] = 256;
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
        self.m2i = (self.m2b * 2 + eb2) as usize;
        self.st[NC + 3] = if self.m2len > 0 {
            i32::from(t.stretch[usize::from(self.m2sp[self.m2i] >> 4)])
        } else {
            0
        };
        let vote2 = if self.m2b > 15 { 1024 } else { 256 };
        self.st[NC + 4] = if self.m2len == 0 {
            0
        } else if eb2 == 1 {
            vote2
        } else {
            -vote2
        };
        self.ri = (self.rrun * 2 + ebr) as usize;
        self.st[NC + 5] = if self.ra {
            i32::from(t.stretch[usize::from(self.rsp[self.ri] >> 4)])
        } else {
            0
        };
        self.st[NC + 6] = if !self.ra {
            0
        } else if ebr == 1 {
            256
        } else {
            -256
        };
        // mixer 1 and the first APM: the partial byte and the two match states
        let msel = if self.profile & F_SEL == 0 {
            0
        } else {
            (if self.m2len > 0 { 1 + eb2 } else { 0 }) + 3 * (if self.ra { 1 + ebr } else { 0 })
        };
        let c1 = (self.c0 + 256 * msel) as usize;
        self.s1 = c1 * NINP;
        self.s2 = ((self.c4 & 255) * 8 + self.bit) as usize * NINP;
        self.s3 = (((self.c4 >> 8) & 255) * 8 + self.bit) as usize * NINP;
        let [a, b, c] = K::dot(
            &self.st,
            row(&self.w1, self.s1),
            row(&self.w2, self.s2),
            row(&self.w3, self.s3),
        );
        let d1 = (a >> 16).clamp(-2047, 2047) as i32;
        let d2 = (b >> 16).clamp(-2047, 2047) as i32;
        let d3 = (c >> 16).clamp(-2047, 2047) as i32;
        self.pm1 = squash(d1);
        self.pm2 = squash(d2);
        self.pm3 = squash(d3);
        let p = squash((((d1 + d2 + d3 + 6144) * 21846) >> 16) - 2048).clamp(1, 4095) as u32;
        let q1 = self.a1.refine(&t.stretch, p, c1);
        let cx2 = h((self.c4 & 0xff_ffff)
            .wrapping_mul(0x2F0B_4C27)
            .wrapping_add(self.c0))
            >> (32 - APM2_BITS);
        let q2 = self.a2.refine(&t.stretch, p, cx2 as usize);
        let pr = ((p as i32 + q1 + 2 * q2) / 4).clamp(1, 4095) as u32;
        self.prefetch_next(pr);
        pr
    }

    #[inline(always)]
    fn update<K: Kernel>(&mut self, y: bool) {
        self.a1.learn(y);
        self.a2.learn(y);
        let rate = &self.t.rate;
        for k in 0..NC {
            let i = self.ix[k] as usize;
            adapt(&mut self.sp[i], &mut self.sn[i], y, rate);
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
        // plasticity: 24 + 104 * 4096 / (4096 + pos), in 1/65536 per unit of x * err
        let lr = 24 + (104 * 4096 / (4096 + u64::from(self.pos))) as i64;
        let yb = i32::from(y) << 12;
        // |err| <= 4096 and lr <= 128, so err * lr < 2^20 and x * err * lr
        // (|x| <= 2047) stays inside i32: the kernels learn in 32-bit lanes.
        let lr = lr as i32;
        let e = [
            (yb - self.pm1) * lr,
            (yb - self.pm2) * lr,
            (yb - self.pm3) * lr,
        ];
        K::train(
            &self.st,
            row_mut(&mut self.w1, self.s1),
            row_mut(&mut self.w2, self.s2),
            row_mut(&mut self.w3, self.s3),
            e,
        );
        self.c0 = (self.c0 << 1) | u32::from(y);
        self.bit += 1;
        if self.bit == 8 {
            let b = self.c0 & 255;
            self.hist.push(b as u8);
            self.pos += 1;
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
}

/// Fresh mixer weights for `rows` contexts: 1/4 on every v0.3 input, 0 on
/// the short match and repeat inputs (they earn their weight) and on padding.
fn weights(rows: usize) -> Vec<i32> {
    let mut w = vec![0; rows * NINP];
    for r in w.chunks_exact_mut(NINP) {
        r[..NC + 3].fill(1 << 14);
    }
    w
}

#[inline(always)]
fn row(w: &[i32], s: usize) -> &[i32; NINP] {
    w[s..s + NINP].try_into().expect("row")
}

#[inline(always)]
fn row_mut(w: &mut [i32], s: usize) -> &mut [i32; NINP] {
    (&mut w[s..s + NINP]).try_into().expect("row")
}

/// The three mixers' arithmetic: the exact dot products of the inputs with
/// three weight rows, and the bounded Hebbian step on those rows. Every kernel
/// computes the same integers; only the instructions differ.
trait Kernel {
    fn dot(x: &[i32; NINP], w1: &[i32; NINP], w2: &[i32; NINP], w3: &[i32; NINP]) -> [i64; 3];
    fn train(
        x: &[i32; NINP],
        w1: &mut [i32; NINP],
        w2: &mut [i32; NINP],
        w3: &mut [i32; NINP],
        e: [i32; 3],
    );
}

/// The portable kernel (wasm, and x86_64 without AVX2).
#[cfg_attr(target_arch = "aarch64", allow(dead_code))]
struct Scalar;

impl Kernel for Scalar {
    #[inline(always)]
    fn dot(x: &[i32; NINP], w1: &[i32; NINP], w2: &[i32; NINP], w3: &[i32; NINP]) -> [i64; 3] {
        let (mut a, mut b, mut c) = (0i64, 0i64, 0i64);
        for i in 0..NIN {
            let x = i64::from(x[i]);
            a += x * i64::from(w1[i]);
            b += x * i64::from(w2[i]);
            c += x * i64::from(w3[i]);
        }
        [a, b, c]
    }

    #[inline(always)]
    fn train(
        x: &[i32; NINP],
        w1: &mut [i32; NINP],
        w2: &mut [i32; NINP],
        w3: &mut [i32; NINP],
        e: [i32; 3],
    ) {
        for i in 0..NIN {
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
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            $f::<Scalar>($($a),*)
        }
    }};
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::{Kernel, NINP, WEIGHT_LIMIT};
    use std::arch::x86_64::*;

    pub(super) struct Avx2;

    /// The encoder loop compiled for AVX2.
    #[target_feature(enable = "avx2")]
    pub(super) fn encoder(history: &[u8], data: &[u8], profile: u32) -> Option<Vec<u8>> {
        super::encoder::<Avx2>(history, data, profile)
    }

    /// The decoder loop compiled for AVX2.
    #[target_feature(enable = "avx2")]
    pub(super) fn decoder(
        history: &[u8],
        payload: &[u8],
        limit: u64,
        canonical: bool,
        profile: u32,
    ) -> Option<Vec<u8>> {
        super::decoder::<Avx2>(history, payload, limit, canonical, profile)
    }

    /// Eight lanes of a row.
    ///
    /// # Safety
    /// AVX2 must be available and `i * 8 + 8 <= NINP`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn lanes(r: &[i32; NINP], i: usize) -> __m256i {
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
    unsafe fn dot(
        x: &[i32; NINP],
        w1: &[i32; NINP],
        w2: &[i32; NINP],
        w3: &[i32; NINP],
    ) -> [i64; 3] {
        let (mut a, mut b, mut c) = (
            _mm256_setzero_si256(),
            _mm256_setzero_si256(),
            _mm256_setzero_si256(),
        );
        for i in 0..NINP / 8 {
            // SAFETY: AVX2 is available and i < NINP / 8.
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
    unsafe fn train(
        x: &[i32; NINP],
        w1: &mut [i32; NINP],
        w2: &mut [i32; NINP],
        w3: &mut [i32; NINP],
        e: [i32; 3],
    ) {
        let (hi, lo) = (
            _mm256_set1_epi32(WEIGHT_LIMIT),
            _mm256_set1_epi32(-WEIGHT_LIMIT),
        );
        for i in 0..NINP / 8 {
            // SAFETY: AVX2 is available and i < NINP / 8.
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
    /// AVX2 must be available and `i * 8 + 8 <= NINP`.
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn step(w: &mut [i32; NINP], i: usize, x: __m256i, e: i32, hi: __m256i, lo: __m256i) {
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
        fn dot(x: &[i32; NINP], w1: &[i32; NINP], w2: &[i32; NINP], w3: &[i32; NINP]) -> [i64; 3] {
            // SAFETY: `Avx2` is only used once AVX2 has been detected.
            unsafe { dot(x, w1, w2, w3) }
        }

        #[inline(always)]
        fn train(
            x: &[i32; NINP],
            w1: &mut [i32; NINP],
            w2: &mut [i32; NINP],
            w3: &mut [i32; NINP],
            e: [i32; 3],
        ) {
            // SAFETY: `Avx2` is only used once AVX2 has been detected.
            unsafe { train(x, w1, w2, w3, e) }
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod arm {
    use super::{Kernel, NINP, WEIGHT_LIMIT};
    use std::arch::aarch64::*;

    pub(super) struct Neon;

    impl Kernel for Neon {
        #[inline(always)]
        fn dot(x: &[i32; NINP], w1: &[i32; NINP], w2: &[i32; NINP], w3: &[i32; NINP]) -> [i64; 3] {
            // SAFETY: NEON is part of aarch64, and every 4-lane load at
            // i * 4 < NINP stays inside its row.
            unsafe {
                let (mut a, mut b, mut c) = (vdupq_n_s64(0), vdupq_n_s64(0), vdupq_n_s64(0));
                for i in 0..NINP / 4 {
                    let xv = vld1q_s32(x.as_ptr().add(i * 4));
                    let xl = vget_low_s32(xv);
                    let acc = |s: int64x2_t, w: &[i32; NINP]| {
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
        fn train(
            x: &[i32; NINP],
            w1: &mut [i32; NINP],
            w2: &mut [i32; NINP],
            w3: &mut [i32; NINP],
            e: [i32; 3],
        ) {
            // SAFETY: NEON is part of aarch64, and every 4-lane access at
            // i * 4 < NINP stays inside its row.
            unsafe {
                let (hi, lo) = (vdupq_n_s32(WEIGHT_LIMIT), vdupq_n_s32(-WEIGHT_LIMIT));
                for i in 0..NINP / 4 {
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

/// [`encode_with`] under the model `profile` ([`V03`] is the v0.3 coder,
/// [`FULL`] every feature).
pub fn encode_profile(history: &[u8], data: &[u8], profile: u32) -> Option<Vec<u8>> {
    dispatch!(encoder(history, data, profile))
}

#[inline(always)]
fn encoder<K: Kernel>(history: &[u8], data: &[u8], profile: u32) -> Option<Vec<u8>> {
    let n = u32::try_from(data.len()).ok()?;
    let cap = 2 * data.len() + 64;
    let mut out = Vec::with_capacity(data.len() / 2 + 16);
    out.extend_from_slice(&n.to_be_bytes());
    let mut m = Model::new(n, history, profile);
    let (mut x1, mut x2) = (0u32, u32::MAX);
    for &byte in data {
        for k in (0..8).rev() {
            let y = (byte >> k) & 1 == 1;
            let p = m.predict::<K>();
            let xm = x1 + ((u64::from(x2 - x1) * u64::from(p)) >> 12) as u32;
            if y {
                x2 = xm;
            } else {
                x1 = xm + 1;
            }
            while (x1 ^ x2) >> 24 == 0 {
                if out.len() - 4 >= cap {
                    return None;
                }
                out.push((x2 >> 24) as u8);
                x1 <<= 8;
                x2 = (x2 << 8) | 255;
            }
            m.update::<K>(y);
        }
    }
    if out.len() - 4 >= cap {
        return None;
    }
    out.push((x1 >> 24) as u8);
    Some(out)
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

fn run_decoder(history: &[u8], payload: &[u8], limit: u64, canonical: bool) -> Option<Vec<u8>> {
    run_profile(history, payload, limit, canonical, V03)
}

/// The decoder under the model `profile`.
pub(crate) fn run_profile(
    history: &[u8],
    payload: &[u8],
    limit: u64,
    canonical: bool,
    profile: u32,
) -> Option<Vec<u8>> {
    dispatch!(decoder(history, payload, limit, canonical, profile))
}

/// Decode a payload [`encode_profile`] wrote under `profile`.
pub fn decode_profile(history: &[u8], payload: &[u8], limit: u64, profile: u32) -> Option<Vec<u8>> {
    run_profile(history, payload, limit, false, profile)
}

#[inline(always)]
fn decoder<K: Kernel>(
    history: &[u8],
    payload: &[u8],
    limit: u64,
    canonical: bool,
    profile: u32,
) -> Option<Vec<u8>> {
    let n = declared_length(payload)?;
    if u64::from(n) > limit {
        return None;
    }
    let cap = 2 * n as usize + 64;
    let mut ip = 4usize;
    let mut x = 0u32;
    for _ in 0..4 {
        x = (x << 8) | u32::from(payload.get(ip).copied().unwrap_or(255));
        ip += 1;
    }
    // coder bytes the encoder has emitted so far
    let mut emitted = 0usize;
    let mut m = Model::new(n, history, profile);
    let (mut x1, mut x2) = (0u32, u32::MAX);
    for _ in 0..n {
        for _ in 0..8 {
            let p = m.predict::<K>();
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
            m.update::<K>(y);
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
    out.drain(..history.len());
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let p = encoder::<Scalar>(&[], data).expect("fits");
            assert_eq!(encode(data).as_deref(), Some(&p[..]));
            let n = data.len() as u64;
            assert_eq!(decode(&p, n).as_deref(), Some(data));
            assert_eq!(decoder::<Scalar>(&[], &p, n, true).as_deref(), Some(data));
            // and after a window history
            let w = encoder::<Scalar>(&text[..999], data).expect("fits");
            assert_eq!(encode_with(&text[..999], data).as_deref(), Some(&w[..]));
            assert_eq!(decode_with(&text[..999], &w, n).as_deref(), Some(data));
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
