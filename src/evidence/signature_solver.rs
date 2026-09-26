//! Signature and call-shape constraint solving from static evidence.
//!
//! Every problem is one code occurrence (a physical body at an exact entry
//! address), never a display name: same-named members of different
//! libraries and anonymous closures stay distinct.
//!
//! Authoritative metadata is attached first:
//!
//! - a retained runtime `FunctionType` proves the *compiled* callable shape
//!   (`retained_runtime_signature`). It describes the program after TFA
//!   signature shaking, which can remove unused or constant parameters and
//!   turn always-passed named parameters into positional ones, so it is not
//!   a statement about the original source declaration;
//! - an exactly bound VM oracle Function proves the same retained shape
//!   (`vm_oracle`).
//!
//! Call sites then contribute typed observations. A direct AOT call loads
//! an `ArgumentsDescriptor` into `ARGS_DESC_REG` immediately before the call
//! exactly when the callee's prologue needs one (`IsGeneric() ||
//! HasOptionalParameters()`, see `EmitOptimizedStaticCall`), so:
//!
//! - a descriptor records the supplied shape of that call: the supplied
//!   value-argument count, how many were positional, and the named
//!   arguments the callee must accept;
//! - its absence at a direct call means the callee takes neither optional
//!   parameters nor a type-argument vector.
//!
//! Minimum required count, supplied counts and total callable capacity are
//! solved separately: agreement between call sites never proves that no
//! optional parameter exists, and an unobserved count is unknown rather
//! than zero.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::call_shape::ArgumentsShape;
use super::tier::EvidenceTier;

/// Solved outcome per code occurrence, keyed by entry address.
pub type SignatureSolutions = BTreeMap<u64, SolvedSignature>;

/// `--emit-ir` representation for [`SignatureSolutions`].
pub(crate) fn serialize_solutions<S>(
    solutions: &Option<SignatureSolutions>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    #[derive(serde::Serialize)]
    struct Entry<'a> {
        address: String,
        #[serde(flatten)]
        solved: &'a SolvedSignature,
    }

    match solutions {
        Some(solutions) => {
            let entries: Vec<Entry<'_>> = solutions
                .iter()
                .map(|(address, solved)| Entry {
                    address: format!("0x{address:x}"),
                    solved,
                })
                .collect();
            serde::Serialize::serialize(&entries, serializer)
        }
        None => serializer.serialize_none(),
    }
}

/// Authoritative parameter counts: (fixed, optional, optional-named,
/// implicit).
pub type DescriptorShape = (usize, usize, bool, usize);

/// Where an authoritative shape came from.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeAuthority {
    /// The snapshot's retained FunctionType for this Function.
    RetainedRuntimeSignature,
    /// An exactly bound Dart VM oracle Function.
    VmOracle,
}

/// Logical identity reported with an occurrence.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct OccurrenceIdentity {
    pub library_uri: Option<String>,
    pub owner: Option<String>,
    pub name: String,
}

/// One typed call-site observation of a callee occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CallObservation {
    /// Address of the calling instruction.
    pub call_address: String,
    /// The descriptor the call passed, when it passed one.
    pub supplied: Option<ArgumentsShape>,
    pub rule: &'static str,
}

/// Solved parameter-shape outcome for one occurrence.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShapeOutcome {
    /// The retained callable shape, from an authority.
    Proven {
        fixed: usize,
        optional: usize,
        optional_named: bool,
        implicit: usize,
        authority: ShapeAuthority,
    },
    /// Call-site constraints only. Every field is a separate fact.
    Constrained {
        /// Distinct supplied value-argument counts seen in descriptors.
        supplied_counts: Vec<usize>,
        /// Required positional parameters cannot exceed the fewest
        /// positional arguments any descriptor supplied.
        required_positional_at_most: Option<usize>,
        /// Capacity is at least the most arguments any call supplied.
        capacity_at_least: Option<usize>,
        /// Named parameters the callee must accept (names may be
        /// obfuscated tokens).
        named_accepted: Vec<String>,
        /// Whether the prologue reads an arguments descriptor (optional
        /// parameters or generic). `None` when calls disagree or none were
        /// observed as direct calls.
        needs_arguments_descriptor: Option<bool>,
        /// Type-argument vector lengths supplied.
        type_argument_counts: Vec<usize>,
        calls: usize,
    },
    /// No usable constraints. Erasure is real; do not guess.
    Unknown,
}

impl ShapeOutcome {
    pub fn tier(&self) -> EvidenceTier {
        match self {
            Self::Proven { .. } => EvidenceTier::Proven,
            Self::Constrained { .. } => EvidenceTier::Inferred,
            Self::Unknown => EvidenceTier::Speculative,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Proven { .. } => "proven",
            Self::Constrained { .. } => "constrained",
            Self::Unknown => "unknown",
        }
    }
}

/// Receiver-type constraint derived from allocation shapes / CIDs.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReceiverConstraint {
    pub class_id: i64,
    pub rule: &'static str,
}

/// Solver input for one code occurrence.
#[derive(Clone, Debug, Default)]
pub struct SignatureProblem {
    pub address: u64,
    pub identity: OccurrenceIdentity,
    /// Other logical functions sharing this physical body.
    pub alternatives: Vec<OccurrenceIdentity>,
    /// Retained runtime FunctionType counts.
    pub retained: Option<DescriptorShape>,
    /// Exactly bound oracle counts.
    pub descriptor: Option<DescriptorShape>,
    pub observations: Vec<CallObservation>,
    pub receivers: Vec<ReceiverConstraint>,
}

/// Solved result for one occurrence.
#[derive(Clone, Debug, Serialize)]
pub struct SolvedSignature {
    #[serde(flatten)]
    pub identity: OccurrenceIdentity,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<OccurrenceIdentity>,
    pub outcome: ShapeOutcome,
    pub tier: EvidenceTier,
    /// Call-site observations, kept even when an authority decided the
    /// outcome so disagreements stay inspectable.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<CallObservation>,
    /// Receiver CID bounds retained for downstream type narrowing.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub receiver_class_ids: Vec<i64>,
    /// Rule identifiers that contributed, in stable order.
    pub rules: Vec<String>,
}

impl SolvedSignature {
    /// Named parameters a call site proved the callee accepts.
    pub fn named_accepted(&self) -> &[String] {
        match &self.outcome {
            ShapeOutcome::Constrained { named_accepted, .. } => named_accepted,
            _ => &[],
        }
    }
}

/// Solve every accumulated problem.
///
/// Deterministic: identical inputs produce identical outputs, and no source
/// can raise another's tier.
pub fn solve(problems: &[SignatureProblem]) -> SignatureSolutions {
    let mut results = BTreeMap::new();
    for problem in problems {
        let mut rules: Vec<String> = problem
            .observations
            .iter()
            .map(|observation| observation.rule.to_owned())
            .collect();
        if problem.retained.is_some() {
            rules.push("retained_runtime_signature".to_owned());
        }
        if problem.descriptor.is_some() {
            rules.push("vm_oracle".to_owned());
        }
        rules.sort();
        rules.dedup();
        let outcome = solve_one(problem);
        results.insert(
            problem.address,
            SolvedSignature {
                identity: problem.identity.clone(),
                alternatives: problem.alternatives.clone(),
                tier: outcome.tier(),
                outcome,
                observations: problem.observations.clone(),
                receiver_class_ids: problem
                    .receivers
                    .iter()
                    .map(|receiver| receiver.class_id)
                    .collect(),
                rules,
            },
        );
    }
    results
}

fn solve_one(problem: &SignatureProblem) -> ShapeOutcome {
    for (shape, authority) in [
        (problem.descriptor, ShapeAuthority::VmOracle),
        (problem.retained, ShapeAuthority::RetainedRuntimeSignature),
    ] {
        if let Some((fixed, optional, optional_named, implicit)) = shape {
            return ShapeOutcome::Proven {
                fixed,
                optional,
                optional_named,
                implicit,
                authority,
            };
        }
    }
    if problem.observations.is_empty() {
        return ShapeOutcome::Unknown;
    }
    let supplied = problem
        .observations
        .iter()
        .filter_map(|observation| observation.supplied.as_ref())
        .collect::<Vec<_>>();
    let with_descriptor = problem
        .observations
        .iter()
        .any(|observation| observation.supplied.is_some());
    let without_descriptor = problem
        .observations
        .iter()
        .any(|observation| observation.supplied.is_none());
    let needs_arguments_descriptor = match (with_descriptor, without_descriptor) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    };
    ShapeOutcome::Constrained {
        supplied_counts: supplied
            .iter()
            .map(|shape| shape.count)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        required_positional_at_most: supplied.iter().map(|shape| shape.positional).min(),
        capacity_at_least: supplied.iter().map(|shape| shape.count).max(),
        named_accepted: supplied
            .iter()
            .flat_map(|shape| shape.named_names().map(str::to_owned))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        needs_arguments_descriptor,
        type_argument_counts: supplied
            .iter()
            .map(|shape| shape.type_args_len)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        calls: problem.observations.len(),
    }
}

/// Builds one problem per recovered body, attaching the retained runtime
/// signature and any exactly bound oracle counts.
pub fn problems_for(functions: &[crate::model::RecoveredFunction]) -> Vec<SignatureProblem> {
    let mut problems = BTreeMap::<u64, SignatureProblem>::new();
    for function in functions {
        let Some(address) = parse_address(&function.address) else {
            continue;
        };
        let retained = function.signature.as_ref().map(|signature| {
            (
                signature.fixed_parameter_count,
                signature.optional_parameter_count,
                signature.optional_parameters_are_named,
                signature.implicit_parameter_count,
            )
        });
        let descriptor = function.vm_evidence.as_ref().and_then(|evidence| {
            Some((
                evidence.fixed_parameter_count?,
                evidence.optional_parameter_count?,
                evidence.optional_parameters_are_named?,
                evidence.implicit_parameter_count?,
            ))
        });
        let identity = OccurrenceIdentity {
            library_uri: function.library_uri.clone(),
            owner: function.owner.clone(),
            name: function.name.clone(),
        };
        // Call sites reach the physical body, so every logical function
        // sharing it is an alternative interpretation of the same calls.
        if let Some(existing) = problems.get_mut(&address) {
            if existing.identity != identity && !existing.alternatives.contains(&identity) {
                existing.alternatives.push(identity);
            }
            continue;
        }
        problems.insert(
            address,
            SignatureProblem {
                address,
                identity,
                alternatives: Vec::new(),
                retained,
                descriptor,
                observations: Vec::new(),
                receivers: Vec::new(),
            },
        );
    }
    problems.into_values().collect()
}

/// Register a direct call loads its arguments descriptor into
/// (`ARGS_DESC_REG`).
fn arguments_descriptor_register(abi: crate::model::Abi) -> &'static str {
    match abi {
        crate::model::Abi::Arm64V8a => "x4",
        crate::model::Abi::ArmeabiV7a => "r4",
        crate::model::Abi::X86_64 => "r10",
    }
}

/// Adds every direct call in `caller` as an observation of its callee.
///
/// Callees are matched by exact code address (normal or unchecked entry).
/// The descriptor must be the pool load into `ARGS_DESC_REG` that the
/// compiler emits immediately before the call; an intervening call, branch
/// or unrelated write to that register means no descriptor was passed.
pub fn accumulate_call_sites(
    abi: crate::model::Abi,
    caller: &crate::model::RecoveredFunction,
    entries: &BTreeMap<u64, u64>,
    problems: &mut BTreeMap<u64, SignatureProblem>,
) {
    let descriptor_register = arguments_descriptor_register(abi);
    for (index, instruction) in caller.instructions.iter().enumerate() {
        let Some(target) = direct_call_target(&instruction.mnemonic, &instruction.operands) else {
            continue;
        };
        let Some(occurrence) = entries.get(&target).copied() else {
            continue;
        };
        let Some(problem) = problems.get_mut(&occurrence) else {
            continue;
        };
        let mut supplied = None;
        // `LoadObject` is at most an address-forming add plus a load.
        for previous in caller.instructions[..index].iter().rev().take(2) {
            let destination = previous
                .operands
                .split(',')
                .next()
                .map(crate::analysis::disassembly::normalized_register);
            if destination.as_deref() != Some(descriptor_register) {
                continue;
            }
            supplied = previous
                .object_pool_value
                .as_deref()
                .and_then(ArgumentsShape::from_label);
            break;
        }
        problem.observations.push(CallObservation {
            call_address: instruction.address.clone(),
            rule: if supplied.is_some() {
                "direct_call_descriptor"
            } else {
                "direct_call_without_descriptor"
            },
            supplied,
        });
    }
}

fn direct_call_target(mnemonic: &str, operands: &str) -> Option<u64> {
    if !matches!(mnemonic, "bl" | "blx" | "call" | "callq") {
        return None;
    }
    parse_address(operands.split(',').next()?.trim().trim_start_matches('#'))
}

fn parse_address(value: &str) -> Option<u64> {
    let digits = value.trim().trim_start_matches("0x");
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

/// Maps every call entry (normal and unchecked) to its occurrence address.
pub fn entry_addresses(functions: &[crate::model::RecoveredFunction]) -> BTreeMap<u64, u64> {
    let mut entries = BTreeMap::new();
    for function in functions {
        let Some(address) = parse_address(&function.address) else {
            continue;
        };
        entries.entry(address).or_insert(address);
        if let Some(offset) = function
            .code_metadata
            .as_ref()
            .and_then(|metadata| metadata.unchecked_entry_offset)
            .filter(|offset| *offset > 0 && *offset < function.size)
        {
            entries.entry(address + offset).or_insert(address);
        }
    }
    entries
}

/// Solves call shapes for `functions`, using `callers` (typically every
/// recovered body, in or out of the output scope) as call-site evidence.
pub fn solve_program(
    abi: crate::model::Abi,
    functions: &[crate::model::RecoveredFunction],
    callers: &[crate::model::RecoveredFunction],
) -> SignatureSolutions {
    let mut problems = problems_for(functions)
        .into_iter()
        .map(|problem| (problem.address, problem))
        .collect::<BTreeMap<_, _>>();
    let entries = entry_addresses(functions);
    let mut seen = BTreeSet::new();
    for caller in callers.iter().chain(functions) {
        // Scope-limited and full lists overlap; count each body once.
        if !seen.insert(caller.address.clone()) {
            continue;
        }
        accumulate_call_sites(abi, caller, &entries, &mut problems);
    }
    solve(&problems.into_values().collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem(address: u64, supplied: &[Option<ArgumentsShape>]) -> SignatureProblem {
        SignatureProblem {
            address,
            identity: OccurrenceIdentity {
                library_uri: Some("package:a/a.dart".to_owned()),
                owner: Some("C".to_owned()),
                name: "m".to_owned(),
            },
            observations: supplied
                .iter()
                .map(|shape| CallObservation {
                    call_address: "0x0".to_owned(),
                    supplied: shape.clone(),
                    rule: "test",
                })
                .collect(),
            ..SignatureProblem::default()
        }
    }

    fn shape(count: usize, positional: usize, named: &[(&str, usize)]) -> ArgumentsShape {
        ArgumentsShape {
            type_args_len: 0,
            count,
            size: 0,
            positional,
            named: named
                .iter()
                .map(|(name, position)| ((*name).to_owned(), *position))
                .collect(),
        }
    }

    fn bare_function() -> crate::model::RecoveredFunction {
        crate::model::RecoveredFunction {
            code_reference: 0,
            code_alias_references: Vec::new(),
            name: "caller".to_owned(),
            name_source: crate::model::RecoveredNameSource::Snapshot,
            snapshot_name: None,
            obfuscated_name: None,
            owner: None,
            library_uri: None,
            source_location: None,
            inlined_functions: Vec::new(),
            inline_regions: Vec::new(),
            kind: None,
            is_static: None,
            loading_unit: None,
            parameter_defaults: Default::default(),
            async_modifier: None,
            lexical_parent: None,
            signature: None,
            signature_source: None,
            parameter_count: None,
            machine_interface: None,
            vm_evidence: None,
            address: "0x100".to_owned(),
            size: 0x20,
            code_metadata: None,
            machine_code: crate::model::MachineCodeEvidence::default(),
            instructions: Vec::new(),
            control_flow: Vec::new(),
            semantic_statements: Vec::new(),
            source_bands: BTreeMap::new(),
            statements: Vec::new(),
        }
    }

    #[test]
    fn authorities_win_and_name_their_source() {
        let mut oracle = problem(0x10, &[Some(shape(2, 2, &[]))]);
        oracle.descriptor = Some((4, 1, true, 1));
        oracle.retained = Some((3, 0, false, 1));
        let solved = solve(&[oracle]);
        assert_eq!(
            solved[&0x10].outcome,
            ShapeOutcome::Proven {
                fixed: 4,
                optional: 1,
                optional_named: true,
                implicit: 1,
                authority: ShapeAuthority::VmOracle,
            }
        );
        assert!(solved[&0x10].tier.at_least(EvidenceTier::Proven));
    }

    #[test]
    fn agreeing_call_sites_do_not_prove_optional_parameters_absent() {
        let solved = solve(&[problem(
            0x10,
            &[Some(shape(2, 2, &[])), Some(shape(2, 2, &[]))],
        )]);
        let ShapeOutcome::Constrained {
            supplied_counts,
            required_positional_at_most,
            capacity_at_least,
            needs_arguments_descriptor,
            ..
        } = &solved[&0x10].outcome
        else {
            panic!("constrained expected");
        };
        assert_eq!(supplied_counts, &vec![2]);
        assert_eq!(*required_positional_at_most, Some(2));
        assert_eq!(*capacity_at_least, Some(2));
        // A descriptor was passed, so the prologue needs one: optional
        // parameters (or a type-argument vector) exist even though every
        // call supplied the same count.
        assert_eq!(*needs_arguments_descriptor, Some(true));
    }

    #[test]
    fn named_arguments_from_descriptors_are_accepted_names() {
        let solved = solve(&[problem(
            0x10,
            &[
                Some(shape(3, 1, &[("price", 1), ("name", 2)])),
                Some(shape(2, 1, &[("category", 1)])),
            ],
        )]);
        assert_eq!(
            solved[&0x10].named_accepted(),
            &["category".to_owned(), "name".to_owned(), "price".to_owned()]
        );
    }

    #[test]
    fn descriptor_free_calls_rule_out_optional_parameters_only() {
        let solved = solve(&[problem(0x10, &[None, None])]);
        let ShapeOutcome::Constrained {
            needs_arguments_descriptor,
            supplied_counts,
            required_positional_at_most,
            ..
        } = &solved[&0x10].outcome
        else {
            panic!("constrained expected");
        };
        assert_eq!(*needs_arguments_descriptor, Some(false));
        // The count itself was never observed: unknown, not zero.
        assert!(supplied_counts.is_empty());
        assert_eq!(*required_positional_at_most, None);
    }

    #[test]
    fn same_named_occurrences_stay_distinct() {
        let first = problem(0x10, &[]);
        let mut second = problem(0x20, &[Some(shape(1, 1, &[]))]);
        second.identity.library_uri = Some("package:b/b.dart".to_owned());
        let solved = solve(&[first, second]);
        assert_eq!(solved.len(), 2);
        assert_eq!(solved[&0x10].outcome, ShapeOutcome::Unknown);
        assert_eq!(solved[&0x20].outcome.label(), "constrained");
    }

    #[test]
    fn observes_descriptors_loaded_immediately_before_direct_calls() {
        use crate::model::MachineInstruction;
        let instruction = |address: &str, mnemonic: &str, operands: &str, pool: Option<&str>| {
            MachineInstruction {
                address: address.to_owned(),
                bytes: "00000000".to_owned(),
                mnemonic: mnemonic.to_owned(),
                operands: operands.to_owned(),
                object_pool_index: pool.map(|_| 1),
                object_pool_value: pool.map(str::to_owned),
            }
        };
        let label = shape(2, 1, &[("colorScheme", 1)]).label();
        let mut caller = bare_function();
        caller.instructions = vec![
            instruction("0x100", "ldr", "x4, [x27, #0x40]", Some(&label)),
            instruction("0x104", "bl", "#0x200", None),
            instruction("0x108", "ldr", "x4, [x27, #0x48]", Some(&label)),
            instruction("0x10c", "mov", "x2, x0", None),
            instruction("0x110", "mov", "x3, x0", None),
            instruction("0x114", "bl", "#0x200", None),
        ];
        let mut problems = BTreeMap::from([(0x200, problem(0x200, &[]))]);
        let entries = BTreeMap::from([(0x200, 0x200)]);
        accumulate_call_sites(
            crate::model::Abi::Arm64V8a,
            &caller,
            &entries,
            &mut problems,
        );
        let observations = &problems[&0x200].observations;
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].rule, "direct_call_descriptor");
        // Two instructions separate the second load from its call: the
        // compiler emits LoadObject(ARGS_DESC_REG) adjacent to the call, so
        // this is not the call's descriptor.
        assert_eq!(observations[1].rule, "direct_call_without_descriptor");
    }
}
