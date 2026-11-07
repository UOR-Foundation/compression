//! Mechanical projections of the varint contract and native core inventory.
use crate::{authority, ModelError};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Source-owned codec contract and proof/negative inventories.
#[derive(Debug, Clone, Serialize)]
pub struct VarintContract {
    /// Versioned projection schema.
    pub spec: String,
    /// Source-owned explanatory fields.
    pub contract: BTreeMap<String, String>,
    /// General theorem roots, distinct from closed examples.
    pub theorem: Vec<String>,
    /// Closed vector roots, including every failure family.
    pub vector: Vec<String>,
    /// Typed failures and their negative families.
    pub failure: Vec<BTreeMap<String, String>>,
    /// Source-owned planted changes and owning kernel declarations.
    pub mutation: Vec<BTreeMap<String, String>>,
}
fn bad(message: impl Into<String>) -> ModelError {
    ModelError::Inconsistent(format!("UC-VAR-01: {}", message.into()))
}
fn record(d: &Value, expected: &[&str]) -> Result<BTreeMap<String, String>, ModelError> {
    let mut out = BTreeMap::new();
    for f in d["body"]["fields"]
        .as_array()
        .ok_or_else(|| bad("expected contract record"))?
    {
        let name = f["field"].as_str().ok_or_else(|| bad("missing field"))?;
        let value = f["value"]["value"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| bad("missing string value"))?;
        if f["value"]["kind"] != "string" || out.insert(name.into(), value.into()).is_some() {
            return Err(bad("non-string or duplicate contract field"));
        }
    }
    if out.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected.iter().copied().collect()
    {
        return Err(bad("contract record fields are not exact"));
    }
    Ok(out)
}
impl VarintContract {
    /// Project source records and reconcile them with actual kernel declarations.
    pub fn load(root: &Path) -> Result<Self, ModelError> {
        let declarations = authority::semantic_declarations(
            &root.join("src/Uorc/VarintContract.lex.tex"),
            "Uorc.VarintContract",
        )?;
        let file = root.join("src/Uorc/Varint.lex.tex");
        let source =
            std::fs::read_to_string(&file).map_err(|source| ModelError::Io(file, source))?;
        let payloads: Vec<_> = source
            .lines()
            .filter_map(|line| {
                line.strip_prefix("\\coredata{")
                    .and_then(|s| s.strip_suffix('}'))
            })
            .collect();
        if payloads.len() != 1 {
            return Err(bad("expected one native core module"));
        }
        let core: Value = serde_json::from_str(payloads[0]).map_err(|e| bad(e.to_string()))?;
        let rows = core["declarations"]
            .as_array()
            .ok_or_else(|| bad("missing core declarations"))?;
        let mut names = BTreeSet::new();
        for d in rows {
            let name = d["name"].as_str().ok_or_else(|| bad("missing core name"))?;
            if !names.insert(name) || d["policy"]["kind"] != "exact" {
                return Err(bad("duplicate core name or non-exact axiom policy"));
            }
            for axiom in d["policy"]["axioms"]
                .as_array()
                .ok_or_else(|| bad("missing axiom inventory"))?
            {
                if !["propext", "Quot.sound", "Classical.choice"]
                    .contains(&axiom.as_str().unwrap_or(""))
                {
                    return Err(bad("unapproved foundational axiom"));
                }
            }
        }
        for name in [
            "encode",
            "decode",
            "decodePrefix",
            "encodeFuel",
            "scan",
            "rawValue",
        ] {
            let name = format!("Uorc.Varint.{name}");
            if !rows.iter().any(|d| {
                d["name"] == name
                    && d["kind"] == "definition"
                    && d["policy"]["axioms"] == serde_json::json!([])
            }) {
                return Err(bad(format!(
                    "missing axiom-free computational entry {name}"
                )));
            }
        }
        let mut contracts = Vec::new();
        let mut theorem = Vec::new();
        let mut failure = Vec::new();
        let mut mutation = Vec::new();
        for d in declarations.iter().filter(|d| d["kind"] == "definition") {
            match d["result"]["member"]["name"].as_str() {
                Some("VarintContract") => contracts.push(record(
                    d,
                    &[
                        "domain",
                        "encoding",
                        "decoding",
                        "errorOrder",
                        "offsets",
                        "proofScope",
                        "axiomPolicy",
                        "authority",
                    ],
                )?),
                Some("KernelRoot") => theorem.push(
                    record(d, &["symbol"])?
                        .get("symbol")
                        .ok_or_else(|| bad("missing theorem symbol"))?
                        .clone(),
                ),
                Some("VarintFailure") => {
                    failure.push(record(d, &["variant", "meaning", "negativeFamily"])?)
                }
                Some("CoreMutation") => mutation.push(record(
                    d,
                    &["id", "definition", "before", "after", "owner"],
                )?),
                _ => return Err(bad("unexpected varint contract definition")),
            }
        }
        if contracts.len() != 1
            || theorem.is_empty()
            || theorem.iter().collect::<BTreeSet<_>>().len() != theorem.len()
        {
            return Err(bad(
                "missing/duplicate contract or general theorem inventory",
            ));
        }
        for name in &theorem {
            if !rows
                .iter()
                .any(|d| d["name"] == *name && d["kind"] == "theorem")
            {
                return Err(bad(format!("missing general theorem {name}")));
            }
        }
        let families = [
            "encode_vector_",
            "decode_vector_",
            "truncated_",
            "overlong_",
            "overflow_",
            "invalid_byte_",
            "trailing_",
            "encode_overflow_",
        ];
        let vector: Vec<String> = rows
            .iter()
            .filter_map(|d| {
                let name = d["name"].as_str()?;
                (d["kind"] == "theorem"
                    && families
                        .iter()
                        .any(|f| name.starts_with(&format!("Uorc.Varint.{f}"))))
                .then(|| name.to_owned())
            })
            .collect();
        if vector.is_empty() || failure.len() != 6 {
            return Err(bad("empty vectors or incomplete typed-failure inventory"));
        }
        let mut variants = BTreeSet::new();
        for row in &failure {
            let variant = row
                .get("variant")
                .ok_or_else(|| bad("missing failure variant"))?;
            let family = row
                .get("negativeFamily")
                .ok_or_else(|| bad("missing negative family"))?;
            if !variants.insert(variant)
                || !rows.iter().any(|d| {
                    d["name"] == format!("Uorc.Varint.Error.{variant}")
                        && d["kind"] == "constructor"
                })
                || !vector
                    .iter()
                    .any(|name| name.starts_with(&format!("Uorc.Varint.{family}")))
            {
                return Err(bad(format!("unowned or untested failure {variant}")));
            }
        }
        if mutation.is_empty()
            || mutation
                .iter()
                .filter_map(|m| m.get("id"))
                .collect::<BTreeSet<_>>()
                .len()
                != mutation.len()
        {
            return Err(bad("empty or duplicate mutation inventory"));
        }
        for row in &mutation {
            let owner = row
                .get("owner")
                .ok_or_else(|| bad("mutation owner missing"))?;
            if !rows
                .iter()
                .any(|d| d["name"] == *owner && d["kind"] == "theorem")
            {
                return Err(bad(format!("missing mutation owner {owner}")));
            }
        }
        Ok(Self {
            spec: "uorc/varint-contract/1".into(),
            contract: contracts.remove(0),
            theorem,
            vector,
            failure,
            mutation,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conformance_uc_var_01() {
        let contract =
            VarintContract::load(&crate::repo_root()).expect("source-owned codec contract");
        assert_eq!(contract.vector.len(), 236);
        assert_eq!(contract.theorem.len(), 9);
        for name in [
            "uleb_encode_decode",
            "uleb_decode_encode",
            "uleb_size_exact",
            "uleb_size_bounded",
            "shortest",
        ] {
            assert!(contract.theorem.contains(&format!("Uorc.Varint.{name}")));
        }
    }
    #[test]
    fn contract_records_reject_schema_drift_before_rendering() {
        let source = authority::semantic_declarations(
            &crate::repo_root().join("src/Uorc/VarintContract.lex.tex"),
            "Uorc.VarintContract",
        )
        .unwrap();
        let row = source
            .iter()
            .find(|d| d["name"] == "failure_truncated")
            .unwrap();
        let fields = ["variant", "meaning", "negativeFamily"];
        assert!(record(row, &fields).is_ok());
        let mut missing = row.clone();
        missing["body"]["fields"].as_array_mut().unwrap().pop();
        assert!(record(&missing, &fields).is_err());
        let mut duplicate = row.clone();
        let field = duplicate["body"]["fields"][0].clone();
        duplicate["body"]["fields"]
            .as_array_mut()
            .unwrap()
            .push(field);
        assert!(record(&duplicate, &fields).is_err());
        let mut empty = row.clone();
        empty["body"]["fields"][0]["value"]["value"] = serde_json::json!("");
        assert!(record(&empty, &fields).is_err());
    }
}
