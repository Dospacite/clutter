//! Physical-body / logical-occurrence graph.
//!
//! A *physical body* is one machine-code range at one isolate-instruction
//! offset. A *logical occurrence* is one Dart function (or closure) identity
//! that may be linked to a body — including several occurrences sharing one
//! optimized body, and closures whose display names collide at the same entry
//! address. Bindings are many-to-many and never overwrite each other: adding
//! occurrence B at the address of occurrence A appends a binding instead of
//! replacing A's claims.

use std::collections::BTreeMap;

use serde::Serialize;

use super::tier::{EvidenceTier, TieredClaim};
use crate::diagnostic::{ClutterError, Result};

/// Offset into the `_kDartIsolateSnapshotInstructions` region.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct IsolateInstructionOffset(pub u64);

/// Identity of one machine-code range: derived from the subject ABI plus the
/// authoritative static entry offset. Size is a claim about a body, never part
/// of its identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize)]
pub struct PhysicalBodyId {
    pub abi: crate::model::Abi,
    pub entry: u64,
}

/// Identity of one logical Dart function occurrence within one evidence source.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize)]
pub struct FunctionOccurrenceId {
    /// 0 = static snapshot parser, 1 = VM oracle, 2 = runtime trace.
    pub source: EvidenceSourceId,
    pub object: u64,
}

pub type EvidenceSourceId = u32;

pub const SOURCE_STATIC: EvidenceSourceId = 0;
pub const SOURCE_VM_ORACLE: EvidenceSourceId = 1;
/// Reserved for the runtime-trace source; consumed by the trace ingestion
/// path once emulator evidence is attached to the graph.
#[allow(dead_code)]
pub const SOURCE_RUNTIME_TRACE: EvidenceSourceId = 2;

/// How a claimed code range relates to the authoritative static extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "relation", rename_all = "snake_case")]
pub enum RangeRelation {
    Exact,
    SameStartDifferentEnd { static_end: u64, oracle_end: u64 },
    OracleContainedByStatic,
    StaticContainedByOracle,
    Overlap,
    Disjoint,
}

impl RangeRelation {
    pub fn classify(static_extent: (u64, u64), oracle_extent: (u64, u64)) -> Self {
        let (static_start, static_end) = static_extent;
        let (oracle_start, oracle_end) = oracle_extent;
        if static_start == oracle_start && static_end == oracle_end {
            return Self::Exact;
        }
        if static_start == oracle_start {
            return Self::SameStartDifferentEnd {
                static_end,
                oracle_end,
            };
        }
        if oracle_start >= static_start && oracle_end <= static_end {
            if oracle_start == static_start && oracle_end == static_end {
                unreachable!("exact ranges handled above");
            }
            return Self::OracleContainedByStatic;
        }
        if static_start >= oracle_start && static_end <= oracle_end {
            return Self::StaticContainedByOracle;
        }
        if static_start < oracle_end && oracle_start < static_end {
            return Self::Overlap;
        }
        Self::Disjoint
    }

    /// Only an exact range join can carry semantic promotion.
    pub fn supports_semantic_promotion(self) -> bool {
        matches!(self, Self::Exact)
    }
}

/// One machine-code range with its byte-level provenance.
#[derive(Clone, Debug, Serialize)]
pub struct PhysicalBody {
    pub id: PhysicalBodyId,
    pub entry: IsolateInstructionOffset,
    /// Authoritative static decode extent [start, end).
    pub static_extent: ByteExtent,
    pub static_bytes_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ByteExtent {
    pub start: u64,
    pub end: u64,
}

/// One logical Dart function identity observed in one evidence source.
#[derive(Clone, Debug, Serialize)]
pub struct FunctionOccurrence {
    pub id: FunctionOccurrenceId,
    /// Display name candidates with their tiers; obfuscated names stay raw.
    pub names: Vec<super::tier::TieredClaim<String>>,
    pub owner: Option<String>,
    pub library_uri: Option<String>,
    pub parent_occurrence: Option<FunctionOccurrenceId>,
    pub is_closure: bool,
}

/// One many-to-many binding between a body and a function occurrence.
#[derive(Clone, Debug, Serialize)]
pub struct BodyBinding {
    pub body: PhysicalBodyId,
    pub function: FunctionOccurrenceId,
    pub relation: RangeRelation,
    pub tier: EvidenceTier,
    /// Oracle/static size disagreement in bytes, when any.
    pub extent_conflict: Option<(u64, u64)>,
}

/// The full graph for one payload subject.
#[derive(Clone, Debug, Default, Serialize)]
pub struct BodyGraph {
    pub bodies: Vec<PhysicalBody>,
    pub occurrences: Vec<FunctionOccurrence>,
    pub bindings: Vec<BodyBinding>,
    /// Bodies shared by more than one occurrence, keyed by body id order.
    pub shared_bodies: Vec<PhysicalBodyId>,
}

impl BodyGraph {
    pub fn insert_body(&mut self, body: PhysicalBody) {
        if !self.bodies.iter().any(|existing| existing.id == body.id) {
            self.bodies.push(body);
        }
    }

    pub fn insert_occurrence(&mut self, occurrence: FunctionOccurrence) {
        if !self
            .occurrences
            .iter()
            .any(|existing| existing.id == occurrence.id)
        {
            self.occurrences.push(occurrence);
        }
    }

    /// Appends a binding without evicting existing ones. Returns false when
    /// the identical binding already exists.
    pub fn bind(&mut self, binding: BodyBinding) -> bool {
        let duplicate = self
            .bindings
            .iter()
            .any(|existing| existing.body == binding.body && existing.function == binding.function);
        if duplicate {
            return false;
        }
        self.bindings.push(binding);
        true
    }

    /// Recomputes which bodies host multiple distinct occurrences.
    pub fn recompute_shared(&mut self) {
        let mut counts: BTreeMap<PhysicalBodyId, usize> = BTreeMap::new();
        for binding in &self.bindings {
            *counts.entry(binding.body).or_default() += 1;
        }
        self.shared_bodies = counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(body, _)| body)
            .collect();
    }

    /// Every occurrence bound to a given body. Used by resolution phases and
    /// unit tests to verify that same-address occurrences accumulate.
    #[allow(dead_code)]
    pub fn occurrences_at(&self, body: PhysicalBodyId) -> Vec<FunctionOccurrenceId> {
        self.bindings
            .iter()
            .filter(|binding| binding.body == body)
            .map(|binding| binding.function)
            .collect()
    }

    #[allow(dead_code)] // lookup API for resolution phases; covered by tests
    pub fn body(&self, id: PhysicalBodyId) -> Option<&PhysicalBody> {
        self.bodies.iter().find(|body| body.id == id)
    }

    /// Resolution report consumed by output and tests.
    pub fn report(&self) -> BodyGraphReport {
        let mut report = BodyGraphReport {
            body_count: self.bodies.len(),
            occurrence_count: self.occurrences.len(),
            binding_count: self.bindings.len(),
            ..BodyGraphReport::default()
        };
        let mut unbound_oracle_functions = 0usize;
        // Oracle-sourced occurrences without at least one exact-range binding.
        for occurrence in &self.occurrences {
            if occurrence.id.source != SOURCE_VM_ORACLE {
                continue;
            }
            let bound_exact = self.bindings.iter().any(|binding| {
                binding.function == occurrence.id && binding.relation.supports_semantic_promotion()
            });
            if !bound_exact {
                unbound_oracle_functions += 1;
            }
        }
        report.unbound_oracle_occurrences = unbound_oracle_functions;
        report.shared_body_count = self.shared_bodies.len();
        report.extent_conflicts = self
            .bindings
            .iter()
            .filter(|binding| binding.extent_conflict.is_some())
            .count();
        report.disjoint_bindings = self
            .bindings
            .iter()
            .filter(|binding| matches!(binding.relation, RangeRelation::Disjoint))
            .count();
        report
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct BodyGraphReport {
    pub body_count: usize,
    pub occurrence_count: usize,
    pub binding_count: usize,
    pub shared_body_count: usize,
    pub unbound_oracle_occurrences: usize,
    pub extent_conflicts: usize,
    pub disjoint_bindings: usize,
}

/// Builds the graph from the recovered program plus an optional bound VM
/// oracle. The static parser is authoritative for extents; oracle sizes are
/// appended as claims and every disagreement is retained.
///
/// `oracle_functions` are the loaded oracle's function evidence (already
/// matched or code-identity-only): every entry with a `code_offset` becomes a
/// logical occurrence bound to the body at that offset, with its tiered name
/// claims. Occurrences at one address accumulate — a closure sharing an
/// optimized body with another function appends bindings instead of
/// overwriting them.
pub fn build(
    program: &crate::model::RecoveredProgram,
    abi: crate::model::Abi,
    instructions_region: Option<&crate::model::SnapshotRegion>,
    oracle_functions: &[crate::model::VmFunctionEvidence],
) -> Result<BodyGraph> {
    let _ = instructions_region;
    let mut graph = BodyGraph::default();
    let mut by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (index, function) in program.functions.iter().enumerate() {
        let entry = parse_hex(&function.address).ok_or_else(|| {
            ClutterError::Analysis(format!(
                "recovered function {} has a non-hex address {:?}",
                function.name, function.address
            ))
        })?;
        by_offset.entry(entry).or_default().push(index);
        let end = entry.saturating_add(function.size);
        graph.insert_body(PhysicalBody {
            id: PhysicalBodyId { abi, entry },
            entry: IsolateInstructionOffset(entry),
            static_extent: ByteExtent { start: entry, end },
            static_bytes_sha256: String::new(),
        });
        graph.insert_occurrence(FunctionOccurrence {
            id: FunctionOccurrenceId {
                source: SOURCE_STATIC,
                object: index as u64,
            },
            names: vec![TieredClaim::new(
                function.name.clone(),
                tier_for_name_source(function.name_source),
                "static_snapshot",
            )],
            owner: function.owner.clone(),
            library_uri: function.library_uri.clone(),
            parent_occurrence: None,
            is_closure: function.kind == Some(crate::model::RecoveredFunctionKind::Closure),
        });
        // Oracle size claim for this same address, when present.
        let relation = match oracle_functions
            .iter()
            .find(|candidate| candidate.code_offset == Some(entry))
        {
            Some(candidate) if candidate.code_size.is_some() => {
                let oracle_end = entry.saturating_add(candidate.code_size.unwrap());
                RangeRelation::classify((entry, end), (entry, oracle_end))
            }
            _ => RangeRelation::Exact,
        };
        let extent_conflict = match relation {
            RangeRelation::SameStartDifferentEnd {
                static_end,
                oracle_end,
            } => Some((static_end - entry, oracle_end - entry)),
            _ => None,
        };
        graph.bind(BodyBinding {
            body: PhysicalBodyId { abi, entry },
            function: FunctionOccurrenceId {
                source: SOURCE_STATIC,
                object: index as u64,
            },
            relation,
            tier: EvidenceTier::Proven,
            extent_conflict,
        });
    }

    // Every oracle function with linked code becomes a logical occurrence.
    // Multiple oracle functions (and closures) can share one offset: each is
    // appended as its own binding on the same physical body.
    for candidate in oracle_functions {
        let Some(offset) = candidate.code_offset else {
            continue;
        };
        let end = offset.saturating_add(candidate.code_size.unwrap_or(0));
        graph.insert_body(PhysicalBody {
            id: PhysicalBodyId { abi, entry: offset },
            entry: IsolateInstructionOffset(offset),
            static_extent: ByteExtent { start: offset, end },
            static_bytes_sha256: String::new(),
        });
        let mut names = Vec::new();
        if !candidate.name.is_empty() {
            names.push(TieredClaim::new(
                candidate.name.clone(),
                EvidenceTier::Proven,
                "vm_oracle_function",
            ));
        }
        if let Some(user_visible) = &candidate.user_visible_name {
            if candidate.raw_name.as_deref() != Some(user_visible.as_str()) {
                names.push(TieredClaim::new(
                    user_visible.clone(),
                    EvidenceTier::Proven,
                    "vm_oracle_user_visible",
                ));
            }
        }
        graph.insert_occurrence(FunctionOccurrence {
            id: FunctionOccurrenceId {
                source: SOURCE_VM_ORACLE,
                object: candidate.object_id,
            },
            names,
            owner: candidate.owner.clone(),
            library_uri: candidate.library_uri.clone(),
            parent_occurrence: candidate.parent_function_object_id.map(|parent| {
                FunctionOccurrenceId {
                    source: SOURCE_VM_ORACLE,
                    object: parent,
                }
            }),
            is_closure: candidate.parent_function_object_id.is_some(),
        });
        let extent_conflict = match (
            by_offset
                .get(&offset)
                .and_then(|indices| program.functions.get(*indices.first()?).map(|f| f.size)),
            candidate.code_size,
        ) {
            (Some(static_size), Some(oracle_size)) if static_size != oracle_size => {
                Some((static_size, oracle_size))
            }
            _ => None,
        };
        graph.bind(BodyBinding {
            body: PhysicalBodyId { abi, entry: offset },
            function: FunctionOccurrenceId {
                source: SOURCE_VM_ORACLE,
                object: candidate.object_id,
            },
            relation: RangeRelation::Exact,
            tier: EvidenceTier::Proven,
            extent_conflict,
        });
    }
    graph.recompute_shared();
    Ok(graph)
}

fn tier_for_name_source(source: crate::model::RecoveredNameSource) -> EvidenceTier {
    match source {
        crate::model::RecoveredNameSource::Snapshot => EvidenceTier::Proven,
        crate::model::RecoveredNameSource::DartVmOracle => EvidenceTier::Proven,
        crate::model::RecoveredNameSource::SplitDebugInfo => EvidenceTier::Proven,
        crate::model::RecoveredNameSource::ObfuscationMap => EvidenceTier::Inferred,
        // LLM-assisted and synthetic names are speculative by construction:
        // they live only in the speculative tier, never proven evidence.
        crate::model::RecoveredNameSource::Synthetic => EvidenceTier::Speculative,
        crate::model::RecoveredNameSource::LlmAssisted => EvidenceTier::Speculative,
    }
}

fn parse_hex(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_range_relations() {
        assert_eq!(
            RangeRelation::classify((0, 100), (0, 100)),
            RangeRelation::Exact
        );
        assert_eq!(
            RangeRelation::classify((0, 100), (0, 120)),
            RangeRelation::SameStartDifferentEnd {
                static_end: 100,
                oracle_end: 120
            }
        );
        assert_eq!(
            RangeRelation::classify((0, 100), (10, 90)),
            RangeRelation::OracleContainedByStatic
        );
        assert_eq!(
            RangeRelation::classify((10, 90), (0, 100)),
            RangeRelation::StaticContainedByOracle
        );
        assert_eq!(
            RangeRelation::classify((0, 50), (40, 90)),
            RangeRelation::Overlap
        );
        assert_eq!(
            RangeRelation::classify((0, 50), (60, 90)),
            RangeRelation::Disjoint
        );
    }

    #[test]
    fn only_exact_relations_support_semantic_promotion() {
        assert!(RangeRelation::Exact.supports_semantic_promotion());
        assert!(
            !RangeRelation::SameStartDifferentEnd {
                static_end: 1,
                oracle_end: 2
            }
            .supports_semantic_promotion()
        );
    }

    #[test]
    fn bindings_at_identical_addresses_do_not_overwrite_each_other() {
        let mut graph = BodyGraph::default();
        let body = PhysicalBodyId {
            abi: crate::model::Abi::Arm64V8a,
            entry: 0x1000,
        };
        let first = FunctionOccurrenceId {
            source: SOURCE_STATIC,
            object: 1,
        };
        let second = FunctionOccurrenceId {
            source: SOURCE_STATIC,
            object: 2,
        };
        for function in [first, second] {
            assert!(graph.bind(BodyBinding {
                body,
                function,
                relation: RangeRelation::Exact,
                tier: EvidenceTier::Proven,
                extent_conflict: None,
            }));
        }
        // Re-binding the identical pair is refused, not duplicated.
        assert!(!graph.bind(BodyBinding {
            body,
            function: first,
            relation: RangeRelation::Exact,
            tier: EvidenceTier::Proven,
            extent_conflict: None,
        }));
        graph.recompute_shared();
        assert_eq!(graph.occurrences_at(body), vec![first, second]);
        assert_eq!(graph.shared_bodies, vec![body]);
        assert_eq!(graph.report().shared_body_count, 1);
    }
}
