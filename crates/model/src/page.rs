//! The Prism product model of the Pages client: `src/Uorc/Page.lex.tex`.
//!
//! The module states the page in Prism's product-modeling vocabulary (PrismPM
//! `stdlib/src/Foundation/Arch.lex.tex` stakeholders, features, journeys and
//! UML state machines; `Foundation/Qual.lex.tex` quality requirements and
//! measures; `Foundation/View/Text/V3/Model.lex.tex` workbench operations,
//! fields, examples, error hints, sections, links and labels). The records
//! are declared locally because the page works on files, which the text/3
//! workbench profile does not carry; each keeps the field names of its Prism
//! counterpart.
//!
//! [`PageModel::load`] reads every row, checks the model the way PrismPM checks
//! its own (PP2014 to PP2018 for features and journeys, one initial state and
//! closed references for the state machine, plus the page's own closed sets),
//! and [`PageModel::render`] generates `web/index.html`: the page's static
//! content and, embedded as JSON, the model `web/app.js` interprets. The page
//! therefore holds exactly what the model declares.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde_json::{Map, Value};

use crate::ModelError;

/// The LexLean module that owns the page's product model.
pub const PAGE_MODEL_PATH: &str = "src/Uorc/Page.lex.tex";
/// The generated page.
pub const PAGE_PATH: &str = "web/index.html";
const MODULE: &str = "Uorc.Page";

/// Every row type of the model, in the order the JSON projection lists them.
const ROW_TYPES: &[&str] = &[
    "Workbench",
    "Labels",
    "Stakeholder",
    "Feature",
    "Journey",
    "UmlState",
    "UmlTransition",
    "Operation",
    "Field",
    "Control",
    "Example",
    "Column",
    "ErrorHint",
    "Section",
    "Usage",
    "Link",
    "QualityRequirement",
    "QualityMeasure",
    "DecodeHost",
];

/// The triggers `web/app.js` raises; a transition names one of them.
pub const TRIGGERS: &[&str] = &[
    "loaded",
    "loadFailed",
    "run",
    "finished",
    "failed",
    "cancel",
    "restarted",
    "edit",
];
/// The field kinds `web/app.js` renders.
pub const FIELD_KINDS: &[&str] = &["files", "folder", "archive", "file"];
/// The example inputs `web/app.js` generates.
pub const EXAMPLE_INPUTS: &[&str] = &["text", "zeros", "counter", "random", "empty"];
/// The operations the page's worker runs.
pub const OPERATIONS: &[&str] = &["compress", "decompress", "inspect", "verify"];
/// The controls `web/app.js` enables by state.
pub const CONTROLS: &[&str] = &["run", "cancel", "operation", "field", "example"];
/// The error codes the page's worker reports: the wasm client's four, a
/// stopped worker and a client that cannot load.
pub const ERROR_CODES: &[&str] = &[
    "budget", "overflow", "decode", "mismatch", "stopped", "load",
];
/// The tables the page draws.
pub const TABLES: &[&str] = &["inspect", "examples", "theorems", "claims", "weights"];
/// The tables of the model view, each filled from the release's `model.json`,
/// with the label of its heading.
const MODEL_TABLES: &[(&str, &str)] = &[
    ("theorems", "theorems"),
    ("claims", "claims"),
    ("weights", "engine"),
];

/// One row: the definition's name and its record's fields as JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The definition name; `Uorc.Page::<name>` is its identifier.
    pub name: String,
    /// The record fields.
    pub fields: Map<String, Value>,
}

impl Row {
    fn str(&self, field: &str) -> &str {
        self.fields.get(field).and_then(Value::as_str).unwrap_or("")
    }
    fn strs(&self, field: &str) -> Vec<&str> {
        self.fields
            .get(field)
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }
    fn nat(&self, field: &str) -> u64 {
        self.fields.get(field).and_then(Value::as_u64).unwrap_or(0)
    }
    fn bool(&self, field: &str) -> bool {
        self.fields
            .get(field)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
}

/// The page's product model: every row, by type, in declaration order.
#[derive(Debug, Clone, PartialEq)]
pub struct PageModel {
    /// Rows by record type.
    pub rows: BTreeMap<String, Vec<Row>>,
}

fn inconsistent(message: impl Into<String>) -> ModelError {
    ModelError::Inconsistent(format!("{PAGE_MODEL_PATH}: {}", message.into()))
}

/// A record field value as JSON: strings, naturals, Booleans and lists of them.
fn value(data: &Value) -> Result<Value, ModelError> {
    match data.get("kind").and_then(Value::as_str) {
        Some("string") => Ok(data.get("value").cloned().unwrap_or(Value::Null)),
        Some("nat") => data
            .get("value")
            .and_then(Value::as_str)
            .and_then(|n| n.parse::<u64>().ok())
            .map(Value::from)
            .ok_or_else(|| inconsistent("a natural number does not fit in 64 bits")),
        Some("bool") => Ok(data.get("value").cloned().unwrap_or(Value::Null)),
        Some("cons" | "nil") => {
            let mut items = Vec::new();
            let mut at = data;
            while at.get("kind").and_then(Value::as_str) == Some("cons") {
                items.push(value(at.get("head").unwrap_or(&Value::Null))?);
                at = at.get("tail").unwrap_or(&Value::Null);
            }
            Ok(Value::Array(items))
        }
        _ => Err(inconsistent(format!(
            "a field value must be a string, natural, Boolean or list literal, not {data}"
        ))),
    }
}

impl PageModel {
    /// Read and check the model.
    pub fn load(root: &Path) -> Result<Self, ModelError> {
        let declarations =
            crate::authority::semantic_declarations(&root.join(PAGE_MODEL_PATH), MODULE)?;
        let mut rows: BTreeMap<String, Vec<Row>> = BTreeMap::new();
        for declaration in &declarations {
            if declaration.get("kind").and_then(Value::as_str) != Some("definition") {
                continue;
            }
            let name = declaration
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let body = declaration.get("body").unwrap_or(&Value::Null);
            let kind = body
                .get("type")
                .and_then(|t| t.get("name"))
                .and_then(Value::as_str)
                .filter(|_| body.get("kind").and_then(Value::as_str) == Some("record"))
                .ok_or_else(|| inconsistent(format!("`{name}` is not a closed record literal")))?;
            if !ROW_TYPES.contains(&kind) {
                return Err(inconsistent(format!(
                    "`{name}` has unknown row type {kind}"
                )));
            }
            let mut fields = Map::new();
            for field in body
                .get("fields")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let key = field
                    .get("field")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                fields.insert(
                    key.to_string(),
                    value(field.get("value").unwrap_or(&Value::Null))?,
                );
            }
            rows.entry(kind.to_string())
                .or_default()
                .push(Row { name, fields });
        }
        let model = PageModel { rows };
        model.check()?;
        Ok(model)
    }

    /// Rows of one type, in declaration order.
    #[must_use]
    pub fn of(&self, kind: &str) -> &[Row] {
        self.rows.get(kind).map_or(&[], Vec::as_slice)
    }

    fn sorted(&self, kind: &str) -> Vec<&Row> {
        let mut rows: Vec<&Row> = self.of(kind).iter().collect();
        rows.sort_by_key(|row| row.nat("order"));
        rows
    }

    fn one(&self, kind: &str) -> &Row {
        &self.of(kind)[0]
    }

    /// The Prism checks: every feature is realized and accepted, journeys name
    /// stakeholders and features, the state machine is closed and reachable,
    /// and every reference and closed set holds.
    pub fn check(&self) -> Result<(), ModelError> {
        for kind in ROW_TYPES {
            if self.of(kind).is_empty() {
                return Err(inconsistent(format!("the model declares no {kind}")));
            }
        }
        for kind in ["Workbench", "Labels"] {
            if self.of(kind).len() != 1 {
                return Err(inconsistent(format!(
                    "the model declares exactly one {kind}"
                )));
            }
        }
        let id = |row: &Row| format!("{MODULE}::{}", row.name);
        let ids = |kind: &str| -> BTreeSet<String> { self.of(kind).iter().map(id).collect() };
        let unique = |kind: &str, field: &str| -> Result<BTreeSet<String>, ModelError> {
            let mut seen = BTreeSet::new();
            for row in self.of(kind) {
                let key = row.str(field).to_string();
                if key.trim().is_empty() || !seen.insert(key.clone()) {
                    return Err(inconsistent(format!("{kind} `{key}` is blank or repeated")));
                }
            }
            Ok(seen)
        };

        // PP2014, PP2015: every feature is realized and accepted, once each.
        let mut acceptance = BTreeSet::new();
        for feature in self.of("Feature") {
            if feature.strs("calls").is_empty() {
                return Err(inconsistent(format!(
                    "PP2014 feature `{}` is realized by no call",
                    feature.name
                )));
            }
            if feature.strs("acceptance").is_empty() {
                return Err(inconsistent(format!(
                    "PP2015 feature `{}` is accepted by no row",
                    feature.name
                )));
            }
            for case in feature.strs("acceptance") {
                if !acceptance.insert(case.to_string()) {
                    return Err(inconsistent(format!(
                        "acceptance case `{case}` is claimed twice"
                    )));
                }
            }
        }
        // PP2016, PP2017: journeys name stakeholders and features; every
        // feature is reached by a journey, which is how acceptance runs it.
        let (features, stakeholders) = (ids("Feature"), ids("Stakeholder"));
        let mut reached = BTreeSet::new();
        for journey in self.of("Journey") {
            if !stakeholders.contains(journey.str("actor")) {
                return Err(inconsistent(format!(
                    "PP2017 journey `{}` has no declared actor",
                    journey.name
                )));
            }
            for step in journey.strs("steps") {
                if !features.contains(step) {
                    return Err(inconsistent(format!(
                        "PP2016 journey `{}` step `{step}` is not a feature",
                        journey.name
                    )));
                }
                reached.insert(step.to_string());
            }
        }
        if let Some(missed) = features.difference(&reached).next() {
            return Err(inconsistent(format!("feature `{missed}` is on no journey")));
        }

        // The state machine: one initial state, closed triggers, every state
        // reachable from the initial one.
        let states = unique("UmlState", "state")?;
        let initial: Vec<&Row> = self
            .of("UmlState")
            .iter()
            .filter(|s| s.bool("initial"))
            .collect();
        if initial.len() != 1 {
            return Err(inconsistent(
                "the state machine needs exactly one initial state",
            ));
        }
        let mut orders = BTreeSet::new();
        let mut pairs = BTreeSet::new();
        for t in self.of("UmlTransition") {
            let (source, target, trigger) = (t.str("source"), t.str("target"), t.str("trigger"));
            if !states.contains(source) || !states.contains(target) {
                return Err(inconsistent(format!(
                    "transition `{}` names an undeclared state",
                    t.name
                )));
            }
            if !TRIGGERS.contains(&trigger) {
                return Err(inconsistent(format!(
                    "transition `{}` has unknown trigger {trigger}",
                    t.name
                )));
            }
            if !orders.insert(t.nat("order"))
                || !pairs.insert((source.to_string(), trigger.to_string()))
            {
                return Err(inconsistent(format!(
                    "transition `{}` repeats an order or a source and trigger",
                    t.name
                )));
            }
        }
        let mut seen = BTreeSet::from([initial[0].str("state").to_string()]);
        loop {
            let next: BTreeSet<String> = self
                .of("UmlTransition")
                .iter()
                .filter(|t| seen.contains(t.str("source")))
                .map(|t| t.str("target").to_string())
                .collect();
            if next.is_subset(&seen) {
                break;
            }
            seen.extend(next);
        }
        if seen != states {
            return Err(inconsistent(
                "a state is unreachable from the initial state",
            ));
        }

        // The workbench: operations, fields, controls, examples, columns,
        // errors and links each use closed sets and resolve their references.
        let fields = unique("Field", "id")?;
        for field in self.of("Field") {
            if !FIELD_KINDS.contains(&field.str("kind")) {
                return Err(inconsistent(format!(
                    "field `{}` has unknown kind",
                    field.str("id")
                )));
            }
            let replaces = field.str("replaces");
            if !replaces.is_empty() && !fields.contains(replaces) {
                return Err(inconsistent(format!(
                    "field `{}` replaces an undeclared field",
                    field.str("id")
                )));
            }
        }
        let operations = unique("Operation", "id")?;
        if operations != OPERATIONS.iter().map(ToString::to_string).collect() {
            return Err(inconsistent(format!(
                "the operations must be exactly {OPERATIONS:?}"
            )));
        }
        let mut used = BTreeSet::new();
        for op in self.of("Operation") {
            if op.strs("fields").is_empty() {
                return Err(inconsistent(format!(
                    "operation `{}` has no field",
                    op.str("id")
                )));
            }
            for field in op.strs("fields") {
                if !fields.contains(field) {
                    return Err(inconsistent(format!(
                        "operation `{}` names undeclared field `{field}`",
                        op.str("id")
                    )));
                }
                used.insert(field.to_string());
            }
        }
        if used != fields {
            return Err(inconsistent("a field belongs to no operation"));
        }
        if !operations.contains(self.one("Workbench").str("initialOperation")) {
            return Err(inconsistent("the initial operation is not declared"));
        }
        if unique("Control", "id")? != CONTROLS.iter().map(ToString::to_string).collect() {
            return Err(inconsistent(format!(
                "the controls must be exactly {CONTROLS:?}"
            )));
        }
        for control in self.of("Control") {
            if !TRIGGERS.contains(&control.str("trigger")) {
                return Err(inconsistent(format!(
                    "control `{}` raises an unknown trigger",
                    control.str("id")
                )));
            }
            for state in control.strs("enabledIn") {
                if !states.contains(state) {
                    return Err(inconsistent(format!(
                        "control `{}` names undeclared state `{state}`",
                        control.str("id")
                    )));
                }
            }
        }
        unique("Example", "id")?;
        for example in self.of("Example") {
            if !EXAMPLE_INPUTS.contains(&example.str("input")) {
                return Err(inconsistent(format!(
                    "example `{}` has unknown input",
                    example.str("id")
                )));
            }
        }
        for column in self.of("Column") {
            if !TABLES.contains(&column.str("table")) {
                return Err(inconsistent(format!(
                    "column `{}` is in an unknown table",
                    column.name
                )));
            }
        }
        for table in TABLES {
            if !self.of("Column").iter().any(|c| c.str("table") == *table) {
                return Err(inconsistent(format!("table `{table}` has no column")));
            }
        }
        let codes = unique("ErrorHint", "code")?;
        if codes != ERROR_CODES.iter().map(ToString::to_string).collect() {
            return Err(inconsistent(format!(
                "the error hints must cover exactly {ERROR_CODES:?}"
            )));
        }
        for hint in self.of("ErrorHint") {
            for field in hint.strs("fields") {
                if !fields.contains(field) {
                    return Err(inconsistent(format!(
                        "error `{}` names undeclared field `{field}`",
                        hint.str("code")
                    )));
                }
            }
        }
        for link in self.of("Link") {
            let href = link.str("href");
            let local = !href.contains(':') && !href.contains('/') && !href.is_empty();
            if !(href.starts_with("https://github.com/") || local) {
                return Err(inconsistent(format!(
                    "link `{href}` is neither a GitHub page nor a site file"
                )));
            }
        }
        let requirements = ids("QualityRequirement");
        for measure in self.of("QualityMeasure") {
            if !requirements.contains(measure.str("requirement")) {
                return Err(inconsistent(format!(
                    "measure `{}` names no requirement",
                    measure.name
                )));
            }
            if !["gate", "measured"].contains(&measure.str("disposition")) {
                return Err(inconsistent(format!(
                    "measure `{}` is neither a gate nor measured",
                    measure.name
                )));
            }
        }
        // One host rule, and the memory gate is its worker's share.
        let hosts = self.of("DecodeHost");
        let [host] = hosts else {
            return Err(inconsistent("the page states one DecodeHost".to_string()));
        };
        let per_gib = host.nat("workersPerGiB");
        if per_gib == 0 || host.nat("coreLimit") == 0 || host.nat("unreportedGiB") == 0 {
            return Err(inconsistent("the page's host rule starts no worker".to_string()));
        }
        let share = 1024 / per_gib;
        for measure in self.of("QualityMeasure") {
            if measure.name == "workerMemory" && measure.nat("threshold") != share {
                return Err(inconsistent(format!(
                    "workerMemory gates {} MiB, the host rule gives a worker {share} MiB",
                    measure.nat("threshold")
                )));
            }
        }
        Ok(())
    }

    /// The model as JSON, by row type, each row with its `name`.
    #[must_use]
    pub fn json(&self) -> Value {
        let mut out = Map::new();
        for kind in ROW_TYPES {
            let rows = self
                .of(kind)
                .iter()
                .map(|row| {
                    let mut fields = Map::new();
                    fields.insert("name".to_string(), Value::from(row.name.clone()));
                    fields.extend(row.fields.clone());
                    Value::Object(fields)
                })
                .collect();
            out.insert((*kind).to_string(), Value::Array(rows));
        }
        Value::Object(out)
    }

    /// The generated page.
    #[must_use]
    pub fn render(&self) -> String {
        let wb = self.one("Workbench");
        let labels = self.one("Labels");
        let initial_state = self
            .of("UmlState")
            .iter()
            .find(|s| s.bool("initial"))
            .expect("checked: one initial state");
        let mut h = String::new();
        let _ = write!(
            h,
            "<!doctype html>\n<!-- @generated from {PAGE_MODEL_PATH}, the Prism product model of this page; do not edit. -->\n\
             <html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
             <title>{}</title>\n<meta name=\"description\" content=\"{}\">\n\
             <link rel=\"stylesheet\" href=\"app.css\">\n\
             <link rel=\"manifest\" href=\"manifest.webmanifest\"><link rel=\"icon\" href=\"icon.svg\" type=\"image/svg+xml\"><link rel=\"apple-touch-icon\" href=\"icon-512.png\">\
             <meta name=\"theme-color\" content=\"#f3f3ee\" media=\"(prefers-color-scheme: light)\"><meta name=\"theme-color\" content=\"#151312\" media=\"(prefers-color-scheme: dark)\">\n\
             </head>\n<body data-state=\"{}\" data-color-scheme=\"{}\" data-density=\"{}\" data-width=\"{}\">\n",
            esc(wb.str("title")),
            esc(wb.str("purpose")),
            esc(initial_state.str("state")),
            esc(wb.str("colorScheme")),
            esc(wb.str("density")),
            esc(wb.str("width")),
        );
        let _ = write!(
            h,
            "<a class=\"skip\" href=\"#workbench\">{}</a>\n<header>\n<h1>{}</h1>\n<p class=\"purpose\">{}</p>\n\
             <p class=\"privacy\">{}</p>\n<p class=\"version\" id=\"version\" data-label=\"{}\"></p>\n</header>\n<main>\n",
            esc(labels.str("skipToWorkbench")),
            esc(wb.str("heading")),
            esc(wb.str("purpose")),
            esc(wb.str("privacy")),
            esc(labels.str("version")),
        );

        // The workbench.
        let _ = write!(
            h,
            "<section id=\"workbench\" aria-labelledby=\"workbench-title\">\n<h2 id=\"workbench-title\">{}</h2>\n<ol class=\"guidance\">\n",
            esc(labels.str("workbench"))
        );
        for step in wb.strs("guidance") {
            let _ = writeln!(h, "<li>{}</li>", esc(step));
        }
        let _ = write!(
            h,
            "</ol>\n<noscript><p class=\"noscript\">{}</p></noscript>\n<div class=\"tabs\" role=\"tablist\" aria-label=\"{}\">\n",
            esc(labels.str("noscript")),
            esc(labels.str("operations"))
        );
        let initial = wb.str("initialOperation");
        for op in self.sorted("Operation") {
            let on = op.str("id") == initial;
            let _ = writeln!(
                h,
                "<button type=\"button\" role=\"tab\" id=\"tab-{0}\" data-operation=\"{0}\" aria-controls=\"panel\" aria-selected=\"{1}\" tabindex=\"{2}\" disabled>{3}</button>",
                esc(op.str("id")),
                on,
                if on { "0" } else { "-1" },
                esc(op.str("label"))
            );
        }
        let _ = write!(
            h,
            "</div>\n<div id=\"panel\" role=\"tabpanel\" aria-labelledby=\"tab-{}\">\n",
            esc(initial)
        );
        for op in self.sorted("Operation") {
            let _ = writeln!(
                h,
                "<div class=\"operation\" data-operation=\"{}\"{}><p class=\"summary\">{}</p><p class=\"help\">{}</p></div>",
                esc(op.str("id")),
                if op.str("id") == initial { "" } else { " hidden" },
                esc(op.str("summary")),
                esc(op.str("help"))
            );
        }
        let initial_fields: BTreeSet<&str> = self
            .of("Operation")
            .iter()
            .find(|op| op.str("id") == initial)
            .map(|op| op.strs("fields").into_iter().collect())
            .unwrap_or_default();
        for field in self.of("Field") {
            let id = field.str("id");
            let attrs = match field.str("kind") {
                "files" => " multiple",
                "folder" => " webkitdirectory",
                "archive" => " accept=\".uorc,application/octet-stream\"",
                _ => "",
            };
            let _ = writeln!(
                h,
                "<div class=\"field\" data-field=\"{0}\"{1}><label for=\"field-{0}\">{2}</label>\
                 <input id=\"field-{0}\" type=\"file\"{3} aria-describedby=\"help-{0}\" disabled>\
                 <p class=\"help\" id=\"help-{0}\">{4}</p><p class=\"chosen\" id=\"chosen-{0}\"></p></div>",
                esc(id),
                if initial_fields.contains(id) { "" } else { " hidden" },
                esc(field.str("label")),
                attrs,
                esc(field.str("help"))
            );
        }
        let run_label = self
            .of("Operation")
            .iter()
            .find(|op| op.str("id") == initial)
            .map_or("", |op| op.str("runLabel"));
        let cancel = self
            .of("Control")
            .iter()
            .find(|c| c.str("id") == "cancel")
            .map_or("", |c| c.str("label"));
        let _ = write!(
            h,
            "<div class=\"actions\"><button type=\"button\" id=\"run\" disabled>{}</button>\
             <button type=\"button\" id=\"cancel\" class=\"secondary\" disabled>{}</button></div>\n\
             <progress id=\"progress\" aria-label=\"{}\" max=\"1\" value=\"0\" hidden></progress>\n\
             <p id=\"status\" role=\"status\" aria-live=\"polite\">{}</p>\n\
             <div id=\"error\" class=\"error\" hidden><p class=\"error-title\" id=\"error-title\"></p>\
             <p id=\"error-explanation\"></p><p id=\"error-recovery\"></p></div>\n\
             <div id=\"result\" hidden></div>\n</div>\n</section>\n",
            esc(run_label),
            esc(cancel),
            esc(labels.str("progress")),
            esc(initial_state.str("status"))
        );

        // Examples.
        let _ = write!(
            h,
            "<section id=\"examples\" aria-labelledby=\"examples-title\">\n<h2 id=\"examples-title\">{}</h2>\n<p>{}</p>\n<div class=\"buttons\">\n",
            esc(labels.str("examples")),
            esc(labels.str("examplesHelp"))
        );
        for example in self.sorted("Example") {
            let _ = writeln!(
                h,
                "<button type=\"button\" class=\"secondary\" data-example=\"{}\" aria-describedby=\"example-{0}\" disabled>{}</button>",
                esc(example.str("id")),
                esc(example.str("label"))
            );
        }
        h.push_str("</div>\n<ul class=\"example-notes\">\n");
        for example in self.sorted("Example") {
            let _ = writeln!(
                h,
                "<li id=\"example-{}\"><strong>{}</strong>: {} ({} bytes)</li>",
                esc(example.str("id")),
                esc(example.str("label")),
                esc(example.str("description")),
                example.nat("bytes")
            );
        }
        h.push_str("</ul>\n<div class=\"table-wrap\"><table id=\"example-results\"><thead><tr>");
        for column in self
            .sorted("Column")
            .into_iter()
            .filter(|c| c.str("table") == "examples")
        {
            let _ = write!(h, "<th scope=\"col\">{}</th>", esc(column.str("label")));
        }
        h.push_str("</tr></thead><tbody></tbody></table></div>\n</section>\n");

        // What the codec claims: the release's theorems, closed claims and
        // engine state, which app.js reads from the published model.json.
        let _ = write!(
            h,
            "<section id=\"model\" aria-labelledby=\"model-title\" data-empty=\"{}\">\n<h2 id=\"model-title\">{}</h2>\n<p>{}</p>\n",
            esc(labels.str("unpublished")),
            esc(labels.str("model")),
            esc(labels.str("modelHelp"))
        );
        for (table, heading) in MODEL_TABLES {
            let _ = writeln!(
                h,
                "<h3 id=\"model-{table}-title\">{}</h3>",
                esc(labels.str(heading))
            );
            if *table == "weights" {
                h.push_str("<p id=\"model-engine\"></p>\n");
            }
            let _ = write!(
                h,
                "<div class=\"table-wrap\"><table id=\"model-{table}\" aria-labelledby=\"model-{table}-title\"><thead><tr>"
            );
            for column in self
                .sorted("Column")
                .into_iter()
                .filter(|c| c.str("table") == *table)
            {
                let _ = write!(h, "<th scope=\"col\">{}</th>", esc(column.str("label")));
            }
            h.push_str("</tr></thead><tbody></tbody></table></div>\n");
        }
        h.push_str("</section>\n");

        // What the page does, and usage elsewhere.
        for section in self.sorted("Section") {
            let _ = write!(
                h,
                "<section id=\"{0}\" aria-labelledby=\"{0}-title\">\n<h2 id=\"{0}-title\">{1}</h2>\n",
                esc(section.str("id")),
                esc(section.str("title"))
            );
            for paragraph in section.strs("paragraphs") {
                let _ = writeln!(h, "<p>{}</p>", esc(paragraph));
            }
            h.push_str("</section>\n");
        }
        let _ = write!(
            h,
            "<section id=\"usage\" aria-labelledby=\"usage-title\">\n<h2 id=\"usage-title\">{}</h2>\n",
            esc(labels.str("usage"))
        );
        for usage in self.sorted("Usage") {
            let _ = write!(
                h,
                "<h3>{}</h3>\n<pre><code class=\"language-{}\">{}</code></pre>\n",
                esc(usage.str("title")),
                esc(usage.str("language")),
                esc(&usage.strs("code").join("\n"))
            );
        }
        let _ = write!(
            h,
            "</section>\n<section id=\"links\" aria-labelledby=\"links-title\">\n<h2 id=\"links-title\">{}</h2>\n<ul>\n",
            esc(labels.str("links"))
        );
        for link in self.sorted("Link") {
            let _ = writeln!(
                h,
                "<li><a href=\"{}\">{}</a></li>",
                esc(link.str("href")),
                esc(link.str("label"))
            );
        }
        h.push_str("</ul>\n</section>\n</main>\n");
        let json = serde_json::to_string(&self.json())
            .expect("serializable page model")
            .replace("</", "<\\/");
        let _ = write!(
            h,
            "<script type=\"application/json\" id=\"page-model\">{json}</script>\n\
             <script type=\"module\" src=\"app.js\"></script>\n</body>\n</html>\n"
        );
        h
    }
}

/// HTML text and attribute escaping.
fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> std::path::PathBuf {
        crate::repo_root()
    }

    /// The model loads, passes the Prism checks, and the generated page holds
    /// every operation, field, example, error hint, usage block and link.
    #[test]
    fn the_page_model_checks_and_renders_every_element_uc_web_01() {
        let model = PageModel::load(&root()).expect("the page model checks");
        let page = model.render();
        for op in model.of("Operation") {
            assert!(page.contains(&format!("id=\"tab-{}\"", op.str("id"))));
        }
        for field in model.of("Field") {
            assert!(page.contains(&format!("id=\"field-{}\"", field.str("id"))));
        }
        for example in model.of("Example") {
            assert!(page.contains(&format!("data-example=\"{}\"", example.str("id"))));
        }
        for link in model.of("Link") {
            assert!(page.contains(&format!("href=\"{}\"", esc(link.str("href")))));
        }
        assert_eq!(page.matches("role=\"status\"").count(), 1);
        assert!(page.contains("id=\"page-model\""));
    }

    /// The model view: one table per model table with the model's columns,
    /// the engine line, and the inspect table carries the codec point columns.
    #[test]
    fn the_model_view_holds_every_model_table_uc_web_02() {
        let model = PageModel::load(&root()).expect("the page model checks");
        let page = model.render();
        assert!(page.contains("<section id=\"model\""));
        assert!(page.contains("id=\"model-engine\""));
        for (table, _) in MODEL_TABLES {
            assert!(page.contains(&format!("id=\"model-{table}\"")));
            for column in model
                .of("Column")
                .iter()
                .filter(|c| c.str("table") == *table)
            {
                assert!(page.contains(&format!(
                    "<th scope=\"col\">{}</th>",
                    esc(column.str("label"))
                )));
            }
        }
        let keys: BTreeSet<&str> = model
            .of("Column")
            .iter()
            .filter(|c| c.str("table") == "inspect")
            .map(|c| c.str("key"))
            .collect();
        for key in [
            "codec_point",
            "steps",
            "state",
            "lanes",
            "ncm_profile",
            "claims",
        ] {
            assert!(keys.contains(key), "inspect shows no {key}");
        }
    }

    /// Each Prism check rejects its planted defect.
    #[test]
    fn planted_model_defects_are_rejected_uc_web_01() {
        let good = PageModel::load(&root()).expect("the page model checks");
        type Plant = fn(&mut PageModel);
        fn set(m: &mut PageModel, kind: &str, i: usize, field: &str, v: Value) {
            m.rows.get_mut(kind).expect("rows")[i]
                .fields
                .insert(field.to_string(), v);
        }
        let plants: Vec<(&str, Plant)> = vec![
            ("PP2014", |m| {
                set(m, "Feature", 0, "calls", Value::Array(vec![]))
            }),
            ("PP2015", |m| {
                set(m, "Feature", 0, "acceptance", Value::Array(vec![]))
            }),
            ("PP2016", |m| {
                set(
                    m,
                    "Journey",
                    0,
                    "steps",
                    serde_json::json!(["Uorc.Page::nothing"]),
                )
            }),
            ("PP2017", |m| {
                set(m, "Journey", 0, "actor", Value::from("Uorc.Page::nobody"))
            }),
            ("two initial states", |m| {
                set(m, "UmlState", 1, "initial", Value::from(true))
            }),
            ("unknown trigger", |m| {
                set(m, "UmlTransition", 0, "trigger", Value::from("teleport"))
            }),
            ("unreachable state", |m| {
                m.rows
                    .get_mut("UmlTransition")
                    .expect("rows")
                    .retain(|t| t.str("target") != "unavailable");
            }),
            ("unknown field", |m| {
                set(m, "Operation", 0, "fields", serde_json::json!(["nowhere"]))
            }),
            ("uncovered error", |m| {
                m.rows.get_mut("ErrorHint").expect("rows").pop();
            }),
            ("table without columns", |m| {
                m.rows
                    .get_mut("Column")
                    .expect("rows")
                    .retain(|c| c.str("table") != "weights");
            }),
            ("remote link", |m| {
                set(m, "Link", 0, "href", Value::from("https://example.com/"))
            }),
            ("orphan measure", |m| {
                set(
                    m,
                    "QualityMeasure",
                    0,
                    "requirement",
                    Value::from("Uorc.Page::none"),
                )
            }),
        ];
        for (label, plant) in plants {
            let mut bad = good.clone();
            plant(&mut bad);
            assert!(bad.check().is_err(), "{label} must be rejected");
        }
    }
}
