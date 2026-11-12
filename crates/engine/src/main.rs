//! `uorc-engine`: run a tick, show the engine's state, or regenerate its files.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use uorc_engine::{decide, model::Laws, render, space, state, tick};

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

/// The corpus a tick decides over unless `--corpus` names another.
const CORPUS: &str = "uorc-v0.3.0";

/// One tick: decide, write the state files and the shipped table, and report
/// the outcome (`--output`: `key=value` lines for the workflow; `--summary`:
/// the tick in Markdown).
fn run_tick(root: &Path, args: &[String]) -> Result<(), String> {
    let laws = Laws::load(root)?;
    let mut engine = state::EngineFile::load(root)?;
    let mut hypotheses = state::HypothesesFile::load(root)?;
    let corpus = arg(args, "--corpus").unwrap_or_else(|| CORPUS.to_owned());
    let by = arg(args, "--by").unwrap_or_else(|| "local".to_owned());
    let mut decider = decide::FrontierDecider::open(root, &corpus, &by)?;
    let report = tick::tick(&laws, &mut engine, &mut hypotheses, &mut decider)?;
    decider.save(root)?;
    engine.save(root)?;
    hypotheses.save(root)?;
    let portfolio = space::Portfolio::of(&engine, &laws)?;
    let weights = root.join(state::WEIGHTS_RS);
    std::fs::write(&weights, render::weights_rs(&portfolio))
        .map_err(|e| format!("{}: {e}", weights.display()))?;
    let title = report.title();
    println!("{title}");
    for h in &report.decided {
        println!("  {}: {}", h.status, h.statement);
    }
    if let Some(out) = arg(args, "--output") {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&out)
            .map_err(|e| format!("{out}: {e}"))?;
        writeln!(f, "outcome={}\ntitle={title}", report.outcome).map_err(|e| e.to_string())?;
    }
    if let Some(out) = arg(args, "--summary") {
        let mut md = format!("{title}\n\nCorpus `{corpus}` ({}). Decided by `{by}`; every claim is in `frontier.toml` under its κ.\n\n| Status | Claim |\n| --- | --- |\n", decider_corpus(root, &corpus)?);
        for h in &report.decided {
            md.push_str(&format!("| {} | {} |\n", h.status, h.statement));
        }
        std::fs::write(&out, md).map_err(|e| format!("{out}: {e}"))?;
    }
    Ok(())
}

fn decider_corpus(root: &Path, name: &str) -> Result<String, String> {
    let register =
        frontier::Register::load(&root.join(decide::REGISTER)).map_err(|e| e.to_string())?;
    let c = register.corpus_named(name).map_err(|e| e.to_string())?;
    Ok(format!(
        "κ {}…, {} members, {} bytes",
        &c.kappa[..16],
        c.members.len(),
        c.bytes()
    ))
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
        Some("tick") => run_tick(&root, &args),
        Some("render") => render(&root, args.iter().any(|a| a == "--check")),
        Some("status") => status(&root),
        _ => {
            eprintln!(
                "uorc-engine <command> [--root DIR]\n\
                 \n\
                 tick     one tick: mint, decide by κ, learn, consolidate\n\
                 \x20        (--corpus NAME, --by local|factory:ID, --output FILE, --summary FILE)\n\
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
