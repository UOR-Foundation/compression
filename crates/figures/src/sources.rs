//! The page's JavaScript and the LexLean model the shipped client is
//! exported from: their scope, read from the factory graph, and their
//! numerals.

use std::collections::BTreeSet;
use std::path::Path;

use crate::{Error, Scan, Site, WIRE_MODULES};

/// A `closure:ROOTS@DIRS` input of the factory graph (`scripts/factory.mjs`
/// reads it the same way): ROOTS are module names, `Prefix.*` names every
/// module under that directory, and a module is read from the first of DIRS
/// (default `src`) that has it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Closure {
    /// Root module names or `Prefix.*` patterns.
    pub roots: Vec<String>,
    /// Source directories, first wins.
    pub dirs: Vec<String>,
}

impl Closure {
    /// Parse the text after `closure:`.
    pub fn parse(spec: &str) -> Closure {
        let (roots, dirs) = spec.split_once('@').unwrap_or((spec, "src"));
        Closure {
            roots: roots.split(',').map(str::to_string).collect(),
            dirs: dirs.split(',').map(str::to_string).collect(),
        }
    }

    /// Whether a module is named by one of the roots (not its imports).
    pub fn names(&self, module: &str) -> bool {
        self.roots.iter().any(|r| match r.strip_suffix(".*") {
            Some(prefix) => module
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('.')),
            None => r == module,
        })
    }

    /// The modules of the closure and the file each is read from, in name
    /// order.
    pub fn modules(&self, root: &Path) -> Result<Vec<(String, String)>, Error> {
        let mut stack = Vec::new();
        for r in &self.roots {
            if r.ends_with(".*") {
                return Err(format!(
                    "closure root {r}: a pattern root names no single module to start from"
                ));
            }
            stack.push(r.clone());
        }
        let mut seen = std::collections::BTreeMap::new();
        while let Some(m) = stack.pop() {
            if seen.contains_key(&m) {
                continue;
            }
            let rel = format!("{}.lex.tex", m.replace('.', "/"));
            let (path, text) = self
                .dirs
                .iter()
                .find_map(|d| {
                    let p = format!("{d}/{rel}");
                    std::fs::read_to_string(root.join(&p)).ok().map(|t| (p, t))
                })
                .ok_or_else(|| format!("module {m} is in no directory of {:?}", self.dirs))?;
            for import in text.split("\\importmodule{").skip(1) {
                if let Some(name) = import.split('}').next() {
                    stack.push(name.to_string());
                }
            }
            seen.insert(m, path);
        }
        Ok(seen.into_iter().collect())
    }
}

/// Where the page and the model are read, from the factory graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reach {
    /// Directories whose JavaScript the page ships: the directory inputs of
    /// an obligation that builds the site (`scripts/site.mjs`).
    pub page_dirs: Vec<String>,
    /// The model the shipped client is exported from: the closure inputs of
    /// an obligation that records an export (`record`).
    pub exported: Vec<Closure>,
    /// Modules judged by an imported standard: the roots of the closure
    /// inputs of an obligation that reads `standards/`.
    pub standards: Vec<Closure>,
}

/// Read the reach from the graph's obligations.
pub fn reach(graph: &serde_json::Value) -> Reach {
    let mut out = Reach::default();
    for o in graph["obligations"].as_array().into_iter().flatten() {
        let run = o["run"].as_str().unwrap_or_default();
        let inputs: Vec<&str> = o["inputs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|i| i.as_str())
            .collect();
        let closures = inputs
            .iter()
            .filter_map(|i| i.strip_prefix("closure:"))
            .map(Closure::parse);
        if run.contains("scripts/site.mjs") {
            for i in &inputs {
                let plain = !i.starts_with(['!', '&', '@']) && !i.contains(':');
                if plain && i.ends_with('/') && !i.is_empty() {
                    out.page_dirs.push(i.trim_end_matches('/').to_string());
                }
            }
        }
        if o["record"].as_str().is_some() {
            out.exported.extend(closures.clone());
        }
        if inputs.contains(&"standards/") {
            out.standards.extend(closures);
        }
    }
    out
}

/// The JavaScript files under the page directories. A directory that
/// carries a `MANIFEST.sha256` is vendored and judged at its source.
pub fn page_files(root: &Path, dirs: &[String], scan: &mut Scan) -> Result<Vec<String>, Error> {
    let mut out = Vec::new();
    for d in dirs {
        let dir = root.join(d);
        if !dir.is_dir() {
            continue;
        }
        if dir.join("MANIFEST.sha256").exists() {
            scan.generated
                .push((format!("{d}/"), format!("{d}/MANIFEST.sha256")));
            continue;
        }
        walk(root, &dir, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), Error> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(root, &p, out)?;
        } else if p.extension().is_some_and(|x| x == "js" || x == "mjs") {
            let rel = p.strip_prefix(root).map_err(|e| e.to_string())?;
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn push(scan: &mut Scan, file: &str, item: &str, value: String) {
    if value == "0" || value == "1" {
        return;
    }
    scan.sites.push(Site {
        file: file.to_string(),
        item: item.to_string(),
        value,
        facet: None,
    });
}

/// The base-10 value of a JavaScript or LexLean numeral (`0x10`, `1_000`,
/// `2.5`, `1e3`, `10n`), or its text when it is not an integer.
fn value(text: &str) -> String {
    let t: String = text.chars().filter(|c| *c != '_').collect();
    let t = t.strip_suffix('n').unwrap_or(&t);
    let lower = t.to_ascii_lowercase();
    let radix = [("0x", 16), ("0b", 2), ("0o", 8)]
        .iter()
        .find_map(|(p, r)| lower.strip_prefix(p).map(|d| (d.to_string(), *r)));
    if let Some((digits, r)) = radix {
        if let Ok(v) = u128::from_str_radix(&digits, r) {
            return v.to_string();
        }
    }
    if let Ok(v) = lower.parse::<u128>() {
        return v.to_string();
    }
    if let Ok(f) = lower.parse::<f64>() {
        if f.fract() == 0.0 && (0.0..1e30).contains(&f) {
            return format!("{f:.0}");
        }
    }
    lower
}

/// Scan a JavaScript file's numerals. Comments, strings, template text and
/// regular expressions are skipped; a numeral's item is the top-level
/// declaration (`function`, `const`, `let`, `var`, `class`) it sits in.
pub fn scan_js(file: &str, text: &str, scan: &mut Scan) {
    scan.files += 1;
    let b: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut depth: i64 = 0;
    let mut item = String::from("(file)");
    let mut expect_name = false;
    // Whether a `/` here starts a regular expression: after an operator or
    // an opening bracket, not after a value.
    let mut regex_ok = true;
    let mut templates: Vec<i64> = Vec::new();
    while i < b.len() {
        let c = b[i];
        let next = b.get(i + 1).copied().unwrap_or('\0');
        if c == '/' && next == '/' {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && next == '*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        if c == '"' || c == '\'' {
            i += 1;
            while i < b.len() && b[i] != c {
                if b[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            regex_ok = false;
            continue;
        }
        if c == '`' || (c == '}' && templates.last() == Some(&depth)) {
            if c == '}' {
                templates.pop();
            }
            i += 1;
            // Template text up to the closing backtick or an expression.
            while i < b.len() && b[i] != '`' {
                if b[i] == '\\' {
                    i += 2;
                    continue;
                }
                if b[i] == '$' && b.get(i + 1) == Some(&'{') {
                    templates.push(depth);
                    i += 1;
                    break;
                }
                i += 1;
            }
            i += 1;
            regex_ok = false;
            continue;
        }
        if c == '/' && regex_ok {
            i += 1;
            let mut class = false;
            while i < b.len() && (b[i] != '/' || class) && b[i] != '\n' {
                match b[i] {
                    '\\' => i += 1,
                    '[' => class = true,
                    ']' => class = false,
                    _ => {}
                }
                i += 1;
            }
            i += 1;
            while i < b.len() && b[i].is_ascii_alphabetic() {
                i += 1;
            }
            regex_ok = false;
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && next.is_ascii_digit()) {
            let start = i;
            while i < b.len()
                && (b[i].is_ascii_alphanumeric()
                    || b[i] == '_'
                    || b[i] == '.'
                    || ((b[i] == '+' || b[i] == '-')
                        && matches!(b[i - 1], 'e' | 'E')
                        && !b[start..i].iter().any(|c| matches!(c, 'x' | 'X'))))
            {
                i += 1;
            }
            let t: String = b[start..i].iter().collect();
            push(scan, file, &item, value(&t));
            regex_ok = false;
            continue;
        }
        if c.is_alphabetic() || c == '_' || c == '$' {
            let start = i;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_' || b[i] == '$') {
                i += 1;
            }
            let word: String = b[start..i].iter().collect();
            if expect_name {
                item = word.clone();
                expect_name = false;
            } else if depth == 0
                && matches!(
                    word.as_str(),
                    "function" | "const" | "let" | "var" | "class"
                )
            {
                expect_name = true;
            }
            regex_ok = matches!(
                word.as_str(),
                "return"
                    | "typeof"
                    | "case"
                    | "in"
                    | "of"
                    | "new"
                    | "delete"
                    | "void"
                    | "throw"
                    | "yield"
                    | "await"
            );
            continue;
        }
        match c {
            '{' | '(' | '[' => {
                if c == '{' {
                    depth += 1;
                }
                regex_ok = true;
            }
            '}' => {
                depth -= 1;
                regex_ok = true;
            }
            ')' | ']' => regex_ok = false,
            ';' if depth == 0 => {
                item = String::from("(file)");
                regex_ok = true;
            }
            c if c.is_whitespace() => {}
            '*' if expect_name => {}
            _ => regex_ok = true,
        }
        i += 1;
    }
}

/// Scan a LexLean module's numerals inside its definitions (theorems state
/// what a figure does; they fix none). A parameterless `Nat` definition of
/// a wire module whose body is a numeral is the model naming a format
/// identity, and is not a fixed figure.
pub fn scan_lean(file: &str, module: &str, text: &str, scan: &mut Scan) {
    scan.files += 1;
    let wire = WIRE_MODULES.iter().any(|p| module.starts_with(p));
    for decl in text.split("definition{").skip(1) {
        let decl = decl.split("\n}").next().unwrap_or(decl);
        let name = crate::field(decl, "name")
            .unwrap_or("(definition)")
            .to_string();
        let numeral_body = crate::field(decl, "body").is_some_and(|b| b.parse::<u128>().is_ok());
        if wire
            && numeral_body
            && decl.contains("parameters: [],")
            && crate::field(decl, "result") == Some("Nat")
        {
            continue;
        }
        let b: Vec<char> = decl.chars().collect();
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            if c == '"' {
                i += 1;
                while i < b.len() && b[i] != '"' {
                    if b[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
                continue;
            }
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '\'' {
                let start = i;
                while i < b.len()
                    && (b[i].is_alphanumeric() || b[i] == '_' || b[i] == '.' || b[i] == '\'')
                {
                    i += 1;
                }
                let word: String = b[start..i].iter().collect();
                if word.chars().all(|c| c.is_ascii_digit()) {
                    push(scan, file, &name, value(&word));
                }
                continue;
            }
            i += 1;
        }
    }
}

/// The modules of the exported closures, less those an imported standard
/// judges, each once.
pub fn model_modules(root: &Path, reach: &Reach) -> Result<Vec<(String, String)>, Error> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for c in &reach.exported {
        for (m, p) in c.modules(root)? {
            if reach.standards.iter().any(|s| s.names(&m)) {
                continue;
            }
            if seen.insert(m.clone()) {
                out.push((m, p));
            }
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn js(src: &str) -> Vec<(String, String)> {
        let mut s = Scan::default();
        scan_js("x.js", src, &mut s);
        s.sites.into_iter().map(|s| (s.item, s.value)).collect()
    }

    fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
        v.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn javascript_numerals_outside_comments_strings_and_regexes() {
        let got = js("// 7 in a comment\nconst A = 64; /* 9 */\nfunction f(x) { return x.replace(/a{3}/g, '5') + `t${x + 12}u 13` + 0x10 + 1e3 + 2.5 + 1_000; }\nlet b = x / 4 / 2;");
        assert_eq!(
            got,
            pairs(&[
                ("A", "64"),
                ("f", "12"),
                ("f", "16"),
                ("f", "1000"),
                ("f", "2.5"),
                ("f", "1000"),
                ("b", "4"),
                ("b", "2")
            ])
        );
    }

    #[test]
    fn lean_numerals_inside_definitions_and_wire_names() {
        let src = "definition{name: v, parameters: [], result: Nat, body: 2}\n\ndefinition{\n  name: f,\n  parameters: [x : Nat],\n  result: Nat,\n  body: (prim add x 17 : Nat)\n}\n\ntheorem{\n  name: t,\n  statement: (f 3 = 20)\n}\n";
        let mut s = Scan::default();
        scan_lean("w.lex.tex", "Uorc.Wire.Frame", src, &mut s);
        let got: Vec<_> = s
            .sites
            .iter()
            .map(|s| (s.item.as_str(), s.value.as_str()))
            .collect();
        assert_eq!(got, [("f", "17")]);
        let mut s = Scan::default();
        scan_lean("p.lex.tex", "Uorc.Performance", src, &mut s);
        assert_eq!(
            s.sites.len(),
            2,
            "outside a wire module a named numeral is a figure"
        );
    }

    #[test]
    fn closure_inputs_parse_and_name_patterns() {
        let c = Closure::parse("Uorc.Integrity.*");
        assert_eq!(c.dirs, ["src"]);
        assert!(c.names("Uorc.Integrity.Sha256"));
        assert!(!c.names("Uorc.IntegrityX"));
        let c = Closure::parse("Uorc.Client@client/src,src");
        assert_eq!(c.dirs, ["client/src", "src"]);
        assert!(c.names("Uorc.Client") && !c.names("Uorc.Core.Types"));
    }

    #[test]
    fn the_graph_names_the_page_the_export_and_the_standards() {
        let g: serde_json::Value = serde_json::from_str(r#"{"obligations":[
            {"run":"node scripts/site.mjs","inputs":["&client","web/","scripts/site.mjs","closure:X"]},
            {"run":"export","record":"generated/rust/id","inputs":["closure:Uorc.Client@client/src,src"]},
            {"run":"just integrity","inputs":["closure:Uorc.Integrity.*","standards/"]}]}"#).unwrap();
        let r = reach(&g);
        assert_eq!(r.page_dirs, ["web"]);
        assert_eq!(r.exported, [Closure::parse("Uorc.Client@client/src,src")]);
        assert_eq!(r.standards, [Closure::parse("Uorc.Integrity.*")]);
    }
}
