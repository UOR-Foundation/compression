//! Native ulz codec: the LZ + adaptive range coder of `Uorc.Codec.LzEncode`
//! and `Uorc.Codec.LzDecode` (client/src/Uorc/Codec), ported operation for
//! operation so that [`encode`] writes exactly the payload the LexLean encoder
//! writes, and [`decode`] reconstructs exactly what the LexLean decoder (and the
//! ulz kernel graph) reconstructs.
//!
//! The decoder is deliberately partial. It accepts only payloads on which every
//! reading of the bitstream agrees: the range coder keeps `code < range`, every
//! match source lies inside the output, no match runs past the declared length,
//! and the coder consumes the payload exactly. Any other payload is declined
//! with `None`, and the caller hands the archive to the exported reference
//! decoder, which applies the model's semantics to it. Every payload [`encode`]
//! writes is accepted (checked in the tests).

/// Probability model size (`Table::fill 2 13 1024`).
const PROBS: usize = 1 << 13;
const TOP: u32 = 1 << 24;
type Probs = [u16; PROBS];
const IS_REP: usize = 12;
const IS_REP_G0: usize = 24;
const IS_REP_G1: usize = 36;
const IS_REP_G2: usize = 48;
const IS_REP0_LONG: usize = 60;
const LITERAL: usize = 72;
const SLOT: usize = 6216;
const SPEC: usize = 6472;
const ALIGN: usize = 6587;
const LEN: usize = 6603;
const REP_LEN: usize = 7149;
const MAX_MATCH: usize = 273;
const NICE: usize = 64;
const DEPTH: usize = 48;

fn after_literal(s: usize) -> usize {
    if s < 4 {
        0
    } else if s < 10 {
        s - 3
    } else {
        s - 6
    }
}

fn after_match(s: usize) -> usize {
    if s < 7 {
        7
    } else {
        10
    }
}

fn after_rep(s: usize) -> usize {
    if s < 7 {
        8
    } else {
        11
    }
}

fn after_short(s: usize) -> usize {
    if s < 7 {
        9
    } else {
        11
    }
}

fn literal_base(prev: u8) -> usize {
    LITERAL + 768 * usize::from(prev >> 5)
}

// ---------------------------------------------------------------- encoder

struct RangeEncoder {
    low: u64,
    range: u32,
    cache: u8,
    pending: u64,
    out: Vec<u8>,
}

impl RangeEncoder {
    fn new(capacity: usize) -> Self {
        RangeEncoder {
            low: 0,
            range: u32::MAX,
            cache: 0,
            pending: 1,
            out: Vec::with_capacity(capacity),
        }
    }

    fn shift_low(&mut self) {
        if self.low < 0xFF00_0000 || self.low >= 1 << 32 {
            let carry = (self.low >> 32) as u8;
            self.out.push(self.cache.wrapping_add(carry));
            for _ in 1..self.pending {
                self.out.push(0xFFu8.wrapping_add(carry));
            }
            self.cache = (self.low >> 24) as u8;
            self.pending = 1;
        } else {
            self.pending += 1;
        }
        self.low = (self.low & 0x00FF_FFFF) << 8;
    }

    #[inline(always)]
    fn normalize(&mut self) {
        if self.range < TOP {
            self.range <<= 8;
            self.shift_low();
        }
    }

    #[inline(always)]
    fn bit(&mut self, probs: &mut [u16], at: usize, bit: u32) {
        let v = u32::from(probs[at]);
        let bound = (self.range >> 11) * v;
        if bit == 0 {
            self.range = bound;
            probs[at] = (v + ((2048 - v) >> 5)) as u16;
        } else {
            self.low += u64::from(bound);
            self.range -= bound;
            probs[at] = (v - (v >> 5)) as u16;
        }
        self.normalize();
    }

    fn direct(&mut self, value: u32, bits: u32) {
        for k in (0..bits).rev() {
            let half = self.range >> 1;
            if (value >> k) & 1 == 1 {
                self.low += u64::from(half);
            }
            self.range = half;
            self.normalize();
        }
    }

    fn tree(&mut self, probs: &mut [u16], base: usize, bits: u32, value: u32) {
        let mut m = 1usize;
        for k in (0..bits).rev() {
            let b = (value >> k) & 1;
            self.bit(probs, base + m, b);
            m = (m << 1) | b as usize;
        }
    }

    fn reverse(&mut self, probs: &mut [u16], base: usize, bits: u32, mut value: u32) {
        let mut m = 1usize;
        for _ in 0..bits {
            let b = value & 1;
            value >>= 1;
            self.bit(probs, base + m, b);
            m = (m << 1) | b as usize;
        }
    }

    fn length(&mut self, probs: &mut [u16], base: usize, v: u32) {
        if v < 8 {
            self.bit(probs, base, 0);
            self.tree(probs, base + 2, 3, v);
        } else if v < 16 {
            self.bit(probs, base, 1);
            self.bit(probs, base + 1, 0);
            self.tree(probs, base + 18, 3, v - 8);
        } else {
            self.bit(probs, base, 1);
            self.bit(probs, base + 1, 1);
            self.tree(probs, base + 34, 8, v - 16);
        }
    }

    fn distance(&mut self, probs: &mut [u16], dist: u32, len_code: u32) {
        let ls = len_code.min(3) as usize;
        let slot = if dist < 4 {
            dist
        } else {
            let top = 31 - dist.leading_zeros();
            2 * top + ((dist >> (top - 1)) & 1)
        };
        self.tree(probs, SLOT + 64 * ls, 6, slot);
        if slot >= 4 {
            let fb = (slot >> 1) - 1;
            let base = (2 + (slot & 1)) << fb;
            let reduced = dist - base;
            if slot < 14 {
                self.reverse(probs, SPEC + base as usize - slot as usize, fb, reduced);
            } else {
                self.direct(reduced >> 4, fb - 4);
                self.reverse(probs, ALIGN, 4, reduced & 15);
            }
        }
    }

    fn literal(&mut self, probs: &mut [u16], base: usize, value: u8, match_byte: Option<u8>) {
        let value = u32::from(value);
        let Some(mb) = match_byte else {
            self.tree(probs, base, 8, value);
            return;
        };
        let mb = u32::from(mb);
        let mut sym = 1usize;
        for k in (0..8).rev() {
            let mbit = (mb >> k) & 1;
            let b = (value >> k) & 1;
            self.bit(probs, base + 256 * (1 + mbit as usize) + sym, b);
            sym = (sym << 1) | b as usize;
            if b != mbit {
                for j in (0..k).rev() {
                    let b = (value >> j) & 1;
                    self.bit(probs, base + sym, b);
                    sym = (sym << 1) | b as usize;
                }
                return;
            }
        }
    }

    fn finish(mut self) -> Vec<u8> {
        for _ in 0..5 {
            self.shift_low();
        }
        self.out
    }
}

const HASH3_BITS: u32 = 16;

/// Bits of the 4-byte hash table for `n` bytes of encoder input: the bit
/// length of `n + 1` clamped to [16, 22] (`LzEncode.start`).
fn hash4_bits(n: usize) -> u32 {
    let mut d = 0u32;
    while (1u64 << d) < n as u64 + 1 {
        d += 1;
    }
    d.clamp(16, 22)
}

struct Finder<'a> {
    data: &'a [u8],
    /// Most recent position (+1) of each 3-byte hash.
    head3: Vec<u32>,
    /// Most recent position (+1) of each 4-byte hash, chained through `chain`.
    head: Vec<u32>,
    bits: u32,
    chain: Vec<u32>,
}

impl Finder<'_> {
    #[inline(always)]
    fn at(&self, i: usize) -> u8 {
        self.data.get(i).copied().unwrap_or(0)
    }

    #[inline(always)]
    fn hash3(&self, i: usize) -> usize {
        let x = (u64::from(self.data[i]) << 16)
            | (u64::from(self.data[i + 1]) << 8)
            | u64::from(self.data[i + 2]);
        ((x * 2_654_435_761) >> 16) as usize & ((1 << HASH3_BITS) - 1)
    }

    #[inline(always)]
    fn hash4(&self, i: usize) -> usize {
        let x = u64::from(u32::from_le_bytes(self.data[i..i + 4].try_into().unwrap_or([0; 4])));
        ((x * 2_654_435_761) >> (32 - self.bits)) as usize & ((1 << self.bits) - 1)
    }

    #[inline(always)]
    fn insert(&mut self, i: usize) {
        if i + 3 < self.data.len() {
            let k = self.hash4(i);
            self.chain[i] = self.head[k];
            self.head[k] = i as u32 + 1;
        }
        if i + 2 < self.data.len() {
            let k = self.hash3(i);
            self.head3[k] = i as u32 + 1;
        }
    }

    /// Length of the common run of `a..` and `b..` (a < b), at most `limit`.
    #[inline(always)]
    fn common(&self, a: usize, b: usize, limit: usize) -> usize {
        let x = &self.data[a..a + limit];
        let y = &self.data[b..b + limit];
        let mut l = 0;
        while l + 8 <= limit {
            let diff = u64::from_le_bytes(x[l..l + 8].try_into().unwrap_or([0; 8]))
                ^ u64::from_le_bytes(y[l..l + 8].try_into().unwrap_or([0; 8]));
            if diff != 0 {
                return l + (diff.trailing_zeros() / 8) as usize;
            }
            l += 8;
        }
        while l < limit && x[l] == y[l] {
            l += 1;
        }
        l
    }

    fn find(&self, i: usize) -> (usize, usize) {
        let n = self.data.len();
        if n <= i + 2 {
            return (0, 0);
        }
        let limit = (n - i).min(MAX_MATCH);
        let (mut best, mut dist) = (0usize, 0usize);
        let c3 = self.head3[self.hash3(i)];
        if c3 != 0 {
            let j = c3 as usize - 1;
            let l = self.common(j, i, limit);
            if l >= 2 {
                best = l;
                dist = i - (j + 1);
            }
        }
        if n <= i + 3 || best >= NICE {
            return (best, dist);
        }
        let mut candidate = self.head[self.hash4(i)];
        for _ in 0..DEPTH {
            if candidate == 0 {
                break;
            }
            let j = candidate as usize - 1;
            let next = self.chain[j];
            if self.at(j + best) == self.at(i + best) {
                let l = self.common(j, i, limit);
                if best < l {
                    best = l;
                    dist = i - (j + 1);
                }
                if l >= NICE {
                    return (best, dist);
                }
            }
            candidate = next;
        }
        (best, dist)
    }
}

/// The ulz payload of `data`, byte for byte `Uorc.Codec.LzEncode::encode data`.
pub fn encode(data: &[u8]) -> Vec<u8> {
    encode_with(&[], data)
}

/// The ulz payload of `input` against a `history` window it may copy from:
/// the encoder runs over `history ++ input` with the match finder already
/// holding every history position, and emits only `input`. With an empty
/// history this is [`encode`].
pub fn encode_with(history: &[u8], input: &[u8]) -> Vec<u8> {
    let joined;
    let data: &[u8] = if history.is_empty() {
        input
    } else {
        joined = [history, input].concat();
        &joined
    };
    let n = data.len();
    let start = history.len();
    let mut rc = RangeEncoder::new(input.len() / 2 + 64);
    let mut probs = vec![1024u16; PROBS];
    let mut f = Finder {
        data,
        head3: vec![0; 1 << HASH3_BITS],
        head: vec![0; 1 << hash4_bits(n)],
        bits: hash4_bits(n),
        chain: vec![0; n],
    };
    for j in 0..start {
        f.insert(j);
    }
    let (mut state, mut reps, mut i) = (0usize, [0usize; 4], start);
    while i < n {
        let limit = (n - i).min(MAX_MATCH);
        let (mut rl, mut ri) = (0usize, 0usize);
        for (k, &r) in reps.iter().enumerate() {
            let l = if i < r + 1 {
                0
            } else {
                f.common(i - (r + 1), i, limit)
            };
            if rl < l {
                rl = l;
                ri = k;
            }
        }
        let (fl, fd) = f.find(i);
        let ml1 = if fl == 2 && fd >= 128 { 0 } else { fl };
        f.insert(i);
        let ml = if ml1 >= 2 && rl + 1 < ml1 && i + 1 < n {
            let (gl, gd) = f.find(i + 1);
            if ml1 + 1 < gl || (gl == ml1 + 1 && gd < 4 * fd) {
                0
            } else {
                ml1
            }
        } else {
            ml1
        };
        let s = state;
        if rl >= 2 && ml <= rl + 1 {
            rc.bit(&mut probs, s, 1);
            rc.bit(&mut probs, IS_REP + s, 1);
            if ri == 0 {
                rc.bit(&mut probs, IS_REP_G0 + s, 0);
                rc.bit(&mut probs, IS_REP0_LONG + s, 1);
            } else {
                rc.bit(&mut probs, IS_REP_G0 + s, 1);
                if ri == 1 {
                    rc.bit(&mut probs, IS_REP_G1 + s, 0);
                } else {
                    rc.bit(&mut probs, IS_REP_G1 + s, 1);
                    rc.bit(&mut probs, IS_REP_G2 + s, ri as u32 - 2);
                }
            }
            rc.length(&mut probs, REP_LEN, rl as u32 - 2);
            let d = reps[ri];
            reps.copy_within(0..ri, 1);
            reps[0] = d;
            for k in 1..rl {
                f.insert(i + k);
            }
            state = after_rep(s);
            i += rl;
        } else if ml >= 2 {
            rc.bit(&mut probs, s, 1);
            rc.bit(&mut probs, IS_REP + s, 0);
            rc.length(&mut probs, LEN, ml as u32 - 2);
            rc.distance(&mut probs, fd as u32, ml as u32 - 2);
            for k in 1..ml {
                f.insert(i + k);
            }
            reps = [fd, reps[0], reps[1], reps[2]];
            state = after_match(s);
            i += ml;
        } else {
            rc.bit(&mut probs, s, 0);
            let prev = if i == 0 { 0 } else { data[i - 1] };
            let match_byte = (s >= 7).then(|| {
                if i < reps[0] + 1 {
                    0
                } else {
                    data[i - (reps[0] + 1)]
                }
            });
            rc.literal(&mut probs, literal_base(prev), data[i], match_byte);
            state = after_literal(s);
            i += 1;
        }
    }
    let body = rc.finish();
    let mut payload = Vec::with_capacity(8 + body.len());
    payload.extend_from_slice(&(input.len() as u64).to_le_bytes());
    payload.extend_from_slice(&body);
    payload
}

// ---------------------------------------------------------------- decoder

struct RangeDecoder<'a> {
    input: &'a [u8],
    pos: usize,
    range: u32,
    code: u32,
}

impl RangeDecoder<'_> {
    #[inline(always)]
    fn next(&mut self) -> u32 {
        let b = self.input.get(self.pos).copied().unwrap_or(0);
        self.pos += 1;
        u32::from(b)
    }

    #[inline(always)]
    fn normalize(&mut self) {
        if self.range < TOP {
            self.range <<= 8;
            self.code = (self.code << 8) | self.next();
        }
    }

    #[inline(always)]
    fn bit(&mut self, probs: &mut Probs, at: usize) -> u32 {
        let p = &mut probs[at & (PROBS - 1)];
        let v = u32::from(*p);
        let bound = (self.range >> 11) * v;
        let b = if self.code < bound {
            self.range = bound;
            *p = (v + ((2048 - v) >> 5)) as u16;
            0
        } else {
            self.code -= bound;
            self.range -= bound;
            *p = (v - (v >> 5)) as u16;
            1
        };
        self.normalize();
        b
    }

    /// Direct bits; `None` when the stream leaves `code < range`.
    fn direct(&mut self, bits: u32) -> Option<u32> {
        let mut acc = 0u32;
        for _ in 0..bits {
            let half = self.range >> 1;
            self.range = half;
            if self.code < half {
                acc <<= 1;
            } else {
                self.code -= half;
                if self.code >= half {
                    return None;
                }
                acc = (acc << 1) | 1;
            }
            self.normalize();
        }
        Some(acc)
    }

    #[inline(always)]
    fn tree(&mut self, probs: &mut Probs, base: usize, bits: u32) -> u32 {
        let mut m = 1usize;
        for _ in 0..bits {
            m = (m << 1) | self.bit(probs, base + m) as usize;
        }
        (m - (1 << bits)) as u32
    }

    fn reverse(&mut self, probs: &mut Probs, base: usize, bits: u32) -> u32 {
        let (mut m, mut acc) = (1usize, 0u32);
        for k in 0..bits {
            let b = self.bit(probs, base + m);
            m = (m << 1) | b as usize;
            acc |= b << k;
        }
        acc
    }

    fn length(&mut self, probs: &mut Probs, base: usize) -> u32 {
        if self.bit(probs, base) == 0 {
            self.tree(probs, base + 2, 3)
        } else if self.bit(probs, base + 1) == 0 {
            8 + self.tree(probs, base + 18, 3)
        } else {
            16 + self.tree(probs, base + 34, 8)
        }
    }

    fn distance(&mut self, probs: &mut Probs, len_code: u32) -> Option<u64> {
        let ls = len_code.min(3) as usize;
        let slot = self.tree(probs, SLOT + 64 * ls, 6);
        if slot < 4 {
            return Some(u64::from(slot));
        }
        let fb = (slot >> 1) - 1;
        let base = u64::from(2 + (slot & 1)) << fb;
        if slot < 14 {
            let r = self.reverse(probs, SPEC + base as usize - slot as usize, fb);
            Some(base + u64::from(r))
        } else {
            let hi = self.direct(fb - 4)?;
            let lo = self.reverse(probs, ALIGN, 4);
            Some(base + 16 * u64::from(hi) + u64::from(lo))
        }
    }
}

/// Reconstruct the input of a ulz payload whose header declares `expected`
/// bytes. `None` declines the payload (see the module documentation); it never
/// means the payload is invalid under the model, only that the native decoder
/// does not vouch for it.
pub fn decode(payload: &[u8], expected: u64) -> Option<Vec<u8>> {
    decode_with(&[], payload, expected)
}

/// [`decode`] against the `history` window the payload was encoded with
/// ([`encode_with`]): the output starts as `history`, matches may reach into
/// it, and only the bytes after it are returned.
pub fn decode_with(history: &[u8], payload: &[u8], expected: u64) -> Option<Vec<u8>> {
    if payload.len() < 13 {
        return None;
    }
    let total = u64::from_le_bytes(payload[..8].try_into().ok()?);
    if total != expected {
        return None;
    }
    let h = history.len();
    let total = usize::try_from(total).ok()?.checked_add(h)?;
    let code = u32::from_be_bytes(payload[9..13].try_into().ok()?);
    if code == u32::MAX {
        return None;
    }
    let mut rc = RangeDecoder {
        input: payload,
        pos: 13,
        range: u32::MAX,
        code,
    };
    let mut probs: Box<Probs> = Box::new([1024u16; PROBS]);
    // A hostile header cannot make this allocate more than the payload could
    // possibly describe up front; the vector grows as output is produced.
    let mut out: Vec<u8> = Vec::with_capacity(total.min(h + payload.len().saturating_mul(64)));
    out.extend_from_slice(history);
    let (mut state, mut reps) = (0usize, [0u64; 4]);
    while out.len() < total {
        let s = state;
        let size = out.len();
        if rc.bit(&mut probs, s) == 0 {
            let prev = out.last().copied().unwrap_or(0);
            let base = literal_base(prev);
            let sym = if s < 7 {
                rc.tree(&mut probs, base, 8)
            } else {
                let rep0 = reps[0];
                let mb = if rep0 < size as u64 {
                    u32::from(out[size - rep0 as usize - 1])
                } else {
                    0
                };
                let mut sym = 1usize;
                let mut k = 8u32;
                while k > 0 {
                    k -= 1;
                    let mbit = (mb >> k) & 1;
                    let b = rc.bit(&mut probs, base + 256 * (1 + mbit as usize) + sym);
                    sym = (sym << 1) | b as usize;
                    if b != mbit {
                        break;
                    }
                }
                while sym < 0x100 {
                    sym = (sym << 1) | rc.bit(&mut probs, base + sym) as usize;
                }
                (sym - 0x100) as u32
            };
            out.push(sym as u8);
            state = after_literal(s);
            continue;
        }
        let len = if rc.bit(&mut probs, IS_REP + s) == 0 {
            let len_code = rc.length(&mut probs, LEN);
            let dist = rc.distance(&mut probs, len_code)?;
            reps = [dist, reps[0], reps[1], reps[2]];
            state = after_match(s);
            len_code as usize + 2
        } else {
            if rc.bit(&mut probs, IS_REP_G0 + s) == 0 {
                if rc.bit(&mut probs, IS_REP0_LONG + s) == 0 {
                    let rep0 = reps[0];
                    if rep0 >= size as u64 {
                        return None;
                    }
                    out.push(out[size - rep0 as usize - 1]);
                    state = after_short(s);
                    continue;
                }
            } else {
                let ri = if rc.bit(&mut probs, IS_REP_G1 + s) == 0 {
                    1
                } else {
                    2 + rc.bit(&mut probs, IS_REP_G2 + s) as usize
                };
                let d = reps[ri];
                reps.copy_within(0..ri, 1);
                reps[0] = d;
            }
            let len_code = rc.length(&mut probs, REP_LEN);
            state = after_rep(s);
            len_code as usize + 2
        };
        let rep0 = reps[0];
        if rep0 >= size as u64 || size + len > total {
            return None;
        }
        let distance = rep0 as usize + 1;
        let from = size - distance;
        if distance >= len {
            out.extend_from_within(from..from + len);
        } else {
            for k in 0..len {
                let b = out[from + k];
                out.push(b);
            }
        }
    }
    if rc.pos != payload.len() {
        return None;
    }
    out.drain(..h);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(n: usize, seed: u64) -> Vec<u8> {
        let mut x = seed;
        let words: [&[u8]; 6] = [b"lorem ", b"ipsum ", b"dolor ", b"sit ", b"amet, ", b"\n"];
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            if x.is_multiple_of(5) {
                out.push((x >> 24) as u8);
            } else {
                out.extend_from_slice(words[(x % 6) as usize]);
            }
        }
        out.truncate(n);
        out
    }

    #[test]
    fn round_trips_and_is_accepted() {
        for (n, seed) in [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (100, 5),
            (5000, 6),
            (200_000, 7),
        ] {
            let data = sample(n, seed);
            let payload = encode(&data);
            assert_eq!(decode(&payload, n as u64), Some(data), "n = {n}");
        }
        let zeros = vec![0u8; 100_000];
        assert_eq!(decode(&encode(&zeros), 100_000), Some(zeros));
        let all: Vec<u8> = (0..=255u8).cycle().take(70_000).collect();
        assert_eq!(decode(&encode(&all), 70_000), Some(all));
    }

    #[test]
    fn history_window_round_trips_and_helps() {
        let history = sample(50_000, 21);
        let mut input = history[10_000..30_000].to_vec();
        input.extend(sample(5_000, 22));
        let alone = encode(&input);
        let with = encode_with(&history, &input);
        assert!(
            with.len() * 3 < alone.len(),
            "{} vs {}",
            with.len(),
            alone.len()
        );
        assert_eq!(
            decode_with(&history, &with, input.len() as u64),
            Some(input.clone())
        );
        assert_eq!(encode_with(&[], &input), alone);
        assert_eq!(decode_with(&[], &alone, input.len() as u64), Some(input));
    }

    #[test]
    fn declines_what_it_does_not_vouch_for() {
        let data = sample(3000, 9);
        let mut payload = encode(&data);
        assert_eq!(decode(&payload, 2999), None);
        payload.push(0);
        assert_eq!(decode(&payload, 3000), None);
    }
}

