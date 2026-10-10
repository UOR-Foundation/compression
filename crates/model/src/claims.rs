//! Mechanical projections and consistency checks for the LexLean claim policy.
//! Classification decisions and their closed kernel cases live in `Uorc.Claims`.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{authority, Level, ModelError};

/// Generated evidence-classification register.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimPolicy {
    /// Versioned projection schema.
    pub spec: String,
    /// Source-owned explanation of the classification/evidence boundary.
    pub summary: String,
    /// Source-owned evidence constructors.
    pub evidence_kinds: Vec<String>,
    /// Source-owned claim constructors.
    pub claim_kinds: Vec<String>,
    /// Source-owned research dispositions.
    pub research_dispositions: Vec<String>,
    /// Exact closed kernel case inventory.
    pub theorem_roots: Vec<String>,
    /// One classification record per evidence constructor.
    pub evidence: Vec<EvidenceRule>,
    /// Current research claims and their explicit source disposition.
    pub open_claim: Vec<OpenClaim>,
    /// Source-authored mutations executed against the actual kernel gate.
    pub mutation: Vec<ClaimMutation>,
}

/// One source-owned negative test of an evidence promotion.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimMutation {
    /// Stable test identifier.
    pub id: String,
    /// Closed source record to mutate.
    pub definition: String,
    /// Field to replace.
    pub field: String,
    /// Replacement constructor or string.
    pub replacement: String,
    /// Exact theorem whose kernel check must reject the promotion.
    pub owner: String,
}

/// One LexLean-authored evidence classification.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRule {
    /// Evidence-kind constructor.
    pub kind: String,
    /// The claim this evidence supports, within its own scope only.
    pub claim: String,
    /// Existing template honesty level; a theorem does not invent a fourth level.
    pub level: String,
    /// Research disposition, independent of implementation evidence.
    pub disposition: String,
    /// Eligibility as implementation evidence, never whole-release acceptance.
    pub implementation_evidence: bool,
}

/// A currently unevidenced research claim, protected by closed source proofs.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenClaim {
    /// Registered claim ID.
    pub id: String,
    /// Template honesty level.
    pub level: String,
    /// Current evidence kind.
    pub evidence: String,
    /// Current research disposition.
    pub disposition: String,
}

fn bad(message: impl Into<String>) -> ModelError {
    ModelError::Inconsistent(format!("UC-HON-02: {}", message.into()))
}
fn token(value: &Value, key: &str) -> Result<String, ModelError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| bad(format!("missing string {key}")))
}
fn enum_values(declarations: &[Value], name: &str) -> Result<Vec<String>, ModelError> {
    let rows: Vec<_> = declarations
        .iter()
        .filter(|d| d["kind"] == "inductive" && d["name"] == name)
        .collect();
    if rows.len() != 1 {
        return Err(bad(format!("missing/duplicate enum {name}")));
    }
    rows[0]["constructors"]
        .as_array()
        .ok_or_else(|| bad("missing constructors"))?
        .iter()
        .map(|c| {
            if c["fields"] != serde_json::json!([]) {
                return Err(bad("evidence constructors must be nullary"));
            }
            token(c, "name")
        })
        .collect()
}
fn fields<'a>(
    declaration: &'a Value,
    expected: &[&str],
) -> Result<BTreeMap<String, &'a Value>, ModelError> {
    let mut out = BTreeMap::new();
    for field in declaration["body"]["fields"]
        .as_array()
        .ok_or_else(|| bad("missing record fields"))?
    {
        let name = token(field, "field")?;
        if out.insert(name, &field["value"]).is_some() {
            return Err(bad("duplicate record field"));
        }
    }
    if out.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected.iter().copied().collect()
    {
        return Err(bad("claim-policy fields are not exact"));
    }
    Ok(out)
}
fn constructor(value: &Value, prefix: &str) -> Result<String, ModelError> {
    if value["kind"] != "constructor" || value["arguments"] != serde_json::json!([]) {
        return Err(bad("classification is not a closed enum constructor"));
    }
    token(&value["constructor"], "name")?
        .strip_prefix(&format!("{prefix}."))
        .map(str::to_owned)
        .ok_or_else(|| bad("wrong classification enum"))
}
fn string(value: &Value) -> Result<String, ModelError> {
    if value["kind"] != "string" {
        return Err(bad("expected a source string"));
    }
    token(value, "value")
}

impl ClaimPolicy {
    /// Project closed source records without reimplementing classification.
    pub fn load_source(root: &Path) -> Result<Self, ModelError> {
        let declarations =
            authority::semantic_declarations(&root.join("src/Uorc/Claims.lex.tex"), "Uorc.Claims")?;
        let evidence_kinds = enum_values(&declarations, "EvidenceKind")?;
        let claim_kinds = enum_values(&declarations, "ClaimKind")?;
        let research_dispositions = enum_values(&declarations, "ResearchDisposition")?;
        let summaries: Vec<_> = declarations
            .iter()
            .filter(|d| d["kind"] == "definition" && d["name"] == "policySummary")
            .collect();
        if summaries.len() != 1 {
            return Err(bad("missing or duplicate policy summary"));
        }
        let summary = string(&summaries[0]["body"])?;
        let mut evidence = Vec::new();
        let mut open_claim = Vec::new();
        let mut mutation = Vec::new();
        let mut expected_proofs = BTreeSet::new();
        for d in &declarations {
            if d["kind"] != "definition" {
                continue;
            }
            if d["result"]["member"]["name"] == "ClaimMutation" {
                let f = fields(d, &["id", "definition", "field", "replacement", "owner"])?;
                mutation.push(ClaimMutation {
                    id: string(f["id"])?,
                    definition: string(f["definition"])?,
                    field: string(f["field"])?,
                    replacement: string(f["replacement"])?,
                    owner: string(f["owner"])?,
                });
            }
            if d["result"]["member"]["name"] == "EvidenceRule" {
                let f = fields(
                    d,
                    &[
                        "evidence",
                        "claim",
                        "level",
                        "disposition",
                        "implementationEvidence",
                    ],
                )?;
                let kind = constructor(f["evidence"], "EvidenceKind")?;
                for function in [
                    "claimForEvidence",
                    "researchDisposition",
                    "implementationEvidence",
                    "honestyLevel",
                ] {
                    expected_proofs.insert(format!("{function}_{kind}"));
                }
                evidence.push(EvidenceRule {
                    kind,
                    claim: constructor(f["claim"], "ClaimKind")?,
                    level: string(f["level"])?,
                    disposition: constructor(f["disposition"], "ResearchDisposition")?,
                    implementation_evidence: f["implementationEvidence"]["value"]
                        .as_bool()
                        .ok_or_else(|| bad("expected source Boolean"))?,
                });
            }
            if d["result"]["member"]["name"] == "OpenClaim" {
                let f = fields(d, &["id", "level", "evidence", "disposition"])?;
                for field in ["level", "evidence", "disposition"] {
                    expected_proofs.insert(format!("{}_{}_unpromoted", token(d, "name")?, field));
                }
                open_claim.push(OpenClaim {
                    id: string(f["id"])?,
                    level: string(f["level"])?,
                    evidence: string(f["evidence"])?,
                    disposition: string(f["disposition"])?,
                });
            }
        }
        expected_proofs.extend([
            "valid_measurement_benchmark_target_unmet".into(),
            "valid_measurement_benchmark_target_met".into(),
        ]);
        let mut actual_proofs = BTreeSet::new();
        for d in declarations.iter().filter(|d| d["kind"] == "theorem") {
            if d["axioms"] != serde_json::json!([]) || !actual_proofs.insert(token(d, "name")?) {
                return Err(bad(
                    "duplicate theorem or nonempty claim-policy axiom allowance",
                ));
            }
        }
        if actual_proofs != expected_proofs {
            return Err(bad("claim-policy kernel case inventory is incomplete"));
        }
        let policy = Self {
            spec: "uorc/claim-policy/1".into(),
            summary,
            evidence_kinds,
            claim_kinds,
            research_dispositions,
            theorem_roots: actual_proofs
                .into_iter()
                .map(|name| format!("Uorc.Claims.{name}"))
                .collect(),
            evidence,
            open_claim,
            mutation,
        };
        policy.check()?;
        Ok(policy)
    }

    /// Reconcile the generated policy's closed inventories.
    pub fn check(&self) -> Result<(), ModelError> {
        if self.spec != "uorc/claim-policy/1" || self.summary.trim().is_empty() {
            return Err(bad("unknown claim policy schema"));
        }
        for values in [
            &self.evidence_kinds,
            &self.claim_kinds,
            &self.research_dispositions,
            &self.theorem_roots,
        ] {
            if values.is_empty()
                || values.iter().any(|s| s.trim().is_empty())
                || values.iter().collect::<BTreeSet<_>>().len() != values.len()
            {
                return Err(bad("empty or duplicated policy inventory"));
            }
        }
        let kinds: BTreeSet<_> = self.evidence.iter().map(|r| &r.kind).collect();
        if kinds.len() != self.evidence.len() || kinds != self.evidence_kinds.iter().collect() {
            return Err(bad("evidence rule/constructor inventory differs"));
        }
        for r in &self.evidence {
            if !self.claim_kinds.contains(&r.claim)
                || !self.research_dispositions.contains(&r.disposition)
                || !["open", "build", "some-true"].contains(&r.level.as_str())
            {
                return Err(bad("unknown rule claim, level or disposition"));
            }
            if r.level == "open" && r.implementation_evidence {
                return Err(bad("open evidence cannot qualify implementation"));
            }
        }
        if self.open_claim.is_empty()
            || self
                .open_claim
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.open_claim.len()
        {
            return Err(bad("empty/duplicate protected claim inventory"));
        }
        for row in &self.open_claim {
            let rule = self.rule(&row.evidence)?;
            if row.level != Level::Open.as_str()
                || row.level != rule.level
                || row.disposition != rule.disposition
                || rule.implementation_evidence
            {
                return Err(bad(format!("{}: research claim was promoted", row.id)));
            }
        }
        if self.mutation.is_empty()
            || self
                .mutation
                .iter()
                .map(|m| &m.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.mutation.len()
        {
            return Err(bad("empty or duplicated mutation inventory"));
        }
        for mutation in &self.mutation {
            if !self
                .theorem_roots
                .contains(&format!("Uorc.Claims.{}", mutation.owner))
            {
                return Err(bad("mutation has no owning kernel theorem"));
            }
        }
        Ok(())
    }

    /// Find the unique source-defined classification for an evidence constructor.
    pub fn rule(&self, kind: &str) -> Result<&EvidenceRule, ModelError> {
        self.evidence
            .iter()
            .find(|r| r.kind == kind)
            .ok_or_else(|| bad(format!("unregistered evidence kind {kind}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthorityGraph;

    #[test]
    fn conformance_uc_hon_01() {
        let policy = ClaimPolicy::load_source(&crate::repo_root()).expect("source policy");
        assert_eq!(policy.evidence.len(), 10);
        assert_eq!(policy.theorem_roots.len(), 48);
        for kind in &policy.evidence_kinds {
            let rule = policy.rule(kind).unwrap();
            assert_eq!(
                rule.claim == "record_validated",
                kind == "external_record_acceptance"
            );
            assert_eq!(rule.claim == "kernel_theorem", kind == "kernel_theorem");
            if rule.level == "open" {
                assert!(!rule.implementation_evidence);
            }
        }
        let unmet = policy.rule("benchmark_target_unmet").unwrap();
        assert_eq!(unmet.claim, "benchmark_result");
        assert_eq!(unmet.disposition, "measured_target_unmet");
        assert!(!unmet.implementation_evidence);
        assert_eq!(
            policy.rule("benchmark_target_met").unwrap().disposition,
            "measured_target_met"
        );
        for kind in [
            "fixture_result",
            "scoped_minimum_certificate",
            "version_metadata",
        ] {
            assert_eq!(policy.rule(kind).unwrap().disposition, "not_measured");
        }
    }

    #[test]
    fn ledger_rejects_omissions_duplicates_and_cross_class_promotion() {
        let graph = AuthorityGraph::load(&crate::repo_root()).unwrap();
        let model = graph.model();
        let mut omitted = model.clone();
        omitted.ledger.claim.pop();
        assert!(omitted.check().is_err());
        let mut duplicate = model.clone();
        duplicate
            .ledger
            .claim
            .push(duplicate.ledger.claim[0].clone());
        assert!(duplicate.check().is_err());
        for id in ["UC-PERF-01", "UC-REC-01"] {
            for kind in &model.claim_policy.evidence_kinds {
                if kind == "none" {
                    continue;
                }
                let mut changed = model.clone();
                let rule = model.claim_policy.rule(kind).unwrap();
                let row = changed
                    .ledger
                    .claim
                    .iter_mut()
                    .find(|row| row.id == id)
                    .unwrap();
                row.evidence_kind = kind.clone();
                row.research_disposition = rule.disposition.clone();
                assert!(changed.check().is_err(), "{id} promoted by {kind}");
            }
            let mut promoted = model.clone();
            promoted
                .ids
                .id
                .iter_mut()
                .find(|row| row.id == id)
                .unwrap()
                .level = Level::Build;
            let claim = promoted
                .ledger
                .claim
                .iter_mut()
                .find(|row| row.id == id)
                .unwrap();
            claim.level = Level::Build;
            claim.evidence_kind = "build_evidence".into();
            assert!(
                promoted.check().is_err(),
                "coordinated ledger/registry promotion of {id}"
            );
        }
        let mut changed = model.clone();
        changed.ledger.claim[0].research_disposition = "externally_accepted_record".into();
        assert!(changed.check().is_err());
        let mut changed = model.clone();
        changed.ledger.claim[0].feature = Some("features/suites/other.feature".into());
        assert!(changed.check().is_err());
    }

    #[test]
    fn classification_inventory_and_open_evidence_fail_closed() {
        let policy = ClaimPolicy::load_source(&crate::repo_root()).unwrap();
        let mut changed = policy.clone();
        changed.evidence.pop();
        assert!(changed.check().is_err());
        let mut changed = policy.clone();
        changed.evidence.push(changed.evidence[0].clone());
        assert!(changed.check().is_err());
        let mut changed = policy.clone();
        changed.open_claim[0].level = "build".into();
        assert!(changed.check().is_err());
        let mut changed = policy.clone();
        changed
            .evidence
            .iter_mut()
            .find(|r| r.kind == "benchmark_target_unmet")
            .unwrap()
            .implementation_evidence = true;
        assert!(changed.check().is_err());
        assert!(policy.rule("fixture_as_proof").is_err());
    }
}
