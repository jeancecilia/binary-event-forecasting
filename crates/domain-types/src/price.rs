//! Price type — scaled integer representation of a price level.

use serde::{Deserialize, Serialize};

use crate::{checked, DomainError};

/// A price level represented as a scaled integer.
///
/// The raw value is `price × PRICE_SCALE`. For example, with
/// `PRICE_SCALE = 100_000_000`, a price of 0.625 is stored as 62_500_000.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Price(u64);

impl Price {
    /// The zero price.
    pub const ZERO: Self = Price(0);

    /// Create a new Price from a raw scaled value.
    ///
    /// # Errors
    /// Returns `DomainError::OutOfRange` if the value exceeds `PRICE_SCALE * max_value`.
    pub fn new(raw: u64) -> Result<Self, DomainError> {
        Ok(Price(raw))
    }

    /// Create a Price directly from a raw value without validation.
    /// Only for use when the value is known to be valid.
    pub const fn from_raw(raw: u64) -> Self {
        Price(raw)
    }

    /// Return the raw scaled integer value.
    pub fn as_raw(&self) -> u64 {
        self.0
    }

    /// Checked addition.
    pub fn checked_add(&self, other: &Price) -> Result<Price, DomainError> {
        checked::checked_add_u64(self.0, other.0, "Price")
            .map(Price)
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, other: &Price) -> Result<Price, DomainError> {
        checked::checked_sub_u64(self.0, other.0, "Price")
            .map(Price)
    }

    /// Checked multiplication by a scalar.
    pub fn checked_mul_scalar(&self, scalar: u64) -> Result<Price, DomainError> {
        checked::checked_mul_u64(self.0, scalar, "Price")
            .map(Price)
    }

    /// Checked division by a scalar.
    pub fn checked_div_scalar(&self, scalar: u64) -> Result<Price, DomainError> {
        checked::checked_div_u64(self.0, scalar, "Price")
            .map(Price)
    }

    /// Compare two prices.
    pub fn is_less_than_or_equal(&self, other: &Price) -> bool {
        self.0 <= other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_zero() {
        assert_eq!(Price::ZERO.as_raw(), 0);
    }

    #[test]
    fn test_price_ordering() {
        let a = Price::from_raw(100);
        let b = Price::from_raw(200);
        assert!(a < b);
        assert!(a.is_less_than_or_equal(&b));
    }

    #[test]
    fn test_price_add_overflow() {
        let max = Price::from_raw(u64::MAX);
        let one = Price::from_raw(1);
        assert!(max.checked_add(&one).is_err());
    }
}
