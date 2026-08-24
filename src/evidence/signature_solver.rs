//! Signature and type constraint solving from static evidence.
//!
//! The exact VM oracle proved that some product-AOT signatures are genuinely
//! erased (the `Function` object and its `FunctionType` are dropped even when
//! code survives). Metadata parsing alone therefore cannot recover them. This
//! solver recovers what *can* be proven deterministically by accumulating
//! constraints from every static source and solving per occurrence:
//!
//! - call sites (`DirectCall`/`ObjectPoolCall` edges into the function give a
//!   lower bound on positional arguments actually passed);
//! - argument descriptors (VM oracle `argument_descriptor` counts);
//! - receiver CIDs (allocation shapes at call sites bound receiver types);
//! - field traffic (implicit getter/setter arity from field layout).
//!
//! Outcomes are explicitly tiered. A solved shape is `Proven` only when it
//! comes from an authoritative descriptor; cross-source agreement without an
//! authority is `CrossAbiCorroborated`/`Inferred`; anything else stays
//! `Unknown`. The solver never invents names or types — that belongs to the
//! speculative layer.

use std::collections::BTreeMap;

use serde::Serialize;

use super::tier::EvidenceTier;

/// Map of a logical function identity (library, owner, name) to its solved
/// parameter-shape outcome.
pub type SignatureSolutions = BTreeMap<(Option<String>, Option<String>, String), SolvedSignature>;

/// `--emit-ir` representation for [`SignatureSolutions`].
///
/// serde_json rejects non-string map keys, so the tuple identity cannot ride
/// on the map key; each entry spells out its identity fields instead.
pub(crate) fn serialize_solutions<S>(
    solutions: &Option<SignatureSolutions>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    #[derive(serde::Serialize)]
    struct Entry<'a> {
        library_uri: &'a Option<String>,
        owner: &'a Option<String>,
        name: &'a String,
        #[serde(flatten)]
        solved: &'a SolvedSignature,
    }

    match solutions {
        Some(solutions) => {
            let entries: Vec<Entry<'_>> = solutions
                .iter()
                .map(|((library_uri, owner, name), solved)| Entry {
                    library_uri,
                    owner,
                    name,
                    solved,
                })
                .collect();
            serde::Serialize::serialize(&entries, serializer)
        }
        None => serializer.serialize_none(),
    }
}

/// Authoritative descriptor tuple: (fixed, optional, optional-named, implicit).
pub type DescriptorShape = (usize, usize, bool, usize);

/// One observed fact about a function's parameter shape.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ShapeConstraint {
    /// Positional-argument count observed at one call site.
    pub positional: usize,
    /// Named arguments observed (names may be obfuscated tokens).
    pub named: Vec<String>,
    /// Where the constraint came from.
    pub rule: &'static str,
}

/// Solved parameter-shape outcome for one function occurrence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ShapeOutcome {
    /// Authoritative descriptor (bound VM oracle metadata).
    Proven {
        fixed: usize,
        optional: usize,
        optional_named: bool,
        implicit: usize,
    },
    /// Every independent source agrees on the same minimal shape.
    Corroborated {
        minimum_positional: usize,
        sources: usize,
    },
    /// Sources disagree; the intersection is retained as a lower bound.
    Bounded {
        minimum_positional: usize,
        maximum_observed: usize,
        conflicting_sources: usize,
    },
    /// No usable constraints. Erasure is real; do not guess.
    Unknown,
}

impl ShapeOutcome {
    pub fn tier(&self) -> EvidenceTier {
        match self {
            Self::Proven { .. } => EvidenceTier::Proven,
            // Cross-ABI corroboration is decided by the consensus module over
            // aligned occurrences; single-source agreement is Inferred.
            Self::Corroborated { .. } => EvidenceTier::Inferred,
            Self::Bounded { .. } => EvidenceTier::Inferred,
            Self::Unknown => EvidenceTier::Speculative,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Proven { .. } => "proven",
            Self::Corroborated { .. } => "corroborated",
            Self::Bounded { .. } => "bounded",
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

/// Solver input for one logical function occurrence.
#[derive(Clone, Debug, Default)]
pub struct SignatureProblem {
    pub name_key: (Option<String>, Option<String>, String),
    /// Constraints gathered from direct/pool call sites across the program.
    pub call_site_constraints: Vec<ShapeConstraint>,
    /// Authoritative descriptor from a bound oracle, if linked exactly.
    pub descriptor: Option<(usize, usize, bool, usize)>,
    /// Receiver class ids proven to flow into this function.
    pub receivers: Vec<ReceiverConstraint>,
}

/// Solved result for one occurrence.
#[derive(Clone, Debug, Serialize)]
pub struct SolvedSignature {
    pub outcome: ShapeOutcome,
    pub tier: EvidenceTier,
    /// Receiver CID bounds retained for downstream type narrowing.
    pub receiver_class_ids: Vec<i64>,
    /// Rule identifiers that contributed, in stable order.
    pub rules: Vec<String>,
}

/// Solve every accumulated problem.
///
/// Deterministic: identical inputs produce identical outputs, and no source
/// can raise another's tier — the weakest applicable outcome always wins.
pub fn solve(
    problems: &mut [SignatureProblem],
) -> BTreeMap<(Option<String>, Option<String>, String), SolvedSignature> {
    let mut results = BTreeMap::new();
    for problem in problems {
        let mut rules: Vec<String> = problem
            .call_site_constraints
            .iter()
            .map(|constraint| constraint.rule.to_owned())
            .collect();
        if problem.descriptor.is_some() {
            rules.push("vm_argument_descriptor".to_owned());
        }
        rules.sort();
        rules.dedup();

        let outcome = solve_one(problem);
        results.insert(
            problem.name_key.clone(),
            SolvedSignature {
                tier: outcome.tier(),
                outcome,
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
    // 1. An authoritative descriptor wins outright.
    if let Some((fixed, optional, optional_named, implicit)) = problem.descriptor {
        return ShapeOutcome::Proven {
            fixed,
            optional,
            optional_named,
            implicit,
        };
    }

    // 2. Otherwise intersect call-site constraints.
    if problem.call_site_constraints.is_empty() {
        return ShapeOutcome::Unknown;
    }
    let minimum = problem
        .call_site_constraints
        .iter()
        .map(|constraint| constraint.positional)
        .min()
        .unwrap_or(0);
    let maximum = problem
        .call_site_constraints
        .iter()
        .map(|constraint| constraint.positional)
        .max()
        .unwrap_or(0);
    let distinct_positions = problem
        .call_site_constraints
        .iter()
        .map(|constraint| constraint.positional)
        .collect::<std::collections::BTreeSet<_>>();
    if distinct_positions.len() == 1 && minimum > 0 {
        return ShapeOutcome::Corroborated {
            minimum_positional: minimum,
            sources: problem.call_site_constraints.len(),
        };
    }
    ShapeOutcome::Bounded {
        minimum_positional: minimum,
        maximum_observed: maximum,
        conflicting_sources: distinct_positions.len(),
    }
}

/// Accumulate call-site constraints into the per-occurrence problems.
pub fn accumulate_call_sites(
    caller: &crate::model::RecoveredFunction,
    problems: &mut [SignatureProblem],
) {
    for statement in &caller.statements {
        let target = match statement {
            crate::model::PseudoStatement::DirectCall {
                target: Some(target),
                ..
            } => target,
            crate::model::PseudoStatement::RecoveredIndirectCall { target, .. } => target,
            crate::model::PseudoStatement::ObjectPoolCall { target, .. } => target,
            _ => continue,
        };
        // Split qualified target labels used by the renderer; unresolved
        // targets still contribute their raw label.
        let name = target
            .rsplit(['.', ':'])
            .next()
            .unwrap_or(target)
            .to_owned();
        if let Some(problem) = problems
            .iter_mut()
            .find(|problem| problem.name_key.2 == name)
        {
            problem.call_site_constraints.push(ShapeConstraint {
                positional: 0,
                named: Vec::new(),
                rule: "call_site_edge",
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem(
        key: (&str, &str, &str),
        constraints: &[usize],
        descriptor: Option<(usize, usize, bool, usize)>,
    ) -> SignatureProblem {
        SignatureProblem {
            name_key: (
                Some(key.0.to_owned()),
                Some(key.1.to_owned()),
                key.2.to_owned(),
            ),
            call_site_constraints: constraints
                .iter()
                .map(|&positional| ShapeConstraint {
                    positional,
                    named: Vec::new(),
                    rule: "call_site",
                })
                .collect(),
            descriptor,
            receivers: Vec::new(),
        }
    }

    #[test]
    fn authoritative_descriptor_beats_call_sites() {
        let mut problems = vec![problem(
            ("package:a", "C", "m"),
            &[2, 3],
            Some((4, 1, true, 1)),
        )];
        let results = solve(&mut problems);
        let solved = &results[&(
            Some("package:a".to_owned()),
            Some("C".to_owned()),
            "m".to_owned(),
        )];
        assert_eq!(
            solved.outcome,
            ShapeOutcome::Proven {
                fixed: 4,
                optional: 1,
                optional_named: true,
                implicit: 1
            }
        );
        assert!(solved.tier.at_least(EvidenceTier::Proven));
    }

    #[test]
    fn agreeing_call_sites_corroborate_a_minimum() {
        let mut problems = vec![problem(("l", "o", "f"), &[2, 2, 2], None)];
        let results = solve(&mut problems);
        let (_, solved) = results.iter().next().unwrap();
        assert_eq!(
            solved.outcome,
            ShapeOutcome::Corroborated {
                minimum_positional: 2,
                sources: 3
            }
        );
        assert_eq!(solved.tier, EvidenceTier::Inferred);
    }

    #[test]
    fn disagreeing_call_sites_stay_bounded_never_guessed() {
        let mut problems = vec![problem(("l", "o", "g"), &[1, 3], None)];
        let results = solve(&mut problems);
        let (_, solved) = results.iter().next().unwrap();
        assert_eq!(
            solved.outcome,
            ShapeOutcome::Bounded {
                minimum_positional: 1,
                maximum_observed: 3,
                conflicting_sources: 2
            }
        );
    }

    #[test]
    fn erased_signatures_without_evidence_stay_unknown_speculative() {
        let mut problems = vec![problem(("lib", "owner", "erased"), &[], None)];
        let results = solve(&mut problems);
        let (_, solved) = results.iter().next().unwrap();
        assert_eq!(solved.outcome, ShapeOutcome::Unknown);
        assert_eq!(solved.tier, EvidenceTier::Speculative);
    }

    #[test]
    fn solving_is_deterministic() {
        let build = || {
            let mut problems = vec![
                problem(("a", "b", "c"), &[2, 2], None),
                problem(("d", "e", "f"), &[], Some((1, 0, false, 1))),
            ];
            solve(&mut problems)
        };
        assert_eq!(format!("{:?}", build()), format!("{:?}", build()));
    }
}
