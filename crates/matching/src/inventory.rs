//! Position inventory tracking.

use domain_types::Quantity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap; // Use BTreeMap for deterministic iteration

/// Uniquely identifies an inventory position.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InventoryKey {
    pub market_id: String,
    pub outcome_id: String,
    pub side: protocol::enums::OutcomeSide,
}

/// A specific inventory line containing free and reserved balances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLine {
    pub free: Quantity,
    pub reserved: Quantity,
    pub total: Quantity,
}

impl Default for InventoryLine {
    fn default() -> Self {
        Self {
            free: Quantity::ZERO,
            reserved: Quantity::ZERO,
            total: Quantity::ZERO,
        }
    }
}

impl InventoryLine {
    /// Verify the free + reserved = total invariant.
    pub fn verify_invariant(&self) -> Result<(), String> {
        let sum = self
            .free
            .as_raw()
            .checked_add(self.reserved.as_raw())
            .ok_or("Inventory invariant overflow")?;
        if sum != self.total.as_raw() {
            return Err(format!(
                "Inventory invariant violated: free({}) + reserved({}) != total({})",
                self.free.as_raw(),
                self.reserved.as_raw(),
                self.total.as_raw()
            ));
        }
        Ok(())
    }
}

/// Position inventory for each market/contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    lines: BTreeMap<InventoryKey, InventoryLine>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            lines: BTreeMap::new(),
        }
    }

    /// Get the current line for a contract.
    pub fn get_line(&self, key: &InventoryKey) -> InventoryLine {
        self.lines.get(key).cloned().unwrap_or_default()
    }

    pub fn insert_line(&mut self, key: InventoryKey, line: InventoryLine) -> Result<(), String> {
        line.verify_invariant()?;
        self.lines.insert(key, line);
        Ok(())
    }
}
