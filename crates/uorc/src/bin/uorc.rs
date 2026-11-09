//! `uorc`: the command-line client.

use std::io::{Read, Write};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage:
  uorc compress   [--steps N] [--stats] <input|-> [-o <output|->]
  uorc decompress [--work N]  [--stats] <input|-> [-o <output|->]
  uorc --version

Compress writes the smaller of the ulz kernel archive and the raw archive;
--steps N also runs the program search for N steps and keeps the smallest.
Without -o, compress writes <input>.uorc and decompress strips a .uorc suffix
(or writes <input>.out). '-' reads stdin or writes stdout.";

struct Options {
    command: String,
    budget: Option<u64>,
    stats: bool,
    input: String,
    output: Option<String>,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Options, String> {
    let command = args.next().ok_or("missing command")?;
    let (mut budget, mut stats, mut input, mut output) = (None, false, None, None);
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
            "--stats" => stats = true,
            "-o" => output = Some(args.next().ok_or("-o needs a path")?),
            _ if input.is_none() => input = Some(arg),
            _ => return Err(format!("unexpected argument: {arg}")),
        }
    }
    Ok(Options {
        command,
        budget,
        stats,
        input: input.ok_or("missing input")?,
        output,
    })
}

fn read(path: &str) -> std::io::Result<Vec<u8>> {
    if path == "-" {
        let mut bytes = Vec::new();
        std::io::stdin().read_to_end(&mut bytes)?;
        Ok(bytes)
    } else {
        std::fs::read(path)
    }
}

fn write(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    if path == "-" {
        std::io::stdout().write_all(bytes)
    } else {
        std::fs::write(path, bytes)
    }
}

fn run(options: Options) -> Result<(), String> {
    let input = read(&options.input).map_err(|e| format!("{}: {e}", options.input))?;
    let started = Instant::now();
    let (output, default_path) = match options.command.as_str() {
        "compress" => (
            uorc::compress(&input, options.budget.unwrap_or(uorc::DEFAULT_STEPS)),
            format!("{}.uorc", options.input),
        ),
        "decompress" => (
            uorc::decompress(&input, options.budget.unwrap_or(uorc::DEFAULT_DECODE_WORK)),
            options
                .input
                .strip_suffix(".uorc")
                .map_or_else(|| format!("{}.out", options.input), str::to_owned),
        ),
        other => return Err(format!("unknown command: {other}")),
    };
    let output = output.map_err(|e| e.to_string())?;
    let elapsed = started.elapsed();
    let path = match options.output {
        Some(path) => path,
        None if options.input == "-" => "-".to_owned(),
        None => default_path,
    };
    write(&path, &output).map_err(|e| format!("{path}: {e}"))?;
    if options.stats {
        eprintln!(
            "{{\"command\":\"{}\",\"input_bytes\":{},\"output_bytes\":{},\"seconds\":{:.6}}}",
            options.command,
            input.len(),
            output.len(),
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
