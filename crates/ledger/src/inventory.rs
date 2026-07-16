//! Position inventory tracking.

use domain_types::Quantity;
use std::collections::HashMap;

/// Position inventory for each market/contract.
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    positions: HashMap<String, Quantity>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    /// Get the current position for a contract.
    pub fn get_position(&self, contract_id: &str) -> Quantity {
        self.positions
            .get(contract_id)
            .copied()
            .unwrap_or(Quantity::ZERO)
    }
}
