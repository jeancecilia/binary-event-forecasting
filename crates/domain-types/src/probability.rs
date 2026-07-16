//! ProbabilityScaled type (TYP-003).
//!
//! Probabilities crossing a process boundary use scaled integers:
//! `0 ≤ ProbabilityScaled ≤ ProbabilityScale`
//!
//! The protocol defines one `ProbabilityScale` constant per schema version.
//! Bounds, uncertainty intervals, and quantization are validated before
//! the message enters the forecast policy.

use serde::{Deserialize, Serialize};

use crate::DomainError;

/// The probability scale for schema version 1.
/// Probability = raw_value / PROBABILITY_SCALE_V1
pub const PROBABILITY_SCALE_V1: u64 = 1_000_000;

/// A scaled probability value.
///
/// Must satisfy: `0 ≤ value ≤ PROBABILITY_SCALE`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProbabilityScaled(u64);

impl ProbabilityScaled {
    /// Zero probability.
    pub const ZERO: Self = ProbabilityScaled(0);

    /// Certainty (probability = 1.0).
    pub const fn certainty(scale: u64) -> Self {
        ProbabilityScaled(scale)
    }

    /// Create a new ProbabilityScaled, validating bounds.
    pub fn new(raw: u64, scale: u64) -> Result<Self, DomainError> {
        if raw > scale {
            return Err(DomainError::OutOfRange {
                value: raw.to_string(),
                min: "0".to_string(),
                max: scale.to_string(),
                detail: "ProbabilityScaled exceeds scale".to_string(),
            });
        }
        Ok(ProbabilityScaled(raw))
    }

    /// Create a ProbabilityScaled without validation.
    /// Only for use when the value is known to be valid.
    pub const fn from_raw(raw: u64) -> Self {
        ProbabilityScaled(raw)
    }

    /// Return the raw scaled integer value.
    pub fn as_raw(&self) -> u64 {
        self.0
    }

    /// Convert to a floating-point representation (for reporting/display only).
    /// NOT for use in state accounting or matching.
    pub fn to_f64_lossy(&self, scale: u64) -> f64 {
        self.0 as f64 / scale as f64
    }

    /// Returns true if this probability is in the valid range.
    pub fn is_valid(&self, scale: u64) -> bool {
        self.0 <= scale
    }
}

/// An uncertainty interval for a probability estimate.
///
/// Invariant: `0 ≤ lower ≤ probability ≤ upper ≤ ProbabilityScale`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UncertaintyInterval {
    /// Lower bound of the uncertainty interval.
    pub lower: ProbabilityScaled,
    /// Upper bound of the uncertainty interval.
    pub upper: ProbabilityScaled,
    /// Coverage level (e.g., 0.90)
    pub coverage_level: f64,
    /// Method used for uncertainty estimation.
    pub method: String,
}

impl UncertaintyInterval {
    /// Validate that the uncertainty interval satisfies its invariants.
    pub fn validate(
        &self,
        probability: &ProbabilityScaled,
        scale: u64,
    ) -> Result<(), DomainError> {
        if self.lower.as_raw() > probability.as_raw()
            || probability.as_raw() > self.upper.as_raw()
        {
            return Err(DomainError::OutOfRange {
                value: format!(
                    "lower={} prob={} upper={}",
                    self.lower.as_raw(),
                    probability.as_raw(),
                    self.upper.as_raw()
                ),
                min: "0".to_string(),
                max: scale.to_string(),
                detail: "Uncertainty interval invariant violated".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probability_bounds() {
        assert!(ProbabilityScaled::new(500_000, PROBABILITY_SCALE_V1).is_ok());
        assert!(ProbabilityScaled::new(1_000_001, PROBABILITY_SCALE_V1).is_err());
    }

    #[test]
    fn test_uncertainty_invariant() {
        let prob = ProbabilityScaled::from_raw(600_000);
        let interval = UncertaintyInterval {
            lower: ProbabilityScaled::from_raw(550_000),
            upper: ProbabilityScaled::from_raw(650_000),
            coverage_level: 0.90,
            method: "conformal".to_string(),
        };
        assert!(interval.validate(&prob, PROBABILITY_SCALE_V1).is_ok());
    }

    #[test]
    fn test_uncertainty_invariant_violated() {
        let prob = ProbabilityScaled::from_raw(600_000);
        let interval = UncertaintyInterval {
            lower: ProbabilityScaled::from_raw(650_000), // lower > prob
            upper: ProbabilityScaled::from_raw(700_000),
            coverage_level: 0.90,
            method: "conformal".to_string(),
        };
        assert!(interval.validate(&prob, PROBABILITY_SCALE_V1).is_err());
    }
}
