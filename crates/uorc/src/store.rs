//! The frames a stream names again, held by κ-label while a later reference
//! frame still needs them.
//!
//! A reference frame ([`crate::archive::reference_archive`]) reproduces an
//! earlier frame of the same stream, named by that frame's κ-label
//! ([`crate::archive::frame_label`]). The decoder learns which labels are named,
//! and how often, from the archive headers before it decodes (see
//! [`Store::want`]), so it labels and keeps a frame only when a reference will
//! read it, and drops it after the last one. A stream with no reference frames
//! labels nothing and keeps nothing.
//!
//! Kept frames stay in memory up to [`STORE_MEMORY`] bytes; beyond that they
//! go to an unnamed temporary file, so memory stays bounded whatever the
//! stream holds.

use std::collections::HashMap;
use std::io;

use crate::archive;

/// Bytes of referenced frames kept in memory before the rest go to a temporary file.
#[cfg_attr(target_family = "wasm", allow(dead_code))]
pub const STORE_MEMORY: usize = 32 * crate::FRAME;

enum Slot {
    Memory(Vec<u8>),
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    Spilled(u64, usize),
}

/// Referenced frames by κ-label (see the module documentation).
#[derive(Default)]
pub(crate) struct Store {
    /// References still to come, per label.
    wanted: HashMap<[u8; 32], u64>,
    held: HashMap<[u8; 32], Slot>,
    in_memory: usize,
    #[cfg(not(target_family = "wasm"))]
    spill: Option<spill::File>,
}

impl Store {
    /// Note the reference frame `archive`, if it is one: its label will be read once more.
    pub(crate) fn want(&mut self, archive: &[u8]) {
        if let Some(label) = archive::reference_of(archive) {
            *self.wanted.entry(label).or_insert(0) += 1;
        }
    }

    /// A store for the archives of a stream held whole.
    pub(crate) fn of(archives: &[&[u8]]) -> Self {
        let mut store = Store::default();
        for archive in archives {
            store.want(archive);
        }
        store
    }

    /// Offer a decoded frame: kept if a reference still to come names it.
    pub(crate) fn offer(&mut self, frame: &[u8]) -> io::Result<()> {
        if self.wanted.is_empty() {
            return Ok(());
        }
        let label = archive::frame_label(frame);
        if !self.wanted.contains_key(&label) || self.held.contains_key(&label) {
            return Ok(());
        }
        let slot = self.keep(frame)?;
        self.held.insert(label, slot);
        Ok(())
    }

    fn keep(&mut self, frame: &[u8]) -> io::Result<Slot> {
        #[cfg(not(target_family = "wasm"))]
        if self.in_memory + frame.len() > STORE_MEMORY {
            if self.spill.is_none() {
                self.spill = Some(spill::File::create()?);
            }
            let file = self.spill.as_mut().expect("just created");
            return Ok(Slot::Spilled(file.append(frame)?, frame.len()));
        }
        self.in_memory += frame.len();
        Ok(Slot::Memory(frame.to_vec()))
    }

    /// The frame labelled `label` for one reference to it, if one was kept.
    pub(crate) fn take(&mut self, label: &[u8; 32]) -> io::Result<Option<Vec<u8>>> {
        let Some(left) = self.wanted.get_mut(label) else {
            return Ok(None);
        };
        *left -= 1;
        let last = *left == 0;
        if last {
            self.wanted.remove(label);
        }
        let slot = if last {
            self.held.remove(label)
        } else {
            match self.held.get(label) {
                Some(Slot::Memory(frame)) => return Ok(Some(frame.clone())),
                Some(Slot::Spilled(at, len)) => Some(Slot::Spilled(*at, *len)),
                None => None,
            }
        };
        match slot {
            None => Ok(None),
            Some(Slot::Memory(frame)) => {
                self.in_memory -= frame.len();
                Ok(Some(frame))
            }
            #[cfg(not(target_family = "wasm"))]
            Some(Slot::Spilled(at, len)) => {
                let file = self.spill.as_mut().expect("a spilled frame has a file");
                Ok(Some(file.read(at, len)?))
            }
            #[cfg(target_family = "wasm")]
            Some(Slot::Spilled(..)) => Ok(None),
        }
    }
}

#[cfg(not(target_family = "wasm"))]
mod spill {
    use std::fs::{self, OpenOptions};
    use std::io::{self, Read, Seek, SeekFrom, Write};
    use std::path::PathBuf;

    /// An append-only temporary file, removed when dropped (on Unix, as soon
    /// as it is open).
    pub(super) struct File {
        file: fs::File,
        end: u64,
        path: Option<PathBuf>,
    }

    impl File {
        pub(super) fn create() -> io::Result<Self> {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos());
            let path = std::env::temp_dir()
                .join(format!("uorc-frames-{}-{nanos:x}.tmp", std::process::id()));
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)?;
            // On Unix the open file outlives its name.
            let path = if cfg!(unix) && fs::remove_file(&path).is_ok() {
                None
            } else {
                Some(path)
            };
            Ok(File { file, end: 0, path })
        }

        pub(super) fn append(&mut self, frame: &[u8]) -> io::Result<u64> {
            let at = self.end;
            self.file.seek(SeekFrom::Start(at))?;
            self.file.write_all(frame)?;
            self.end += frame.len() as u64;
            Ok(at)
        }

        pub(super) fn read(&mut self, at: u64, len: usize) -> io::Result<Vec<u8>> {
            let mut frame = vec![0; len];
            self.file.seek(SeekFrom::Start(at))?;
            self.file.read_exact(&mut frame)?;
            Ok(frame)
        }
    }

    impl Drop for File {
        fn drop(&mut self) {
            if let Some(path) = &self.path {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_named_frames_until_their_last_reference() {
        let (a, b) = (vec![1u8; 100], vec![2u8; 100]);
        let reference = archive::reference_archive(&archive::frame_label(&a), &[100]);
        let mut store = Store::of(&[&reference, &reference]);
        store.offer(&b).expect("offer");
        store.offer(&a).expect("offer");
        assert_eq!(store.held.len(), 1);
        assert_eq!(
            store.take(&archive::frame_label(&a)).expect("take"),
            Some(a.clone())
        );
        assert_eq!(
            store.take(&archive::frame_label(&a)).expect("take"),
            Some(a.clone())
        );
        assert!(store.held.is_empty() && store.in_memory == 0);
        assert_eq!(store.take(&archive::frame_label(&a)).expect("take"), None);
    }

    #[test]
    fn spills_past_the_memory_budget() {
        let frames: Vec<Vec<u8>> = (0..3u8).map(|i| vec![i; 64]).collect();
        let refs: Vec<Vec<u8>> = frames
            .iter()
            .map(|f| archive::reference_archive(&archive::frame_label(f), &[64]))
            .collect();
        let views: Vec<&[u8]> = refs.iter().map(Vec::as_slice).collect();
        let mut store = Store::of(&views);
        store.in_memory = STORE_MEMORY - 100; // room for one frame only
        for f in &frames {
            store.offer(f).expect("offer");
        }
        assert!(matches!(
            store.held[&archive::frame_label(&frames[1])],
            Slot::Spilled(..)
        ));
        for f in frames.iter().rev() {
            assert_eq!(
                store.take(&archive::frame_label(f)).expect("take").as_ref(),
                Some(f)
            );
        }
    }
}
