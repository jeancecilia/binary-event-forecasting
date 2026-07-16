//! Experiment Control (EXP-001, EXP-002)
//!
//! Manages experiment registration, manifest validation, holdout access
//! auditing, and policy versioning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A registered experiment manifest (EXP-001).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentManifest {
    /// Experiment identifier
    pub experiment_id: String,
    /// Primary hypothesis
    pub primary_hypothesis: String,
    /// Primary metrics
    pub primary_metrics: Vec<String>,
    /// Model artifact version
    pub model_version: String,
    /// Prompt version
    pub prompt_version: String,
    /// Ensemble version
    pub ensemble_version: String,
    /// Calibration model version
    pub calibration_version: String,
    /// Forecast selection rule
    pub selection_rule: String,
    /// Baseline definitions
    pub baseline_definitions: Vec<String>,
    /// Matching model version
    pub matching_model_version: String,
    /// Latency model version
    pub latency_model_version: String,
    /// Cost model version
    pub cost_model_version: String,
    /// Settlement model version
    pub settlement_model_version: String,
    /// Event group weighting
    pub event_group_weighting: String,
    /// Exclusion rules
    pub exclusion_rules: Vec<String>,
    /// Planned comparisons
    pub planned_comparisons: Vec<String>,
    /// Statistical correction method
    pub statistical_correction: String,
    /// Software build hash
    pub software_build_hash: String,
    /// Configuration hash
    pub configuration_hash: String,
    /// When the manifest was frozen
    pub frozen_at: DateTime<Utc>,
}

/// Holdout access audit entry (EXP-002).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldoutAccessEntry {
    /// Access timestamp
    pub accessed_at: DateTime<Utc>,
    /// Who/what accessed
    pub accessor: String,
    /// What was accessed
    pub resource: String,
    /// Whether this access was registered
    pub registered: bool,
    /// Experiment ID (if applicable)
    pub experiment_id: Option<String>,
}

/// Check if a design change requires a new experiment.
pub fn requires_new_experiment(
    _original_manifest: &ExperimentManifest,
    proposed_changes: &[String],
    _holdout_accessed: bool,
) -> bool {
    // Any change to model, policy, threshold, sizing, latency,
    // matching, cost, or exclusion after holdout access requires
    // a new experiment ID and untouched evaluation period.
    !proposed_changes.is_empty()
}
