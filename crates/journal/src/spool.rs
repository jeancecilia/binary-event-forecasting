//! Bounded local spool for PostgreSQL reconciliation (AUD-003).

use super::SpoolConfig;

/// Local spool for buffering records when PostgreSQL is unavailable.
pub struct Spool {
    config: SpoolConfig,
    current_records: u64,
    current_bytes: u64,
}

impl Spool {
    /// Create a new spool.
    pub fn new(config: SpoolConfig) -> Self {
        Self {
            config,
            current_records: 0,
            current_bytes: 0,
        }
    }

    /// Check if the spool has remaining capacity.
    pub fn has_capacity(&self) -> bool {
        self.current_records < self.config.max_records && self.current_bytes < self.config.max_bytes
    }

    /// Returns true if the spool is exhausted.
    pub fn is_exhausted(&self) -> bool {
        !self.has_capacity()
    }

    /// Returns the configured exhaustion behavior.
    pub fn exhaustion_behavior(&self) -> super::SpoolExhaustionBehavior {
        self.config.on_exhaustion
    }
}
