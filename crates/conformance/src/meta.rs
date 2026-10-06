//! The honesty meta-gate (R2).
//!
//! Three levels only mean something if the suite respects them, and nothing in
//! the arithmetic can enforce that. What can is a check on the *names* and the
//! *prose*: an `open` claim that a test asserts, or a `some-true` authority
//! that the README presents as this repository's own result, is exactly the
//! blurring of registers the discipline exists to prevent.
//!
//! The gate is deliberately about language, because that is where the failure
//! mode lives. Nobody sets out to claim they proved an upstream theorem; they
//! write "proves" where they meant "is evidence for", and six months later the
//! sentence is load-bearing.

use std::collections::BTreeSet;
use std::path::Path;

use repo_model::{Level, Model};

use crate::runner::SuiteReport;

/// What the meta-gate found.
#[derive(Clone, Debug, Default)]
pub struct HonestyReport {
    /// Every problem, each naming the rule it breaks.
    pub violations: Vec<String>,
    /// How many registered IDs were checked.
    pub ids_checked: usize,
    /// How many scenarios were read.
    pub scenarios_checked: usize,
}

impl HonestyReport {
    /// Did everything hold?
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Word stems that assert a claim as established.
///
/// A `build` claim may use them --- it *is* evidence, constructed here. An
/// `open` claim may not, because it is a measurement, and a `some-true` claim
/// may not, because it belongs to someone else.
///
/// Stems, matched against the start of a whole word: `prove` covers "proves",
/// "proved", and "proven" without also matching "improves", which a substring
/// search does.
const ASSERTIVE_STEMS: &[&str] = &[
    "prove",
    "proof",
    "guarantee",
    "establish",
    "demonstrat",
    "confirm",
    "verif",
    "certif",
    "validated",
    "settled",
    "achieve",
    "beats",
    "outperform",
    "surpass",
    "superior",
];

/// Phrases that assert, where no single word does.
const ASSERTIVE_PHRASES: &[&str] = &["shows that", "is true", "holds for", "world record"];

/// The assertive word or phrase in `line`, if it has one.
pub fn assertive_term(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();
    if let Some(phrase) = ASSERTIVE_PHRASES.iter().find(|p| lower.contains(*p)) {
        return Some(phrase);
    }
    lower
        .split(|c: char| !c.is_alphanumeric())
        .find_map(|word| ASSERTIVE_STEMS.iter().find(|s| word.starts_with(*s)))
        .copied()
}

/// Every Markdown document in the repository, as root-relative paths.
///
/// Discovered rather than listed: a document nobody scans reports nothing, and
/// a fixed list goes stale the first time one is added.
fn markdown_documents(root: &Path) -> std::io::Result<Vec<String>> {
    const SKIPPED: &[&str] = &[
        ".git",
        ".lake",
        ".lexlean",
        ".prism",
        "target",
        "vendor",
        "node_modules",
    ];
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| SKIPPED.contains(&n))
                {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "md") {
                let relative = path.strip_prefix(root).unwrap_or(&path);
                out.push(relative.display().to_string().replace('\\', "/"));
            }
        }
    }
    out.sort();
    Ok(out)
}

/// The registered-looking ID a test name ends in, if it is not registered.
///
/// A registered ID `UC-CHR-01` is discharged by a test whose name ends in
/// `uc_chr_01`. A name ending in the same shape --- the same leading prefix, the
/// same number of parts, a numeric last part --- names an ID too, whatever the
/// register's scheme is: two parts (`CT-04`) or three (`UC-CHR-01`).
fn unregistered_id_named_by(tail: &str, model: &Model) -> Option<String> {
    let tokens: Vec<&str> = tail.split('_').collect();
    let shapes: BTreeSet<(String, usize)> = model
        .ids
        .id
        .iter()
        .filter_map(|row| {
            let parts: Vec<&str> = row.id.split('-').collect();
            let numeric = parts
                .last()
                .is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
            (parts.len() >= 2 && numeric).then(|| (parts[0].to_lowercase(), parts.len()))
        })
        .collect();
    for (prefix, len) in shapes {
        if tokens.len() <= len {
            continue;
        }
        let parts = &tokens[tokens.len() - len..];
        let numeric = parts[len - 1].bytes().all(|b| b.is_ascii_digit());
        if parts[0] != prefix || parts.iter().any(|p| p.is_empty()) || !numeric {
            continue;
        }
        let id = parts.join("-").to_uppercase();
        if model.ids.get(&id).is_none() {
            return Some(id);
        }
    }
    None
}

/// Run the meta-gate.
///
/// `root` is the repository root. `tests` are the test names collected from the
/// workspace, which the caller gathers because it knows how to run `cargo`.
pub fn check_honesty(root: &Path, tests: &BTreeSet<String>) -> std::io::Result<HonestyReport> {
    let mut report = HonestyReport::default();
    let model = match Model::load(&root.join("model")) {
        Ok(m) => m,
        Err(e) => {
            report
                .violations
                .push(format!("R1: the model does not load: {e}"));
            return Ok(report);
        }
    };
    let suites: SuiteReport = crate::runner::scenarios_in(&root.join("features/suites"))?;
    report.scenarios_checked = suites.scenarios.len();
    report.ids_checked = model.ids.id.len();

    let scenario_ids = suites.ids();

    // R3, CM-02: every registered ID has a scenario, and a test named for it.
    for row in &model.ids.id {
        if !scenario_ids.contains(row.id.as_str()) {
            report.violations.push(format!(
                "R3: {} is registered but has no scenario in features/suites/. Every \
                 capability begins as a Gherkin scenario.",
                row.id
            ));
        }
        let slug = row.id.to_lowercase().replace('-', "_");
        if !tests.iter().any(|t| t.ends_with(&slug)) {
            report.violations.push(format!(
                "CM-02: {} is registered but no test name ends in `{slug}`. A claim with \
                 no test is an assertion.",
                row.id
            ));
        }
    }

    // CM-02, the other direction: every scenario names a registered ID.
    for s in &suites.scenarios {
        if model.ids.get(&s.id).is_none() {
            report.violations.push(format!(
                "CM-02: scenario `{}` in {} names `{}`, which is not in the register.",
                s.statement, s.suite, s.id
            ));
        }
        if s.steps.is_empty() {
            report.violations.push(format!(
                "R3: scenario `{}` in {} has no steps. There are no pending steps.",
                s.statement, s.suite
            ));
        }
        // R2: the scenario's tag must agree with the register.
        if let Some(row) = model.ids.get(&s.id) {
            if s.level != row.level.as_str() {
                report.violations.push(format!(
                    "R2: {} is tagged `{}` in {} but `{}` in the register.",
                    s.id,
                    s.level,
                    s.suite,
                    row.level.as_str()
                ));
            }
        }
    }

    // CM-02, the third direction: every ID a *test* names is registered.
    //
    // Without this the register can be a subset of what the suite claims: a test
    // called `..._ct_04` looks like it discharges `CT-04`, but if `CT-04` is not
    // a row then nothing checks it has a scenario and `CONFORMANCE.md` does not
    // list it. Three IDs were in exactly that state --- `CK-08`, `CT-04`, and
    // `CT-05` --- with passing tests and no register rows, which is a claim made
    // by a name and by nothing else.
    //
    // Only shapes the register already uses are checked, so a test whose name
    // merely happens to end in letters and digits is not a claim.
    for name in tests {
        let tail = name.rsplit("::").next().unwrap_or(name);
        if let Some(id) = unregistered_id_named_by(tail, &model) {
            report.violations.push(format!(
                "CM-02: test `{name}` names `{id}`, which is not in the register. An ID \
                 that exists only in a test name has no scenario and no row in \
                 CONFORMANCE.md."
            ));
        }
    }

    // R2: an `open` claim must not be asserted as established, anywhere.
    let open_ids: Vec<&str> = model
        .ids
        .id
        .iter()
        .filter(|r| r.level == Level::Open)
        .map(|r| r.id.as_str())
        .collect();
    let documents = markdown_documents(root)?;
    for doc in &documents {
        let Ok(text) = std::fs::read_to_string(root.join(doc)) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            for id in &open_ids {
                if !line.contains(id) {
                    continue;
                }
                if let Some(word) = assertive_term(line) {
                    report.violations.push(format!(
                        "R2: {doc}:{}: `{id}` is an `open` claim --- measured and reported, \
                         never asserted --- but this line says `{word}`.\n    {}",
                        i + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    // R2: a `some-true` authority is reproduced, not established here.
    for authority in &model.authorities.authority {
        for doc in &documents {
            let Ok(text) = std::fs::read_to_string(root.join(doc)) else {
                continue;
            };
            for (i, line) in text.lines().enumerate() {
                if !line.contains(&authority.id) {
                    continue;
                }
                if let Some(word) = assertive_term(line) {
                    report.violations.push(format!(
                        "R2: {doc}:{}: `{}` is cited, not established here, but this line \
                         says `{word}`.\n    {}",
                        i + 1,
                        authority.id,
                        line.trim()
                    ));
                }
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vocabulary check must fire on assertion and stay quiet on report.
    #[test]
    fn the_assertive_vocabulary_is_recognised() {
        for line in [
            "UC-PERF-01 proves the target",
            "UC-PERF-01 is proven",
            "UC-PERF-01 is settled: verified and certified",
            "UC-PERF-01 Guarantees a smaller archive",
            "UC-PERF-01 shows that the archive is smaller",
            "UC-PERF-01 outperforms every comparator",
            "UC-REC-01 is a world record",
        ] {
            assert!(assertive_term(line).is_some(), "must be recognised: {line}");
        }
        for line in [
            "UC-PERF-01 reports the measured ratio with its confidence interval",
            "UC-PERF-01 measured public-corpus compression improvement; open until measured",
            "UC-PERF-01 improves nothing by being registered",
        ] {
            assert_eq!(assertive_term(line), None, "must be permitted: {line}");
        }
    }
}
