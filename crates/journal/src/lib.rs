//! Durable Event Journal (AUD-001 through AUD-004)
//!
//! Provides a crash-recoverable local journal using SQLite WAL.
//! Implements the transition protocol:
//! 1. Append DispositionPlanned durably
//! 2. Apply the ledger transition idempotently
//! 3. Append DispositionCommitted durably
//!
//! Supports bounded local spooling for PostgreSQL reconciliation.

pub mod db;
pub mod spool;
pub mod recovery;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A journal record with hash-linked integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalRecord {
    /// Unique record identifier
    pub record_id: String,
    /// Message or intent ID
    pub entity_id: String,
    /// Lifecycle state
    pub lifecycle_state: String,
    /// Transition identifier
    pub transition_id: String,
    /// Logical simulation timestamp
    pub logical_timestamp: i64,
    /// Canonical payload hash (SHA-256)
    pub canonical_payload_hash: String,
    /// Previous record hash (hash chain)
    pub previous_record_hash: String,
    /// Record-level checksum
    pub checksum: String,
    /// Wall-clock timestamp (telemetry only, not deterministic)
    pub created_at_runtime: DateTime<Utc>,
}

/// Transition protocol phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    /// DispositionPlanned has been durably written
    Planned,
    /// Ledger transition has been applied
    Applied,
    /// DispositionCommitted has been durably written
    Committed,
}

/// Configuration for the local journal.
#[derive(Debug, Clone)]
pub struct JournalConfig {
    /// Path to the SQLite journal database
    pub db_path: String,
    /// Spool configuration
    pub spool: SpoolConfig,
}

/// Configuration for the bounded local spool.
#[derive(Debug, Clone)]
pub struct SpoolConfig {
    /// Path to the spool SQLite database
    pub db_path: String,
    /// Maximum spool size in bytes
    pub max_bytes: u64,
    /// Maximum number of spooled records
    pub max_records: u64,
    /// Behavior when spool is exhausted
    pub on_exhaustion: SpoolExhaustionBehavior,
}

/// Behavior when the spool reaches capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpoolExhaustionBehavior {
    /// Halt all state-changing work
    HaltStateChangingWork,
    /// Enter degraded mode (accept but don't process)
    Degraded,
}

impl Default for SpoolConfig {
    fn default() -> Self {
        Self {
            db_path: "var/spool/research-store-spool.sqlite".to_string(),
            max_bytes: 1_073_741_824, // 1 GiB
            max_records: 1_000_000,
            on_exhaustion: SpoolExhaustionBehavior::HaltStateChangingWork,
        }
    }
}
