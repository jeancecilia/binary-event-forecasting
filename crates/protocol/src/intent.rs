//! Simulation intent schema (FCP-002).
//!
//! A simulation_intent is immutable and deterministically derived
//! from a forecast message via the versioned forecast-to-intent policy.

use crate::enums::{BookSide, OrderClass, OutcomeSide, TimeInForce};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An immutable simulation intent derived from a forecast message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationIntent {
    /// Deterministically derived from canonical inputs
    pub simulation_intent_id: String,
    /// Experiment identifier
    pub experiment_id: String,
    /// Parent forecast message ID
    pub source_forecast_message_id: String,
    /// Forecast policy version used
    pub forecast_policy_version: String,
    /// Hash of policy configuration
    pub configuration_hash: String,
    /// Target market
    pub market_id: String,
    /// Target contract/outcome
    pub contract_or_outcome_id: String,
    /// Human-readable target
    pub forecast_target: String,
    /// Order class
    pub order_class: OrderClass,
    /// Book side
    pub book_side: BookSide,
    /// Outcome side (Yes/No)
    pub outcome_side: OutcomeSide,
    /// Order quantity (scaled integer)
    pub quantity: u64,
    /// Limit price (scaled integer)
    pub price_limit: u64,
    /// Time in force
    pub time_in_force: TimeInForce,
    /// Priority for tie-breaking
    pub policy_priority: u32,
    /// When the policy decision was made
    pub decision_timestamp: DateTime<Utc>,
    /// Simulated arrival at matcher
    pub simulated_arrival_timestamp: DateTime<Utc>,
    /// Latency model version
    pub latency_scenario_version: String,
    /// Matching model version
    pub matching_model_version: String,
    /// Cost model version
    pub cost_model_version: String,
    /// Acknowledgement latency model version
    pub acknowledgement_latency_version: String,
    /// Cancellation latency model version
    pub cancellation_latency_version: String,
    /// Account state version
    pub account_state_version: String,
    /// Input snapshot version
    pub input_snapshot_version: String,
    /// Expiry time
    pub expires_at: DateTime<Utc>,
}
