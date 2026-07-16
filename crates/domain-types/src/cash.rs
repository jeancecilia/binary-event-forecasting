//! Cash and ReservedCash types.

use serde::{Deserialize, Serialize};

use crate::{checked, DomainError};

/// Cash balance represented as a scaled integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cash(i128);

/// Cash reserved for open orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReservedCash(u64);

impl Cash {
    /// Zero cash.
    pub const ZERO: Self = Cash(0);

    /// Create a new Cash value.
    pub fn new(raw: i128) -> Self {
        Cash(raw)
    }

    /// Return the raw scaled value.
    pub fn as_raw(&self) -> i128 {
        self.0
    }

    /// Checked addition.
    pub fn checked_add(&self, other: &Cash) -> Result<Cash, DomainError> {
        self.0
            .checked_add(other.0)
            .map(Cash)
            .ok_or_else(|| DomainError::Overflow {
                detail: format!("Cash overflow: {} + {}", self.0, other.0),
            })
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, other: &Cash) -> Result<Cash, DomainError> {
        self.0
            .checked_sub(other.0)
            .map(Cash)
            .ok_or_else(|| DomainError::Overflow {
                detail: format!("Cash underflow: {} - {}", self.0, other.0),
            })
    }

    /// Returns true if cash is sufficient for the requirement.
    pub fn is_at_least(&self, required: i128) -> bool {
        self.0 >= required
    }
}

impl ReservedCash {
    /// Zero reserved cash.
    pub const ZERO: Self = ReservedCash(0);

    /// Create new ReservedCash.
    pub fn new(raw: u64) -> Self {
        ReservedCash(raw)
    }

    /// Return the raw value.
    pub fn as_raw(&self) -> u64 {
        self.0
    }

    /// Checked addition.
    pub fn checked_add(&self, other: &ReservedCash) -> Result<ReservedCash, DomainError> {
        checked::checked_add_u64(self.0, other.0, "ReservedCash").map(ReservedCash)
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, other: &ReservedCash) -> Result<ReservedCash, DomainError> {
        checked::checked_sub_u64(self.0, other.0, "ReservedCash").map(ReservedCash)
    }
}
