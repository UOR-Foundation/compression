//! File trees as ordered UORC objects.
//!
//! A tree of `n > 0` entries is stored as the objects
//! `[manifest, file₁, …, fileₖ]`, one object per regular file in manifest
//! order. An empty tree is the archive of zero objects. A one-object archive is
//! always a single plain byte stream, never a tree, so the two never collide.
//!
//! The manifest is:
//!
//! ```text
//! "uorc-tree" 0x01  uleb(entryCount)
//! entry* = kind[1] uleb(pathLength) path        -- kind 0 file, 1 file +x, 2 directory
//! ```
//!
//! Paths are relative, `/`-separated UTF-8, with no empty, `.` or `..`
//! components, so restoring an archive never writes outside its destination.

use crate::Error;

const MAGIC: &[u8] = b"uorc-tree\x01";

/// What a tree entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A regular file.
    File,
    /// A regular file with its executable bit set.
    Executable,
    /// A directory (listed so empty directories survive).
    Directory,
}

/// One entry of a tree: a safe relative path and, for files, its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Relative `/`-separated path.
    pub path: String,
    /// What the entry is.
    pub kind: Kind,
    /// File contents; empty for directories.
    pub data: Vec<u8>,
}

/// Whether `path` is a relative path that stays inside its destination.
pub fn is_safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains('\0')
        && !path.contains(':')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn push_uleb(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn read_uleb(data: &[u8], at: &mut usize) -> Result<u64, Error> {
    let (mut value, mut shift) = (0u64, 0u32);
    loop {
        let b = *data.get(*at).ok_or(Error::Decode)?;
        *at += 1;
        if shift >= 64 {
            return Err(Error::Decode);
        }
        value |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
    }
}

/// The ordered objects that store `entries`. Paths must be safe and unique.
pub fn to_objects(entries: &[Entry]) -> Result<Vec<Vec<u8>>, Error> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let mut seen = std::collections::HashSet::new();
    let mut manifest = MAGIC.to_vec();
    push_uleb(&mut manifest, entries.len() as u64);
    let mut objects = vec![Vec::new()];
    for entry in entries {
        if !is_safe_path(&entry.path) || !seen.insert(entry.path.as_str()) {
            return Err(Error::Decode);
        }
        manifest.push(match entry.kind {
            Kind::File => 0,
            Kind::Executable => 1,
            Kind::Directory => 2,
        });
        push_uleb(&mut manifest, entry.path.len() as u64);
        manifest.extend_from_slice(entry.path.as_bytes());
        if entry.kind != Kind::Directory {
            objects.push(entry.data.clone());
        }
    }
    objects[0] = manifest;
    Ok(objects)
}

/// The entries the ordered `objects` store, checking every path is safe.
pub fn from_objects(mut objects: Vec<Vec<u8>>) -> Result<Vec<Entry>, Error> {
    if objects.is_empty() {
        return Ok(Vec::new());
    }
    let manifest = std::mem::take(&mut objects[0]);
    let mut files = objects.into_iter().skip(1);
    let rest = manifest.strip_prefix(MAGIC).ok_or(Error::Decode)?;
    let mut at = 0;
    let count = read_uleb(rest, &mut at)?;
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for _ in 0..count {
        let kind = match rest.get(at).ok_or(Error::Decode)? {
            0 => Kind::File,
            1 => Kind::Executable,
            2 => Kind::Directory,
            _ => return Err(Error::Decode),
        };
        at += 1;
        let len = usize::try_from(read_uleb(rest, &mut at)?).map_err(|_| Error::Decode)?;
        let end = at.checked_add(len).ok_or(Error::Decode)?;
        let path = std::str::from_utf8(rest.get(at..end).ok_or(Error::Decode)?)
            .map_err(|_| Error::Decode)?
            .to_owned();
        at = end;
        if !is_safe_path(&path) || !seen.insert(path.clone()) {
            return Err(Error::Decode);
        }
        let data = if kind == Kind::Directory {
            Vec::new()
        } else {
            files.next().ok_or(Error::Decode)?
        };
        entries.push(Entry { path, kind, data });
    }
    if at != rest.len() || files.next().is_some() {
        return Err(Error::Decode);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_paths_are_refused() {
        for bad in [
            "",
            "/etc/passwd",
            "../x",
            "a/../b",
            "a//b",
            "./a",
            "a/",
            "c:x",
            "a\\b",
        ] {
            assert!(!is_safe_path(bad), "{bad:?}");
        }
        assert!(is_safe_path("a/b/c.txt"));
    }

    #[test]
    fn manifest_round_trip() {
        let entries = vec![
            Entry {
                path: "d".into(),
                kind: Kind::Directory,
                data: vec![],
            },
            Entry {
                path: "d/empty".into(),
                kind: Kind::File,
                data: vec![],
            },
            Entry {
                path: "run.sh".into(),
                kind: Kind::Executable,
                data: b"#!/bin/sh\n".to_vec(),
            },
        ];
        let objects = to_objects(&entries).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(from_objects(objects).unwrap(), entries);
        assert!(to_objects(&[]).unwrap().is_empty());
        assert!(from_objects(vec![]).unwrap().is_empty());
    }

    #[test]
    fn hostile_manifest_is_rejected() {
        let mut manifest = MAGIC.to_vec();
        manifest.extend_from_slice(&[1, 0, 5]);
        manifest.extend_from_slice(b"../up");
        assert_eq!(
            from_objects(vec![manifest, b"x".to_vec()]),
            Err(Error::Decode)
        );
    }
}
