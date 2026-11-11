//! `uorc-engine`: run a tick, show the engine's state, or regenerate its files.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use uorc_engine::{model::Laws, render, space, state};

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn root(args: &[String]) -> PathBuf {
    arg(args, "--root").map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// Regenerate (or with `--check` compare) `weights.rs` and `engine.yml`.
fn render(root: &Path, check: bool) -> Result<(), String> {
    let laws = Laws::load(root)?;
    if !laws.valid_workflow() {
        return Err("Uorc.Engine's workflow schedule or budgets are not valid".into());
    }
    let engine = state::EngineFile::load(root)?;
    let portfolio = space::Portfolio::of(&engine, &laws)?;
    for (rel, text) in [
        (state::WEIGHTS_RS, render::weights_rs(&portfolio)),
        (render::WORKFLOW, render::workflow(&laws)),
    ] {
        let path = root.join(rel);
        if check {
            let committed = std::fs::read_to_string(&path).unwrap_or_default();
            if committed != text {
                return Err(format!("{rel} is stale: run `uorc-engine render`"));
            }
        } else {
            std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
            println!("wrote {rel}");
        }
    }
    Ok(())
}

/// Print the engine's state.
fn status(root: &Path) -> Result<(), String> {
    let laws = Laws::load(root)?;
    let engine = state::EngineFile::load(root)?;
    let portfolio = space::Portfolio::of(&engine, &laws)?;
    println!(
        "engine: tick {}, rule 30 register {}",
        engine.tick, engine.rule30_state
    );
    for (f, name) in uorc::portfolio::FAMILY_NAMES.iter().enumerate() {
        let s = portfolio.strategy_bytes();
        let w = portfolio.weight_bytes();
        let table = uorc::portfolio::Table {
            strategies: &s,
            weights: &w,
        };
        let order: Vec<String> = table
            .schedule(f)
            .into_iter()
            .map(|i| {
                format!(
                    "{} ({})",
                    space::name(&portfolio.entries[i]),
                    portfolio.weights[f][i]
                )
            })
            .collect();
        println!("  {name}: {}", order.join(", "));
    }
    for t in &engine.ticks {
        println!(
            "  tick {}: {} ({} minted, {} proved, {} refuted, {} known)",
            t.tick, t.outcome, t.minted, t.proved, t.refuted, t.known
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = root(&args);
    let result = match args.first().map(String::as_str) {
        Some("render") => render(&root, args.iter().any(|a| a == "--check")),
        Some("status") => status(&root),
        _ => {
            eprintln!(
                "uorc-engine <command> [--root DIR]\n\
                 \n\
                 tick     one tick: mint, decide by κ, learn, consolidate\n\
                 status   the portfolio's schedule per family and the recent ticks\n\
                 render   regenerate crates/uorc/src/weights.rs and .github/workflows/engine.yml\n\
                 \x20        (--check: refuse drift instead)"
            );
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("uorc-engine: {e}");
            ExitCode::FAILURE
        }
    }
}
