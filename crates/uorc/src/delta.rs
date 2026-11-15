//! Native delta filter family, byte-identical to `Uorc.Codec.Delta`.
//!
//! A frame read as samples `width` bytes wide (little-endian), interleaved in
//! `channels` channels: each sample becomes its difference from the sample
//! of the same channel before it, modulo `2^(8 width)`; samples before the
//! first read as 0, and the bytes past the last whole sample are kept. The
//! inverse adds the differences back in order, so each direction undoes the
//! other on every input. A frame whose samples change slowly (sound, images,
//! measurements) codes its differences in fewer bits than its samples.
//!
//! Which member a frame is coded over is recorded in its word
//! ([`crate::ncm::streams::DELTA_AT`]) and decided per frame by outcome
//! (`Client.stepDelta`): the member whose differences need the fewest
//! magnitude bits ([`candidate`]) is tried on the chosen coding, and kept only
//! when the archive is strictly shorter.

/// A member of the family: the sample width in bytes and the channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Delta {
    /// Bytes of a sample: one of [`Delta::WIDTHS`].
    pub width: u32,
    /// Interleaved channels, 1 to [`Delta::CHANNELS`].
    pub channels: u32,
}

impl Delta {
    /// The widest sample: the samples a 32-bit word holds.
    pub const WIDEST: u32 = u32::BITS / u8::BITS;
    /// The widths, 2^0 to [`Delta::WIDEST`] bytes.
    pub const WIDTHS: u32 = Delta::WIDEST.trailing_zeros() + 1;
    /// Bits of the width field: it holds the base-2 logarithm of the width
    /// plus one (1 to [`Delta::WIDTHS`]), 0 for no delta.
    pub const WIDTH_FIELD: u32 = u32::BITS - Delta::WIDTHS.leading_zeros();
    /// Bits of the channels field, the delta field's remaining bits: it
    /// holds the channels less one.
    pub const CHANNEL_FIELD: u32 =
        crate::ncm::streams::PLAN_BITS - crate::ncm::streams::DELTA_AT - Delta::WIDTH_FIELD;
    /// The most channels the channels field names.
    pub const CHANNELS: u32 = 1 << Delta::CHANNEL_FIELD;

    /// The member a word's delta field names (`None` for 0, and for a
    /// field that names no member).
    #[must_use]
    pub fn of_field(field: u32) -> Option<Delta> {
        let code = field & ((1 << Delta::WIDTH_FIELD) - 1);
        let channels = (field >> Delta::WIDTH_FIELD) + 1;
        if code == 0 || field >> (Delta::WIDTH_FIELD + Delta::CHANNEL_FIELD) != 0 {
            return None;
        }
        Some(Delta {
            width: 1 << (code - 1),
            channels,
        })
    }

    /// The word's delta field naming this member.
    #[must_use]
    pub fn field(self) -> u32 {
        (self.width.trailing_zeros() + 1) | (self.channels - 1) << Delta::WIDTH_FIELD
    }

    /// Whether the fields can name this member.
    #[must_use]
    pub fn valid(self) -> bool {
        self.width.is_power_of_two()
            && self.width.trailing_zeros() < Delta::WIDTHS
            && (1..=Delta::CHANNELS).contains(&self.channels)
    }

    /// Every member, widths then channels ascending.
    pub fn all() -> impl Iterator<Item = Delta> {
        (0..Delta::WIDTHS).flat_map(|w| {
            (1..=Delta::CHANNELS).map(move |channels| Delta {
                width: 1 << w,
                channels,
            })
        })
    }

    fn bits(self) -> u32 {
        self.width * u8::BITS
    }

    fn sample(self, data: &[u8], k: usize) -> u64 {
        let w = self.width as usize;
        data[k * w..(k + 1) * w]
            .iter()
            .rev()
            .fold(0u64, |v, &b| (v << u8::BITS) | u64::from(b))
    }

    fn put(self, out: &mut [u8], k: usize, v: u64) {
        let w = self.width as usize;
        for (j, b) in out[k * w..(k + 1) * w].iter_mut().enumerate() {
            *b = (v >> (j as u32 * u8::BITS)) as u8;
        }
    }

    fn mask(self) -> u64 {
        u64::MAX >> (u64::BITS - self.bits())
    }

    /// The bits of `v` as a signed sample of this width, less its sign: 0
    /// for 0 and -1.
    fn magnitude(self, v: u64) -> u32 {
        let top = 1 << (self.bits() - 1);
        let m = if v & top == 0 { v } else { !v & self.mask() };
        u64::BITS - m.leading_zeros()
    }
}

/// The filter (`decode` false) or its inverse of `data` under `d`.
#[must_use]
pub fn filter(data: &[u8], decode: bool, d: Delta) -> Vec<u8> {
    let mut out = data.to_vec();
    let samples = data.len() / d.width as usize;
    let c = d.channels as usize;
    let mask = d.mask();
    for k in c..samples {
        // the inverse reads the sample before as it decoded it
        let before = if decode {
            d.sample(&out, k - c)
        } else {
            d.sample(data, k - c)
        };
        let v = d.sample(data, k);
        let r = if decode {
            v.wrapping_add(before)
        } else {
            v.wrapping_sub(before)
        };
        d.put(&mut out, k, r & mask);
    }
    out
}

/// The magnitude bits of `data`'s samples under `d`: of its differences, or
/// with `raw` of the samples themselves (`Delta.measure`).
#[must_use]
pub fn magnitude_bits(data: &[u8], d: Delta, raw: bool) -> u64 {
    let samples = data.len() / d.width as usize;
    let c = d.channels as usize;
    (0..samples)
        .map(|k| {
            let v = d.sample(data, k);
            let v = if raw || k < c {
                v
            } else {
                v.wrapping_sub(d.sample(data, k - c))
            };
            u64::from(d.magnitude(v & d.mask()))
        })
        .sum()
}

/// The member a frame is tried over (`Delta.candidate`): of every member,
/// the one whose differences save the most magnitude bits over the frame's
/// own samples at its width (the first such, widths then channels
/// ascending); `None` when no member saves a bit.
#[must_use]
pub fn candidate(frame: &[u8]) -> Option<Delta> {
    let raw: Vec<u64> = (0..Delta::WIDTHS)
        .map(|w| {
            let d = Delta {
                width: 1 << w,
                channels: 1,
            };
            magnitude_bits(frame, d, true)
        })
        .collect();
    let mut best: Option<(u64, Delta)> = None;
    for d in Delta::all() {
        let saved =
            raw[d.width.trailing_zeros() as usize].saturating_sub(magnitude_bits(frame, d, false));
        if saved > best.map_or(0, |(s, _)| s) {
            best = Some((saved, d));
        }
    }
    best.map(|(_, d)| d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverts_both_ways() {
        let mut s = 7u32;
        let data: Vec<u8> = (0..1003)
            .map(|_| {
                s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                (s >> 16) as u8
            })
            .collect();
        for d in Delta::all() {
            let f = filter(&data, false, d);
            assert_eq!(filter(&f, true, d), data, "{d:?}");
            let g = filter(&data, true, d);
            assert_eq!(filter(&g, false, d), data, "{d:?}");
            assert_eq!(Delta::of_field(d.field()), Some(d));
            assert!(d.valid());
        }
        assert_eq!(Delta::of_field(0), None);
    }

    #[test]
    fn a_slow_wave_is_tried_at_its_width_and_channels() {
        // two 16-bit channels, each a slow ramp
        let mut data = Vec::new();
        for k in 0..4096i32 {
            for ch in 0..2 {
                let v = (k * (ch + 3) - 2000) as i16;
                data.extend_from_slice(&v.to_le_bytes());
            }
        }
        assert_eq!(
            candidate(&data),
            Some(Delta {
                width: 2,
                channels: 2
            })
        );
        assert_eq!(candidate(&[]), None);
    }
}
