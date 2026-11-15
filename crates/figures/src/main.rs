//! `figures [check|write|seed|report|native] [--root <dir>]`: hold the numbers in
//! uorc's implementation against the Prism model (see the library docs).

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let all: Vec<String> = std::env::args().skip(1).collect();
    if all.first().is_some_and(|a| a == "derive") {
        return derive(&all[1..]);
    }
    let mut args = all.into_iter();
    let mut command = String::from("check");
    let mut root = PathBuf::from(".");
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => match args.next() {
                Some(r) => root = PathBuf::from(r),
                None => return usage(),
            },
            "check" | "write" | "seed" | "report" | "native" => command = a,
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
    eprintln!("usage: figures [check|write|seed|report|native] [--root <dir>]");
    ExitCode::from(2)
}

fn run(command: &str, root: &std::path::Path) -> Result<ExitCode, String> {
    match command {
        "native" => {
            figures::native::write(root)?;
            for (out, src) in figures::native::GENERATED {
                println!("figures: generated {out} from {src}");
            }
            Ok(ExitCode::SUCCESS)
        }
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

/// `figures derive [--dry] <generated module> <file>…`: restate the literals
/// of native files that are quantities of their domains as the model's
/// quantities (see `figures::derive`).
fn derive(args: &[String]) -> ExitCode {
    let dry = args.iter().any(|a| a == "--dry");
    let lines = args.iter().any(|a| a == "--lines");
    let rest: Vec<String> = args
        .iter()
        .filter(|a| *a != "--dry" && *a != "--lines")
        .cloned()
        .collect();
    let Some((generated, files)) = rest.split_first() else {
        eprintln!("usage: figures derive [--dry] <generated module> <file>...");
        return ExitCode::from(2);
    };
    match figures::derive::rewrite(
        std::path::Path::new("."),
        generated,
        files,
        &["uorc", "uorc-wasm"],
        dry,
    ) {
        Ok(r) => {
            for (f, n) in &r.restated {
                println!("figures: {f}: {n} literals restated as model quantities");
            }
            if r.dropped > 0 {
                println!(
                    "figures: {} placed literals kept (no type view the compiler admits)",
                    r.dropped
                );
            }
            if lines {
                for (f, placed) in &r.placed {
                    for (l, v, q) in placed {
                        println!("{f}:{l} {v} -> {q}");
                    }
                }
                for (f, left) in &r.left {
                    for (l, v, sc) in left {
                        println!("{f}:{l} {v} [{sc}]");
                    }
                }
            }
            for (f, res) in &r.residue {
                let mut v: Vec<_> = res.iter().collect();
                v.sort_by(|a, b| b.1.cmp(a.1));
                let total: u64 = res.values().sum();
                println!("figures: {f}: {total} literals left; by value and domains in scope:");
                for ((val, scope), n) in v.iter().take(40) {
                    println!(
                        "  {val:>12} {n:>4}  {}",
                        if scope.is_empty() { "(none)" } else { scope }
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("figures: {e}");
            ExitCode::FAILURE
        }
    }
}
