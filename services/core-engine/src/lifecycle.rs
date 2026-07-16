//! Lifecycle management for forecast messages.
//!
//! Tracks the full lifecycle of a forecast message from receipt
//! through validation, policy evaluation, matching, and terminal disposition.

use chrono::{DateTime, Utc};

/// The current lifecycle state of a forecast message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleState {
    /// Message received, not yet validated
    Received,
    /// Validation in progress
    Validating,
    /// Validation passed
    Validated,
    /// Forecast policy evaluation in progress
    Evaluating,
    /// Policy evaluation complete
    Evaluated,
    /// Simulation intent submitted to matcher
    SimulationSubmitted,
    /// Simulation complete
    Simulated,
    /// Terminal: message was rejected
    Rejected { reason: String },
    /// Terminal: message was superseded
    Superseded { by_message_id: String },
    /// Terminal: message evicted from queue
    Evicted { reason: String },
}

/// A lifecycle transition record.
#[derive(Debug, Clone)]
pub struct LifecycleTransition {
    /// Unique transition identifier
    pub transition_id: String,
    /// The message this transition applies to
    pub message_id: String,
    /// Previous state
    pub from_state: LifecycleState,
    /// New state
    pub to_state: LifecycleState,
    /// Logical timestamp of the transition
    pub logical_timestamp: i64,
    /// Wall-clock timestamp (telemetry only)
    pub runtime_timestamp: DateTime<Utc>,
    /// Canonical payload hash
    pub payload_hash: String,
}
