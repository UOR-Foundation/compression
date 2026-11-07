//! Repository gates.
//!
//! `cargo xtask <task>`; `just vv` runs the whole normative acceptance gate.
//! Each task below enforces one of the rules `AGENTS.md` sets out, and each
//! names the rule it enforces when it fails.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use repo_model::{codegen, AuthorityGraph, Model};

mod audit;
mod bootstrap;
#[cfg(test)]
mod claims;
#[cfg(test)]
mod integrity;
#[cfg(test)]
mod offline;
#[cfg(test)]
mod open_claims;
mod product_execution;
#[cfg(test)]
mod product_receipts;
#[cfg(test)]
mod varint;

fn main() -> ExitCode {
    let task = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "help".to_string());
    let write = std::env::args().any(|a| a == "--write");
    let root = repo_model::repo_root();

    let result = match task.as_str() {
        "check-model" => check_model(&root, write),
        "projection-inventory" => projection_inventory(&root),
        "audit-source-authority" => audit::audit_source_authority(&root),
        "audit-limits" => audit::audit_limits(&root),
        "audit-deferral" => audit::audit_deferral(&root),
        "audit-bootstrap" => bootstrap::audit(&root),
        "audit-dependency-updates" => bootstrap::audit_dependency_updates(&root),
        "validate" => validate(&root),
        "data-form" => convert(true),
        "notation" => convert(false),
        "runtime-sources" => runtime_sources(),
        _ => {
            eprintln!(
                "cargo xtask <task>\n\
                 \n\
                 check-model       R1: LexLean authority graph is the source; regenerate projections\n\
                 audit-source-authority   R1: reject a second handwritten UORC specification\n\
                 audit-limits      R5: no bound that cannot be traced to a parameter\n\
                 audit-deferral    R4: no deferral marker, no stub, no capability behind a flag\n\
                 audit-bootstrap   immutable SDK and least-privilege workflow trust root\n\
                 audit-dependency-updates   SDK ownership and ordinary dependency maintenance\n\
                 validate          run every gate above\n\
                 data-form         stdin source to its canonical JSON data form (for reading)\n\
                 notation          stdin source to the canonical language-1.2 notation\n\
                 runtime-sources   <dir>: the pinned LexLean runtime Lean modules, in import order\n\
                 \n\
                 --write           check-model only: rewrite generated model/document projections"
            );
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gate failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// A gate failure, reported with the rule it broke.
type Fail = Box<dyn std::error::Error>;

/// Language-1.2 modules are written in the notation; gates and controls that
/// inspect or plant module data convert a source text (stdin to stdout) only
/// through the SDK's own conversions (LexLean SPEC 17.21.1, 24.8). The data
/// form is for reading: a 1.2 project refuses it, so planted sources are
/// written back in the notation.
fn convert(to_data: bool) -> Result<(), Fail> {
    use std::io::{Read, Write};
    let mut text = String::new();
    std::io::stdin().lock().read_to_string(&mut text)?;
    let converted = if to_data {
        lexlean::sdk::to_data_form(&text)
    } else {
        lexlean::sdk::to_notation(&text)
    }
    .map_err(|e| e.to_string())?;
    std::io::stdout().lock().write_all(converted.as_bytes())?;
    Ok(())
}

/// Write the LexLean runtime's generated Lean modules (which every
/// language-1.2 module imports) under `<dir>`, and print their names in
/// dependency order, so a standalone `lean` can elaborate generated modules.
fn runtime_sources() -> Result<(), Fail> {
    let dir = std::env::args()
        .nth(2)
        .ok_or("runtime-sources needs an output directory")?;
    let modules = lexlean::backend::runtime::modules().map_err(|e| format!("{e:?}"))?;
    for module in modules {
        let file = Path::new(&dir).join(format!("{}.lean", module.name.replace('.', "/")));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file, &module.text)?;
        println!("{}", module.name);
    }
    Ok(())
}

/// Expose the renderer-owned artifact inventory to validation orchestration.
fn projection_inventory(root: &Path) -> Result<(), Fail> {
    let graph = AuthorityGraph::load(root)?;
    let mut paths: Vec<String> = codegen::render_all(&graph)
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    paths.sort();
    println!("{}", serde_json::to_string(&paths)?);
    Ok(())
}

/// R1: regenerate every committed projection from the LexLean authority graph.
fn check_model(root: &Path, write: bool) -> Result<(), Fail> {
    let graph = AuthorityGraph::load(root)?;
    let projected_model = graph.model();
    projected_model.check()?;

    let rendered = codegen::render_all(&graph);
    let expected_features: BTreeSet<String> = rendered
        .iter()
        .map(|(relative, _)| relative)
        .filter(|relative| {
            relative.starts_with("features/suites/") && relative.ends_with(".feature")
        })
        .cloned()
        .collect();

    for (relative, expected) in &rendered {
        let path: PathBuf = root.join(relative);
        if write {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, expected)?;
            println!("wrote {}", path.display());
            continue;
        }

        let committed = std::fs::read_to_string(&path).map_err(|e| {
            format!(
                "{}: {e}\nrun `cargo xtask check-model --write`",
                path.display()
            )
        })?;
        if committed != *expected {
            return Err(format!(
                "{} is stale: it disagrees with the LexLean authority graph.\n\
                 R1: generated model/document projections cannot become a second source. \
                 Run `cargo xtask check-model --write`.",
                path.display()
            )
            .into());
        }
    }

    let suite_dir = root.join("features/suites");
    let mut actual_features = BTreeSet::new();
    for entry in std::fs::read_dir(&suite_dir)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "feature")
        {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string()
                .replace('\\', "/");
            actual_features.insert(relative);
        }
    }
    if write {
        for stale in actual_features.difference(&expected_features) {
            std::fs::remove_file(root.join(stale))?;
            println!("removed stale generated suite {stale}");
        }
    } else if actual_features != expected_features {
        return Err(format!(
            "R1: generated feature inventory differs from the LexLean registry. expected {:?}, observed {:?}",
            expected_features, actual_features
        )
        .into());
    }

    let committed_model = Model::load(&root.join("model"))?;
    committed_model.check()?;
    if committed_model.ids.id.len() != projected_model.ids.id.len() {
        return Err("R1: committed ID projection count differs from LexLean source".into());
    }

    if write {
        println!(
            "check-model: regenerated {} LexLean-owned projections",
            rendered.len()
        );
    } else {
        println!(
            "check-model: all projections equal the LexLean authority graph, {} ids (CM-01)",
            projected_model.ids.id.len()
        );
    }
    Ok(())
}

/// The whole normative acceptance gate, in one place.
fn validate(root: &Path) -> Result<(), Fail> {
    check_model(root, false)?;
    audit::audit_source_authority(root)?;
    audit::audit_limits(root)?;
    audit::audit_deferral(root)?;
    bootstrap::audit(root)?;
    println!("validate: every gate passed");
    Ok(())
}
