//! Runtime trace ingestion: dynamic evidence from an emulator or device.
//!
//! A trace records what actually executed: PCs hit, receiver class ids seen
//! at dispatch sites, dispatch targets taken, and argument descriptors of
//! called functions. This is *dynamic* evidence and is deliberately kept in a
//! separate namespace from static facts:
//!
//! - trace facts never promote, replace, or overwrite static/proven claims;
//! - they attach to occurrences as refinement candidates with their own tier
//!   ([`TraceFact`]) and can only narrow *ambiguity* (e.g. pick between
//!   disputed cross-ABI variants), never manufacture semantics;
//! - the merge result records every refinement so it remains auditable.
//!
//! The schema version gates acceptance: unknown versions are rejected rather
//! than parsed loosely.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::tier::EvidenceTier;

pub const TRACE_SCHEMA: &str = "clutter.runtime-trace/v1";

/// One executed-PC observation inside one function body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExecutedPc {
    /// Isolate-instruction offset of the body entry.
    pub body_entry_offset: u64,
    /// Offset relative to the body entry.
    pub pc_offset: u64,
    pub hit_count: u64,
}

/// Receiver class id observed at one dispatch/call site.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObservedReceiverCid {
    pub call_site_pc_offset: u64,
    pub class_id: i64,
    pub hit_count: u64,
}

/// Dispatch target actually taken from one selector slot.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DispatchObservation {
    pub selector_index: u64,
    /// Isolate-instruction offset of the target that executed.
    pub target_offset: u64,
    pub hit_count: u64,
}

/// Argument descriptor observed for one callee invocation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObservedArgumentDescriptor {
    pub callee_offset: u64,
    pub positional_count: usize,
    #[serde(default)]
    pub named_names: Vec<String>,
    pub hit_count: u64,
}

/// Full runtime-trace document.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuntimeTrace {
    pub schema: String,
    /// ABI the trace was collected on.
    pub abi: crate::model::Abi,
    /// Snapshot hash of the payload that produced the trace. Traces from a
    /// different snapshot must not refine this subject's evidence.
    pub snapshot_hash: String,
    #[serde(default)]
    pub executed_pcs: Vec<ExecutedPc>,
    #[serde(default)]
    pub receiver_cids: Vec<ObservedReceiverCid>,
    #[serde(default)]
    pub dispatch_targets: Vec<DispatchObservation>,
    #[serde(default)]
    pub argument_descriptors: Vec<ObservedArgumentDescriptor>,
}

#[derive(Debug, thiserror::Error)]
pub enum TraceError {
    #[error("runtime trace schema {found:?} is not {expected:?}")]
    SchemaMismatch { found: String, expected: String },
    #[error("failed to parse runtime trace: {0}")]
    Parse(#[from] serde_json::Error),
}

impl RuntimeTrace {
    /// Load and validate a trace document.
    pub fn load(bytes: &[u8]) -> Result<Self, TraceError> {
        let trace: Self = serde_json::from_slice(bytes)?;
        if trace.schema != TRACE_SCHEMA {
            return Err(TraceError::SchemaMismatch {
                found: trace.schema,
                expected: TRACE_SCHEMA.to_owned(),
            });
        }
        Ok(trace)
    }

    /// Bodies actually entered during the run, with total hit counts.
    pub fn hot_bodies(&self) -> BTreeMap<u64, u64> {
        let mut bodies = BTreeMap::new();
        for pc in &self.executed_pcs {
            *bodies.entry(pc.body_entry_offset).or_insert(0) += pc.hit_count;
        }
        bodies
    }

    /// For each dispatch slot, the targets observed (with counts).
    pub fn dispatch_profile(&self) -> BTreeMap<u64, Vec<(u64, u64)>> {
        let mut profile: BTreeMap<u64, Vec<(u64, u64)>> = BTreeMap::new();
        for observation in &self.dispatch_targets {
            profile
                .entry(observation.selector_index)
                .or_default()
                .push((observation.target_offset, observation.hit_count));
        }
        profile
    }
}

/// A refinement candidate produced by merging trace evidence into resolved
/// static facts. Never mutates the static claim; consumers decide.
#[derive(Clone, Debug, Serialize)]
pub struct TraceFact<T> {
    pub value: T,
    /// Dynamic evidence lives below proven tiers by construction: even a
    /// million executions cannot prove source-level semantics.
    pub tier: EvidenceTier,
    pub observations: u64,
}

impl<T> TraceFact<T> {
    pub fn new(value: T, observations: u64) -> Self {
        Self {
            value,
            // Execution frequency ranks plausibility, not proof.
            tier: EvidenceTier::Inferred,
            observations,
        }
    }
}

/// Refinement output: what the trace suggests, per category, without having
/// touched any static fact.
#[derive(Clone, Debug, Default, Serialize)]
pub struct TraceRefinement {
    /// Body offsets confirmed reachable (executed at least once).
    pub executed_bodies: Vec<TraceFact<u64>>,
    /// For each selector slot, the most frequently taken target offset.
    pub dominant_dispatch_targets: Vec<(u64, TraceFact<u64>)>,
    /// Observed positional arities per callee offset.
    pub observed_arities: Vec<(u64, TraceFact<usize>)>,
    /// Receiver CIDs observed per call-site offset.
    pub observed_receivers: Vec<(u64, TraceFact<i64>)>,
}

impl TraceRefinement {
    /// Derive refinements from a validated trace.
    pub fn derive(trace: &RuntimeTrace) -> Self {
        let mut refinement = TraceRefinement::default();

        for (offset, hits) in trace.hot_bodies() {
            refinement
                .executed_bodies
                .push(TraceFact::new(offset, hits));
        }

        for (selector, mut targets) in trace.dispatch_profile() {
            targets.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
            if let Some((target_offset, hits)) = targets.first().copied() {
                refinement
                    .dominant_dispatch_targets
                    .push((selector, TraceFact::new(target_offset, hits)));
            }
        }

        let mut arities: BTreeMap<u64, (usize, u64)> = BTreeMap::new();
        for descriptor in &trace.argument_descriptors {
            let entry = arities
                .entry(descriptor.callee_offset)
                .or_insert((descriptor.positional_count, 0));
            entry.1 += descriptor.hit_count;
            entry.0 = entry.0.max(descriptor.positional_count);
        }
        for (offset, (positional, hits)) in arities {
            refinement
                .observed_arities
                .push((offset, TraceFact::new(positional, hits)));
        }

        let mut receivers: BTreeMap<u64, BTreeMap<i64, u64>> = BTreeMap::new();
        for observation in &trace.receiver_cids {
            *receivers
                .entry(observation.call_site_pc_offset)
                .or_default()
                .entry(observation.class_id)
                .or_insert(0) += observation.hit_count;
        }
        for (site, cids) in receivers {
            for (class_id, hits) in cids {
                refinement
                    .observed_receivers
                    .push((site, TraceFact::new(class_id, hits)));
            }
        }

        refinement
            .dominant_dispatch_targets
            .sort_by_key(|(selector, _)| *selector);
        refinement
            .observed_arities
            .sort_by_key(|(offset, _)| *offset);
        refinement.observed_receivers.sort_by_key(|(site, _)| *site);
        refinement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "schema": "clutter.runtime-trace/v1",
        "abi": "arm64-v8a",
        "snapshot_hash": "abc123",
        "executed_pcs": [
            {"body_entry_offset": 4096, "pc_offset": 0, "hit_count": 3},
            {"body_entry_offset": 4096, "pc_offset": 16, "hit_count": 1},
            {"body_entry_offset": 8192, "pc_offset": 0, "hit_count": 7}
        ],
        "receiver_cids": [
            {"call_site_pc_offset": 32, "class_id": 62, "hit_count": 5}
        ],
        "dispatch_targets": [
            {"selector_index": 100, "target_offset": 8192, "hit_count": 2},
            {"selector_index": 100, "target_offset": 4096, "hit_count": 9},
            {"selector_index": 200, "target_offset": 4096, "hit_count": 1}
        ],
        "argument_descriptors": [
            {"callee_offset": 4096, "positional_count": 2, "named_names": [], "hit_count": 4},
            {"callee_offset": 8192, "positional_count": 1, "named_names": ["x"], "hit_count": 6}
        ]
    }"#;

    #[test]
    fn loads_valid_trace_and_rejects_wrong_schema() {
        let trace = RuntimeTrace::load(SAMPLE.as_bytes()).unwrap();
        assert_eq!(trace.abi, crate::model::Abi::Arm64V8a);

        let bad = SAMPLE.replace("v1", "v999");
        assert!(matches!(
            RuntimeTrace::load(bad.as_bytes()),
            Err(TraceError::SchemaMismatch { .. })
        ));
    }

    #[test]
    fn hot_bodies_sum_hit_counts_per_body() {
        let trace = RuntimeTrace::load(SAMPLE.as_bytes()).unwrap();
        let hot = trace.hot_bodies();
        assert_eq!(hot[&4096], 4);
        assert_eq!(hot[&8192], 7);
    }

    #[test]
    fn dominant_dispatch_target_is_most_frequent_with_stable_ties() {
        let trace = RuntimeTrace::load(SAMPLE.as_bytes()).unwrap();
        let refinement = TraceRefinement::derive(&trace);
        let slot_100 = refinement
            .dominant_dispatch_targets
            .iter()
            .find(|(selector, _)| *selector == 100)
            .unwrap();
        assert_eq!(slot_100.1.value, 4096);
        assert_eq!(slot_100.1.observations, 9);
        // Dynamic evidence never claims proven strength.
        assert!(!slot_100.1.tier.at_least(EvidenceTier::Proven));
    }

    #[test]
    fn observed_arities_keep_maximum_positional_count() {
        let trace = RuntimeTrace::load(SAMPLE.as_bytes()).unwrap();
        let refinement = TraceRefinement::derive(&trace);
        let arity_4096 = refinement
            .observed_arities
            .iter()
            .find(|(offset, _)| *offset == 4096)
            .unwrap();
        assert_eq!(arity_4096.1.value, 2);
        assert_eq!(arity_4096.1.observations, 4);
    }

    #[test]
    fn receiver_cids_are_grouped_by_call_site() {
        let trace = RuntimeTrace::load(SAMPLE.as_bytes()).unwrap();
        let refinement = TraceRefinement::derive(&trace);
        assert_eq!(refinement.observed_receivers.len(), 1);
        assert_eq!(refinement.observed_receivers[0].1.value, 62);
    }
}
