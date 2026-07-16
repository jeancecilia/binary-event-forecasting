//! Domain Types — Scaled Integer Primitives (TYP-001, TYP-002, TYP-003)
//!
//! This crate defines the explicit scaled integer domain types used throughout
//! the system. Binary floating-point arithmetic is prohibited in market-state
//! accounting, matching, ledger transitions, fees, and P&L.
//!
//! ## Types
//!
//! - [`Price`] — Scaled integer price
//! - [`Quantity`] — Scaled integer quantity
//! - [`Notional`] — `round(Price × Quantity / PriceScale)`
//! - [`Cash`] — Scaled integer cash balance
//! - [`ReservedCash`] — Cash reserved for open orders
//! - [`SignedPnl`] — Signed profit/loss
//! - [`ProbabilityScaled`] — Probability × ProbabilityScale
//!
//! ## Safety
//!
//! All arithmetic uses checked operations. Overflows and division-by-zero
//! return errors rather than panicking. State mutations are atomic — an
//! arithmetic failure must not partially mutate state.

pub mod cash;
pub mod notional;
pub mod price;
pub mod probability;
pub mod quantity;
pub mod signed_pnl;

mod checked;

pub use cash::{Cash, ReservedCash};
pub use notional::Notional;
pub use price::Price;
pub use probability::ProbabilityScaled;
pub use quantity::Quantity;
pub use signed_pnl::SignedPnl;

/// The default price scale (divisor for all price calculations).
pub const PRICE_SCALE: u64 = 100_000_000; // 8 decimal places

/// Errors that can occur during domain type operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("Overflow in arithmetic operation: {detail}")]
    Overflow { detail: String },

    #[error("Division by zero in: {detail}")]
    DivisionByZero { detail: String },

    #[error("Value {value} out of valid range [{min}, {max}]: {detail}")]
    OutOfRange {
        value: String,
        min: String,
        max: String,
        detail: String,
    },

    #[error("Invalid conversion: {detail}")]
    InvalidConversion { detail: String },

    #[error("Rounding error exceeds maximum: actual {actual}, max {max}")]
    RoundingError { actual: u64, max: u64 },
}
