//! Signed P&L type.

use serde::{Deserialize, Serialize};

use crate::DomainError;

/// Signed profit/loss represented as a scaled integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SignedPnl(i128);

impl SignedPnl {
    /// Zero P&L.
    pub const ZERO: Self = SignedPnl(0);

    /// Create a new SignedPnl value.
    pub fn new(raw: i128) -> Self {
        SignedPnl(raw)
    }

    /// Return the raw scaled value.
    pub fn as_raw(&self) -> i128 {
        self.0
    }

    /// Is this a profit (positive)?
    pub fn is_profit(&self) -> bool {
        self.0 > 0
    }

    /// Is this a loss (negative)?
    pub fn is_loss(&self) -> bool {
        self.0 < 0
    }

    /// Checked addition.
    pub fn checked_add(&self, other: &SignedPnl) -> Result<SignedPnl, DomainError> {
        self.0
            .checked_add(other.0)
            .map(SignedPnl)
            .ok_or_else(|| DomainError::Overflow {
                detail: format!("P&L overflow: {} + {}", self.0, other.0),
            })
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, other: &SignedPnl) -> Result<SignedPnl, DomainError> {
        self.0
            .checked_sub(other.0)
            .map(SignedPnl)
            .ok_or_else(|| DomainError::Overflow {
                detail: format!("P&L underflow: {} - {}", self.0, other.0),
            })
    }
}
