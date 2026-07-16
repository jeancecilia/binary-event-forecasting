//! Shared virtual depth tracking (SIM-003).
//!
//! All strategies submit intents to one shared matching adapter.
//! Observable depth, cash, reserved cash, inventory, and margin
//! must not be consumed more than once.

use domain_types::Quantity;
use std::collections::HashMap;

/// Tracks virtual depth consumption across all policies.
#[derive(Debug, Clone, Default)]
pub struct VirtualDepth {
    /// Quantity consumed at each price level (price raw → consumed qty)
    consumed: HashMap<u64, Quantity>,
}

impl VirtualDepth {
    /// Create a new virtual depth tracker.
    pub fn new() -> Self {
        Self {
            consumed: HashMap::new(),
        }
    }

    /// Check if consuming additional quantity at a price level would exceed available.
    pub fn can_consume(
        &self,
        price_raw: u64,
        requested: &Quantity,
        available: &Quantity,
    ) -> bool {
        let already_consumed = self
            .consumed
            .get(&price_raw)
            .map(|q| q.as_raw())
            .unwrap_or(0);
        already_consumed + requested.as_raw() <= available.as_raw()
    }

    /// Consume quantity at a price level. Returns error if it would exceed available.
    pub fn consume(
        &mut self,
        price_raw: u64,
        quantity: &Quantity,
        available: &Quantity,
    ) -> Result<(), String> {
        if !self.can_consume(price_raw, quantity, available) {
            return Err(format!(
                "Virtual depth exceeded at price {}: consumed {} + requested {} > available {}",
                price_raw,
                self.consumed.get(&price_raw).map(|q| q.as_raw()).unwrap_or(0),
                quantity.as_raw(),
                available.as_raw()
            ));
        }
        let entry = self.consumed.entry(price_raw).or_insert(Quantity::ZERO);
        *entry = Quantity::from_raw(entry.as_raw() + quantity.as_raw());
        Ok(())
    }

    /// Reset consumed depth (e.g., after state rebuild).
    pub fn reset(&mut self) {
        self.consumed.clear();
    }
}
