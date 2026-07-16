//! Crash recovery and idempotency (AUD-002, AUD-004).

/// Recovery state after a restart.
#[derive(Debug, Clone)]
pub struct RecoveryState {
    /// Processed message IDs retained from before crash
    pub processed_messages: Vec<String>,
    /// Last known sender sequences
    pub sender_sequences: std::collections::HashMap<String, u64>,
    /// Planned transitions not yet applied
    pub pending_plans: Vec<String>,
    /// Applied transitions not yet committed
    pub pending_commits: Vec<String>,
}

impl RecoveryState {
    /// Create an empty recovery state.
    pub fn new() -> Self {
        Self {
            processed_messages: Vec::new(),
            sender_sequences: std::collections::HashMap::new(),
            pending_plans: Vec::new(),
            pending_commits: Vec::new(),
        }
    }

    /// Check if a message has already been processed.
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        self.processed_messages.contains(&message_id.to_string())
    }
}

impl Default for RecoveryState {
    fn default() -> Self {
        Self::new()
    }
}
