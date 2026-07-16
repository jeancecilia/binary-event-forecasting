//! Settlement and ledger finalization (SIM-005).

use protocol::enums::{ResolutionStatus, TerminalOutcome};

/// Settlement result for a resolved event.
#[derive(Debug, Clone)]
pub struct SettlementResult {
    pub market_id: String,
    pub resolution_status: ResolutionStatus,
    pub terminal_outcome: TerminalOutcome,
    pub settlement_price: u64,
}

impl SettlementResult {
    /// Returns true if this outcome is eligible for primary binary forecasting scores.
    pub fn is_scorable(&self) -> bool {
        self.resolution_status == ResolutionStatus::Final
            && matches!(
                self.terminal_outcome,
                TerminalOutcome::Yes | TerminalOutcome::No
            )
    }
}
