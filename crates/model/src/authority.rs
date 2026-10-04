//! The UORC LexLean authority graph.
//!
//! `src/Uorc/Specification.lex.tex` owns the project charter and evidence
//! boundaries. `src/Uorc/Registry.lex.tex` owns the project claim register.
//! `model/*.toml` and project Markdown are generated projections of this graph,
//! never a second source of UORC semantics.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::{Authorities, IdRow, Ids, Ledger, Level, Model, ModelError};

/// The LexLean module that owns the product charter.
pub const SPECIFICATION_PATH: &str = "src/Uorc/Specification.lex.tex";
/// The LexLean module that owns the claim register.
pub const REGISTRY_PATH: &str = "src/Uorc/Registry.lex.tex";

/// Product scope and source-ownership policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charter {
    /// Human-readable product name.
    pub product_name: String,
    /// Product purpose.
    pub purpose: String,
    /// Which source graph is authoritative.
    pub authority_source: String,
    /// Version-one scope.
    pub scope: String,
    /// Explicit non-scope.
    pub non_scope: String,
    /// Authority precedence.
    pub precedence: String,
    /// Source-ownership boundary.
    pub source_ownership: String,
}

/// The five claim classes that must never be conflated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimBoundaries {
    /// Complete implementation acceptance.
    pub implementation_acceptance: String,
    /// Formal losslessness.
    pub losslessness: String,
    /// Scoped minimum certification.
    pub scoped_minimum: String,
    /// Measured benchmark improvement.
    pub benchmark_improvement: String,
    /// External record acceptance.
    pub external_record_acceptance: String,
}

/// Production, validation, measurement, and open-research boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceBoundaries {
    /// Dependencies permitted in shipped execution.
    pub production_dependencies: String,
    /// Validation-only imported authorities.
    pub validation_authorities: String,
    /// Empirical measurement evidence.
    pub measurements: String,
    /// Claims which remain open research questions.
    pub open_research_claims: String,
}

/// Prior-art and research-hypothesis positioning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchPosition {
    /// Relevant prior-art boundary.
    pub prior_art: String,
    /// The research hypothesis under test.
    pub hypothesis: String,
    /// What is and is not claimed as novel.
    pub novelty_status: String,
    /// What is and is not claimed about compression performance.
    pub performance_status: String,
}

/// The complete project-owned authority graph needed by the repository model.
#[derive(Debug, Clone)]
pub struct AuthorityGraph {
    /// Product charter.
    pub charter: Charter,
    /// Claim-class distinctions.
    pub claim_boundaries: ClaimBoundaries,
    /// Trust/evidence distinctions.
    pub evidence_boundaries: EvidenceBoundaries,
    /// Prior-art/research positioning.
    pub research_position: ResearchPosition,
    /// Conformance IDs authored in `Registry.lex.tex`.
    pub ids: Ids,
    /// Imported authorities represented by this authority slice.
    pub authorities: Authorities,
    /// Non-ID ledger claims represented by this authority slice.
    pub ledger: Ledger,
}

impl AuthorityGraph {
    /// Load and structurally validate the two LexLean authority modules.
    pub fn load(root: &Path) -> Result<Self, ModelError> {
        let specification =
            semantic_declarations(&root.join(SPECIFICATION_PATH), "Uorc.Specification")?;
        let registry = semantic_declarations(&root.join(REGISTRY_PATH), "Uorc.Registry")?;

        validate_specification_declarations(&specification)?;
        validate_registry_declarations(&registry)?;

        let charter = record_strings(
            &specification,
            "charter",
            "Charter",
            &[
                "productName",
                "purpose",
                "authoritySource",
                "scope",
                "nonScope",
                "precedence",
                "sourceOwnership",
            ],
        )?;
        let claim_boundaries = record_strings(
            &specification,
            "claimBoundaries",
            "ClaimBoundaries",
            &[
                "implementationAcceptance",
                "losslessness",
                "scopedMinimum",
                "benchmarkImprovement",
                "externalRecordAcceptance",
            ],
        )?;
        let evidence_boundaries = record_strings(
            &specification,
            "evidenceBoundaries",
            "EvidenceBoundaries",
            &[
                "productionDependencies",
                "validationAuthorities",
                "measurements",
                "openResearchClaims",
            ],
        )?;
        let research_position = record_strings(
            &specification,
            "researchPosition",
            "ResearchPosition",
            &[
                "priorArt",
                "hypothesis",
                "noveltyStatus",
                "performanceStatus",
            ],
        )?;

        let mut ids = Vec::new();
        for declaration in &registry {
            if definition_result_name(declaration) != Some("ConformanceRow") {
                continue;
            }
            let fields = definition_string_fields(
                declaration,
                "ConformanceRow",
                &["id", "level", "suite", "statement"],
            )?;
            ids.push(IdRow {
                id: required(&fields, "id")?,
                level: parse_level(&required(&fields, "level")?)?,
                suite: required(&fields, "suite")?,
                statement: required(&fields, "statement")?,
            });
        }
        if ids.is_empty() {
            return Err(ModelError::Inconsistent(
                "Uorc.Registry contains no ConformanceRow definitions".to_string(),
            ));
        }

        let graph = Self {
            charter: Charter {
                product_name: required(&charter, "productName")?,
                purpose: required(&charter, "purpose")?,
                authority_source: required(&charter, "authoritySource")?,
                scope: required(&charter, "scope")?,
                non_scope: required(&charter, "nonScope")?,
                precedence: required(&charter, "precedence")?,
                source_ownership: required(&charter, "sourceOwnership")?,
            },
            claim_boundaries: ClaimBoundaries {
                implementation_acceptance: required(&claim_boundaries, "implementationAcceptance")?,
                losslessness: required(&claim_boundaries, "losslessness")?,
                scoped_minimum: required(&claim_boundaries, "scopedMinimum")?,
                benchmark_improvement: required(&claim_boundaries, "benchmarkImprovement")?,
                external_record_acceptance: required(
                    &claim_boundaries,
                    "externalRecordAcceptance",
                )?,
            },
            evidence_boundaries: EvidenceBoundaries {
                production_dependencies: required(&evidence_boundaries, "productionDependencies")?,
                validation_authorities: required(&evidence_boundaries, "validationAuthorities")?,
                measurements: required(&evidence_boundaries, "measurements")?,
                open_research_claims: required(&evidence_boundaries, "openResearchClaims")?,
            },
            research_position: ResearchPosition {
                prior_art: required(&research_position, "priorArt")?,
                hypothesis: required(&research_position, "hypothesis")?,
                novelty_status: required(&research_position, "noveltyStatus")?,
                performance_status: required(&research_position, "performanceStatus")?,
            },
            ids: Ids {
                spec: "template/1".to_string(),
                id: ids,
            },
            authorities: Authorities {
                spec: "template/1".to_string(),
                authority: Vec::new(),
            },
            ledger: Ledger {
                spec: "template/1".to_string(),
                claim: Vec::new(),
            },
        };

        graph.model().check()?;
        Ok(graph)
    }

    /// Project the LexLean graph into the template repository model.
    #[must_use]
    pub fn model(&self) -> Model {
        Model {
            ledger: self.ledger.clone(),
            ids: self.ids.clone(),
            authorities: self.authorities.clone(),
        }
    }
}

fn semantic_declarations(path: &Path, module: &str) -> Result<Vec<Value>, ModelError> {
    let text = std::fs::read_to_string(path).map_err(|e| ModelError::Io(path.to_path_buf(), e))?;
    let open = format!("\\begin{{lexlean}}{{{module}}}");
    if !text.contains(&open)
        || !text.contains("\\begin{semanticmodule}")
        || !text.contains("\\end{semanticmodule}")
        || !text.contains("\\end{lexlean}")
    {
        return Err(ModelError::Inconsistent(format!(
            "{} is not the expected LexLean semantic module `{module}`",
            path.display()
        )));
    }

    const MARKER: &str = "\\semanticdata{";
    if text.matches(MARKER).count() != 1 {
        return Err(ModelError::Inconsistent(format!(
            "{} must contain exactly one semanticdata value",
            path.display()
        )));
    }
    let start = text.find(MARKER).expect("counted one semanticdata marker") + MARKER.len();
    let tail = &text[start..];
    let end = tail.find("\\end{semanticmodule}").ok_or_else(|| {
        ModelError::Inconsistent(format!(
            "{} has no semanticmodule terminator after semanticdata",
            path.display()
        ))
    })?;
    let wrapped = tail[..end].trim();
    let json = wrapped.strip_suffix('}').ok_or_else(|| {
        ModelError::Inconsistent(format!(
            "{} semanticdata macro has no closing brace",
            path.display()
        ))
    })?;
    let value: Value = serde_json::from_str(json).map_err(|e| {
        ModelError::Inconsistent(format!(
            "{} contains invalid semantic JSON: {e}",
            path.display()
        ))
    })?;
    if value.get("spec").and_then(Value::as_str) != Some("lexlean/semantic-module/1") {
        return Err(ModelError::Inconsistent(format!(
            "{} must use lexlean/semantic-module/1",
            path.display()
        )));
    }
    value
        .get("declarations")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            ModelError::Inconsistent(format!(
                "{} has no semantic declaration array",
                path.display()
            ))
        })
}

fn validate_specification_declarations(declarations: &[Value]) -> Result<(), ModelError> {
    let expected: BTreeSet<(&str, &str)> = [
        ("structure", "Charter"),
        ("structure", "ClaimBoundaries"),
        ("structure", "EvidenceBoundaries"),
        ("structure", "ResearchPosition"),
        ("definition", "charter"),
        ("definition", "claimBoundaries"),
        ("definition", "evidenceBoundaries"),
        ("definition", "researchPosition"),
    ]
    .into_iter()
    .collect();

    let mut observed = BTreeSet::new();
    for declaration in declarations {
        let kind = declaration
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::Inconsistent("authority declaration has no kind".to_string())
            })?;
        let name = declaration
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::Inconsistent("authority declaration has no name".to_string())
            })?;
        if !observed.insert((kind, name)) {
            return Err(ModelError::Inconsistent(format!(
                "Uorc.Specification repeats declaration `{name}`"
            )));
        }
    }

    if observed != expected {
        return Err(ModelError::Inconsistent(format!(
            "Uorc.Specification projection declaration set is not exact: expected {:?}, observed {:?}",
            expected, observed
        )));
    }
    Ok(())
}

fn validate_registry_declarations(declarations: &[Value]) -> Result<(), ModelError> {
    let mut structure_count = 0usize;
    let mut definition_names = BTreeSet::new();

    for declaration in declarations {
        let kind = declaration
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::Inconsistent("registry declaration has no kind".to_string())
            })?;
        let name = declaration
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::Inconsistent("registry declaration has no name".to_string())
            })?;
        match kind {
            "structure" if name == "ConformanceRow" => {
                structure_count += 1;
            }
            "definition" if definition_result_name(declaration) == Some("ConformanceRow") => {
                if !definition_names.insert(name) {
                    return Err(ModelError::Inconsistent(format!(
                        "Uorc.Registry repeats row definition `{name}`"
                    )));
                }
            }
            _ => {
                return Err(ModelError::Inconsistent(format!(
                    "Uorc.Registry contains unprojected declaration `{kind} {name}`"
                )));
            }
        }
    }

    if structure_count != 1 {
        return Err(ModelError::Inconsistent(format!(
            "Uorc.Registry must contain exactly one ConformanceRow structure, observed {structure_count}"
        )));
    }
    if definition_names.is_empty() {
        return Err(ModelError::Inconsistent(
            "Uorc.Registry contains no ConformanceRow definitions".to_string(),
        ));
    }
    Ok(())
}

fn record_strings(
    declarations: &[Value],
    definition: &str,
    type_name: &str,
    expected_fields: &[&str],
) -> Result<BTreeMap<String, String>, ModelError> {
    let declaration = declarations
        .iter()
        .find(|value| {
            value.get("kind").and_then(Value::as_str) == Some("definition")
                && value.get("name").and_then(Value::as_str) == Some(definition)
        })
        .ok_or_else(|| {
            ModelError::Inconsistent(format!(
                "LexLean authority graph is missing definition `{definition}`"
            ))
        })?;
    definition_string_fields(declaration, type_name, expected_fields)
}

fn definition_result_name(declaration: &Value) -> Option<&str> {
    declaration
        .get("result")?
        .get("member")?
        .get("name")?
        .as_str()
}

fn definition_string_fields(
    declaration: &Value,
    type_name: &str,
    expected_fields: &[&str],
) -> Result<BTreeMap<String, String>, ModelError> {
    if definition_result_name(declaration) != Some(type_name) {
        return Err(ModelError::Inconsistent(format!(
            "definition `{}` does not return `{type_name}`",
            declaration
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("<unnamed>")
        )));
    }
    let body = declaration
        .get("body")
        .ok_or_else(|| ModelError::Inconsistent("authority definition has no body".to_string()))?;
    if body.get("kind").and_then(Value::as_str) != Some("record")
        || body
            .get("type")
            .and_then(|v| v.get("name"))
            .and_then(Value::as_str)
            != Some(type_name)
    {
        return Err(ModelError::Inconsistent(format!(
            "definition `{}` must be a `{type_name}` record",
            declaration
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("<unnamed>")
        )));
    }
    let fields = body
        .get("fields")
        .and_then(Value::as_array)
        .ok_or_else(|| ModelError::Inconsistent(format!("`{type_name}` record has no fields")))?;
    let mut out = BTreeMap::new();
    for field in fields {
        let name = field.get("field").and_then(Value::as_str).ok_or_else(|| {
            ModelError::Inconsistent(format!("`{type_name}` has a field without a name"))
        })?;
        let value = field.get("value").ok_or_else(|| {
            ModelError::Inconsistent(format!("`{type_name}.{name}` has no value"))
        })?;
        if value.get("kind").and_then(Value::as_str) != Some("string") {
            return Err(ModelError::Inconsistent(format!(
                "`{type_name}.{name}` must be a string"
            )));
        }
        let string = value.get("value").and_then(Value::as_str).ok_or_else(|| {
            ModelError::Inconsistent(format!("`{type_name}.{name}` has no string value"))
        })?;
        if out.insert(name.to_string(), string.to_string()).is_some() {
            return Err(ModelError::Inconsistent(format!(
                "`{type_name}` repeats field `{name}`"
            )));
        }
    }

    let observed: BTreeSet<&str> = out.keys().map(String::as_str).collect();
    let expected: BTreeSet<&str> = expected_fields.iter().copied().collect();
    if observed != expected {
        return Err(ModelError::Inconsistent(format!(
            "`{type_name}` projection fields are not exact: expected {:?}, observed {:?}",
            expected, observed
        )));
    }

    Ok(out)
}

fn required(fields: &BTreeMap<String, String>, name: &str) -> Result<String, ModelError> {
    let value = fields.get(name).ok_or_else(|| {
        ModelError::Inconsistent(format!("LexLean authority record is missing `{name}`"))
    })?;
    if value.trim().is_empty() {
        return Err(ModelError::Inconsistent(format!(
            "LexLean authority field `{name}` is empty"
        )));
    }
    Ok(value.clone())
}

fn parse_level(token: &str) -> Result<Level, ModelError> {
    match token {
        "some-true" => Ok(Level::SomeTrue),
        "build" => Ok(Level::Build),
        "open" => Ok(Level::Open),
        other => Err(ModelError::Inconsistent(format!(
            "unknown honesty level `{other}` in Uorc.Registry"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_graph_loads_from_lexlean_uc_chr_01() {
        let graph = AuthorityGraph::load(&crate::repo_root()).expect("authority graph loads");
        assert!(!graph.charter.product_name.is_empty());
        assert!(!graph.ids.id.is_empty());
    }

    #[test]
    fn projection_schema_is_fail_closed_uc_chr_01() {
        let declaration = serde_json::json!({
            "kind": "definition",
            "name": "row",
            "result": {
                "kind": "named",
                "member": {"module": "Uorc.Registry", "name": "ConformanceRow"},
                "arguments": []
            },
            "body": {
                "kind": "record",
                "type": {"module": "Uorc.Registry", "name": "ConformanceRow"},
                "fields": [
                    {"field": "id", "value": {"kind": "string", "value": "X"}},
                    {"field": "level", "value": {"kind": "string", "value": "build"}},
                    {"field": "suite", "value": {"kind": "string", "value": "x"}},
                    {"field": "statement", "value": {"kind": "string", "value": "x"}},
                    {"field": "hidden", "value": {"kind": "string", "value": "unprojected"}}
                ]
            }
        });
        assert!(
            definition_string_fields(
                &declaration,
                "ConformanceRow",
                &["id", "level", "suite", "statement"],
            )
            .is_err(),
            "an unprojected LexLean field must fail closed"
        );
    }
}
