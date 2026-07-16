//! Forecast message schema (IPC-003).
//!
//! A `forecast_message` contains protocol identity, target identity,
//! source provenance, model provenance, forecast values, and lifecycle timestamps.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A forecast message from the Python intelligence plane.
///
/// This is NOT itself a simulation intent. It must be transformed
/// through a deterministic forecast-to-intent policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl ForecastMessage {
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

    /// Returns true if this message has expired relative to the given time.
    pub fn is_expired_at(&self, time: &DateTime<Utc>) -> bool {
        time >= &self.expires_at
    }
}
