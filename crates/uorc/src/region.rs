//! The host model and region placement of the Prism model (`Uorc.Host`), as
//! the native client reads and applies them.
//!
//! A **host** is the machine a codec runs on, read once: its threads, its
//! memory, its base page, its large page (zero when the host offers none),
//! its cache line and its cache levels. A **region** is where one model table
//! lives: the table's working set, the bytes its lookups range over, decides
//! the granule it is placed on. A table whose working set spans at least one
//! large page is placed on large pages, so a lookup anywhere in it is covered
//! by one translation entry per large page instead of one per base page; a
//! smaller table stays on base pages, where a large page would only pad it.
//! Placement moves where a table's entries live, never their values, so an
//! archive's bytes never depend on it (tests: `region_placement_keeps_bytes`).

// WebAssembly memory has no pages to place: its tables stay plain vectors.
#![cfg_attr(target_family = "wasm", allow(dead_code))]

use std::sync::OnceLock;

/// One cache level of the host, innermost first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cache {
    /// The level (1 is innermost).
    pub level: u32,
    /// Bytes the level holds for one core's data (a unified level counts).
    pub bytes: u64,
}

/// The host as `Uorc.Host` models it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Host {
    /// Threads the host runs at once.
    pub threads: u64,
    /// Bytes of memory the host holds.
    pub memory: u64,
    /// Bytes of a base page.
    pub page: u64,
    /// Bytes of a large page a table can be placed on; 0 when the host
    /// offers none.
    pub large_page: u64,
    /// Bytes of a cache line.
    pub line: u64,
    /// The data cache levels, innermost first.
    pub caches: Vec<Cache>,
}

impl Host {
    /// The host this process runs on, read once.
    pub fn current() -> &'static Host {
        static HOST: OnceLock<Host> = OnceLock::new();
        HOST.get_or_init(probe)
    }

    /// Runs `f` with tables placed as on `host` instead of the running
    /// machine, on this thread. Placement never changes a value, so `f`
    /// returns the same bytes on every host (`region_placement_keeps_bytes`).
    pub fn placing<R>(host: Host, f: impl FnOnce() -> R) -> R {
        let host: &'static Host = Box::leak(Box::new(host));
        let before = PLACING.with(|p| p.replace(Some(host)));
        let r = f();
        PLACING.with(|p| p.set(before));
        r
    }

    /// The host tables are placed on: [`Host::current`], unless
    /// [`Host::placing`] says otherwise on this thread.
    fn placement() -> &'static Host {
        PLACING
            .with(std::cell::Cell::get)
            .unwrap_or_else(Host::current)
    }

    /// The granule a table whose lookups range over `working_set` bytes is
    /// placed on (`Uorc.Host.granule`): the large page when the host offers
    /// one and the working set spans at least one, else the base page.
    pub fn granule(&self, working_set: u64) -> u64 {
        if self.large_page != 0 && self.large_page <= working_set {
            self.large_page
        } else {
            self.page
        }
    }

    /// The host as one JSON object (`uorc host`).
    pub fn json(&self) -> String {
        let caches: Vec<String> = self
            .caches
            .iter()
            .map(|c| format!("{{\"level\":{},\"bytes\":{}}}", c.level, c.bytes))
            .collect();
        format!(
            "{{\"threads\":{},\"memory\":{},\"page\":{},\"largePage\":{},\"line\":{},\"caches\":[{}]}}",
            self.threads,
            self.memory,
            self.page,
            self.large_page,
            self.line,
            caches.join(",")
        )
    }
}

thread_local! {
    static PLACING: std::cell::Cell<Option<&'static Host>> = const { std::cell::Cell::new(None) };
}

fn read(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
}

/// A size as Linux's sysfs writes it (`48K`, `2048K`, `32M`, or bytes).
fn size(s: &str) -> Option<u64> {
    let (digits, unit) = match s.as_bytes().last()? {
        b'K' => (&s[..s.len() - 1], 1 << 10),
        b'M' => (&s[..s.len() - 1], 1 << 20),
        b'G' => (&s[..s.len() - 1], 1 << 30),
        _ => (s, 1),
    };
    digits.parse::<u64>().ok().map(|v| v * unit)
}

#[cfg(unix)]
fn base_page() -> u64 {
    // SAFETY: sysconf reads a constant of the running system.
    let p = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    u64::try_from(p).unwrap_or(0)
}

#[cfg(not(unix))]
fn base_page() -> u64 {
    0
}

fn probe() -> Host {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get() as u64);
    let memory = read("/proc/meminfo")
        .and_then(|m| {
            let line = m.lines().find(|l| l.starts_with("MemTotal:"))?;
            let kib = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
            Some(kib << 10)
        })
        .unwrap_or(0);
    // Transparent large pages: offered unless the kernel says `[never]`.
    let large_page = match read("/sys/kernel/mm/transparent_hugepage/enabled") {
        Some(mode) if !mode.contains("[never]") => {
            read("/sys/kernel/mm/transparent_hugepage/hpage_pmd_size")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
        }
        _ => 0,
    };
    let mut caches = Vec::new();
    let mut line = 0;
    for i in 0.. {
        let dir = format!("/sys/devices/system/cpu/cpu0/cache/index{i}");
        let Some(kind) = read(&format!("{dir}/type")) else {
            break;
        };
        if kind == "Instruction" {
            continue;
        }
        let level = read(&format!("{dir}/level")).and_then(|s| s.parse().ok());
        let bytes = read(&format!("{dir}/size")).and_then(|s| size(&s));
        if let (Some(level), Some(bytes)) = (level, bytes) {
            caches.push(Cache { level, bytes });
        }
        if line == 0 {
            line = read(&format!("{dir}/coherency_line_size"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }
    caches.sort_by_key(|c| c.level);
    Host {
        threads,
        memory,
        page: base_page(),
        large_page,
        line,
        caches,
    }
}

/// `len` entries of a model table, placed by the host's granule for its
/// working set (the whole table: every lookup is hashed over all of it).
/// A region is mapped memory on Unix and an ordinary allocation elsewhere.
pub(crate) struct Region<T: Copy> {
    ptr: std::ptr::NonNull<T>,
    len: usize,
    /// Mapped bytes (0: an ordinary allocation, or no entries).
    mapped: usize,
}

// SAFETY: a region owns its entries exclusively, like a `Vec<T>`.
unsafe impl<T: Copy + Send> Send for Region<T> {}
// SAFETY: shared access only reads, like `&[T]`.
unsafe impl<T: Copy + Sync> Sync for Region<T> {}

impl<T: Copy> Region<T> {
    /// `len` entries of `value`.
    pub(crate) fn new(len: usize, value: T) -> Region<T> {
        let r = Region::<T>::uninit(len);
        // Mapped memory is zero: a zero value needs no writes, so its pages
        // are only touched when a lookup first reaches them.
        let zero = {
            // SAFETY: `value` is plain data of `size_of::<T>()` bytes.
            let b = unsafe {
                std::slice::from_raw_parts(
                    std::ptr::from_ref(&value).cast::<u8>(),
                    std::mem::size_of::<T>(),
                )
            };
            b.iter().all(|&x| x == 0)
        };
        if r.mapped == 0 || !zero {
            for i in 0..len {
                // SAFETY: `i < len`, inside the allocation.
                unsafe { r.ptr.as_ptr().add(i).write(value) };
            }
        }
        r
    }

    /// `len` entries whose values the caller writes before reading.
    fn uninit(len: usize) -> Region<T> {
        let bytes = len * std::mem::size_of::<T>();
        if bytes == 0 {
            return Region {
                ptr: std::ptr::NonNull::dangling(),
                len,
                mapped: 0,
            };
        }
        #[cfg(unix)]
        if let Some((ptr, mapped)) = map(bytes) {
            return Region {
                ptr: ptr.cast(),
                len,
                mapped,
            };
        }
        let layout = std::alloc::Layout::array::<T>(len).expect("table size");
        // SAFETY: the layout has nonzero size.
        let ptr = unsafe { std::alloc::alloc(layout) }.cast::<T>();
        let Some(ptr) = std::ptr::NonNull::new(ptr) else {
            std::alloc::handle_alloc_error(layout)
        };
        Region {
            ptr,
            len,
            mapped: 0,
        }
    }
}

/// Maps `bytes` for a table, on large pages when its working set (all of it)
/// spans one: the mapping is aligned to the granule and advised as a large
/// page region. `None` when the host refuses the mapping.
#[cfg(unix)]
fn map(bytes: usize) -> Option<(std::ptr::NonNull<u8>, usize)> {
    let host = Host::placement();
    if host.page == 0 {
        return None;
    }
    let g = host.granule(bytes as u64) as usize;
    let round = |n: usize, to: usize| n.div_ceil(to) * to;
    let len = round(bytes, g);
    let large = g != host.page as usize;
    // A large-page region is mapped with one granule of slack, then trimmed
    // to an aligned span.
    let ask = if large { len + g } else { len };
    // SAFETY: an anonymous private mapping aliases nothing.
    let p = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            ask,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if p == libc::MAP_FAILED {
        return None;
    }
    let start = p as usize;
    let at = if large { round(start, g) } else { start };
    if large {
        // SAFETY: both spans lie inside the mapping just made and outside
        // the kept `[at, at + len)`.
        unsafe {
            if at > start {
                libc::munmap(p, at - start);
            }
            let end = start + ask;
            if end > at + len {
                libc::munmap((at + len) as *mut libc::c_void, end - at - len);
            }
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        // SAFETY: advice on a span this process mapped changes no value.
        unsafe {
            libc::madvise(at as *mut libc::c_void, len, libc::MADV_HUGEPAGE);
        }
    }
    std::ptr::NonNull::new(at as *mut u8).map(|p| (p, len))
}

impl<T: Copy> Drop for Region<T> {
    fn drop(&mut self) {
        if self.len == 0 || std::mem::size_of::<T>() == 0 {
            return;
        }
        #[cfg(unix)]
        if self.mapped != 0 {
            // SAFETY: the span is the mapping `map` returned.
            unsafe { libc::munmap(self.ptr.as_ptr().cast(), self.mapped) };
            return;
        }
        let layout = std::alloc::Layout::array::<T>(self.len).expect("table size");
        // SAFETY: allocated in `uninit` with this layout.
        unsafe { std::alloc::dealloc(self.ptr.as_ptr().cast(), layout) };
    }
}

impl<T: Copy> Clone for Region<T> {
    fn clone(&self) -> Region<T> {
        let r = Region::<T>::uninit(self.len);
        // SAFETY: both spans hold `len` entries and do not overlap.
        unsafe { std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), r.ptr.as_ptr(), self.len) };
        r
    }
}

impl<T: Copy> std::ops::Deref for Region<T> {
    type Target = [T];
    #[inline(always)]
    fn deref(&self) -> &[T] {
        // SAFETY: `len` initialized entries at `ptr`.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T: Copy> std::ops::DerefMut for Region<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        // SAFETY: `len` initialized entries at `ptr`, owned exclusively.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn granule_follows_the_working_set() {
        let h = Host {
            threads: 4,
            memory: 1 << 34,
            page: 4096,
            large_page: 2 << 20,
            line: 64,
            caches: vec![],
        };
        assert_eq!(h.granule(4 << 20), 2 << 20);
        assert_eq!(h.granule(2 << 20), 2 << 20);
        assert_eq!(h.granule((2 << 20) - 1), 4096);
        let none = Host { large_page: 0, ..h };
        assert_eq!(none.granule(1 << 30), 4096);
    }

    #[test]
    fn regions_hold_their_values() {
        for len in [0usize, 1, 1000, (4 << 20) / 4 + 3] {
            let mut r = Region::new(len, 7u32);
            assert!(r.iter().all(|&v| v == 7));
            let z = Region::new(len, 0u32);
            assert!(z.iter().all(|&v| v == 0));
            if len > 0 {
                r[len - 1] = 9;
                let c = r.clone();
                assert_eq!(c[len - 1], 9);
                assert_eq!(c[0], if len > 1 { 7 } else { 9 });
            }
        }
        assert_eq!(size("48K"), Some(48 << 10));
        assert_eq!(size("32M"), Some(32 << 20));
    }

    /// The archive's bytes never depend on where its tables live: the same
    /// input codes to the same payload, and decodes to the input, with the
    /// tables in ordinary allocations (a host without mappings, as in
    /// WebAssembly), on base pages only, and with every table on a large page.
    #[test]
    fn region_placement_keeps_bytes() {
        let data: Vec<u8> = (0..300_000u32)
            .map(|i| {
                (i.wrapping_mul(2_654_435_761) >> 13) as u8 & if i % 7 == 0 { 0xff } else { 0x3f }
            })
            .collect();
        let me = Host::current().clone();
        let page = me.page.max(4096);
        let hosts = [
            Host {
                page: 0,
                large_page: 0,
                ..me.clone()
            },
            Host {
                page,
                large_page: 0,
                ..me.clone()
            },
            Host {
                page,
                large_page: 2 * page,
                ..me.clone()
            },
            me.clone(),
        ];
        let mut first: Option<Vec<u8>> = None;
        for h in hosts {
            let (payload, back) = Host::placing(h, || {
                let p = crate::ncm::encode(&data).expect("codes");
                let back = crate::ncm::decode(&p, data.len() as u64).expect("decodes");
                (p, back)
            });
            assert_eq!(back, data);
            match &first {
                Some(f) => assert_eq!(&payload, f),
                None => first = Some(payload),
            }
        }
    }
}
