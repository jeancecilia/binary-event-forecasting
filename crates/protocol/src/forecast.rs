//! Forecast message schema (IPC-003).
//!
//! A `forecast_message` contains protocol identity, target identity,
//! source provenance, model provenance, forecast values, and lifecycle timestamps.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The probability scale for schema version 1.
pub const PROBABILITY_SCALE_V1: u64 = 1_000_000;

/// A forecast message from the Python intelligence plane.
///
/// This is NOT itself a simulation intent. It must be transformed
/// through a deterministic forecast-to-intent policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForecastMessage {
    // ── Protocol identity ──
    pub schema_version: u32,
    pub message_id: String,
    pub sender_instance_id: String,
    pub sender_sequence: u64,

    // ── Target identity ──
    pub market_id: String,
    pub contract_or_outcome_id: String,
    pub market_definition_version: String,
    pub event_id: String,
    pub underlying_event_group_id: String,
    pub forecast_target: String,
    pub forecast_horizon: String,

    // ── Source provenance ──
    pub source_id: String,
    pub source_version: String,
    pub evidence_set_hash: String,
    pub published_at: DateTime<Utc>,
    pub first_source_available_at: DateTime<Utc>,
    pub ingested_at: DateTime<Utc>,
    pub revision_id: String,

    // ── Model provenance ──
    pub model_artifact_hash: String,
    pub model_training_cutoff: DateTime<Utc>,
    pub ensemble_version: String,
    pub component_model_versions: serde_json::Value,
    pub prompt_version: String,
    pub retrieval_corpus_version: String,
    pub calibration_model_version: String,
    pub calibration_training_cutoff: DateTime<Utc>,

    // ── Forecast values ──
    pub raw_model_probability: u64,
    pub calibrated_probability: u64,
    pub uncertainty_lower: u64,
    pub uncertainty_upper: u64,
    pub uncertainty_coverage_level: f64,
    pub uncertainty_method: String,
    pub abstention_reason: Option<String>,

    // ── Lifecycle timestamps ──
    pub decision_cutoff_at: DateTime<Utc>,
    pub forecast_created_at: DateTime<Utc>,
    pub forecast_emitted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Validation errors for forecast messages.
#[derive(Debug, Clone, PartialEq)]
pub enum ForecastValidationError {
    SchemaVersion { expected: u32, got: u32 },
    ProbabilityBounds { detail: String },
    RawProbabilityOutOfRange { value: u64, max: u64 },
    CoverageLevelOutOfRange { value: f64 },
    TimestampOrder { detail: String },
    TemporalEligibility { detail: String },
    MissingField { field: String },
}

impl std::fmt::Display for ForecastValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaVersion { expected, got } => {
                write!(f, "Schema version mismatch: expected {expected}, got {got}")
            }
            Self::ProbabilityBounds { detail } => {
                write!(f, "Probability bounds violation: {detail}")
            }
            Self::RawProbabilityOutOfRange { value, max } => {
                write!(f, "Raw probability {value} exceeds max {max}")
            }
            Self::CoverageLevelOutOfRange { value } => {
                write!(f, "Coverage level {value} not in [0.0, 1.0]")
            }
            Self::TimestampOrder { detail } => {
                write!(f, "Timestamp order violation: {detail}")
            }
            Self::TemporalEligibility { detail } => {
                write!(f, "Temporal eligibility: {detail}")
            }
            Self::MissingField { field } => {
                write!(f, "Missing required field: {field}")
            }
        }
    }
}

impl ForecastMessage {
    /// Perform full validation of the forecast message.
    ///
    /// Validates schema version, probability bounds, raw probability,
    /// coverage level, timestamp ordering, and temporal eligibility.
    pub fn validate(&self, probability_scale: u64) -> Result<(), ForecastValidationError> {
        // Validate schema version
        if self.schema_version != crate::SCHEMA_VERSION {
            return Err(ForecastValidationError::SchemaVersion {
                expected: crate::SCHEMA_VERSION,
                got: self.schema_version,
            });
        }
        
        if self.message_id.is_empty() || self.sender_instance_id.is_empty() || self.market_id.is_empty() {
            return Err(ForecastValidationError::MissingField { field: "ID fields cannot be empty".into() });
        }

        // Validate probability bounds invariant
        self.validate_probability_bounds(probability_scale)
            .map_err(|detail| ForecastValidationError::ProbabilityBounds { detail })?;

        // Validate raw_model_probability
        if self.raw_model_probability > probability_scale {
            return Err(ForecastValidationError::RawProbabilityOutOfRange {
                value: self.raw_model_probability,
                max: probability_scale,
            });
        }

        // Validate coverage level
        if !(0.0..=1.0).contains(&self.uncertainty_coverage_level) {
            return Err(ForecastValidationError::CoverageLevelOutOfRange {
                value: self.uncertainty_coverage_level,
            });
        }

        // Validate timestamp ordering
        self.validate_timestamp_order()?;

        // Validate temporal eligibility (CAL-003)
        self.validate_temporal_eligibility()?;

        Ok(())
    }

    /// Validate the probability invariance:
    /// `0 ≤ uncertainty_lower ≤ calibrated_probability ≤ uncertainty_upper ≤ ProbabilityScale`
    pub fn validate_probability_bounds(&self, probability_scale: u64) -> Result<(), String> {
        if self.uncertainty_lower > self.calibrated_probability {
            return Err(format!(
                "uncertainty_lower ({}) > calibrated_probability ({})",
                self.uncertainty_lower, self.calibrated_probability
            ));
        }
        if self.calibrated_probability > self.uncertainty_upper {
            return Err(format!(
                "calibrated_probability ({}) > uncertainty_upper ({})",
                self.calibrated_probability, self.uncertainty_upper
            ));
        }
        if self.uncertainty_upper > probability_scale {
            return Err(format!(
                "uncertainty_upper ({}) > probability_scale ({})",
                self.uncertainty_upper, probability_scale
            ));
        }
        Ok(())
    }

    /// Validate timestamp causal ordering.
    pub fn validate_timestamp_order(&self) -> Result<(), ForecastValidationError> {
        // first_source_available_at ≤ decision_cutoff_at
        if self.first_source_available_at > self.decision_cutoff_at {
            return Err(ForecastValidationError::TimestampOrder {
                detail: "first_source_available_at > decision_cutoff_at".to_string(),
            });
        }
        // Timestamps must be strictly ordered
        if self.decision_cutoff_at >= self.forecast_created_at {
            return Err(ForecastValidationError::TimestampOrder {
                detail: "decision_cutoff_at >= forecast_created_at".to_string(),
            });
        }
        if self.forecast_created_at >= self.forecast_emitted_at {
            return Err(ForecastValidationError::TimestampOrder {
                detail: "forecast_created_at >= forecast_emitted_at".to_string(),
            });
        }
        if self.forecast_emitted_at >= self.expires_at {
            return Err(ForecastValidationError::TimestampOrder {
                detail: "forecast_emitted_at >= expires_at".to_string(),
            });
        }
        Ok(())
    }

    /// Validate temporal eligibility: model training cutoff must precede forecast cutoff.
    pub fn validate_temporal_eligibility(&self) -> Result<(), ForecastValidationError> {
        if self.model_training_cutoff >= self.decision_cutoff_at {
            return Err(ForecastValidationError::TemporalEligibility {
                detail: "model_training_cutoff >= decision_cutoff_at".to_string(),
            });
        }
        if self.calibration_training_cutoff >= self.decision_cutoff_at {
            return Err(ForecastValidationError::TemporalEligibility {
                detail: "calibration_training_cutoff >= decision_cutoff_at".to_string(),
            });
        }
        Ok(())
    }

    /// Returns true if this message has expired relative to the given time.
    pub fn is_expired_at(&self, time: &DateTime<Utc>) -> bool {
        time >= &self.expires_at
    }
}
