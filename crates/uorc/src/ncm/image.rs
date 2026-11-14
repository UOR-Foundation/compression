//! Stored warmed models: a model state replayed once per host, kept as an
//! image in the running host's persistent store (`Uorc.Host`, field
//! `store`), and read back instead of replayed by every later process.
//!
//! An image is the warmed model's every field, written and read in one order
//! fixed by an exhaustive destructuring of [`Model`], so a field added to the
//! model and left out here does not compile. It is keyed by the build that
//! wrote it (the executable's identity and the kernel), the state's bytes,
//! the profile and the table mask, and is used only by a process of the same
//! build: a read model is the replayed model, value for value, so it changes
//! the time a frame takes and never its bytes (`stored_warm_keeps_bytes`).
//!
//! Whether a host keeps an image is decided by outcome on that host: the
//! image is kept when writing it took less time than replaying the state,
//! so reading it back is the cheaper way to the same model there. The store
//! holds at most the host's memory in images, least recently used out.

use super::{Apm, Bucket, Kernel, Model, Table, Twin, NC};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const TAG: &[u8] = b"uorc/warmed-image/1";

/// Plain data an image stores by its bytes.
trait Raw: Copy {}
impl Raw for u8 {}
impl Raw for u16 {}
impl Raw for u32 {}
impl Raw for i32 {}
impl Raw for i64 {}
impl Raw for Bucket {}

fn bytes<T: Raw>(s: &[T]) -> &[u8] {
    // SAFETY: `T` is plain data without padding (integers, or a bucket of
    // sixteen u32), so its bytes are initialized.
    unsafe { std::slice::from_raw_parts(s.as_ptr().cast::<u8>(), std::mem::size_of_val(s)) }
}

fn bytes_mut<T: Raw>(s: &mut [T]) -> &mut [u8] {
    // SAFETY: as in `bytes`; every byte pattern is a valid `T`.
    unsafe { std::slice::from_raw_parts_mut(s.as_mut_ptr().cast::<u8>(), std::mem::size_of_val(s)) }
}

struct Out<W: Write>(W);

impl<W: Write> Out<W> {
    fn gain(&mut self, g: Option<&[i64; NC]>) -> io::Result<()> {
        self.u(u64::from(g.is_some()))?;
        match g {
            Some(g) => self.s(g),
            None => Ok(()),
        }
    }
    fn u(&mut self, v: u64) -> io::Result<()> {
        self.0.write_all(&v.to_le_bytes())
    }
    fn s<T: Raw>(&mut self, s: &[T]) -> io::Result<()> {
        self.u(s.len() as u64)?;
        self.0.write_all(bytes(s))
    }
}

/// A reader of an image with `left` bytes still in its file: no length it
/// reads may ask for more, so a damaged image is refused before anything
/// is allocated for it.
struct In<R: Read> {
    r: R,
    left: u64,
}

impl<R: Read> In<R> {
    fn raw(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let n = buf.len() as u64;
        if n > self.left {
            return Err(bad());
        }
        self.left -= n;
        self.r.read_exact(buf)
    }
    fn u(&mut self) -> io::Result<u64> {
        let mut b = [0; 8];
        self.raw(&mut b)?;
        Ok(u64::from_le_bytes(b))
    }
    fn us(&mut self) -> io::Result<usize> {
        usize::try_from(self.u()?).map_err(|_| bad())
    }
    fn u32(&mut self) -> io::Result<u32> {
        u32::try_from(self.u()?).map_err(|_| bad())
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(self.u()? as i64 as i32)
    }
    fn b(&mut self) -> io::Result<bool> {
        Ok(self.u()? != 0)
    }
    fn len(&mut self) -> io::Result<usize> {
        let n = self.us()?;
        if n as u64 > self.left {
            return Err(bad());
        }
        Ok(n)
    }
    fn into<T: Raw>(&mut self, dst: &mut [T]) -> io::Result<()> {
        if self.len()? != dst.len() {
            return Err(bad());
        }
        self.raw(bytes_mut(dst))
    }
    fn arr<T: Raw + Default, const N: usize>(&mut self) -> io::Result<[T; N]> {
        let mut a = [T::default(); N];
        self.into(&mut a)?;
        Ok(a)
    }
    fn vec<T: Raw + Default>(&mut self) -> io::Result<Vec<T>> {
        let n = self.len()?;
        let mut v = vec![T::default(); n];
        self.raw(bytes_mut(&mut v))?;
        Ok(v)
    }
    fn table<T: Raw + super::Spare + Default>(&mut self) -> io::Result<Table<T>> {
        let n = self.len()?;
        let mut t = Table::new(n, T::default());
        self.raw(bytes_mut(&mut t))?;
        Ok(t)
    }
    fn gain(&mut self) -> io::Result<Option<Box<[i64; NC]>>> {
        Ok(if self.b()? {
            Some(Box::new(self.arr()?))
        } else {
            None
        })
    }
    fn twin(&mut self) -> io::Result<Option<Box<Twin>>> {
        if !self.b()? {
            return Ok(None);
        }
        let sel = self.b()?;
        let s1 = self.us()?;
        let pm1 = self.i32()?;
        let pr = self.u32()?;
        let dot1 = self.u()? as i64;
        let gain = self.gain()?;
        Ok(Some(Box::new(Twin {
            sel,
            w1: self.vec()?,
            a1: self.apm()?,
            a2: self.apm()?,
            s1,
            pm1,
            pr,
            dot1,
            gain,
        })))
    }
    fn apm(&mut self) -> io::Result<Apm> {
        let idx = self.us()?;
        Ok(Apm {
            t: self.table()?,
            idx,
        })
    }
}

fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "warmed image does not match")
}

fn usizes<const N: usize>(a: &[usize; N]) -> [u32; N] {
    a.map(|v| v as u32)
}

/// Writes every field of a warmed model (one with no lane, learning or twin).
fn save(m: &Model, w: impl Write) -> io::Result<()> {
    let Model {
        t: _,
        slots,
        act,
        na,
        stride,
        mask,
        mb,
        sl,
        hist,
        hoff,
        pos,
        seen,
        lane,
        c4,
        c8,
        wordh,
        word1,
        c0,
        bit,
        mtab,
        profile,
        pbits: _,
        m2tab,
        m2ptr,
        m2len,
        m2exp,
        m2b,
        m2sp,
        m2sn,
        m2i,
        rd,
        rrun,
        rexp,
        rok,
        ra,
        rsp,
        rsn,
        ri,
        mptr,
        mlen,
        mexp,
        mbucket,
        msp,
        msn,
        lastp,
        lastd,
        rcnt,
        r,
        hb,
        spec_key,
        spec_hb,
        bb,
        // derived again from bb before the next bit's update reads them
        distinct: _,
        sv: _,
        ix,
        st,
        mi,
        w1,
        w2,
        w3,
        s1,
        s2,
        s3,
        pm1,
        pm2,
        pm3,
        dots,
        gain,
        a1,
        a2,
        twin,
    } = m;
    if lane.is_some() {
        return Err(bad());
    }
    let mut o = Out(w);
    o.gain(gain.as_deref())?;
    o.u(u64::from(twin.is_some()))?;
    if let Some(t) = twin {
        let Twin {
            sel,
            w1,
            a1,
            a2,
            s1,
            pm1,
            pr,
            dot1,
            gain,
        } = &**t;
        o.u(u64::from(*sel))?;
        o.u(*s1 as u64)?;
        o.u(*pm1 as i64 as u64)?;
        o.u(u64::from(*pr))?;
        o.u(*dot1 as u64)?;
        o.gain(gain.as_deref())?;
        o.s(w1)?;
        o.u(a1.idx as u64)?;
        o.s(&a1.t)?;
        o.u(a2.idx as u64)?;
        o.s(&a2.t)?;
    }
    o.s(&usizes(act))?;
    for v in [*na, *stride, *m2i, *ri, *mi, *s1, *s2, *s3] {
        o.u(v as u64)?;
    }
    for v in [
        *mask, *mb, *hoff, *pos, *seen, *c4, *c8, *wordh, *word1, *c0, *bit, *profile, *m2ptr,
        *m2len, *m2exp, *m2b, *rd, *rrun, *rexp, *mptr, *mlen, *mexp, *mbucket, *r,
    ] {
        o.u(u64::from(v))?;
    }
    o.u(u64::from(slots.top))?;
    for v in [*rok, *ra] {
        o.u(u64::from(v))?;
    }
    for v in [*pm1, *pm2, *pm3] {
        o.u(v as i64 as u64)?;
    }
    o.s(dots)?;
    o.s(m2sp)?;
    o.s(m2sn)?;
    o.s(rsp)?;
    o.s(rsn)?;
    o.s(msp)?;
    o.s(msn)?;
    o.s(lastp)?;
    o.s(lastd)?;
    o.s(hb)?;
    o.s(spec_key)?;
    o.s(spec_hb)?;
    o.s(bb)?;
    o.s(ix)?;
    o.s(st)?;
    o.s(hist)?;
    o.s(rcnt)?;
    o.s(w1)?;
    o.s(w2)?;
    o.s(w3)?;
    o.u(a1.idx as u64)?;
    o.s(&a1.t)?;
    o.u(a2.idx as u64)?;
    o.s(&a2.t)?;
    o.s(sl)?;
    o.s(mtab)?;
    o.s(m2tab)?;
    o.0.flush()
}

/// Reads a model [`save`] wrote, every field in the same order.
fn load(r: impl Read, left: u64) -> io::Result<Model> {
    let mut i = In { r, left };
    let gain = i.gain()?;
    let twin = i.twin()?;
    let act = i.arr::<u32, { super::NC }>()?.map(|v| v as usize);
    let mut sz = [0usize; 8];
    for v in &mut sz {
        *v = i.us()?;
    }
    let [na, stride, m2i, ri, mi, s1, s2, s3] = sz;
    let mut w = [0u32; 24];
    for v in &mut w {
        *v = i.u32()?;
    }
    let [mask, mb, hoff, pos, seen, c4, c8, wordh, word1, c0, bit, profile, m2ptr, m2len, m2exp, m2b, rd, rrun, rexp, mptr, mlen, mexp, mbucket, r] =
        w;
    let top = i.u32()?;
    let (rok, ra) = (i.b()?, i.b()?);
    let (pm1, pm2, pm3) = (i.i32()?, i.i32()?, i.i32()?);
    Ok(Model {
        t: super::tables(),
        slots: super::Rates {
            top,
            ..super::tables().rate
        },
        act,
        na,
        stride,
        mask,
        mb,
        dots: i.arr()?,
        m2sp: i.arr()?,
        m2sn: i.arr()?,
        rsp: i.arr()?,
        rsn: i.arr()?,
        msp: i.arr()?,
        msn: i.arr()?,
        lastp: i.arr()?,
        lastd: i.arr()?,
        hb: i.arr()?,
        spec_key: i.arr()?,
        spec_hb: i.arr()?,
        bb: i.arr()?,
        // a repeated bucket's slots adapt one after another until the next
        // nibble checks them; sensing reads the words before an update
        distinct: false,
        sv: [0; NC],
        ix: i.arr()?,
        st: i.arr()?,
        hist: i.vec()?,
        rcnt: i.vec()?,
        w1: i.vec()?,
        w2: i.vec()?,
        w3: i.vec()?,
        a1: i.apm()?,
        a2: i.apm()?,
        sl: i.table()?,
        mtab: i.table()?,
        m2tab: i.table()?,
        hoff,
        pos,
        seen,
        lane: None,
        c4,
        c8,
        wordh,
        word1,
        c0,
        bit,
        profile,
        pbits: 12,
        m2ptr,
        m2len,
        m2exp,
        m2b,
        m2i,
        rd,
        rrun,
        rexp,
        rok,
        ra,
        ri,
        mptr,
        mlen,
        mexp,
        mbucket,
        r,
        mi,
        s1,
        s2,
        s3,
        pm1,
        pm2,
        pm3,
        gain,
        twin,
    })
}

/// The identity of the running build: the executable's path, length and
/// modification time, so a rebuilt or replaced client never reads an image
/// another build wrote.
fn build() -> Option<&'static [u8]> {
    static BUILD: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
    BUILD
        .get_or_init(|| {
            let exe = std::env::current_exe().ok()?;
            let meta = std::fs::metadata(&exe).ok()?;
            let since = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?;
            let mut b = exe.to_string_lossy().into_owned().into_bytes();
            b.extend_from_slice(&meta.len().to_le_bytes());
            b.extend_from_slice(&since.as_nanos().to_le_bytes());
            b.extend_from_slice(env!("CARGO_PKG_VERSION").as_bytes());
            Some(b)
        })
        .as_deref()
}

/// The image's path in the store and its key, for a model warmed by kernel
/// `K` from `state` under `profile` with table mask `mask`.
fn place<K: Kernel>(
    state: &[u8],
    profile: super::Point,
    (mask, mb): (u32, u32),
    form: u8,
) -> Option<(PathBuf, [u8; 32])> {
    let store = crate::region::Host::current().store.as_ref()?;
    let mut h = Sha256::new();
    h.update(TAG);
    h.update(build()?);
    h.update(std::any::type_name::<K>().as_bytes());
    h.update(profile.to_le_bytes());
    h.update(mask.to_le_bytes());
    h.update(mb.to_le_bytes());
    h.update([form]);
    h.update(Sha256::digest(state));
    let key: [u8; 32] = h.finalize().into();
    let name: String = key.iter().map(|b| format!("{b:02x}")).collect();
    Some((store.join(format!("{name}.warm")), key))
}

/// The warmed model the store holds for this key, if any.
/// `form` names what was warmed beside the model: 0 nothing, otherwise a
/// shared pass's twin (1 not learning, 2 learning).
pub(super) fn read<K: Kernel>(
    state: &[u8],
    profile: super::Point,
    dims: (u32, u32),
    form: u8,
) -> Option<Model> {
    let (path, key) = place::<K>(state, profile, dims, form)?;
    let f = std::fs::File::open(&path).ok()?;
    let size = f.metadata().ok()?.len();
    // a read makes the image the most recently used ([`prune`])
    let _ = f.set_modified(std::time::SystemTime::now());
    let mut r = io::BufReader::with_capacity(1 << 20, f);
    let mut head = [0u8; 32];
    r.read_exact(&mut head).ok()?;
    if head != key {
        return None;
    }
    let m = load(&mut r, size.checked_sub(32)?).ok()?;
    // the whole file was read: a trailing byte means another writer's image
    let mut rest = [0u8; 1];
    if r.read(&mut rest).ok()? != 0 || (m.mask, m.mb) != dims {
        return None;
    }
    Some(m)
}

/// Stores `m`, replayed in `replay`, when writing it takes less time than
/// the replay did on this host; otherwise leaves the store as it was.
pub(super) fn write<K: Kernel>(
    m: &Model,
    state: &[u8],
    profile: super::Point,
    dims: (u32, u32),
    form: u8,
    replay: Duration,
) {
    let Some((path, key)) = place::<K>(state, profile, dims, form) else {
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    // a writer's own temporary name: threads of one process write too
    static WRITER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp = path.with_extension(format!(
        "{}.{}.tmp",
        std::process::id(),
        WRITER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let t = Instant::now();
    let written = (|| -> io::Result<()> {
        let f = std::fs::File::create(&tmp)?;
        let mut w = io::BufWriter::with_capacity(1 << 20, f);
        w.write_all(&key)?;
        save(m, &mut w)?;
        w.into_inner()
            .map_err(io::IntoInnerError::into_error)?
            .sync_data()
    })();
    if written.is_ok() && t.elapsed() < replay && std::fs::rename(&tmp, &path).is_ok() {
        prune(dir, crate::region::Host::current().memory);
        return;
    }
    let _ = std::fs::remove_file(&tmp);
}

/// Keeps the store within `bound` bytes, the running host's memory: an image
/// is worth keeping while the host can hold it where a read is fast, so the
/// least recently used images (by the time each was last read or written)
/// leave first.
fn prune(dir: &std::path::Path, bound: u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut images: Vec<(std::time::SystemTime, u64, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "warm"))
        .filter_map(|e| {
            let m = e.metadata().ok()?;
            Some((m.modified().ok()?, m.len(), e.path()))
        })
        .collect();
    images.sort();
    let mut total: u64 = images.iter().map(|i| i.1).sum();
    for (_, len, path) in images {
        if total <= bound {
            break;
        }
        if std::fs::remove_file(&path).is_ok() {
            total -= len;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    /// A model read back from its image is the replayed model: the frame
    /// coded after it, and decoded after it, is the same to the byte.
    #[test]
    fn stored_warm_keeps_bytes() {
        let state: Vec<u8> = (0..40_000u32)
            .map(|i| b"the quick brown fox; "[i as usize % 21] ^ (i >> 9) as u8)
            .collect();
        let data: Vec<u8> = (0..30_000u32)
            .map(|i| b"jumps over the lazy dog. "[i as usize % 25] ^ (i >> 11) as u8)
            .collect();
        let profile = FULL;
        let total = (data.len() + state.len()) as u32;
        for twin in [false, true] {
            same_after_image(&state, &data, total, profile, twin);
        }
    }

    fn same_after_image(state: &[u8], data: &[u8], total: u32, profile: u32, twin: bool) {
        let mut m = Model::new(total, &[], u64::from(profile));
        if twin {
            m.twin = Some(Box::new(Twin {
                sel: profile & F_SEL == 0,
                w1: weights(256 * 9, m.na + 3, m.stride),
                a1: Apm::new(256 * 9),
                a2: Apm::new(1 << APM2_BITS),
                s1: 0,
                pm1: 0,
                pr: 2048,
                dot1: 0,
                gain: Some(Box::new([0; NC])),
            }));
        }
        m = if m.stride == NINP_NARROW {
            m.warm::<Scalar, NINP_NARROW>(state)
        } else {
            m.warm::<Scalar, NINP>(state)
        };
        let mut img = Vec::new();
        super::save(&m, &mut img).expect("saves");
        let back = super::load(&img[..], img.len() as u64).expect("loads");
        let mut again = Vec::new();
        super::save(&back, &mut again).expect("saves");
        assert_eq!(img, again, "an image reads back to the same fields");
        let half = img.len() / 2;
        assert!(
            super::load(&img[..half], half as u64).is_err(),
            "a cut image is refused"
        );
        // with a twin, both coders' payloads
        let stops = [usize::MAX, if twin { usize::MAX } else { 0 }];
        let a = run_encoder::<Scalar>(m, data, false, stops);
        let b = run_encoder::<Scalar>(back, data, false, stops);
        assert!(a.0.is_some());
        assert_eq!(a, b);
    }
}
