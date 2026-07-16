//! Quantity type — scaled integer representation of a quantity.

use serde::{Deserialize, Serialize};

use crate::{checked, DomainError};

/// A quantity represented as a scaled integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Quantity(u64);

impl Quantity {
    /// Zero quantity.
    pub const ZERO: Self = Quantity(0);

    /// Create a new Quantity from a raw scaled value.
    pub fn new(raw: u64) -> Result<Self, DomainError> {
        Ok(Quantity(raw))
    }

    /// Create a Quantity directly from a raw value.
    pub const fn from_raw(raw: u64) -> Self {
        Quantity(raw)
    }

    /// Return the raw scaled integer value.
    pub fn as_raw(&self) -> u64 {
        self.0
    }

    /// Checked addition.
    pub fn checked_add(&self, other: &Quantity) -> Result<Quantity, DomainError> {
        checked::checked_add_u64(self.0, other.0, "Quantity").map(Quantity)
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, other: &Quantity) -> Result<Quantity, DomainError> {
        checked::checked_sub_u64(self.0, other.0, "Quantity").map(Quantity)
    }

    /// Returns true if this quantity is sufficient for the requested amount.
    pub fn is_at_least(&self, required: &Quantity) -> bool {
        self.0 >= required.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_zero() {
        assert_eq!(Quantity::ZERO.as_raw(), 0);
    }

    #[test]
    fn test_quantity_sufficient() {
        let available = Quantity::from_raw(100);
        let required = Quantity::from_raw(50);
        assert!(available.is_at_least(&required));
    }
}
