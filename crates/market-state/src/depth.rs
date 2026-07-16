//! Order book depth calculations (STA-003).

use domain_types::{Price, Quantity};
use super::MarketSnapshot;

/// Result of a depth check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepthResult {
    /// Valid state with sufficient depth
    Sufficient { available: Quantity },
    /// Valid state with insufficient depth
    InsufficientDepth { available: Quantity, required: Quantity },
    /// Invalid or stale market state
    StateInvalid { reason: String },
    /// Unobservable queue progression
    Unobservable { reason: String },
}

/// Evaluate whether a snapshot has sufficient depth for a requested quantity.
pub fn evaluate_depth(
    snapshot: &MarketSnapshot,
    is_buy: bool,
    price_limit: &Price,
    required_quantity: &Quantity,
) -> DepthResult {
    if !snapshot.is_usable() {
        return DepthResult::StateInvalid {
            reason: format!("Market state is {:?}", snapshot.sync_status),
        };
    }

    let available = if is_buy {
        snapshot.available_buy_quantity(price_limit)
    } else {
        snapshot.available_sell_quantity(price_limit)
    };

    if available.as_raw() >= required_quantity.as_raw() {
        DepthResult::Sufficient { available }
    } else {
        DepthResult::InsufficientDepth {
            available,
            required: *required_quantity,
        }
    }
}
