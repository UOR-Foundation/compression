//! The grep-shaped gates.
//!
//! These are crude on purpose. A gate that needs interpretation is a gate that
//! gets argued with; these ones read the source, find a token, and fail. Each
//! carries the rule it enforces in its failure message, because the point of a
//! red gate is to name the promise that was broken.

use std::path::{Path, PathBuf};

use crate::Fail;

/// The crates that ship: every crate under `crates/` that is not
/// `publish = false`. The rules below apply to those and not to the
/// dev-and-CI-only crates, which may use `std`, `alloc`, and floats freely.
///
/// Derived from the manifests rather than listed here. A list would be a second
/// place to remember, and the failure mode of a second place is that a crate
/// added to one is missing from the other --- silently, because a gate that
/// skips a crate cannot report on it. Reading `publish = false` asks the same
/// question `cargo publish` asks.
fn shipped(root: &Path) -> Result<Vec<String>, Fail> {
    let mut out = Vec::new();
    let dir = root.join("crates");
    if !dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&dir)? {
        let path = entry?.path();
        let manifest = path.join("Cargo.toml");
        let Ok(toml) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        if toml.lines().any(|l| l.trim() == "publish = false") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            out.push(name.to_string());
        }
    }
    out.sort();
    Ok(out)
}

struct Source {
    rel: String,
    text: String,
}

fn shipped_sources(root: &Path) -> Result<Vec<Source>, Fail> {
    let mut out = Vec::new();
    for name in shipped(root)? {
        let dir = root.join("crates").join(&name).join("src");
        if !dir.exists() {
            continue;
        }
        collect(&dir, root, &mut out)?;
    }
    Ok(out)
}

fn collect(dir: &Path, root: &Path, out: &mut Vec<Source>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, root, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = std::fs::read_to_string(&path)?;
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            out.push(Source { rel, text });
        }
    }
    Ok(())
}

/// Lines of `text` outside comments and outside `#[cfg(test)]` modules.
///
/// A rule about what the code *does* must not be tripped by a doc comment
/// explaining what it does not do, nor by a test that deliberately constructs
/// the forbidden thing to prove it is caught.
fn effective_lines(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut in_test = false;
    let mut test_depth = 0i32;
    let mut depth = 0i32;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with("//") || line.starts_with("#!") {
            // A `#![deny(...)]` or a comment states policy; it never is the
            // behaviour the gate is looking for.
            continue;
        }
        let opens = raw.matches('{').count() as i32;
        let closes = raw.matches('}').count() as i32;
        if line.starts_with("#[cfg(test)]") {
            in_test = true;
            test_depth = depth;
        }
        if !in_test {
            let code = line.split("//").next().unwrap_or("");
            if !code.trim().is_empty() {
                out.push((i + 1, code));
            }
        }
        depth += opens - closes;
        if in_test && depth <= test_depth && closes > 0 {
            in_test = false;
        }
    }
    out
}

/// R1: project semantics have exactly one project-owned source: the LexLean
/// authority graph. Generated Markdown is checked by `check-model`; handwritten
/// project prose may record evidence, rationale, operations, or planning, but it
/// may not introduce a second UORC specification or BCP-14 semantic rule.
pub fn audit_source_authority(root: &Path) -> Result<(), Fail> {
    const UNIVERSAL: &[&str] = &[
        "AGENTS.md",
        "TEMPLATE-CONTRACT.md",
        "TEMPLATE-VERIFICATION.md",
        "VERIFICATION.md",
    ];

    let mut markdown = Vec::new();
    gather_markdown(root, root, &mut markdown)?;
    markdown.sort();

    let mut violations = Vec::new();
    for path in markdown {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");
        if UNIVERSAL.contains(&rel.as_str()) {
            continue;
        }

        let contents = std::fs::read_to_string(&path)?;
        if is_generated_projection(&contents) {
            continue;
        }

        if specification_shaped_path(&rel) {
            violations.push(format!(
                "{rel}: handwritten specification-shaped project prose is not admitted"
            ));
        }
        if let Some(reason) = handwritten_semantic_rule(&contents) {
            violations.push(format!("{rel}: {reason}"));
        }
    }

    if !violations.is_empty() {
        return Err(format!(
            "R1: UORC project semantics are authored only in src/Uorc/Specification.lex.tex \
             and src/Uorc/Registry.lex.tex. Handwritten prose may carry evidence or planning, \
             but cannot become a second authority.\n\n{}",
            violations.join("\n")
        )
        .into());
    }

    println!("audit-source-authority: no handwritten project specification or semantic rule (R1)");
    Ok(())
}

fn is_generated_projection(contents: &str) -> bool {
    contents
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("<!-- @generated from src/Uorc/"))
}

fn specification_shaped_path(relative: &str) -> bool {
    let name = relative
        .rsplit('/')
        .next()
        .unwrap_or(relative)
        .to_ascii_lowercase();
    name.starts_with("spec")
        || name.starts_with("requirements")
        || name.starts_with("semantics")
}

fn handwritten_semantic_rule(contents: &str) -> Option<&'static str> {
    const MARKERS: &[(&str, &str)] = &[
        (
            "UORC MUST ",
            "contains a handwritten BCP-14 UORC requirement",
        ),
        (
            "UORC MUST NOT ",
            "contains a handwritten BCP-14 UORC prohibition",
        ),
        (
            "UORC SHALL ",
            "contains a handwritten BCP-14 UORC requirement",
        ),
        (
            "UORC REQUIRED ",
            "contains a handwritten BCP-14 UORC requirement",
        ),
        (
            "# UORC Specification",
            "declares a handwritten UORC specification",
        ),
        ("## UORC Semantics", "declares handwritten UORC semantics"),
        (
            "## UORC Requirements",
            "declares handwritten UORC requirements",
        ),
    ];
    MARKERS
        .iter()
        .find_map(|(marker, reason)| contents.contains(marker).then_some(*reason))
}

fn gather_markdown(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let skip = rel.components().any(|component| {
                matches!(
                    component.as_os_str().to_str(),
                    Some(".git" | ".lake" | ".lexlean" | "target")
                )
            });
            if !skip {
                gather_markdown(&path, root, out)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
    Ok(())
}

/// R5: no arbitrary limitation. Every bound is a property of the caller's
/// chosen instantiation, never of the code.
///
/// Concretely: no shipped crate may return an error the model does not
/// sanction. The absence of a class of error is checked here --- the absence of
/// negative testing is only honest if there is nothing to test negatively.
pub fn audit_limits(root: &Path) -> Result<(), Fail> {
    let sources = shipped_sources(root)?;
    // The only error type a shipped crate may name, plus the declaration check at
    // a declared boundary, which is not the operation failing.
    let sanctioned = ["NotAProduct", "ObservedBound", "KappaError"];

    let mut violations = Vec::new();
    for src in &sources {
        for (line_no, line) in effective_lines(&src.text) {
            let Some(pos) = line.find("Result<") else {
                continue;
            };
            let tail = &line[pos..];
            if sanctioned.iter().any(|s| tail.contains(s)) {
                continue;
            }
            violations.push(format!("{}:{line_no}:{}", src.rel, line.trim()));
        }
    }
    if !violations.is_empty() {
        return Err(format!(
            "R5: every bound is derived from declared parameters and is a \
             property of the caller's chosen instantiation. The only reportable \
             condition is that the requested object does not exist, reported at view \
             construction. A `Result` over anything else is a limitation the model does \
             not sanction.\n\n{}",
            violations.join("\n")
        )
        .into());
    }
    println!("audit-limits: no shipped crate returns an unsanctioned error (R5)");
    Ok(())
}

/// R4: nothing is deferred. No deferral marker, no stub, no placeholder
/// document section, no capability behind a flag that turns it off.
///
/// The markers are spelled in halves. This gate reads every crate *and*
/// `xtask`, which means it reads this file, and a list of forbidden tokens
/// written out in full is a list that matches itself --- the gate would fail on
/// its own definition, for ever. The alternative is exempting this file, and an
/// exemption is a hole: a real deferral parked in the gate would then be the one
/// place nothing looks. Split, the gate scans itself like anything else.
pub fn audit_deferral(root: &Path) -> Result<(), Fail> {
    let markers = [
        concat!("TO", "DO"),
        concat!("FIX", "ME"),
        concat!("XX", "X"),
        concat!("unimplemented", "!"),
        concat!("to", "do!"),
        concat!("for ", "now"),
        concat!("later ", "version"),
    ];
    let mut violations = Vec::new();

    // Every crate, not only the shipped ones, and `xtask` with them: R4 is a
    // promise about the repository, and a deferral parked in a gate is the same
    // deferral as one parked in shipped code.
    let mut files: Vec<PathBuf> = Vec::new();
    for dir in [root.join("crates"), root.join("xtask")] {
        if dir.exists() {
            gather_all(&dir, &mut files)?;
        }
    }
    // Every Markdown file at the root, discovered rather than listed. A list
    // here would go stale the first time a document was added or renamed, and
    // the failure would be silent: a document nobody scans reports nothing.
    for entry in std::fs::read_dir(root)? {
        let p = entry?.path();
        if p.extension().is_some_and(|e| e == "md") {
            files.push(p);
        }
    }

    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        for (i, line) in text.lines().enumerate() {
            for marker in markers {
                if !line.contains(marker) {
                    continue;
                }
                // A backticked marker is a *mention*, not a use: the
                // documentation has to be able to name what the gate catches
                // without tripping it. Anything outside code spans is real.
                if outside_code_spans(line, marker) {
                    violations.push(format!("{rel}:{}: {}", i + 1, line.trim()));
                }
            }
        }
    }

    if !violations.is_empty() {
        return Err(format!(
            "R4: nothing is deferred. None of {} may appear, and no stub, no \
             placeholder section, and no capability behind a flag that turns it \
             off. Every capability ships in the one release.\n\n{}",
            markers.join(", "),
            violations.join("\n")
        )
        .into());
    }
    println!("audit-deferral: nothing is deferred (R4)");
    Ok(())
}

/// Does `marker` occur in `line` outside every backtick-delimited span?
fn outside_code_spans(line: &str, marker: &str) -> bool {
    let mut rest = line;
    let mut at = 0usize;
    while let Some(pos) = rest.find(marker) {
        let absolute = at + pos;
        // An odd number of backticks before this occurrence means it is inside
        // a span.
        if line[..absolute].matches('`').count().is_multiple_of(2) {
            return true;
        }
        at = absolute + marker.len();
        rest = &line[at..];
    }
    false
}

fn gather_all(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            gather_all(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|e| e == "rs" || e == "md" || e == "toml")
        {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod source_authority_tests {
    use super::*;

    #[test]
    fn handwritten_authority_conflicts_are_rejected_uc_chr_02() {
        assert!(handwritten_semantic_rule("UORC MUST decode an archive this way.").is_some());
        assert!(handwritten_semantic_rule("# UORC Specification\nA second source.").is_some());
        assert!(specification_shaped_path("docs/SPEC.md"));
        assert!(specification_shaped_path("notes/requirements-v1.md"));
        assert!(!specification_shaped_path("docs/governance/evidence.md"));
        assert!(is_generated_projection(
            "<!-- @generated from src/Uorc/Specification.lex.tex and src/Uorc/Registry.lex.tex. -->\n"
        ));
        assert!(handwritten_semantic_rule("This is project-owned governance evidence.").is_none());
    }
}
