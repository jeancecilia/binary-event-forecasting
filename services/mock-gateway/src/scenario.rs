//! Scenario scripting for the mock gateway.
//!
//! Each scenario defines a sequence of lifecycle events:
//! acknowledgements, rejections, partial fills, cancellations, and settlements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A mock gateway scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Unique scenario identifier
    pub scenario_id: String,
    /// Scenario version
    pub version: String,
    /// Environment (must be LOCAL_MOCK_DEMO)
    pub environment: String,
    /// Configuration hash at scenario creation time
    pub config_hash: String,
    /// Sequence of lifecycle events
    pub events: Vec<LifecycleEvent>,
}

/// A single lifecycle event in a mock scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LifecycleEvent {
    /// Acknowledge receipt of a forecast message
    Acknowledgement {
        message_id: String,
        timestamp: DateTime<Utc>,
        status: String,
    },
    /// Reject a forecast message
    Rejection {
        message_id: String,
        timestamp: DateTime<Utc>,
        reason: String,
    },
    /// Partial fill
    PartialFill {
        intent_id: String,
        timestamp: DateTime<Utc>,
        quantity: u64,
        price: u64,
    },
    /// Complete fill
    CompleteFill {
        intent_id: String,
        timestamp: DateTime<Utc>,
        quantity: u64,
        average_price: u64,
    },
    /// Cancellation
    Cancellation {
        intent_id: String,
        timestamp: DateTime<Utc>,
        effective: bool,
    },
    /// Settlement event
    Settlement {
        market_id: String,
        timestamp: DateTime<Utc>,
        outcome: String,
    },
}
