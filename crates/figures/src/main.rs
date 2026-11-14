//! `figures [check|write|seed|report] [--root <dir>]`: hold the numbers in
//! uorc's implementation against the Prism model (see the library docs).

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut command = String::from("check");
    let mut root = PathBuf::from(".");
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => match args.next() {
                Some(r) => root = PathBuf::from(r),
                None => return usage(),
            },
            "check" | "write" | "seed" | "report" => command = a,
            _ => return usage(),
        }
    }
    match run(&command, &root) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("figures: {e}");
            eprintln!("figures: {}", figures::rule());
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: figures [check|write|seed|report] [--root <dir>]");
    ExitCode::from(2)
}

fn run(command: &str, root: &std::path::Path) -> Result<ExitCode, String> {
    match command {
        "write" | "seed" => {
            let out = figures::write(root, command == "seed")?;
            print!("{}", figures::summary(&out));
            println!("figures: wrote {}", figures::REGISTER);
            Ok(ExitCode::SUCCESS)
        }
        _ => {
            let register = figures::load(root)?
                .ok_or_else(|| format!("{} does not exist", figures::REGISTER))?;
            let out = figures::check(root, Some(&register))?;
            if command == "report" {
                print!("{}", figures::summary(&out));
            }
            if !out.rejected.is_empty() {
                return Err(format!("rejected:\n  {}", out.rejected.join("\n  ")));
            }
            if !out.stale.is_empty() {
                return Err(format!(
                    "{} lists what the implementation no longer holds; record it with `just figures-write`:\n  {}",
                    figures::REGISTER,
                    out.stale.join("\n  ")
                ));
            }
            println!(
                "figures: {} fixed-figure sites, none beyond the register ({} under named facets)",
                register.sites, register.under_facets
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}
