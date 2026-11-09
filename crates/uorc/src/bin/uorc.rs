//! `uorc`: the command-line client.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage:
  uorc compress   [--steps N] [--threads N] [--stats] <input|-> [-o <output|->]
  uorc decompress [--work N]  [--threads N] [--stats] <input|-> [-o <output|->]
  uorc --version

Compress writes the smaller of the ulz kernel archive and the raw archive;
--steps N also runs the program search for N steps and keeps the smallest.
Input of any size streams through 8 MiB frames, --threads N at a time
(default: every core); each frame is one self-contained archive.
Without -o, compress writes <input>.uorc and decompress strips a .uorc suffix
(or writes <input>.out). '-' reads stdin or writes stdout.";

struct Options {
    command: String,
    budget: Option<u64>,
    threads: Option<usize>,
    stats: bool,
    input: String,
    output: Option<String>,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Options, String> {
    let command = args.next().ok_or("missing command")?;
    let (mut budget, mut threads, mut stats, mut input, mut output) =
        (None, None, false, None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--steps" | "--work" => {
                let value = args.next().ok_or_else(|| format!("{arg} needs a number"))?;
                budget = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("{arg}: not a number: {value}"))?,
                );
            }
            "--threads" => {
                let value = args.next().ok_or("--threads needs a number")?;
                threads = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("--threads: not a number: {value}"))?,
                );
            }
            "--stats" => stats = true,
            "-o" => output = Some(args.next().ok_or("-o needs a path")?),
            _ if input.is_none() => input = Some(arg),
            _ => return Err(format!("unexpected argument: {arg}")),
        }
    }
    Ok(Options {
        command,
        budget,
        threads,
        stats,
        input: input.ok_or("missing input")?,
        output,
    })
}

fn reader(path: &str) -> std::io::Result<Box<dyn Read>> {
    Ok(if path == "-" {
        Box::new(std::io::stdin().lock())
    } else {
        Box::new(BufReader::with_capacity(1 << 20, File::open(path)?))
    })
}

fn writer(path: &str) -> std::io::Result<Box<dyn Write>> {
    Ok(if path == "-" {
        Box::new(std::io::stdout().lock())
    } else {
        Box::new(BufWriter::with_capacity(1 << 20, File::create(path)?))
    })
}

fn run(options: Options) -> Result<(), String> {
    let default_path = match options.command.as_str() {
        "compress" => format!("{}.uorc", options.input),
        "decompress" => options
            .input
            .strip_suffix(".uorc")
            .map_or_else(|| format!("{}.out", options.input), str::to_owned),
        other => return Err(format!("unknown command: {other}")),
    };
    let path = match &options.output {
        Some(path) => path.clone(),
        None if options.input == "-" => "-".to_owned(),
        None => default_path,
    };
    let input = reader(&options.input).map_err(|e| format!("{}: {e}", options.input))?;
    let output = writer(&path).map_err(|e| format!("{path}: {e}"))?;
    let threads = options.threads.unwrap_or_else(uorc::default_threads);
    let started = Instant::now();
    let totals = if options.command == "compress" {
        let steps = options.budget.unwrap_or(uorc::DEFAULT_STEPS);
        uorc::compress_stream(input, output, steps, threads)
    } else {
        let work = options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK);
        uorc::decompress_stream(input, output, work, threads)
    }
    .map_err(|e| e.to_string())?;
    let elapsed = started.elapsed();
    if options.stats {
        eprintln!(
            "{{\"command\":\"{}\",\"input_bytes\":{},\"output_bytes\":{},\"frames\":{},\"seconds\":{:.6}}}",
            options.command,
            totals.read,
            totals.written,
            totals.frames,
            elapsed.as_secs_f64()
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
    match parse(args).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("uorc: {message}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}
