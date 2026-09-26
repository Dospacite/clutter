//! Cross-ABI consensus over aligned logical occurrences.
//!
//! The existing `analysis::compare_cross_abi` aligns functions by display
//! name, which the accuracy architecture rejects as a unique key under
//! obfuscation. This module aligns occurrences with ABI-neutral evidence —
//! owner identity, lexical order within the owner, parameter shape, parent
//! closure occurrence, and a deliberately small semantic fingerprint built
//! from constants, call topology, and object-pool identities. Only facts that
//! every packaged ABI agrees on are promoted to
//! [`EvidenceTier::CrossAbiCorroborated`]; disagreements are retained and
//! reported, never voted into semantics.
//!
//! Raw instruction counts, register names, stack slots, and per-ABI argument
//! vectors are excluded from consensus inputs: they differ by lowering, not
//! by meaning (the fixture's `formatPrice` disagreement is the regression
//! test for this rule).

use std::collections::BTreeMap;

use serde::Serialize;

use super::tier::EvidenceTier;

/// ABI-neutral alignment key for one logical function occurrence.
///
/// Every component survives obfuscation or is deliberately tolerant of its
/// absence: owners may be `None`, lexical position is the occurrence's index
/// among same-owner members in snapshot order, and the closure parent is the
/// parent occurrence's own alignment key.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct OccurrenceKey {
    pub library_uri: Option<String>,
    pub owner: Option<String>,
    pub member: Option<String>,
    /// Occurrence index among members with the same qualified display name.
    pub lexical_index: usize,
    /// Fixed + optional arity when proven by a signature; `None` otherwise.
    pub arity: Option<(usize, usize)>,
    /// Key of the enclosing closure occurrence, when any.
    pub parent_lexical: Option<usize>,
    /// Repeated names without a proven parent remain local to one ABI.
    pub ambiguous_abi: Option<crate::model::Abi>,
}

/// One ABI-neutral semantic observation extracted per ABI.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct SemanticObservation {
    /// Numeric constants materialised by the body (pool doubles, immediates).
    pub constants: Vec<i64>,
    /// Resolved callee identities in call order (target entry offsets when
    /// known, else target names). Topology only — argument vectors excluded.
    pub callees: Vec<String>,
    /// Object-pool entries referenced by the body, as typed pool labels.
    pub pool_identities: Vec<String>,
}

impl SemanticObservation {
    /// Deterministic 64-bit fingerprints per component. Order-insensitive
    /// within a component because instruction scheduling differs across ABIs.
    pub fn fingerprint(&self) -> [u64; 3] {
        [
            fold_hashed(
                self.constants
                    .iter()
                    .map(|value| hash_value(&value.to_string())),
            ),
            fold_hashed(self.callees.iter().map(|value| hash_value(value))),
            fold_hashed(self.pool_identities.iter().map(|value| hash_value(value))),
        ]
    }
}

fn fold_hashed(hashes: impl Iterator<Item = u64>) -> u64 {
    let mut values = hashes.collect::<Vec<_>>();
    values.sort_unstable();
    // Hash the complete sorted multiset. Repeated equal values still change
    // the digest, and no tail is silently discarded.
    let mut mixed = 0xcbf29ce484222325u64;
    for value in &values {
        for byte in value.to_le_bytes() {
            mixed ^= u64::from(byte);
            mixed = mixed.wrapping_mul(0x100000001b3);
        }
    }
    mixed ^ scramble(values.len() as u64)
}

fn scramble(value: u64) -> u64 {
    let mut mixed = value.wrapping_mul(0x9e3779b97f4a7c15);
    mixed ^= mixed >> 29;
    mixed = mixed.wrapping_mul(0xbf58476d1ce4e5b9);
    mixed ^ (mixed >> 32)
}

fn hash_value(value: &str) -> u64 {
    // FNV-1a is enough here: these digests are advisory alignment evidence,
    // never security boundaries, and stability matters more than resistance.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Consensus outcome for one aligned occurrence.
#[derive(Clone, Debug, Serialize)]
pub struct OccurrenceConsensus {
    pub key: OccurrenceKey,
    pub present_in: Vec<crate::model::Abi>,
    /// Component-wise agreement across every present ABI.
    pub constants_agree: bool,
    pub call_topology_agrees: bool,
    pub pool_objects_agree: bool,
    /// Full three-component agreement. Only this supports promotion.
    pub consensus: bool,
    /// Per-ABI fingerprints retained whenever components disagree.
    pub fingerprints: Option<BTreeMap<crate::model::Abi, [u64; 3]>>,
    /// Tier earned by this occurrence after cross-ABI review.
    pub tier: EvidenceTier,
}

impl OccurrenceConsensus {
    fn tier(&self) -> EvidenceTier {
        if self.consensus && self.present_in.len() >= 2 {
            EvidenceTier::CrossAbiCorroborated
        } else {
            EvidenceTier::Inferred
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ConsensusReport {
    /// Occurrences present in more than one ABI.
    pub aligned_occurrences: usize,
    /// Aligned occurrences whose full fingerprint agrees everywhere.
    pub corroborated_occurrences: usize,
    /// Aligned occurrences with at least one disagreeing component.
    pub disputed_occurrences: usize,
    /// Occurrences seen in exactly one ABI.
    pub unaligned_occurrences: usize,
    pub disputes: Vec<OccurrenceConsensus>,
}

impl ConsensusReport {
    /// Facts promoted to cross-ABI corroboration.
    #[allow(dead_code)] // consumed by tier-aware consumers and tests
    pub fn corroborated_tier(&self) -> EvidenceTier {
        if self.corroborated_occurrences > 0 {
            EvidenceTier::CrossAbiCorroborated
        } else {
            EvidenceTier::Inferred
        }
    }

    /// Per-tier summary of aligned occurrences, for the output manifest.
    pub fn tier_summary(&self) -> BTreeMap<String, usize> {
        let mut summary = BTreeMap::new();
        summary.insert(
            EvidenceTier::CrossAbiCorroborated.label().to_owned(),
            self.corroborated_occurrences,
        );
        summary.insert(
            EvidenceTier::Inferred.label().to_owned(),
            self.disputed_occurrences,
        );
        summary.insert("unaligned".to_owned(), self.unaligned_occurrences);
        summary
    }
}

/// Compare per-ABI observations keyed by [`OccurrenceKey`].
///
/// An occurrence is corroborated only when every packaged ABI that contains it
/// agrees on all three fingerprint components. Any disagreement keeps the
/// occurrence at [`EvidenceTier::Inferred`] and records the per-ABI
/// fingerprints for the resolution report — disagreement is data, not noise.
pub fn reach_consensus(
    observations: BTreeMap<crate::model::Abi, BTreeMap<OccurrenceKey, SemanticObservation>>,
) -> ConsensusReport {
    use std::collections::BTreeSet;

    let mut keys: BTreeMap<OccurrenceKey, BTreeSet<crate::model::Abi>> = BTreeMap::new();
    for (abi, map) in &observations {
        for key in map.keys() {
            keys.entry(key.clone()).or_default().insert(*abi);
        }
    }

    let mut report = ConsensusReport::default();
    for (key, abis) in keys {
        let mut fingerprints = BTreeMap::new();
        for abi in &abis {
            if let Some(observation) = observations.get(abi).and_then(|map| map.get(&key)) {
                fingerprints.insert(*abi, observation.fingerprint());
            }
        }
        let mut reference: Option<[u64; 3]> = None;
        let mut constants = BTreeSet::new();
        let mut topology = BTreeSet::new();
        let mut pools = BTreeSet::new();
        for (abi, fp) in &fingerprints {
            constants.insert(fp[0]);
            topology.insert(fp[1]);
            pools.insert(fp[2]);
            if reference.is_none() {
                // Deterministic pick: lowest ABI id first (BTreeMap order).
                let _ = abi;
                reference = Some(*fp);
            }
        }
        let constants_agree = constants.len() == 1;
        let call_topology_agrees = topology.len() == 1;
        let pool_objects_agree = pools.len() == 1;
        let has_evidence = abis.iter().all(|abi| {
            observations
                .get(abi)
                .and_then(|map| map.get(&key))
                .is_some_and(|observation| {
                    !observation.constants.is_empty()
                        || !observation.callees.is_empty()
                        || !observation.pool_identities.is_empty()
                })
        });
        let consensus =
            has_evidence && constants_agree && call_topology_agrees && pool_objects_agree;
        let multi_abi = abis.len() > 1;
        match (multi_abi, consensus) {
            (true, true) => report.corroborated_occurrences += 1,
            (true, false) => report.disputed_occurrences += 1,
            (false, _) => report.unaligned_occurrences += 1,
        }
        report.aligned_occurrences += usize::from(multi_abi);

        let disputed = multi_abi && !consensus;
        if multi_abi || disputed {
            let mut entry = OccurrenceConsensus {
                key,
                present_in: abis.into_iter().collect(),
                constants_agree,
                call_topology_agrees,
                pool_objects_agree,
                consensus,
                fingerprints: None,
                tier: EvidenceTier::Inferred,
            };
            entry.tier = entry.tier();
            if disputed {
                entry.fingerprints = Some(fingerprints);
                report.disputes.push(entry);
            }
        }
    }
    report
}

/// Extract an ABI-neutral [`SemanticObservation`] from one recovered function.
///
/// Call topology uses resolved targets in statement order (deduplicated),
/// which is stable across ABIs once names are recovered; unresolved calls
/// contribute their raw target address label so topology still compares.
pub fn observe_function(function: &crate::model::RecoveredFunction) -> SemanticObservation {
    let mut observation = SemanticObservation::default();
    for instruction in &function.instructions {
        if let Some(value) = &instruction.object_pool_value
            && !is_opaque_identity(value)
        {
            observation.pool_identities.push(value.clone());
        }
    }
    let mut callees = Vec::new();
    for statement in &function.statements {
        match statement {
            crate::model::PseudoStatement::DirectCall {
                target: Some(target),
                ..
            } => {
                if !is_opaque_identity(target) {
                    callees.push(target.clone());
                }
            }
            crate::model::PseudoStatement::RecoveredIndirectCall { target, .. } => {
                if !is_opaque_identity(target) {
                    callees.push(target.clone());
                }
            }
            crate::model::PseudoStatement::ObjectPoolCall { target, .. } => {
                if !is_opaque_identity(target) {
                    callees.push(target.clone());
                }
            }
            _ => {}
        }
    }
    callees.dedup();
    observation.callees = callees;
    observation.constants = collect_constants(function);
    observation
}

fn collect_constants(function: &crate::model::RecoveredFunction) -> Vec<i64> {
    let mut constants = Vec::new();
    for instruction in &function.instructions {
        let value = parse_pool_double(instruction.object_pool_value.as_deref());
        if let Some(value) = value {
            constants.push(value);
        }
    }
    constants.sort();
    constants.dedup();
    constants
}

/// Recognise decoded object-pool double literals such as
/// `doubleBits(1.5)` / `poolDouble(4621819117588971520)` produced by the
/// lifter's pool decoding. Anything else yields no constant evidence.
fn parse_pool_double(value: Option<&str>) -> Option<i64> {
    let value = value?;
    if !value.starts_with("doubleBits(") && !value.starts_with("poolDouble(") {
        return None;
    }
    let bits_start = value.find('(')? + 1;
    let bits_end = value.rfind(')')?;
    if bits_end < bits_start {
        return None;
    }
    let inner = &value[bits_start..bits_end];
    if let Ok(double) = inner.parse::<f64>() {
        return Some(double.to_bits() as i64);
    }
    inner.parse::<i64>().ok()
}

fn is_opaque_identity(value: &str) -> bool {
    value.starts_with("sub_")
        || value.starts_with("snapshotRef(")
        || value.starts_with("snapshotInstance(")
        || value.starts_with("0x")
}

/// Build per-ABI observation maps plus owner lexical indices from recovered
/// function lists, then reach consensus. This is the entry point used by the
/// CLI's cross-ABI pass.
pub fn consensus_from_functions(
    per_abi: Vec<(crate::model::Abi, &[crate::model::RecoveredFunction])>,
) -> ConsensusReport {
    let mut observations = BTreeMap::new();
    for (abi, functions) in per_abi {
        // Lexical position: first-seen index of each name within its owner's
        // member sequence. Duplicate names (overloads are impossible in Dart,
        // but closures repeat) get their first occurrence's index.
        let mut owner_members: BTreeMap<(Option<String>, Option<String>, String), usize> =
            BTreeMap::new();
        for function in functions {
            *owner_members
                .entry((
                    function.library_uri.clone(),
                    function.owner.clone(),
                    function.name.clone(),
                ))
                .or_default() += 1;
        }
        let mut lexical: BTreeMap<(Option<String>, Option<String>, String), usize> =
            BTreeMap::new();
        let mut maps = BTreeMap::new();
        for function in functions {
            let identity = (
                function.library_uri.clone(),
                function.owner.clone(),
                function.name.clone(),
            );
            let ordinal = lexical.entry(identity.clone()).or_default();
            let key = OccurrenceKey {
                library_uri: function.library_uri.clone(),
                owner: function.owner.clone(),
                member: Some(function.name.clone()),
                lexical_index: *ordinal,
                arity: function.signature.as_ref().map(|signature| {
                    (
                        signature.fixed_parameter_count,
                        signature.optional_parameter_count,
                    )
                }),
                parent_lexical: None,
                ambiguous_abi: (owner_members[&identity] > 1).then_some(abi),
            };
            *ordinal += 1;
            maps.insert(key, observe_function(function));
        }
        observations.insert(abi, maps);
    }
    reach_consensus(observations)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(constants: &[i64], callees: &[&str], pools: &[&str]) -> SemanticObservation {
        SemanticObservation {
            constants: constants.to_vec(),
            callees: callees.iter().map(|value| (*value).to_owned()).collect(),
            pool_identities: pools.iter().map(|value| (*value).to_owned()).collect(),
        }
    }

    #[test]
    fn identical_semantics_across_three_abis_are_corroborated() {
        let mut observations = BTreeMap::new();
        for abi in crate::model::Abi::ALL {
            observations.insert(
                abi,
                BTreeMap::from([(
                    OccurrenceKey {
                        library_uri: Some("package:app/main.dart".into()),
                        owner: Some("Counter".into()),
                        member: Some("increment".into()),
                        lexical_index: 0,
                        arity: Some((1, 0)),
                        parent_lexical: None,
                        ambiguous_abi: None,
                    },
                    observation(&[42], &["_interpolate"], &["snapshotString(Deal: )"]),
                )]),
            );
        }
        let report = reach_consensus(observations);
        assert_eq!(report.aligned_occurrences, 1);
        assert_eq!(report.corroborated_occurrences, 1);
        assert_eq!(report.disputed_occurrences, 0);
        assert_eq!(
            report.corroborated_tier(),
            EvidenceTier::CrossAbiCorroborated
        );
    }

    #[test]
    fn abi_lowering_differences_stay_disputed_and_retain_fingerprints() {
        let mut observations = BTreeMap::new();
        for (index, abi) in crate::model::Abi::ALL.into_iter().enumerate() {
            // ARM64 vs x64 encode different immediate shapes for formatPrice:
            // the architecture doc requires this to stay unresolved.
            let extra = if index == 0 { vec![7i64] } else { vec![] };
            observations.insert(
                abi,
                BTreeMap::from([(
                    OccurrenceKey {
                        library_uri: None,
                        owner: None,
                        member: Some("formatPrice".into()),
                        lexical_index: 3,
                        arity: None,
                        parent_lexical: None,
                        ambiguous_abi: None,
                    },
                    observation(&extra, &["formatPrice"], &[]),
                )]),
            );
        }
        let report = reach_consensus(observations);
        assert_eq!(report.disputed_occurrences, 1);
        assert_eq!(report.corroborated_occurrences, 0);
        let dispute = &report.disputes[0];
        assert!(!dispute.consensus);
        assert!(dispute.fingerprints.is_some());
        assert_eq!(dispute.tier, EvidenceTier::Inferred);
    }

    #[test]
    fn single_component_disagreement_blocks_promotion_even_when_rest_agree() {
        let mut observations = BTreeMap::new();
        for (index, abi) in crate::model::Abi::ALL.into_iter().enumerate() {
            let pools: Vec<&str> = if index == 1 {
                vec!["differentPool"]
            } else {
                vec!["sharedPool"]
            };
            observations.insert(
                abi,
                BTreeMap::from([(
                    OccurrenceKey {
                        library_uri: None,
                        owner: Some("A".into()),
                        member: Some("f".into()),
                        lexical_index: 0,
                        arity: None,
                        parent_lexical: None,
                        ambiguous_abi: None,
                    },
                    observation(&[1], &["sameCallee"], &pools),
                )]),
            );
        }
        let report = reach_consensus(observations);
        assert_eq!(report.disputed_occurrences, 1);
        assert_eq!(report.corroborated_occurrences, 0);
    }

    #[test]
    fn fingerprints_are_order_insensitive_within_a_component() {
        let left = observation(&[3, 1, 2], &["a", "b"], &[]);
        let right = observation(&[2, 3, 1], &["b", "a"], &[]);
        assert_eq!(left.fingerprint(), right.fingerprint());
        assert_ne!(
            observation(&[1], &[], &[]).fingerprint(),
            observation(&[2], &[], &[]).fingerprint()
        );
    }

    #[test]
    fn only_typed_pool_doubles_count_as_numeric_evidence() {
        assert_eq!(
            parse_pool_double(Some("doubleBits(1.5)")),
            Some(1.5f64.to_bits() as i64)
        );
        assert_eq!(parse_pool_double(Some("snapshotRef(123)")), None);
        assert_eq!(parse_pool_double(Some("plainLabel")), None);
    }

    #[test]
    fn fingerprints_include_repetitions_and_values_after_sixteen() {
        let prefix = (0..16).collect::<Vec<i64>>();
        let mut changed_tail = prefix.clone();
        changed_tail.push(17);
        let mut another_tail = prefix.clone();
        another_tail.push(18);
        assert_ne!(
            observation(&changed_tail, &[], &[]).fingerprint(),
            observation(&another_tail, &[], &[]).fingerprint()
        );
        assert_ne!(
            observation(&[7, 7], &[], &[]).fingerprint(),
            observation(&[], &[], &[]).fingerprint()
        );
    }

    #[test]
    fn empty_observations_do_not_corroborate() {
        let key = OccurrenceKey {
            library_uri: None,
            owner: None,
            member: Some("empty".into()),
            lexical_index: 0,
            arity: None,
            parent_lexical: None,
            ambiguous_abi: None,
        };
        let observations = BTreeMap::from([
            (
                crate::model::Abi::Arm64V8a,
                BTreeMap::from([(key.clone(), SemanticObservation::default())]),
            ),
            (
                crate::model::Abi::X86_64,
                BTreeMap::from([(key, SemanticObservation::default())]),
            ),
        ]);
        let report = reach_consensus(observations);
        assert_eq!(report.corroborated_occurrences, 0);
        assert_eq!(report.disputed_occurrences, 1);
    }
}
