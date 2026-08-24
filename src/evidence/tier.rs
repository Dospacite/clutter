use serde::{Deserialize, Serialize};

/// Explicit epistemic tiers for every recovered fact.
///
/// - `Proven`: derived from an exactly bound oracle, a byte-exact range join,
///   or authoritative VM metadata. Never contradicted by another proven fact.
/// - `CrossAbiCorroborated`: agreed independently by ARM64, ARM32, and x64
///   analyses of the same logical occurrence.
/// - `Inferred`: produced by a deterministic static rule (call-site joins,
///   allocation shapes, field traffic) from proven inputs.
/// - `Speculative`: plausible reconstruction only — including any LLM-assisted
///   naming or signature recovery. Rendered output must label it as such and
///   downstream consumers must not promote it into proven evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTier {
    Proven,
    CrossAbiCorroborated,
    Inferred,
    Speculative,
}

impl EvidenceTier {
    pub fn label(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::CrossAbiCorroborated => "cross_abi_corroborated",
            Self::Inferred => "inferred",
            Self::Speculative => "speculative",
        }
    }

    /// Explicit strength rank. Declaration order must not encode strength:
    /// the enum reads strongest-first for humans, while comparisons need
    /// weakest-is-smallest so `merge` below cannot accidentally upgrade.
    #[allow(dead_code)] // shared by merge (tests) and at_least consumers
    fn strength(self) -> u8 {
        match self {
            Self::Speculative => 0,
            Self::Inferred => 1,
            Self::CrossAbiCorroborated => 2,
            Self::Proven => 3,
        }
    }

    /// Weakest tier wins when claims merge; disagreement never upgrades.
    #[cfg(test)]
    pub(crate) fn merge(self, other: Self) -> Self {
        if self.strength() <= other.strength() {
            self
        } else {
            other
        }
    }

    /// A claim may only be promoted into a stronger tier by an explicit rule,
    /// never by accumulation of weaker claims.
    #[allow(dead_code)] // consumed in solver/trace tests and future resolvers
    pub fn at_least(self, floor: Self) -> bool {
        self.strength() >= floor.strength()
    }
}

/// A fact attached to a tier plus the rule that produced it.
#[derive(Clone, Debug, Serialize)]
pub struct TieredClaim<T> {
    pub value: T,
    pub tier: EvidenceTier,
    pub rule: String,
}

impl<T> TieredClaim<T> {
    pub fn new(value: T, tier: EvidenceTier, rule: impl Into<String>) -> Self {
        Self {
            value,
            tier,
            rule: rule.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EvidenceTier;

    #[test]
    fn merging_disagreement_keeps_the_weaker_tier() {
        assert_eq!(
            EvidenceTier::Proven.merge(EvidenceTier::Inferred),
            EvidenceTier::Inferred
        );
        assert_eq!(
            EvidenceTier::Speculative.merge(EvidenceTier::Proven),
            EvidenceTier::Speculative
        );
        assert_eq!(
            EvidenceTier::Proven.merge(EvidenceTier::Proven),
            EvidenceTier::Proven
        );
    }

    #[test]
    fn serializes_snake_case_labels() {
        assert_eq!(
            serde_json::to_string(&EvidenceTier::CrossAbiCorroborated).unwrap(),
            "\"cross_abi_corroborated\""
        );
    }
}
