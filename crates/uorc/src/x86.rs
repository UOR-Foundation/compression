//! Native x86 branch filter, byte-identical to `Uorc.Codec.X86`
//! (client/gen/x86_model.py).
//!
//! At every opcode E8 or E9 (near call, near jump) with four bytes after it, a
//! rel32 displacement `v` whose top byte is 00 or FF becomes the absolute
//! target `sext25(v + base + i + 5)`; the inverse subtracts. The four bytes are
//! skipped either way, so both directions scan the same positions and each
//! undoes the other on every input. `base` places the bytes in a longer stream.

/// The filter (`decode` false) or its inverse of `data` placed at `base`, and
/// the number of displacements rewritten (`X86.hits` counts them for `base` 0;
/// the count never depends on `base`).
pub fn filter(data: &[u8], decode: bool, base: u64) -> (Vec<u8>, u64) {
    let mut out = data.to_vec();
    let n = data.len();
    let mut hits = 0;
    let mut i = 0;
    while i < n {
        let b = data[i];
        if (b == 0xE8 || b == 0xE9) && i + 5 <= n {
            let top = data[i + 4];
            if top == 0 || top == 0xFF {
                let v = u32::from_le_bytes([data[i + 1], data[i + 2], data[i + 3], top]);
                let pos = (base + i as u64 + 5) as u32;
                let w0 = if decode {
                    v.wrapping_sub(pos)
                } else {
                    v.wrapping_add(pos)
                };
                let w25 = w0 & 0x01FF_FFFF;
                let w = if w25 < 1 << 24 {
                    w25
                } else {
                    w25 + 0xFE00_0000
                };
                out[i + 1..i + 5].copy_from_slice(&w.to_le_bytes());
                hits += 1;
            }
            i += 5;
        } else {
            i += 1;
        }
    }
    (out, hits)
}

/// Whether a frame is x86 code (`Archive.x86Dense`): one rewritten call or
/// jump per 256 bytes or more. Denser frames measured 1-8% smaller under the
/// filter, sparser ones up to 3% larger.
pub fn dense(data: &[u8]) -> bool {
    let (mut hits, mut i) = (0u64, 0usize);
    while i + 5 <= data.len() {
        if data[i] == 0xE8 || data[i] == 0xE9 {
            hits += u64::from(data[i + 4] == 0 || data[i + 4] == 0xFF);
            i += 5;
        } else {
            i += 1;
        }
    }
    hits > 0 && data.len() as u64 <= 256 * hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverts_both_ways() {
        let mut x = Vec::new();
        let mut s = 7u32;
        for _ in 0..5000 {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            x.push([0xE8, 0xE9, 0, 0xFF, (s >> 16) as u8][(s >> 8) as usize % 5]);
        }
        for base in [0, 7, (1 << 32) - 3] {
            let (y, h) = filter(&x, false, base);
            assert!(h > 0);
            assert_eq!(filter(&y, true, base).0, x);
            assert_eq!(filter(&filter(&x, true, base).0, false, base).0, x);
        }
    }
}
