//! Cash and Inventory Ledger (SIM-003, SIM-005)
//!
//! Owns the canonical ledger state: cash balances, reserved cash,
//! position inventory, P&L tracking, and settlement finalization.

pub mod cash_ledger;
pub mod settlement;

use domain_types::{Cash, ReservedCash, SignedPnl};
use serde::{Deserialize, Serialize};

/// Represents an idempotent transition to the ledger state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerTransition {
    pub transition_id: String,
    pub free_cash_delta: i128,
    pub reserved_cash_delta: i128,
    pub total_cash_delta: i128,
    // (We could add inventory deltas here, but omitting for brevity in the slice)
}

/// The canonical ledger state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    /// Free cash available for new orders
    pub free_cash: Cash,
    /// Cash reserved for open orders
    pub reserved_cash: ReservedCash,
    /// Total cash (invariant: free + reserved = total)
    pub total_cash: Cash,
    /// Realized P&L
    pub realized_pnl: SignedPnl,
    /// Unrealized P&L (marked to market)
    pub unrealized_pnl: SignedPnl,
    /// Position inventory keyed by market/outcome/side
    pub inventory: matching::inventory::Inventory,
    /// Version of the ledger
    pub version: u64,
    /// Set of applied transition IDs for idempotency
    pub applied_transitions: std::collections::HashSet<String>,
}

impl Ledger {
    /// Create a new ledger with initial cash.
    pub fn new(initial_cash: Cash) -> Self {
        Self {
            free_cash: initial_cash,
            reserved_cash: ReservedCash::ZERO,
            total_cash: initial_cash,
            realized_pnl: SignedPnl::ZERO,
            unrealized_pnl: SignedPnl::ZERO,
            inventory: matching::inventory::Inventory::new(),
            version: 0,
            applied_transitions: std::collections::HashSet::new(),
        }
    }

    /// Verify the cash invariant on a candidate state.
    fn verify_candidate_cash_invariant(free: i128, res: u64, total: i128) -> Result<(), String> {
        let sum = free
            .checked_add(res as i128)
            .ok_or("Cash invariant overflow")?;
        if sum != total {
            return Err(format!(
                "Cash invariant violated: free({}) + reserved({}) != total({})",
                free, res, total
            ));
        }
        Ok(())
    }

    /// Verify the cash invariant.
    pub fn verify_cash_invariant(&self) -> Result<(), String> {
        Self::verify_candidate_cash_invariant(
            self.free_cash.as_raw(),
            self.reserved_cash.as_raw(),
            self.total_cash.as_raw(),
        )
    }

    /// Bump the ledger version.
    ///
    /// Returns an error on overflow rather than silently wrapping.
    pub fn increment_version(&mut self) -> Result<(), domain_types::DomainError> {
        self.version = self.version.checked_add(1).ok_or(
            domain_types::DomainError::Overflow {
                detail: format!("Ledger version overflow at {}", self.version),
            },
        )?;
        Ok(())
    }

    /// Apply a transition idempotently.
    pub fn apply_transition(&mut self, transition: &LedgerTransition) -> Result<(), String> {
        if self.applied_transitions.contains(&transition.transition_id) {
            return Ok(());
        }

        let new_free = self.free_cash.as_raw().checked_add(transition.free_cash_delta)
            .ok_or("Free cash overflow")?;
        let new_res = (self.reserved_cash.as_raw() as i128).checked_add(transition.reserved_cash_delta)
            .ok_or("Reserved cash overflow")?;
        let new_total = self.total_cash.as_raw().checked_add(transition.total_cash_delta)
            .ok_or("Total cash overflow")?;

        if new_free < 0 || new_res < 0 || new_total < 0 {
            return Err("Negative cash balance not allowed".into());
        }

        Self::verify_candidate_cash_invariant(new_free, new_res as u64, new_total)?;

        // Atomic swap
        self.free_cash = Cash::new(new_free);
        self.reserved_cash = ReservedCash::new(new_res as u64);
        self.total_cash = Cash::new(new_total);
        self.applied_transitions.insert(transition.transition_id.clone());

        self.increment_version().map_err(|e| e.to_string())?;

        Ok(())
    }
}
