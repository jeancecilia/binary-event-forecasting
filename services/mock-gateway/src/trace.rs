//! Immutable trace recording for mock gateway scenarios.
//!
//! All mock-gateway market events and lifecycle responses are captured
//! in an immutable, versioned trace for subsequent deterministic replay.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A recorded mock gateway trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// Trace identifier (matches scenario ID)
    pub trace_id: String,
    /// Environment
    pub environment: String,
    /// Gateway build hash
    pub build_hash: String,
    /// Scenario ID
    pub scenario_id: String,
    /// Configuration hash
    pub config_hash: String,
    /// Trace creation timestamp
    pub created_at: DateTime<Utc>,
    /// Recorded events in order
    pub events: Vec<TraceEvent>,
    /// SHA-256 of the trace content
    pub content_hash: String,
}

/// A single recorded event in a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Experiment ID
    pub experiment_id: String,
    /// Intent ID (if applicable)
    pub intent_id: Option<String>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: String,
    /// Canonical request hash
    pub request_hash: String,
    /// Canonical response hash
    pub response_hash: String,
}

impl Trace {
    /// Create a new trace for a scenario.
    pub fn new(
        scenario_id: String,
        config_hash: String,
        build_hash: String,
    ) -> Self {
        Self {
            trace_id: uuid::Uuid::now_v7().to_string(),
            environment: "LOCAL_MOCK_DEMO".to_string(),
            build_hash,
            scenario_id,
            config_hash,
            created_at: Utc::now(),
            events: Vec::new(),
            content_hash: String::new(),
        }
    }
}
