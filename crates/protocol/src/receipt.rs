//! Receipt acknowledgement schema.
//!
//! Sent by the Rust core to acknowledge receipt of a forecast message.

use crate::enums::ReceiptStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Acknowledgement of forecast message receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptAcknowledgement {
    /// Schema version
    pub schema_version: u32,
    /// Reference to the original message
    pub message_id: String,
    /// Receipt status
    pub receipt_status: ReceiptStatus,
    /// When the acknowledgement was generated
    pub timestamp: DateTime<Utc>,
    /// Receipt sequence number
    pub receipt_id: String,
    /// Optional detail for rejections
    pub detail: Option<String>,
}
