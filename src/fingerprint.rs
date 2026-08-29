//! Exact, ABI-local function fingerprinting for known-framework reference joins.
//!
//! Fingerprints deliberately exclude names and absolute addresses. A match is
//! emitted only when the same fingerprint is unique in both documents; no
//! nearest-neighbour or majority guess is allowed.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::diagnostic::{ClutterError, IoContext, Result};

#[derive(Debug, Serialize)]
pub struct FingerprintReport {
    pub schema: &'static str,
    pub target_functions: usize,
    pub reference_functions: usize,
    pub reference_framework_functions: usize,
    pub unique_reference_fingerprints: usize,
    pub unique_target_fingerprints: usize,
    pub matches: Vec<FingerprintMatch>,
    pub ambiguous_fingerprints: usize,
    pub contradictory_name_matches_rejected: usize,
}

#[derive(Debug, Serialize)]
pub struct FingerprintMatch {
    pub target_address: String,
    pub target_name: String,
    pub reference_name: String,
    pub reference_owner: Option<String>,
    pub reference_library_uri: String,
    pub confidence: &'static str,
    pub evidence: &'static str,
}

pub fn match_programs(target: &Path, reference: &Path) -> Result<FingerprintReport> {
    let target_document = read_document(target)?;
    let reference_document = read_document(reference)?;
    let target_functions = functions(&target_document, target)?;
    let reference_functions = functions(&reference_document, reference)?;
    verify_compatible_toolchains(&target_document, &reference_document)?;

    let framework = reference_functions
        .iter()
        .copied()
        .filter(|function| is_known_framework(function))
        .collect::<Vec<_>>();
    let target_index = fingerprint_index(&target_functions);
    let reference_index = fingerprint_index(&framework);
    let unique_target_fingerprints = target_index.values().filter(|rows| rows.len() == 1).count();
    let unique_reference_fingerprints = reference_index
        .values()
        .filter(|rows| rows.len() == 1)
        .count();
    let ambiguous_fingerprints = reference_index
        .iter()
        .filter(|(fingerprint, rows)| {
            rows.len() != 1
                || target_index
                    .get(*fingerprint)
                    .is_some_and(|targets| targets.len() != 1)
        })
        .count();

    let mut matches = Vec::new();
    let mut contradictory_name_matches_rejected = 0usize;
    for (fingerprint, references) in &reference_index {
        let [reference_function] = references.as_slice() else {
            continue;
        };
        let Some(targets) = target_index.get(fingerprint) else {
            continue;
        };
        let [target_function] = targets.as_slice() else {
            continue;
        };
        let library_uri = string(reference_function, "library_uri").unwrap_or_default();
        let reference_name = string(reference_function, "name").unwrap_or("<unnamed>");
        if reference_name.starts_with("sub_") || reference_name.is_empty() {
            continue;
        }
        let target_name = string(target_function, "name").unwrap_or("<unnamed>");
        let target_is_synthetic = target_name.starts_with("sub_");
        // Exact machine bodies are not unique source identities: trivial
        // initializers and tear-offs can compile identically. A surviving,
        // contradictory name is therefore negative evidence, not something
        // the fingerprint may overrule. Promotion is limited to corroborating
        // an existing name (which can still restore owner/library) or naming
        // a genuinely synthetic target.
        if !target_is_synthetic && target_name != reference_name {
            contradictory_name_matches_rejected += 1;
            continue;
        }
        if reference_name == "<anonymous closure>" {
            continue;
        }
        matches.push(FingerprintMatch {
            target_address: string(target_function, "address").unwrap_or_default().to_owned(),
            target_name: target_name.to_owned(),
            reference_name: reference_name.to_owned(),
            reference_owner: string(reference_function, "owner").map(str::to_owned),
            reference_library_uri: library_uri.to_owned(),
            confidence: "corroborated",
            evidence: "unique exact normalized body fingerprint in target and known-framework reference",
        });
    }
    // Shared Code aliases can produce more than one logical function at one
    // address. Keep one proposal only when every proposal for that address
    // agrees on the exact reference identity; conflicting aliases stay out.
    let mut by_address = BTreeMap::<String, Vec<FingerprintMatch>>::new();
    for matched in matches {
        by_address
            .entry(matched.target_address.clone())
            .or_default()
            .push(matched);
    }
    let mut matches = Vec::new();
    for proposals in by_address.into_values() {
        let first = &proposals[0];
        let agrees = proposals.iter().all(|proposal| {
            proposal.reference_name == first.reference_name
                && proposal.reference_owner == first.reference_owner
                && proposal.reference_library_uri == first.reference_library_uri
        });
        if agrees {
            matches.push(proposals.into_iter().next().expect("non-empty proposals"));
        }
    }

    Ok(FingerprintReport {
        schema: "clutter.fingerprint-match/v1",
        target_functions: target_functions.len(),
        reference_functions: reference_functions.len(),
        reference_framework_functions: framework.len(),
        unique_reference_fingerprints,
        unique_target_fingerprints,
        matches,
        ambiguous_fingerprints,
        contradictory_name_matches_rejected,
    })
}

fn verify_compatible_toolchains(target: &Value, reference: &Value) -> Result<()> {
    for key in ["snapshot_hash", "dart_commit", "target_arch"] {
        let target_value = target
            .get("vm_oracle")
            .and_then(|oracle| string(oracle, key));
        let reference_value = reference
            .get("vm_oracle")
            .and_then(|oracle| string(oracle, key));
        if let (Some(target_value), Some(reference_value)) = (target_value, reference_value)
            && target_value != reference_value
        {
            return Err(ClutterError::Analysis(format!(
                "reference mismatch: target {key} is `{target_value}`, reference is `{reference_value}`"
            )));
        }
    }
    Ok(())
}

fn read_document(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).at(path)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        ClutterError::Analysis(format!(
            "failed to parse IR document {}: {error}",
            path.display()
        ))
    })
}

fn functions<'a>(document: &'a Value, path: &Path) -> Result<Vec<&'a Value>> {
    document
        .get("functions")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().collect())
        .ok_or_else(|| {
            ClutterError::Analysis(format!(
                "IR document {} has no `functions` array; pass ir/program.json",
                path.display()
            ))
        })
}

fn is_known_framework(function: &Value) -> bool {
    string(function, "library_uri").is_some_and(|uri| {
        uri.starts_with("dart:")
            || uri.starts_with("package:flutter/")
            || uri.starts_with("package:flutter_test/")
    })
}

fn fingerprint_index<'a>(functions: &[&'a Value]) -> BTreeMap<String, Vec<&'a Value>> {
    let mut index = BTreeMap::<String, Vec<&Value>>::new();
    for function in functions {
        if let Some(fingerprint) = fingerprint(function) {
            index.entry(fingerprint).or_default().push(function);
        }
    }
    index
}

fn fingerprint(function: &Value) -> Option<String> {
    let instructions = function.get("instructions")?.as_array()?;
    if instructions.len() < 4 {
        return None;
    }
    let mut parts = Vec::new();
    parts.push(format!("size:{}", function.get("size")?.as_u64()?));
    parts.push(format!("kind:{}", string(function, "kind").unwrap_or("?")));

    let mnemonics = instructions
        .iter()
        .filter_map(|instruction| string(instruction, "mnemonic"))
        .map(normalize_mnemonic)
        .collect::<Vec<_>>();
    parts.push(format!("insn:{}", mnemonics.join(",")));

    let semantics = function
        .get("semantic_statements")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| string(row, "kind"))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    parts.push(format!("semantic:{}", semantics.join(",")));

    let edges = function
        .get("control_flow")
        .and_then(Value::as_array)
        .map(|rows| normalized_edges(rows))
        .unwrap_or_default();
    parts.push(format!("cfg:{}", edges.join(",")));

    let constants = instructions
        .iter()
        .filter_map(|instruction| string(instruction, "object_pool_value"))
        .filter(|value| is_stable_constant(value))
        .collect::<BTreeSet<_>>();
    parts.push(format!(
        "constants:{}",
        constants.into_iter().collect::<Vec<_>>().join("|")
    ));
    Some(parts.join(";"))
}

fn normalize_mnemonic(mnemonic: &str) -> &str {
    mnemonic
        .strip_suffix("eq")
        .or_else(|| mnemonic.strip_suffix("ne"))
        .unwrap_or(mnemonic)
}

fn normalized_edges(edges: &[Value]) -> Vec<String> {
    let mut addresses = edges
        .iter()
        .flat_map(|edge| [string(edge, "from"), string(edge, "to")])
        .flatten()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    addresses.sort_by_key(|address| parse_address(address).unwrap_or(u64::MAX));
    let ranks = addresses
        .into_iter()
        .enumerate()
        .map(|(rank, address)| (address, rank))
        .collect::<BTreeMap<_, _>>();
    let mut normalized = edges
        .iter()
        .filter_map(|edge| {
            let from = ranks.get(string(edge, "from")?)?;
            let to = ranks.get(string(edge, "to")?)?;
            let kind = string(edge, "kind")?;
            Some(format!("{from}>{to}:{kind}"))
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized
}

fn is_stable_constant(value: &str) -> bool {
    value.starts_with('"')
        || matches!(value, "true" | "false" | "null")
        || value.starts_with("Type(")
        || value.starts_with("Class(")
}

fn parse_address(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

fn string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn fingerprint_ignores_addresses_and_names() {
        let make = |name: &str, base: &str, next: &str| {
            json!({
                "name": name,
                "kind": "regular",
                "size": 16,
                "instructions": [
                    {"mnemonic":"ldr"}, {"mnemonic":"cmp"},
                    {"mnemonic":"bne"}, {"mnemonic":"ret"}
                ],
                "semantic_statements": [{"kind":"condition"}],
                "control_flow": [{"from":base,"to":next,"kind":"conditional_true"}]
            })
        };
        assert_eq!(
            super::fingerprint(&make("plain", "0x100", "0x110")),
            super::fingerprint(&make("a", "0x900", "0x910"))
        );
    }

    #[test]
    fn short_bodies_are_not_matchable() {
        let function = json!({
            "size": 4,
            "instructions": [{"mnemonic":"ret"}],
            "semantic_statements": [],
            "control_flow": []
        });
        assert_eq!(super::fingerprint(&function), None);
    }
}
