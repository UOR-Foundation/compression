//! `uorc`: the command-line client.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage:
  uorc compress   [--best] [--steps N] [--threads N] [--force] [--stats] <input|-> [-o <output|->]
  uorc compress   [--best] [--steps N] [--force] [--stats] <file|dir>... -o <archive>
  uorc decompress [--work N]  [--threads N] [--force|--resume] [--stats] <input|-> [-o <output|->]
  uorc inspect    [--json] <archive|->
  uorc verify     [--work N] [--json] <archive|-> [original]
  uorc --version

Compress writes the smaller of the ulz archive and the raw archive; --best
also tries the context-mixing coder (ncm: smallest, but well under 1 MB/s
each way), and --steps N also runs the program search for N steps; the
smallest is kept. Archives name the built-in decoder basis by identity, so
they carry no decoder kernel.
Input of any size streams through 8 MiB frames, compressed --threads N at a
time (default: every core) and written back to back; each frame after the
first may match into the two before it, so frames decode in order. A frame
identical to an earlier one is stored once, as a reference to it. Without -o, compress writes <input>.uorc and decompress strips a .uorc suffix
(or writes <input>.out). '-' reads stdin or writes stdout.

Given a directory or several paths, compress stores them as one archive of
ordered objects: a manifest of relative paths, then every file in order
(symbolic links are skipped). Decompressing such an archive restores the tree
under the output directory (default: the archive name without .uorc). Paths
that are absolute or contain '..' are refused. --resume keeps files that
already exist with the right contents, so an interrupted restore can finish.

Outputs are written to a temporary file next to the destination and renamed
into place, so a failed run leaves nothing behind. An existing output is never
replaced unless --force is given.

inspect reads the frame header and checks the integrity seal without decoding.
verify decodes in memory and checks integrity and declared lengths, and, given
an original, that the archive reproduces it byte for byte.

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
    level: uorc::Level,
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
            "--stats" => options.stats = true,
            "--best" => options.level = uorc::Level::Best,
            "--force" | "-f" => options.force = true,
            "--json" => options.json = true,
            "--resume" => options.resume = true,
            "-o" => options.output = Some(args.next().ok_or_else(|| usage("-o needs a path"))?),
            _ => options.paths.push(arg),
        }
    }
    Ok(options)
}

fn read(path: &str) -> Result<Vec<u8>, Failure> {
    if path == "-" {
        let mut bytes = Vec::new();
        io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|e| io_failure(path, e))?;
        Ok(bytes)
    } else {
        fs::read(path).map_err(|e| io_failure(path, e))
    }
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

fn write(path: &str, bytes: &[u8], force: bool) -> Result<(), Failure> {
    let fail = |e| io_failure(path, e);
    if path == "-" {
        let mut out = io::stdout().lock();
        return out
            .write_all(bytes)
            .and_then(|()| out.flush())
            .map_err(fail);
    }
    let dest = Path::new(path);
    let mut pending = Pending::create(dest).map_err(fail)?;
    pending.write_all(bytes).map_err(fail)?;
    pending.commit(dest, force).map_err(fail)
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

/// Collect `root` (a file or directory) into `entries`, with paths relative
/// to the directory that holds `root`.
fn collect(root: &Path, name: &str, entries: &mut Vec<uorc::tree::Entry>) -> Result<(), Failure> {
    use uorc::tree::{Entry, Kind};
    let shown = root.display().to_string();
    let meta = fs::symlink_metadata(root).map_err(|e| io_failure(&shown, e))?;
    if meta.file_type().is_symlink() {
        eprintln!("uorc: skipping symbolic link {shown}");
    } else if meta.is_dir() {
        entries.push(Entry {
            path: name.to_owned(),
            kind: Kind::Directory,
            data: Vec::new(),
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
            collect(&child.path(), &format!("{name}/{child_name}"), entries)?;
        }
    } else if meta.is_file() {
        entries.push(Entry {
            path: name.to_owned(),
            kind: if executable(&meta) {
                Kind::Executable
            } else {
                Kind::File
            },
            data: fs::read(root).map_err(|e| io_failure(&shown, e))?,
        });
    } else {
        eprintln!("uorc: skipping special file {shown}");
    }
    Ok(())
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
    check_free(&output, options.force)?;
    let mut entries = Vec::new();
    for path in &options.paths {
        let root = Path::new(path);
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| usage(format!("{path}: no usable file name")))?;
        collect(root, name, &mut entries)?;
    }
    let mut seen = std::collections::HashSet::new();
    if let Some(dup) = entries.iter().find(|e| !seen.insert(e.path.clone())) {
        return Err(usage(format!("two inputs are both named {}", dup.path)));
    }
    let input_bytes: usize = entries.iter().map(|e| e.data.len()).sum();
    let started = Instant::now();
    let archive = uorc::compress_tree(
        &entries,
        options.budget.unwrap_or(uorc::DEFAULT_STEPS),
        options.level,
    )?;
    let elapsed = started.elapsed();
    write(&output, &archive, options.force)?;
    if options.stats {
        eprintln!(
            "{{\"command\":\"compress\",\"entries\":{},\"input_bytes\":{input_bytes},\"output_bytes\":{},\"seconds\":{:.6}}}",
            entries.len(),
            archive.len(),
            elapsed.as_secs_f64()
        );
    }
    Ok(())
}

/// Restore a tree under `dest`. With `resume`, files already holding the
/// right bytes are kept; otherwise an existing file is refused unless `force`.
fn restore(
    entries: &[uorc::tree::Entry],
    dest: &Path,
    options: &Options,
) -> Result<usize, Failure> {
    use uorc::tree::Kind;
    let shown = dest.display().to_string();
    fs::create_dir_all(dest).map_err(|e| io_failure(&shown, e))?;
    let mut written = 0;
    for entry in entries {
        let path = dest.join(&entry.path);
        let shown = path.display().to_string();
        // A path component that is a symbolic link could lead outside `dest`.
        if let Some(parent) = Path::new(&entry.path).parent() {
            let mut at = dest.to_path_buf();
            for part in parent.components() {
                at.push(part);
                if fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
                    return Err(io_failure(
                        &shown,
                        io::Error::other("a parent directory is a symbolic link"),
                    ));
                }
            }
        }
        if entry.kind == Kind::Directory {
            fs::create_dir_all(&path).map_err(|e| io_failure(&shown, e))?;
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| io_failure(&shown, e))?;
        }
        if options.resume && fs::read(&path).is_ok_and(|bytes| bytes == entry.data) {
            continue;
        }
        write(&shown, &entry.data, options.force || options.resume)?;
        if entry.kind == Kind::Executable {
            set_executable(&path).map_err(|e| io_failure(&shown, e))?;
        }
        written += 1;
    }
    Ok(written)
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

fn decompress_tree(
    options: &Options,
    mut input: Box<dyn Source>,
    path: &str,
) -> Result<(), Failure> {
    if path == "-" {
        return Err(usage(
            "a multi-object archive restores to a directory; give -o <dir>",
        ));
    }
    let mut stream = Vec::new();
    input
        .read_to_end(&mut stream)
        .map_err(|e| io_failure(&options.paths[0], e))?;
    let started = Instant::now();
    let entries =
        uorc::decompress_tree(&stream, options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK))?;
    let written = restore(&entries, Path::new(path), options)?;
    if options.stats {
        eprintln!(
            "{{\"command\":\"decompress\",\"entries\":{},\"written\":{written},\"input_bytes\":{},\"seconds\":{:.6}}}",
            entries.len(),
            stream.len(),
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
            uorc::compress_stream(input, output, steps, options.level, threads)
        } else {
            let work = options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK);
            let stream = stream.take().expect("decompress reads its stream once");
            uorc::decompress_stream(stream, output, work, threads)
        }
    };
    let failed = |e: io::Error| match uorc::Error::of(&e) {
        Some(error) => Failure::from(error),
        None => io_failure(&format!("{input_path} -> {path}"), e),
    };
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
            format!("conditional on {hex}")
        },
    );
    format!(
        "format          {} (machine profile {})\n\
         scope           {scope}\n\
         representation  {}\n\
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

fn inspect(options: &Options) -> Result<(), Failure> {
    let stream = read(one_path(options)?)?;
    let infos = uorc::frames(&stream)?
        .into_iter()
        .map(uorc::inspect)
        .collect::<Result<Vec<_>, _>>()?;
    if infos.is_empty() {
        return Err(uorc::Error::Decode.into());
    }
    if options.json {
        let all: Vec<String> = infos.iter().map(uorc::Info::to_json).collect();
        println!("[{}]", all.join(","));
    } else {
        let all: Vec<String> = infos.iter().map(describe).collect();
        println!("{}", all.join("\n\n"));
    }
    if infos.iter().all(|info| info.integrity_ok) {
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
    let archive = read(archive_path)?;
    let original = original_path.map(|p| read(p)).transpose()?;
    let started = Instant::now();
    let infos = uorc::verify(
        &archive,
        original.as_deref(),
        options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK),
    )?;
    if options.json {
        let all: Vec<String> = infos.iter().map(uorc::Info::to_json).collect();
        println!(
            "{{\"ok\":true,\"compared\":{},\"seconds\":{:.6},\"archives\":[{}]}}",
            original.is_some(),
            started.elapsed().as_secs_f64(),
            all.join(",")
        );
    } else {
        println!(
            "ok: {} archive(s), {} objects, {} bytes from {} archive bytes{}",
            infos.len(),
            infos.iter().map(|i| i.object_lengths.len()).sum::<usize>(),
            infos.iter().map(|i| i.output_len).sum::<u64>(),
            archive.len(),
            if original.is_some() {
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
