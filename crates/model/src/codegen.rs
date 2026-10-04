//! Generated projections of the UORC LexLean authority graph.
//!
//! The committed TOML and Markdown files are reviewable outputs. `xtask
//! check-model` regenerates them in memory and rejects drift, so none can become
//! a second source for project semantics or claim truth.

use core::fmt::Write as _;

use crate::authority::AuthorityGraph;
use crate::Model;

/// Generated conformance register.
pub const CONFORMANCE_PATH: &str = "CONFORMANCE.md";
/// Generated project README.
pub const README_PATH: &str = "README.md";
/// Generated project verification summary.
pub const PROJECT_VERIFICATION_PATH: &str = "UORC-VERIFICATION.md";
/// Generated ID projection.
pub const IDS_PATH: &str = "model/ids.toml";
/// Generated authority projection.
pub const AUTHORITIES_PATH: &str = "model/authorities.toml";
/// Generated honesty-ledger projection.
pub const LEDGER_PATH: &str = "model/ledger.toml";

/// Render every committed projection of the LexLean graph.
#[must_use]
pub fn render_all(graph: &AuthorityGraph) -> Vec<(&'static str, String)> {
    let model = graph.model();
    vec![
        (IDS_PATH, render_ids(&model)),
        (AUTHORITIES_PATH, render_authorities(&model)),
        (LEDGER_PATH, render_ledger(&model)),
        (CONFORMANCE_PATH, render_conformance(&model)),
        (README_PATH, render_readme(graph)),
        (
            PROJECT_VERIFICATION_PATH,
            render_project_verification(graph),
        ),
    ]
}

/// Render `model/ids.toml`.
#[must_use]
pub fn render_ids(model: &Model) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`."
    );
    let _ = writeln!(
        out,
        "# Do not edit: the LexLean graph is the project authority."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "spec = \"template/1\"");
    for row in &model.ids.id {
        let _ = writeln!(out);
        let _ = writeln!(out, "[[id]]");
        let _ = writeln!(out, "id = {}", quoted(&row.id));
        let _ = writeln!(out, "level = {}", quoted(row.level.as_str()));
        let _ = writeln!(out, "suite = {}", quoted(&row.suite));
        let _ = writeln!(out, "statement = {}", quoted(&row.statement));
    }
    out
}

/// Render `model/authorities.toml`.
#[must_use]
pub fn render_authorities(model: &Model) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# @generated from the UORC LexLean authority graph by `cargo xtask check-model --write`."
    );
    let _ = writeln!(
        out,
        "# Do not edit: imported authorities are authored in LexLean."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "spec = \"template/1\"");
    if model.authorities.authority.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "authority = []");
        return out;
    }
    for row in &model.authorities.authority {
        let _ = writeln!(out);
        let _ = writeln!(out, "[[authority]]");
        let _ = writeln!(out, "id = {}", quoted(&row.id));
        let _ = writeln!(out, "name = {}", quoted(&row.name));
        let _ = writeln!(out, "citation = {}", quoted(&row.citation));
        let _ = writeln!(out, "checksum = {}", quoted(&row.checksum));
        if !row.checksum_reason.is_empty() {
            let _ = writeln!(out, "checksum_reason = {}", quoted(&row.checksum_reason));
        }
        let _ = writeln!(out, "statement = {}", quoted(&row.statement));
        if !row.realized_by.is_empty() {
            let values = row
                .realized_by
                .iter()
                .map(|value| quoted(value))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(out, "realized_by = [{values}]");
        }
    }
    out
}

/// Render `model/ledger.toml`.
#[must_use]
pub fn render_ledger(model: &Model) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# @generated from the UORC LexLean authority graph by `cargo xtask check-model --write`."
    );
    let _ = writeln!(
        out,
        "# Do not edit: claim disposition is authored in LexLean."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "spec = \"template/1\"");
    if model.ledger.claim.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "claim = []");
        return out;
    }
    for row in &model.ledger.claim {
        let _ = writeln!(out);
        let _ = writeln!(out, "[[claim]]");
        let _ = writeln!(out, "id = {}", quoted(&row.id));
        let _ = writeln!(out, "level = {}", quoted(row.level.as_str()));
        let _ = writeln!(out, "statement = {}", quoted(&row.statement));
        if let Some(feature) = &row.feature {
            let _ = writeln!(out, "feature = {}", quoted(feature));
        }
        if let Some(authority) = &row.authority {
            let _ = writeln!(out, "authority = {}", quoted(authority));
        }
        if let Some(sample_size) = row.sample_size {
            let _ = writeln!(out, "sample_size = {sample_size}");
        }
        if let Some(seed) = row.seed {
            let _ = writeln!(out, "seed = {seed}");
        }
    }
    out
}

/// Render `CONFORMANCE.md` from the generated model projection.
#[must_use]
pub fn render_conformance(model: &Model) -> String {
    let mut out = String::new();
    let w = &mut out;
    let _ = writeln!(
        w,
        "<!-- @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`. -->"
    );
    let _ = writeln!(
        w,
        "<!-- Do not edit. R1: the LexLean graph is the project authority. -->"
    );
    let _ = writeln!(w);
    let _ = writeln!(w, "# CONFORMANCE");
    let _ = writeln!(w);
    let _ = writeln!(
        w,
        "Every registered ID has one generated projection here, one scenario, and"
    );
    let _ = writeln!(
        w,
        "at least one executed test whose name ends in the normalized ID."
    );
    let _ = writeln!(w);
    let _ = writeln!(w, "The three honesty levels (R2):");
    let _ = writeln!(w);
    let _ = writeln!(w, "| Level | Meaning |");
    let _ = writeln!(w, "| --- | --- |");
    let _ = writeln!(
        w,
        "| `some-true` | A fact reproduced from an authority. **Not established here.** |"
    );
    let _ = writeln!(
        w,
        "| `build` | Constructed here and validated against its oracle. Evidence, not a proof. |"
    );
    let _ = writeln!(w, "| `open` | Measured and reported. **Never asserted.** |");

    let mut suites: Vec<&str> = model.ids.id.iter().map(|r| r.suite.as_str()).collect();
    suites.sort_unstable();
    suites.dedup();

    for suite in suites {
        let rows: Vec<_> = model.ids.id.iter().filter(|r| r.suite == suite).collect();
        if rows.is_empty() {
            continue;
        }
        let _ = writeln!(w);
        let _ = writeln!(w, "## {suite}");
        let _ = writeln!(w);
        let _ = writeln!(w, "| ID | Level | Statement |");
        let _ = writeln!(w, "| --- | --- | --- |");
        for row in rows {
            let statement = markdown_cell(&row.statement);
            let _ = writeln!(
                w,
                "| `{}` | `{}` | {} |",
                row.id,
                row.level.as_str(),
                statement
            );
        }
    }

    let _ = writeln!(w);
    let _ = writeln!(w, "## Cited authorities");
    let _ = writeln!(w);
    let _ = writeln!(w, "| Authority | Citation | Evidence here |");
    let _ = writeln!(w, "| --- | --- | --- |");
    for row in &model.authorities.authority {
        let realized = if row.realized_by.is_empty() {
            "--".to_string()
        } else {
            row.realized_by
                .iter()
                .map(|id| format!("`{id}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(
            w,
            "| `{}` | {} | {realized} |",
            row.id,
            markdown_cell(&row.citation)
        );
    }

    let _ = writeln!(w);
    let _ = writeln!(w, "## Claims that are not conformance IDs");
    let _ = writeln!(w);
    let _ = writeln!(w, "| ID | Level | Claim |");
    let _ = writeln!(w, "| --- | --- | --- |");
    for row in &model.ledger.claim {
        let _ = writeln!(
            w,
            "| `{}` | `{}` | {} |",
            row.id,
            row.level.as_str(),
            markdown_cell(&row.statement)
        );
    }

    out
}

/// Render the repository README from the charter.
#[must_use]
pub fn render_readme(graph: &AuthorityGraph) -> String {
    let c = &graph.charter;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<!-- @generated from src/Uorc/Specification.lex.tex and src/Uorc/Registry.lex.tex. -->"
    );
    let _ = writeln!(
        out,
        "<!-- Regenerate with `just model-write`; do not edit by hand. -->"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "# {}", c.product_name);
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", c.purpose);
    let _ = writeln!(out);
    let _ = writeln!(out, "## Authority");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", c.authority_source);
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", c.precedence);
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", c.source_ownership);
    let _ = writeln!(out);
    let _ = writeln!(out, "## Scope");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", c.scope);
    let _ = writeln!(out);
    let _ = writeln!(out, "### Non-scope");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", c.non_scope);

    render_claim_table(&mut out, graph);
    render_evidence_table(&mut out, graph);

    let r = &graph.research_position;
    let _ = writeln!(out);
    let _ = writeln!(out, "## Research position");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", r.prior_art);
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", r.hypothesis);
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", r.novelty_status);
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", r.performance_status);

    let _ = writeln!(out);
    let _ = writeln!(out, "## Repository evidence");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- [CONFORMANCE.md](CONFORMANCE.md) is the generated claim projection."
    );
    let _ = writeln!(
        out,
        "- [UORC-VERIFICATION.md](UORC-VERIFICATION.md) is the generated project verification summary."
    );
    let _ = writeln!(
        out,
        "- [AGENTS.md](AGENTS.md), [TEMPLATE-CONTRACT.md](TEMPLATE-CONTRACT.md), and [VERIFICATION.md](VERIFICATION.md) are inherited repository policy."
    );
    let _ = writeln!(
        out,
        "- `just vv` is the complete repository acceptance gate; targeted commands are subsets only."
    );

    out
}

/// Render a project-specific verification summary without modifying the
/// template-owned universal `VERIFICATION.md`.
#[must_use]
pub fn render_project_verification(graph: &AuthorityGraph) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<!-- @generated from src/Uorc/Specification.lex.tex and src/Uorc/Registry.lex.tex. -->"
    );
    let _ = writeln!(
        out,
        "<!-- Regenerate with `just model-write`; do not edit by hand. -->"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "# UORC verification summary");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "This project-owned summary states evidence boundaries from the LexLean"
    );
    let _ = writeln!(
        out,
        "authority graph. It does not replace the template-owned `VERIFICATION.md`."
    );

    render_claim_table(&mut out, graph);
    render_evidence_table(&mut out, graph);

    let _ = writeln!(out);
    let _ = writeln!(out, "## Source and projection checks");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- `lexlean lock --check` binds the authority project to the exact compiler semantics and workspace inputs."
    );
    let _ = writeln!(
        out,
        "- `lexlean check --all` requires the locked LexLean compiler to accept every authority module before repository projection checks run."
    );
    let _ = writeln!(
        out,
        "- `cargo xtask check-model` extracts the admitted authority records and rejects stale generated projections."
    );
    let _ = writeln!(
        out,
        "- `cargo xtask audit-source-authority` rejects a second handwritten UORC specification or BCP-14 semantic rule in project prose."
    );
    let _ = writeln!(
        out,
        "- `just bdd` reconciles every registered ID with its scenario, honesty level, and executed test root."
    );
    let _ = writeln!(
        out,
        "- `just vv` remains the only complete implementation-acceptance boundary."
    );

    out
}

fn render_claim_table(out: &mut String, graph: &AuthorityGraph) {
    let c = &graph.claim_boundaries;
    let _ = writeln!(out);
    let _ = writeln!(out, "## Claim boundaries");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Claim | Meaning |");
    let _ = writeln!(out, "| --- | --- |");
    let rows = [
        ("Implementation acceptance", &c.implementation_acceptance),
        ("Losslessness", &c.losslessness),
        ("Scoped minimum certification", &c.scoped_minimum),
        ("Benchmark improvement", &c.benchmark_improvement),
        ("External record acceptance", &c.external_record_acceptance),
    ];
    for (name, value) in rows {
        let _ = writeln!(out, "| {name} | {} |", markdown_cell(value));
    }
}

fn render_evidence_table(out: &mut String, graph: &AuthorityGraph) {
    let e = &graph.evidence_boundaries;
    let _ = writeln!(out);
    let _ = writeln!(out, "## Evidence and dependency boundaries");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Class | Boundary |");
    let _ = writeln!(out, "| --- | --- |");
    let rows = [
        ("Production dependencies", &e.production_dependencies),
        ("Validation authorities", &e.validation_authorities),
        ("Measurements", &e.measurements),
        ("Open research claims", &e.open_research_claims),
    ];
    for (name, value) in rows {
        let _ = writeln!(out, "| {name} | {} |", markdown_cell(value));
    }
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a Rust string cannot fail")
}

fn markdown_cell(value: &str) -> String {
    value.replace('\n', " ").replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthorityGraph, Level};

    #[test]
    fn generated_projections_are_exact_uc_src_02() {
        let root = crate::repo_root();
        let graph = AuthorityGraph::load(&root).expect("authority graph loads");
        for (path, expected) in render_all(&graph) {
            let actual = std::fs::read_to_string(root.join(path))
                .unwrap_or_else(|error| panic!("reading {path}: {error}"));
            assert_eq!(actual, expected, "{path} is not the LexLean projection");
        }
    }

    #[test]
    fn performance_claim_is_open_uc_perf_01() {
        let graph = AuthorityGraph::load(&crate::repo_root()).expect("authority graph loads");
        let row = graph
            .ids
            .get("UC-PERF-01")
            .expect("performance claim is registered");
        assert_eq!(row.level, Level::Open);
    }

    #[test]
    fn record_claim_is_open_uc_rec_01() {
        let graph = AuthorityGraph::load(&crate::repo_root()).expect("authority graph loads");
        let row = graph
            .ids
            .get("UC-REC-01")
            .expect("record claim is registered");
        assert_eq!(row.level, Level::Open);
    }
}
