//! Mock gateway message schemas (DEM-001 through DEM-006).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A message to or from the local mock gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MockGatewayMessage {
    /// Submit a forecast for mock processing
    ForecastRequest {
        environment: String,
        scenario_id: String,
        message_id: String,
        payload: serde_json::Value,
    },
    /// Mock response to a forecast
    ForecastResponse {
        environment: String,
        scenario_id: String,
        message_id: String,
        status: String,
        timestamp: DateTime<Utc>,
    },
    /// Submit a simulation intent to the mock gateway
    IntentRequest {
        environment: String,
        scenario_id: String,
        intent_id: String,
        payload: serde_json::Value,
    },
    /// Mock response to an intent
    IntentResponse {
        environment: String,
        scenario_id: String,
        intent_id: String,
        status: String,
        fill_quantity: Option<u64>,
        fill_price: Option<u64>,
        timestamp: DateTime<Utc>,
    },
}
