//! Cash and Inventory Ledger (SIM-003, SIM-005)
//!
//! Owns the canonical ledger state: cash balances, reserved cash,
//! position inventory, P&L tracking, and settlement finalization.

pub mod cash_ledger;
pub mod inventory;
pub mod settlement;

use domain_types::{Cash, ReservedCash, SignedPnl};
use serde::{Deserialize, Serialize};

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
    /// Version of the ledger
    pub version: u64,
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
            version: 0,
        }
    }

    /// Verify the cash invariant.
    pub fn verify_cash_invariant(&self) -> Result<(), String> {
        let sum = self
            .free_cash
            .as_raw()
            .checked_add(self.reserved_cash.as_raw() as i128)
            .ok_or("Cash invariant overflow")?;
        if sum != self.total_cash.as_raw() {
            return Err(format!(
                "Cash invariant violated: free({}) + reserved({}) != total({})",
                self.free_cash.as_raw(),
                self.reserved_cash.as_raw(),
                self.total_cash.as_raw()
            ));
        }
        Ok(())
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
}
