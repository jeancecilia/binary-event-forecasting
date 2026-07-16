//! Checked arithmetic helpers.
//!
//! All arithmetic uses checked operations. None of these functions panic.

use crate::DomainError;

/// Checked u64 addition.
pub fn checked_add_u64(a: u64, b: u64, type_name: &str) -> Result<u64, DomainError> {
    a.checked_add(b).ok_or_else(|| DomainError::Overflow {
        detail: format!("{type_name} overflow: {a} + {b}"),
    })
}

/// Checked u64 subtraction.
pub fn checked_sub_u64(a: u64, b: u64, type_name: &str) -> Result<u64, DomainError> {
    a.checked_sub(b).ok_or_else(|| DomainError::Overflow {
        detail: format!("{type_name} underflow: {a} - {b}"),
    })
}

/// Checked u64 multiplication.
pub fn checked_mul_u64(a: u64, b: u64, type_name: &str) -> Result<u64, DomainError> {
    a.checked_mul(b).ok_or_else(|| DomainError::Overflow {
        detail: format!("{type_name} overflow: {a} × {b}"),
    })
}

/// Checked u64 division.
pub fn checked_div_u64(a: u64, b: u64, type_name: &str) -> Result<u64, DomainError> {
    if b == 0 {
        return Err(DomainError::DivisionByZero {
            detail: format!("{type_name} division by zero"),
        });
    }
    Ok(a / b)
}
