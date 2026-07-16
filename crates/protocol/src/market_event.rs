//! Market event schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A market event (trade, quote update, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    /// Event identifier
    pub event_id: String,
    /// Market identifier
    pub market_id: String,
    /// Event type
    pub event_type: MarketEventType,
    /// Source timestamp
    pub source_timestamp: DateTime<Utc>,
    /// Logical observation timestamp
    pub logical_timestamp: i64,
    /// Source sequence number
    pub source_sequence: Option<u64>,
    /// Event payload
    pub payload: serde_json::Value,
}

/// Types of market events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketEventType {
    Trade,
    QuoteUpdate,
    OrderBookDelta,
    OrderBookSnapshot,
    FeedStatusChange,
    Settlement,
    Correction,
}
