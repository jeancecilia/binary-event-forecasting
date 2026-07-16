//! Lifecycle disposition schema.
//!
//! Terminal lifecycle state for a forecast message after processing.

use crate::enums::DispositionStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Terminal disposition of a forecast message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleDisposition {
    /// Schema version
    pub schema_version: u32,
    /// Reference to the original message
    pub message_id: String,
    /// Terminal disposition status
    pub disposition_status: DispositionStatus,
    /// When the disposition was finalized
    pub timestamp: DateTime<Utc>,
    /// Transition ID
    pub transition_id: String,
    /// Optional detail
    pub detail: Option<String>,
    /// Previous status (for traceability)
    pub previous_status: Option<DispositionStatus>,
}
