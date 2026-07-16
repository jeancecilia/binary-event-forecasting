//! Notional type — `round(Price × Quantity / PriceScale)`

use serde::{Deserialize, Serialize};

use crate::{price::Price, quantity::Quantity, DomainError, PRICE_SCALE};

/// Notional value computed from price and quantity.
///
/// Formula: `Notional = round(Price × Quantity / PriceScale)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Notional(u64);

impl Notional {
    /// Zero notional.
    pub const ZERO: Self = Notional(0);

    /// Create a Notional directly from a raw value.
    pub const fn from_raw(raw: u64) -> Self {
        Notional(raw)
    }

    /// Compute notional from price and quantity using wide intermediate arithmetic.
    ///
    /// Uses 128-bit intermediates to prevent overflow:
    /// `notional = round(price_raw × quantity_raw / PRICE_SCALE)`
    pub fn compute(price: &Price, quantity: &Quantity) -> Result<Self, DomainError> {
        let price_raw = price.as_raw() as u128;
        let quantity_raw = quantity.as_raw() as u128;

        let product = price_raw
            .checked_mul(quantity_raw)
            .ok_or_else(|| DomainError::Overflow {
                detail: format!("Notional overflow: {} × {}", price_raw, quantity_raw),
            })?;

        // Round to nearest: add PRICE_SCALE/2 before division
        let half_scale = PRICE_SCALE as u128 / 2;
        let rounded = product
            .checked_add(half_scale)
            .ok_or_else(|| DomainError::Overflow {
                detail: "Notional overflow during rounding".to_string(),
            })?;

        let result = rounded / PRICE_SCALE as u128;

        if result > u64::MAX as u128 {
            return Err(DomainError::Overflow {
                detail: format!("Notional result {result} exceeds u64::MAX"),
            });
        }

        Ok(Notional(result as u64))
    }

    /// Return the raw value.
    pub fn as_raw(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_notional_basic() {
        // price = 0.50 (50_000_000 at scale 100_000_000)
        // quantity = 100
        // notional = round(50_000_000 * 100 / 100_000_000) = 50
        let price = Price::from_raw(50_000_000);
        let quantity = Quantity::from_raw(100);
        let notional = Notional::compute(&price, &quantity).unwrap();
        assert_eq!(notional.as_raw(), 50);
    }

    #[test]
    fn test_notional_rounding_up() {
        // price = 0.625 (62_500_000), quantity = 1
        // product = 62_500_000, half_scale = 50_000_000
        // (62_500_000 + 50_000_000) / 100_000_000 = 1
        let price = Price::from_raw(62_500_000);
        let quantity = Quantity::from_raw(1);
        let notional = Notional::compute(&price, &quantity).unwrap();
        assert_eq!(notional.as_raw(), 1);
    }
}
