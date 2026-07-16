//! Shared virtual depth tracking (SIM-003).
//!
//! All strategies submit intents to one shared matching adapter.
//! Observable depth, cash, reserved cash, inventory, and margin
//! must not be consumed more than once.

use domain_types::{Price, Quantity};
use protocol::enums::BookSide;
use std::collections::HashMap;

/// A unique key for virtual depth tracking.
/// Prevents collisions across different markets, contracts, sides, and feed generations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DepthKey {
    pub market_id: String,
    pub contract_or_outcome_id: String,
    pub book_side: BookSide,
    pub price_raw: u64,
    pub feed_generation: u64,
}

/// Tracks virtual depth consumption across all policies.
#[derive(Debug, Clone, Default)]
pub struct VirtualDepth {
    /// Quantity consumed at each depth key
    consumed: HashMap<DepthKey, Quantity>,
}

impl VirtualDepth {
    /// Create a new virtual depth tracker.
    pub fn new() -> Self {
        Self {
            consumed: HashMap::new(),
        }
    }

    /// Check if consuming additional quantity at a depth key would exceed available.
    /// Uses checked arithmetic.
    pub fn can_consume(
        &self,
        key: &DepthKey,
        requested: &Quantity,
        available: &Quantity,
    ) -> bool {
        let already = self
            .consumed
            .get(key)
            .map(|q| q.as_raw())
            .unwrap_or(0);
        already.checked_add(requested.as_raw())
            .map(|total| total <= available.as_raw())
            .unwrap_or(false)
    }

    /// Consume quantity at a depth key. Returns error on overflow or exceeding available.
    pub fn consume(
        &mut self,
        key: &DepthKey,
        quantity: &Quantity,
        available: &Quantity,
    ) -> Result<(), String> {
        let already = self
            .consumed
            .get(key)
            .map(|q| q.as_raw())
            .unwrap_or(0);

        let new_total = already
            .checked_add(quantity.as_raw())
            .ok_or_else(|| format!(
                "Virtual depth overflow at {:?}: {} + {}",
                key, already, quantity.as_raw()
            ))?;

        if new_total > available.as_raw() {
            return Err(format!(
                "Virtual depth exceeded at {:?}: consumed {} + requested {} > available {}",
                key, already, quantity.as_raw(), available.as_raw()
            ));
        }

        self.consumed.insert(key.clone(), Quantity::from_raw(new_total));
        Ok(())
    }

    /// Reset consumed depth (e.g., after state rebuild).
    pub fn reset(&mut self) {
        self.consumed.clear();
    }
}
