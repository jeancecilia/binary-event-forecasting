//! Trace manifest schema.
//!
//! Enforces strict schema definitions and hashing for offline replay.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Trace manifest defining the exact expected artifacts for a deterministic replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceManifest {
    pub schema_version: u32,
    pub trace_id: String,
    pub trace_format_version: String,
    pub market_events_file: String,
    pub forecast_messages_file: String,
    pub market_events_sha256: String,
    pub forecast_messages_sha256: String,
    pub configuration_sha256: String,
    pub software_build_sha256: String,
    pub logical_epoch: DateTime<Utc>,
    pub expected_event_count: u64,
    pub expected_forecast_count: u64,
}

impl TraceManifest {
    pub fn validate_schema(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "Unsupported schema version: {}",
                self.schema_version
            ));
        }
        Ok(())
    }
}
