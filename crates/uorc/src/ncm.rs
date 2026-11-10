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
const NIN: usize = NC + 3;
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
    pos: u32,
    c4: u32,
    c8: u32,
    wordh: u32,
    word1: u32,
    c0: u32,
    bit: u32,
    mtab: Vec<u32>,
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
    st: [i32; NIN],
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
    fn new(n: u32) -> Model {
        let mut tb = 16;
        while tb < TB_MAX && (1u64 << tb) < u64::from(n) * 1024 {
            tb += 1;
        }
        let mb = if tb > 22 { 22 } else { tb - 2 };
        let mut msp = [0u16; 64];
        for (i, p) in msp.iter_mut().enumerate() {
            *p = if i % 2 == 1 { 60000 } else { 5536 };
        }
        let mut m = Model {
            t: tables(),
            mask: (1u32 << tb) - 1,
            mb,
            sp: vec![32768; 1 << tb],
            sn: vec![0; 1 << tb],
            hist: Vec::with_capacity(n as usize),
            pos: 0,
            c4: 0,
            c8: 0,
            wordh: 0,
            word1: 0,
            c0: 1,
            bit: 0,
            mtab: vec![0; 1 << mb],
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
            st: [0; NIN],
            mi: 0,
            w1: vec![1 << 14; 256 * NIN],
            w2: vec![1 << 14; 2048 * NIN],
            w3: vec![1 << 14; 2048 * NIN],
            s1: 0,
            s2: 0,
            s3: 0,
            pm1: 0,
            pm2: 0,
            pm3: 0,
            a1: Apm::new(256),
            a2: Apm::new(1 << APM2_BITS),
        };
        m.new_byte();
        m
    }

    #[inline]
    fn at(&self, i: u32) -> u32 {
        u32::from(self.hist[i as usize])
    }

    fn new_byte(&mut self) {
        let pos = self.pos;
        if self.mlen > 0 {
            if self.mlen < 65535 {
                self.mlen += 1;
            }
            self.mptr += 1;
        }
        if pos >= 7 {
            let mut k = 0u32;
            for i in 1..=7 {
                k = k
                    .wrapping_mul(0x2F0B_4C27)
                    .wrapping_add(self.at(pos - i))
                    .wrapping_add(1);
            }
            let k = (h(k) >> (32 - self.mb)) as usize;
            if self.mlen == 0 {
                let cand = self.mtab[k];
                if cand > 0 {
                    let mut l = 0;
                    while l < 1024 && l < cand && self.at(cand - 1 - l) == self.at(pos - 1 - l) {
                        l += 1;
                    }
                    if l >= 7 {
                        self.mptr = cand;
                        self.mlen = l;
                    }
                }
            }
            self.mtab[k] = pos;
        }
        if self.mlen > 0 {
            let l = self.mlen;
            self.mexp = self.at(self.mptr);
            self.mbucket = if l < 16 {
                l
            } else if l < 32 {
                16 + ((l - 16) >> 2)
            } else if l < 64 {
                20 + ((l - 32) >> 3)
            } else if l < 512 {
                24 + ((l - 64) >> 6)
            } else {
                31
            };
        } else {
            self.mexp = 0;
            self.mbucket = 0;
        }
        let (c4, c8, r) = (self.c4, self.c8, self.r);
        let b1 = c4 & 255;
        let p3 = pos & 3;
        let ab = if r > 0 && pos >= r {
            self.at(pos - r)
        } else {
            0
        };
        let ab1 = if r > 0 && pos + 1 >= r {
            self.at(pos + 1 - r)
        } else {
            0
        };
        let ab2 = if r > 0 && pos >= 2 * r {
            self.at(pos - 2 * r)
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
            self.wordh.wrapping_add(h(self.word1)),
            self.wordh,
            (c4 & 0xff00) + (p3 << 16),
            (c4 & 0xffff_0000) + p3,
            b1 + (p3 << 8) + 0x80_0000,
            (c4 & 0xffff) + (p3 << 16) + 0x400_0000,
            c4 & 0x00ff_00ff,
            ab.wrapping_add(b1 << 8)
                .wrapping_add(r << 16)
                .wrapping_add(0x1000_0000),
            ab.wrapping_add(ab1 << 8)
                .wrapping_add(ab2 << 16)
                .wrapping_add(r << 24)
                .wrapping_add(0x8000_0000),
            (if self.mlen > 0 { self.mexp + 0x100 } else { 0 })
                .wrapping_add(self.mbucket << 9)
                .wrapping_add(0x7700_0000),
        ];
        for (k, (hb, v)) in self.hb.iter_mut().zip(v).enumerate() {
            *hb = h(v.wrapping_add((k as u32).wrapping_mul(0x0100_0193)));
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

    fn predict(&mut self) -> u32 {
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
        self.s1 = self.c0 as usize * NIN;
        self.s2 = ((self.c4 & 255) * 8 + self.bit) as usize * NIN;
        self.s3 = (((self.c4 >> 8) & 255) * 8 + self.bit) as usize * NIN;
        let w1 = &self.w1[self.s1..self.s1 + NIN];
        let w2 = &self.w2[self.s2..self.s2 + NIN];
        let w3 = &self.w3[self.s3..self.s3 + NIN];
        let (mut a, mut b, mut c) = (0i64, 0i64, 0i64);
        for i in 0..NIN {
            let x = i64::from(self.st[i]);
            a += x * i64::from(w1[i]);
            b += x * i64::from(w2[i]);
            c += x * i64::from(w3[i]);
        }
        let d1 = (a >> 16).clamp(-2047, 2047) as i32;
        let d2 = (b >> 16).clamp(-2047, 2047) as i32;
        let d3 = (c >> 16).clamp(-2047, 2047) as i32;
        self.pm1 = squash(d1);
        self.pm2 = squash(d2);
        self.pm3 = squash(d3);
        let p = squash((((d1 + d2 + d3 + 6144) * 21846) >> 16) - 2048).clamp(1, 4095) as u32;
        let q1 = self.a1.refine(&t.stretch, p, self.c0 as usize);
        let cx2 = h((self.c4 & 0xff_ffff)
            .wrapping_mul(0x2F0B_4C27)
            .wrapping_add(self.c0))
            >> (32 - APM2_BITS);
        let q2 = self.a2.refine(&t.stretch, p, cx2 as usize);
        ((p as i32 + q1 + 2 * q2) / 4).clamp(1, 4095) as u32
    }

    fn update(&mut self, y: bool) {
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
        // plasticity: 24 + 104 * 4096 / (4096 + pos), in 1/65536 per unit of x * err
        let lr = 24 + (104 * 4096 / (4096 + u64::from(self.pos))) as i64;
        let yb = i32::from(y) << 12;
        let e1 = i64::from(yb - self.pm1) * lr;
        let e2 = i64::from(yb - self.pm2) * lr;
        let e3 = i64::from(yb - self.pm3) * lr;
        let w1 = &mut self.w1[self.s1..self.s1 + NIN];
        let w2 = &mut self.w2[self.s2..self.s2 + NIN];
        let w3 = &mut self.w3[self.s3..self.s3 + NIN];
        for i in 0..NIN {
            let x = i64::from(self.st[i]);
            let learn = |w: &mut i32, e: i64| {
                *w = (*w + ((x * e) >> 16) as i32).clamp(-WEIGHT_LIMIT, WEIGHT_LIMIT);
            };
            learn(&mut w3[i], e3);
            learn(&mut w1[i], e1);
            learn(&mut w2[i], e2);
        }
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

/// Encode `data` as an ncm payload, or `None` when the coder output would pass
/// `2 * len + 64` bytes (the model abandons the encoding there) or the input
/// does not fit the 4-byte length.
pub fn encode(data: &[u8]) -> Option<Vec<u8>> {
    let n = u32::try_from(data.len()).ok()?;
    let cap = 2 * data.len() + 64;
    let mut out = Vec::with_capacity(data.len() / 2 + 16);
    out.extend_from_slice(&n.to_be_bytes());
    let mut m = Model::new(n);
    let (mut x1, mut x2) = (0u32, u32::MAX);
    for &byte in data {
        for k in (0..8).rev() {
            let y = (byte >> k) & 1 == 1;
            let p = m.predict();
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
            m.update(y);
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
    run_decoder(payload, limit, false)
}

/// Decode a payload of exactly `n` bytes that [`encode`] writes for its own
/// output, as `Archive.linkedNcm` requires (`encode(decode(p)) == p`), in one
/// pass: the encoder and the decoder share their coder state, so the bytes the
/// encoder would emit are checked against the payload as the decoder shifts.
pub fn decode_canonical(payload: &[u8], n: u64) -> Option<Vec<u8>> {
    if declared_length(payload).map(u64::from) != Some(n) {
        return None;
    }
    run_decoder(payload, n, true)
}

fn run_decoder(payload: &[u8], limit: u64, canonical: bool) -> Option<Vec<u8>> {
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
    let mut m = Model::new(n);
    let (mut x1, mut x2) = (0u32, u32::MAX);
    for _ in 0..n {
        for _ in 0..8 {
            let p = m.predict();
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
            m.update(y);
        }
    }
    if canonical
        && (emitted >= cap
            || payload.len() != 4 + emitted + 1
            || payload[4 + emitted] != (x1 >> 24) as u8)
    {
        return None;
    }
    Some(m.hist)
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
