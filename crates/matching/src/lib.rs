//! Matching Engine — Immediate Execution and Passive Queue Simulation
//! (TIM-002, SIM-001 through SIM-006)
//!
//! The matcher evaluates immutable simulation intents against arrival-state
//! market books. It performs all-or-none immediate matching and passive
//! queue lifecycle simulation under conservative assumptions.

pub mod immediate;
pub mod passive_queue;
pub mod virtual_depth;

use domain_types::{Cash, Notional, Price, Quantity, ReservedCash};
use market_state::MarketSnapshot;
use protocol::{SimulationIntent, enums::*};

/// Result of a matching operation.
#[derive(Debug, Clone)]
pub enum MatchResult {
    /// Intent was fully filled
    Filled {
        /// Total fill quantity
        filled_quantity: Quantity,
        /// Weighted average fill price
        average_price: Price,
        /// Total notional (before costs)
        notional: Notional,
        /// Cash reserved for this fill
        cash_reserved: Cash,
    },
    /// Intent was rejected (insufficient depth, invalid state, etc.)
    Rejected { reason: String },
    /// Intent is passive and acknowledged (queued)
    Queued {
        /// Quantity ahead in queue at insertion
        quantity_ahead: Quantity,
        /// Position in queue
        queue_position: u64,
    },
    /// Queue progression is unobservable
    Unobservable { reason: String },
}

/// The shared virtual matching state that all policies submit to.
#[derive(Debug, Clone)]
pub struct VirtualMatchingState {
    /// Free cash available
    pub free_cash: Cash,
    /// Cash reserved for open orders
    pub reserved_cash: ReservedCash,
    /// Total cash (invariant: FreeCash + ReservedCash = TotalCash)
    pub total_cash: Cash,
    /// Shared virtual depth tracker
    pub virtual_depth: virtual_depth::VirtualDepth,
}

impl VirtualMatchingState {
    /// Create a new virtual matching state with initial cash.
    pub fn new(initial_cash: Cash) -> Self {
        Self {
            free_cash: initial_cash,
            reserved_cash: ReservedCash::ZERO,
            total_cash: initial_cash,
            virtual_depth: virtual_depth::VirtualDepth::new(),
        }
    }

    /// Verify the cash invariant: FreeCash + ReservedCash = TotalCash
    pub fn verify_cash_invariant(&self) -> Result<(), String> {
        let sum = self
            .free_cash
            .as_raw()
            .checked_add(self.reserved_cash.as_raw() as i128)
            .ok_or("Cash invariant overflow during check")?;
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
}
