//! `uorc`: the command-line client.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage:
  uorc compress   [--steps N] [--threads N] [--host-memory B] [--force] [--stats] <input|-> [-o <output|->]
  uorc compress   [--steps N] [--threads N] [--host-memory B] [--force] [--stats] <file|dir>... -o <archive>
  uorc decompress [--work N]  [--threads N] [--force|--resume] [--stats] <input|-> [-o <output|->]
  uorc inspect    [--json] <archive|->
  uorc verify     [--work N]  [--threads N] [--json] <archive|-> [original|-]
  uorc --version

Compress codes with the context-mixing coder (ncm), x86 code over a branch
filter, and keeps the raw archive when that is smaller; --steps N also runs
the program search for N steps and keeps the smallest. --best is accepted for
earlier scripts and changes nothing. Archives name the built-in decoder basis
by identity, so they carry no decoder kernel.

Each frame's context table grows one bit at a time past the size its length
gives while that makes the archive smaller and a decoder on the declared host
can hold it; the frame records the table it was coded with. The declared host
is one of the page's workers (512 MiB, with the tables it keeps between
frames), so every archive decodes on the page; --host-memory B declares a
native decoder of B bytes instead.

Input of any size streams through 8 MiB frames, compressed --threads N at a
time (default: every core) and written back to back; each frame may match
into up to two frames before it in its block of four, and blocks decompress
on separate cores. Archives are written for a decode host that runs a frame
on one thread (one of the page's workers).
The archive depends on the input and the declared host, never on --threads.
Content repeated anywhere in the input (64 KiB or more,
found by content-defined anchors) is stored once, as a reference to its
earlier copy. Without -o, compress writes <input>.uorc and decompress
strips a .uorc suffix (or writes <input>.out). '-' reads stdin or writes
stdout.

Given a directory or several paths, compress stores them as one archive of
ordered objects: a manifest of relative paths, then every file in order
(symbolic links are skipped), read one file at a time. Decompressing such an
archive restores the tree under the output directory (default: the archive
name without .uorc). Paths that are absolute or contain '..' are refused.
--resume replaces files that already exist, so an interrupted restore can
finish.

Outputs are written to a temporary file next to the destination and renamed
into place, so a failed run leaves no partial file behind (a failed tree
restore keeps the files it completed). An existing output is never replaced
unless --force is given.

inspect reads every frame header and checks each integrity seal without
decoding, and names each frame's codec point (basis, profile word, model
state, lanes, and its κ) and the exact steps its decode takes. verify decodes without keeping the output and checks integrity and
declared lengths, and, given an original, that the archive reproduces it byte
for byte. All commands stream, so memory stays bounded whatever the size.

exit status: 0 ok, 2 usage, 3 I/O (including a refused overwrite),
4 not a valid archive or failed integrity, 5 step budget, 6 overflow,
7 archive does not match the original.";

/// A failure and the exit status it maps to.
struct Failure {
    code: u8,
    message: String,
}

const USAGE_ERROR: u8 = 2;
const IO_ERROR: u8 = 3;

fn usage(message: impl Into<String>) -> Failure {
    Failure {
        code: USAGE_ERROR,
        message: format!("{}\n\n{USAGE}", message.into()),
    }
}

fn io_failure(path: &str, error: io::Error) -> Failure {
    Failure {
        code: IO_ERROR,
        message: format!("{path}: {error}"),
    }
}

impl From<uorc::Error> for Failure {
    fn from(error: uorc::Error) -> Self {
        let code = match error {
            uorc::Error::Decode => 4,
            uorc::Error::Budget => 5,
            uorc::Error::Overflow => 6,
            uorc::Error::Mismatch => 7,
        };
        Failure {
            code,
            message: error.to_string(),
        }
    }
}

#[derive(Default)]
struct Options {
    command: String,
    budget: Option<u64>,
    stats: bool,
    force: bool,
    resume: bool,
    json: bool,
    threads: Option<usize>,
    paths: Vec<String>,
    output: Option<String>,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Options, Failure> {
    let mut options = Options {
        command: args.next().ok_or_else(|| usage("missing command"))?,
        ..Options::default()
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--steps" | "--work" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage(format!("{arg} needs a number")))?;
                options.budget = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| usage(format!("{arg}: not a number: {value}")))?,
                );
            }
            "--threads" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage("--threads needs a number"))?;
                options.threads = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| usage(format!("--threads: not a number: {value}")))?,
                );
            }
            "--host-memory" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage("--host-memory needs a number of bytes"))?;
                let memory = value
                    .parse::<u64>()
                    .ok()
                    .filter(|&m| m > 0)
                    .ok_or_else(|| usage(format!("--host-memory: not a byte count: {value}")))?;
                uorc::performance::declare_host(memory);
            }
            "--stats" => options.stats = true,
            "--best" => {}
            "--force" | "-f" => options.force = true,
            "--json" => options.json = true,
            "--resume" => options.resume = true,
            "-o" => options.output = Some(args.next().ok_or_else(|| usage("-o needs a path"))?),
            _ => options.paths.push(arg),
        }
    }
    Ok(options)
}

/// A file being written next to its destination. It is removed on drop
/// unless [`Pending::commit`] moved it into place.
struct Pending {
    temp: PathBuf,
    file: Option<File>,
}

impl Pending {
    fn create(dest: &Path) -> io::Result<Pending> {
        let dir = match dest.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir,
            _ => Path::new("."),
        };
        let name = dest
            .file_name()
            .map_or_else(|| "output".into(), |n| n.to_string_lossy().into_owned());
        for attempt in 0u32.. {
            let temp = dir.join(format!(".{name}.{}.{attempt}.uorc-tmp", std::process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&temp) {
                Ok(file) => {
                    return Ok(Pending {
                        temp,
                        file: Some(file),
                    })
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists && attempt < 1000 => {}
                Err(e) => return Err(e),
            }
        }
        unreachable!("the attempt loop returns")
    }
}

impl Write for Pending {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file.as_mut().expect("open").write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.as_mut().expect("open").flush()
    }
}

impl Pending {
    /// Flush to disk and move into place: atomically, and without replacing
    /// an existing `dest` unless `force`.
    fn commit(mut self, dest: &Path, force: bool) -> io::Result<()> {
        let file = self.file.take().expect("open");
        file.sync_all()?;
        drop(file);
        if force {
            fs::rename(&self.temp, dest)?;
        } else {
            // A hard link fails if `dest` exists, so no other writer can be
            // overwritten between a check and the move. Filesystems without
            // hard links fall back to a check then rename.
            match fs::hard_link(&self.temp, dest) {
                Ok(()) => fs::remove_file(&self.temp)?,
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => return Err(exists(dest)),
                Err(_) if dest.exists() => return Err(exists(dest)),
                Err(_) => fs::rename(&self.temp, dest)?,
            }
        }
        self.temp = PathBuf::new();
        Ok(())
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.temp.as_os_str().is_empty() {
            let _ = fs::remove_file(&self.temp);
        }
    }
}

fn exists(dest: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("{} exists (use --force to replace it)", dest.display()),
    )
}

/// Refuse early, before any work, when the output already exists.
fn check_free(path: &str, force: bool) -> Result<(), Failure> {
    if path != "-" && !force && Path::new(path).exists() {
        return Err(io_failure(path, exists(Path::new(path))));
    }
    Ok(())
}

fn one_path(options: &Options) -> Result<&str, Failure> {
    match options.paths.as_slice() {
        [path] => Ok(path),
        [] => Err(usage("missing input")),
        [_, extra, ..] => Err(usage(format!("unexpected argument: {extra}"))),
    }
}

fn is_tree_request(options: &Options) -> bool {
    options.paths.len() > 1 || (options.paths.len() == 1 && Path::new(&options.paths[0]).is_dir())
}

/// One entry of a tree to compress: where it is read from, and for a
/// regular file its length when listed.
struct Item {
    path: String,
    kind: uorc::tree::Kind,
    source: PathBuf,
    len: u64,
}

/// Collect `root` (a file or directory) into `items`, with paths relative to
/// the directory that holds `root`. File contents are read later, one file at
/// a time.
fn collect(root: &Path, name: &str, items: &mut Vec<Item>) -> Result<(), Failure> {
    use uorc::tree::Kind;
    let shown = root.display().to_string();
    let meta = fs::symlink_metadata(root).map_err(|e| io_failure(&shown, e))?;
    if meta.file_type().is_symlink() {
        eprintln!("uorc: skipping symbolic link {shown}");
    } else if meta.is_dir() {
        items.push(Item {
            path: name.to_owned(),
            kind: Kind::Directory,
            source: root.to_path_buf(),
            len: 0,
        });
        let mut children: Vec<_> = fs::read_dir(root)
            .map_err(|e| io_failure(&shown, e))?
            .collect::<Result<_, _>>()
            .map_err(|e| io_failure(&shown, e))?;
        children.sort_by_key(fs::DirEntry::file_name);
        for child in children {
            let child_name = child.file_name();
            let child_name = child_name.to_str().ok_or_else(|| Failure {
                code: IO_ERROR,
                message: format!("{}: file name is not UTF-8", child.path().display()),
            })?;
            collect(&child.path(), &format!("{name}/{child_name}"), items)?;
        }
    } else if meta.is_file() {
        items.push(Item {
            path: name.to_owned(),
            kind: if executable(&meta) {
                Kind::Executable
            } else {
                Kind::File
            },
            source: root.to_path_buf(),
            len: meta.len(),
        });
    } else {
        eprintln!("uorc: skipping special file {shown}");
    }
    Ok(())
}

/// The regular files of a tree read back to back, each exactly as long as it
/// was when listed: a file that changed length in between is an error.
struct Files {
    next: std::vec::IntoIter<(PathBuf, u64)>,
    current: Option<(BufReader<File>, u64, PathBuf)>,
}

impl Read for Files {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let changed = |path: &Path| {
            io::Error::other(format!(
                "{}: changed while it was being compressed",
                path.display()
            ))
        };
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            if let Some((file, left, path)) = &mut self.current {
                if *left == 0 {
                    if file.read(&mut [0])? != 0 {
                        return Err(changed(path));
                    }
                    self.current = None;
                    continue;
                }
                let want = buf.len().min(usize::try_from(*left).unwrap_or(usize::MAX));
                let n = file.read(&mut buf[..want])?;
                if n == 0 {
                    return Err(changed(path));
                }
                *left -= n as u64;
                return Ok(n);
            }
            match self.next.next() {
                Some((path, len)) => {
                    let file = File::open(&path).map_err(|e| {
                        io::Error::new(e.kind(), format!("{}: {e}", path.display()))
                    })?;
                    self.current = Some((BufReader::with_capacity(1 << 20, file), len, path));
                }
                None => return Ok(0),
            }
        }
    }
}

#[cfg(unix)]
fn executable(meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn executable(_: &fs::Metadata) -> bool {
    false
}

#[cfg(unix)]
fn set_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn set_executable(_: &Path) -> io::Result<()> {
    Ok(())
}

fn compress_tree(options: &Options) -> Result<(), Failure> {
    let output = match (&options.output, options.paths.as_slice()) {
        (Some(path), _) => path.clone(),
        (None, [dir]) => format!("{}.uorc", dir.trim_end_matches('/')),
        (None, _) => return Err(usage("several inputs need -o <archive>")),
    };
    if output == "-" {
        return Err(usage(
            "a tree archive is written to a file; give -o <archive>",
        ));
    }
    check_free(&output, options.force)?;
    let mut items = Vec::new();
    for path in &options.paths {
        let root = Path::new(path);
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| usage(format!("{path}: no usable file name")))?;
        collect(root, name, &mut items)?;
    }
    let mut seen = std::collections::HashSet::new();
    if let Some(dup) = items.iter().find(|e| !seen.insert(e.path.as_str())) {
        return Err(usage(format!("two inputs are both named {}", dup.path)));
    }
    let layout = uorc::tree::layout(items.iter().map(|i| (i.path.as_str(), i.kind, i.len)))?;
    let files = Files {
        next: items
            .iter()
            .filter(|i| i.kind != uorc::tree::Kind::Directory)
            .map(|i| (i.source.clone(), i.len))
            .collect::<Vec<_>>()
            .into_iter(),
        current: None,
    };
    let input = io::Cursor::new(&layout.manifest).chain(files);
    let threads = options.threads.unwrap_or_else(uorc::default_threads);
    let steps = options.budget.unwrap_or(uorc::DEFAULT_STEPS);
    let dest = Path::new(&output);
    let failed = stream_failure(&output);
    let started = Instant::now();
    let mut pending = Pending::create(dest).map_err(|e| io_failure(&output, e))?;
    let mut writer = BufWriter::with_capacity(1 << 20, &mut pending);
    let totals = uorc::compress_objects_stream(&layout.lengths, input, &mut writer, steps, threads)
        .map_err(failed)?;
    drop(writer);
    pending
        .commit(dest, options.force)
        .map_err(|e| io_failure(&output, e))?;
    if options.stats {
        eprintln!(
            "{{\"command\":\"compress\",\"entries\":{},\"input_bytes\":{},\"output_bytes\":{},\"frames\":{},\"seconds\":{:.6}}}",
            items.len(),
            totals.read - layout.manifest.len() as u64,
            totals.written,
            totals.frames,
            started.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

/// A tree being restored under `dest` as its objects stream in: the manifest
/// first, then each regular file into a temporary file that is moved into
/// place once complete. A failed restore leaves only complete files, and
/// `--resume` finishes it.
struct Restore<'a> {
    dest: &'a Path,
    options: &'a Options,
    objects: u64,
    manifest: Vec<u8>,
    /// The regular files still to come, in order (after the manifest).
    files: std::vec::IntoIter<(String, uorc::tree::Kind)>,
    /// Whether the tree has directories only, so its one file object is the
    /// empty end object.
    directories_only: bool,
    current: Option<(Pending, PathBuf, bool)>,
    written: usize,
    entries: usize,
}

impl Restore<'_> {
    fn piece(&mut self, starts: bool, bytes: &[u8]) -> Result<(), Failure> {
        if starts {
            self.objects += 1;
            if self.objects == 2 {
                self.begin()?;
            }
            if self.objects >= 2 {
                self.finish_file()?;
                self.open_file()?;
            }
        }
        match (&mut self.current, self.objects) {
            (_, 1) => self.manifest.extend_from_slice(bytes),
            (Some((file, path, _)), _) => file
                .write_all(bytes)
                .map_err(|e| io_failure(&path.display().to_string(), e))?,
            // The end object of a tree of directories only is empty.
            (None, _) if bytes.is_empty() => {}
            (None, _) => return Err(uorc::Error::Decode.into()),
        }
        Ok(())
    }

    /// The manifest is complete: create every directory it lists.
    fn begin(&mut self) -> Result<(), Failure> {
        use uorc::tree::Kind;
        let listed = uorc::tree::read_manifest(&self.manifest)?;
        self.entries = listed.len();
        let shown = self.dest.display().to_string();
        fs::create_dir_all(self.dest).map_err(|e| io_failure(&shown, e))?;
        let mut files = Vec::new();
        for (path, kind) in listed {
            if kind == Kind::Directory {
                let at = self.place(&path)?;
                fs::create_dir_all(&at).map_err(|e| io_failure(&at.display().to_string(), e))?;
            } else {
                files.push((path, kind));
            }
        }
        self.directories_only = files.is_empty();
        self.files = files.into_iter();
        Ok(())
    }

    /// Where `path` goes under the destination, refusing a path whose parent
    /// directory is a symbolic link (it could lead outside the destination).
    fn place(&self, path: &str) -> Result<PathBuf, Failure> {
        let target = self.dest.join(path);
        if let Some(parent) = Path::new(path).parent() {
            let mut at = self.dest.to_path_buf();
            for part in parent.components() {
                at.push(part);
                if fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
                    return Err(io_failure(
                        &target.display().to_string(),
                        io::Error::other("a parent directory is a symbolic link"),
                    ));
                }
            }
        }
        Ok(target)
    }

    fn open_file(&mut self) -> Result<(), Failure> {
        let Some((path, kind)) = self.files.next() else {
            // Only a tree of directories has an object past its files: the
            // empty end object.
            return if self.directories_only && self.objects == 2 {
                Ok(())
            } else {
                Err(uorc::Error::Decode.into())
            };
        };
        let target = self.place(&path)?;
        let shown = target.display().to_string();
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| io_failure(&shown, e))?;
        }
        check_free(&shown, self.options.force || self.options.resume)?;
        let file = Pending::create(&target).map_err(|e| io_failure(&shown, e))?;
        self.current = Some((file, target, kind == uorc::tree::Kind::Executable));
        Ok(())
    }

    fn finish_file(&mut self) -> Result<(), Failure> {
        let Some((file, path, executable)) = self.current.take() else {
            return Ok(());
        };
        let shown = path.display().to_string();
        file.commit(&path, self.options.force || self.options.resume)
            .map_err(|e| io_failure(&shown, e))?;
        if executable {
            set_executable(&path).map_err(|e| io_failure(&shown, e))?;
        }
        self.written += 1;
        Ok(())
    }

    /// The stream ended: every listed file must have come.
    fn end(mut self) -> Result<(usize, usize), Failure> {
        match self.objects {
            0 => return Ok((0, 0)),
            1 => self.begin()?,
            _ => self.finish_file()?,
        }
        let ended = !self.directories_only || self.objects == 2;
        if self.files.next().is_some() || !ended {
            return Err(uorc::Error::Decode.into());
        }
        Ok((self.entries, self.written))
    }
}

fn open_input(path: &str) -> Result<Box<dyn Read>, Failure> {
    Ok(if path == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(BufReader::with_capacity(
            1 << 20,
            File::open(path).map_err(|e| io_failure(path, e))?,
        ))
    })
}

/// A stream to decompress: read twice, headers first (see `uorc::decompress_stream`).
trait Source: Read + Seek {}
impl<T: Read + Seek> Source for T {}

/// The stream at `path`, seekable. Standard input is used as is when it is a
/// regular file, and otherwise copied to an unnamed temporary file first.
fn open_stream(path: &str) -> Result<Box<dyn Source>, Failure> {
    if path != "-" {
        let file = File::open(path).map_err(|e| io_failure(path, e))?;
        return Ok(Box::new(BufReader::with_capacity(1 << 20, file)));
    }
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        if let Ok(fd) = io::stdin().as_fd().try_clone_to_owned() {
            let mut file = File::from(fd);
            if file.metadata().is_ok_and(|m| m.is_file()) && file.stream_position().is_ok() {
                return Ok(Box::new(BufReader::with_capacity(1 << 20, file)));
            }
        }
    }
    let spool = |e| io_failure("-", e);
    let name = std::env::temp_dir().join(format!("uorc-stdin-{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&name)
        .map_err(spool)?;
    #[cfg(unix)]
    let _ = fs::remove_file(&name);
    let copied = io::copy(&mut io::stdin().lock(), &mut file).and_then(|_| file.rewind());
    #[cfg(not(unix))]
    let _ = fs::remove_file(&name);
    copied.map_err(spool)?;
    Ok(Box::new(BufReader::with_capacity(1 << 20, file)))
}

/// Read enough of a stream to see its first archive header, and whether that
/// archive holds several objects (a tree). The stream is rewound to where it began.
fn peek_tree(input: &mut dyn Source) -> io::Result<bool> {
    let start = input.stream_position()?;
    let mut prefix = Vec::new();
    let mut want = uorc::HEADER_HINT;
    loop {
        (&mut *input)
            .take((want - prefix.len()) as u64)
            .read_to_end(&mut prefix)?;
        match uorc::frame_length(&prefix) {
            Ok(None) if prefix.len() == want => want *= 2,
            _ => break,
        }
    }
    input.seek(SeekFrom::Start(start))?;
    Ok(uorc::object_lengths(&prefix).is_ok_and(|lengths| lengths.len() != 1))
}

fn decompress_tree(options: &Options, input: Box<dyn Source>, path: &str) -> Result<(), Failure> {
    if path == "-" {
        return Err(usage(
            "a multi-object archive restores to a directory; give -o <dir>",
        ));
    }
    let threads = options.threads.unwrap_or_else(uorc::default_threads);
    let work = options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK);
    let started = Instant::now();
    let mut restore = Restore {
        dest: Path::new(path),
        options,
        objects: 0,
        manifest: Vec::new(),
        files: Vec::new().into_iter(),
        directories_only: false,
        current: None,
        written: 0,
        entries: 0,
    };
    // A restore failure stops the decoder; it is reported as it happened.
    let mut failure = None;
    let decoded = uorc::decompress_objects_stream(input, work, threads, |starts, bytes| {
        restore.piece(starts, bytes).map_err(|f| {
            failure = Some(f);
            io::Error::other("restore failed")
        })
    });
    let totals = match (decoded, failure) {
        (_, Some(failure)) => return Err(failure),
        (decoded, None) => decoded.map_err(stream_failure(&options.paths[0]))?,
    };
    let (entries, written) = restore.end()?;
    if options.stats {
        eprintln!(
            "{{\"command\":\"decompress\",\"entries\":{entries},\"written\":{written},\"input_bytes\":{},\"frames\":{},\"seconds\":{:.6}}}",
            totals.read,
            totals.frames,
            started.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

fn transform(options: &Options) -> Result<(), Failure> {
    let compress = options.command == "compress";
    if compress && is_tree_request(options) {
        return compress_tree(options);
    }
    let input_path = one_path(options)?;
    let path = match &options.output {
        Some(path) => path.clone(),
        None if input_path == "-" => "-".to_owned(),
        None if compress => format!("{input_path}.uorc"),
        None => input_path
            .strip_suffix(".uorc")
            .map_or_else(|| format!("{input_path}.out"), str::to_owned),
    };
    let mut input: Option<Box<dyn Read>> = None;
    let mut stream: Option<Box<dyn Source>> = None;
    if compress {
        input = Some(open_input(input_path)?);
    } else {
        let mut source = open_stream(input_path)?;
        if peek_tree(source.as_mut()).map_err(|e| io_failure(input_path, e))? {
            return decompress_tree(options, source, &path);
        }
        stream = Some(source);
    }
    check_free(&path, options.force)?;
    let threads = options.threads.unwrap_or_else(uorc::default_threads);
    let mut run = |output: &mut dyn Write| {
        if compress {
            let steps = options.budget.unwrap_or(uorc::DEFAULT_STEPS);
            let input = input.take().expect("compress reads its input once");
            uorc::compress_stream(input, output, steps, threads)
        } else {
            let work = options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK);
            let stream = stream.take().expect("decompress reads its stream once");
            uorc::decompress_stream(stream, output, work, threads)
        }
    };
    let shown = format!("{input_path} -> {path}");
    let failed = stream_failure(&shown);
    let started = Instant::now();
    let totals = if path == "-" {
        run(&mut BufWriter::with_capacity(1 << 20, io::stdout().lock())).map_err(failed)?
    } else {
        let dest = Path::new(&path);
        let mut pending = Pending::create(dest).map_err(|e| io_failure(&path, e))?;
        let mut output = BufWriter::with_capacity(1 << 20, &mut pending);
        let totals = run(&mut output).map_err(failed)?;
        drop(output);
        pending
            .commit(dest, options.force)
            .map_err(|e| io_failure(&path, e))?;
        totals
    };
    if options.stats {
        eprintln!(
            "{{\"command\":\"{}\",\"input_bytes\":{},\"output_bytes\":{},\"frames\":{},\"seconds\":{:.6}}}",
            options.command,
            totals.read,
            totals.written,
            totals.frames,
            started.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

fn describe(info: &uorc::Info) -> String {
    let scope = info.basis.map_or_else(
        || "self-contained".to_owned(),
        |b| {
            let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
            match info.window {
                0 => format!("conditional on {hex}"),
                1 => format!("conditional on {hex} (the library and the frame before)"),
                k => format!("conditional on {hex} (the library and the {k} frames before)"),
            }
        },
    );
    let point = uorc::performance::CodecPoint::of(info);
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let codec = format!(
        "profile word    {}\n\
         model state     {}\n\
         lanes           {}\n\
         steps           {}\n\
         codec point     {}",
        info.ncm_profile.map_or_else(
            || "none".to_owned(),
            |p| match info.representation {
                uorc::Representation::NcmX86 => format!("0x{p:05x} x86"),
                _ => format!("0x{p:05x}"),
            }
        ),
        if point.state == [0; 32] {
            "none".to_owned()
        } else {
            hex(&point.state)
        },
        point.lanes,
        uorc::performance::frame_steps(info).map_or_else(
            || "not modelled for this representation".to_owned(),
            |s| s.to_string()
        ),
        hex(&point.kappa()),
    );
    format!(
        "format          {} (machine profile {})\n\
         scope           {scope}\n\
         representation  {}\n\
         {codec}\n\
         objects         {} ({} bytes)\n\
         body            {} bytes\n\
         archive         {} bytes\n\
         integrity       {} {} ({})",
        info.format,
        info.profile,
        info.representation.name(),
        info.object_lengths.len(),
        info.output_len,
        info.body_len,
        info.archive_len,
        info.integrity_profile(),
        info.digest_hex(),
        if info.integrity_ok { "ok" } else { "MISMATCH" },
    )
}

/// The failure an I/O error from streaming `path` maps to.
fn stream_failure(path: &str) -> impl Fn(io::Error) -> Failure + '_ {
    move |e| match uorc::Error::of(&e) {
        Some(error) => error.into(),
        None => io_failure(path, e),
    }
}

fn inspect(options: &Options) -> Result<(), Failure> {
    let path = one_path(options)?;
    let mut out = io::stdout().lock();
    let printed = |e| io_failure("-", e);
    let (mut count, mut intact) = (0usize, true);
    for archive in uorc::archives(open_input(path)?) {
        let info = uorc::inspect(&archive.map_err(stream_failure(path))?)?;
        intact &= info.integrity_ok;
        let (separator, text) = if options.json {
            (if count == 0 { "[" } else { "," }, info.to_json())
        } else {
            (if count == 0 { "" } else { "\n\n" }, describe(&info))
        };
        write!(out, "{separator}{text}").map_err(printed)?;
        count += 1;
    }
    if count == 0 {
        return Err(uorc::Error::Decode.into());
    }
    writeln!(out, "{}", if options.json { "]" } else { "" }).map_err(printed)?;
    if intact {
        Ok(())
    } else {
        Err(Failure {
            code: 4,
            message: "integrity digest does not match the archive".to_owned(),
        })
    }
}

fn verify(options: &Options) -> Result<(), Failure> {
    let (archive_path, original_path) = match options.paths.as_slice() {
        [archive] => (archive, None),
        [archive, original] => (archive, Some(original)),
        [] => return Err(usage("missing archive")),
        [_, _, extra, ..] => return Err(usage(format!("unexpected argument: {extra}"))),
    };
    if archive_path == "-" && original_path.is_some_and(|p| p == "-") {
        return Err(usage("the archive and the original cannot both be stdin"));
    }
    let archive = open_stream(archive_path)?;
    let original = original_path.map(|p| open_input(p)).transpose()?;
    let started = Instant::now();
    let compared = original.is_some();
    let infos = uorc::verify_stream(
        archive,
        original,
        options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK),
        options.threads.unwrap_or_else(uorc::default_threads),
    )
    .map_err(stream_failure(archive_path))?;
    if options.json {
        let all: Vec<String> = infos.iter().map(uorc::Info::to_json).collect();
        println!(
            "{{\"ok\":true,\"compared\":{compared},\"seconds\":{:.6},\"archives\":[{}]}}",
            started.elapsed().as_secs_f64(),
            all.join(",")
        );
    } else {
        println!(
            "ok: {} archive(s), {} objects, {} bytes from {} archive bytes{}",
            infos.len(),
            infos.iter().map(|i| i.object_lengths.len()).sum::<usize>(),
            infos.iter().map(|i| i.output_len).sum::<u64>(),
            infos.iter().map(|i| i.archive_len).sum::<u64>(),
            if compared {
                ", identical to the original"
            } else {
                ""
            }
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1).peekable();
    match args.peek().map(String::as_str) {
        Some("--version") => {
            println!("uorc {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Some("--help" | "-h") | None => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => {}
    }
    let result = parse(args).and_then(|options| match options.command.as_str() {
        "compress" | "decompress" => transform(&options),
        "inspect" => inspect(&options),
        "verify" => verify(&options),
        other => Err(usage(format!("unknown command: {other}"))),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("uorc: {}", failure.message);
            ExitCode::from(failure.code)
        }
    }
}
