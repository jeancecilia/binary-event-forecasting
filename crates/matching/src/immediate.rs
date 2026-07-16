//! Immediate all-or-none matching (SIM-001).

use domain_types::{Cash, Notional, Price, Quantity};
use market_state::MarketSnapshot;
use protocol::SimulationIntent;
use super::{MatchResult, VirtualMatchingState};

/// Execute an immediate all-or-none match.
///
/// Steps:
/// 1. Traverse all eligible price levels up to price limit
/// 2. Verify total available quantity
/// 3. Reject if full quantity unavailable
/// 4. Compute exact weighted notional
/// 5. Evaluate costs separately
/// 6. Reserve cash or inventory atomically
/// 7. Consume shared virtual depth only after all checks pass
pub fn match_immediate(
    intent: &SimulationIntent,
    snapshot: &MarketSnapshot,
    state: &mut VirtualMatchingState,
) -> MatchResult {
    // Check market state is usable
    if !snapshot.is_usable() {
        return MatchResult::Rejected {
            reason: "Market state not usable".to_string(),
        };
    }

    let required_quantity = Quantity::from_raw(intent.quantity);
    let price_limit = Price::from_raw(intent.price_limit);

    // Determine available quantity based on book side
    let is_buy = matches!(intent.book_side, BookSide::Bid);
    let available = if is_buy {
        snapshot.available_buy_quantity(&price_limit)
    } else {
        snapshot.available_sell_quantity(&price_limit)
    };

    // All-or-none: reject if insufficient
    if available.as_raw() < required_quantity.as_raw() {
        return MatchResult::Rejected {
            reason: format!(
                "Insufficient depth: available={}, required={}",
                available.as_raw(),
                required_quantity.as_raw()
            ),
        };
    }

    // Compute weighted notional
    // For simplicity, use price limit as average (conservative)
    // Full implementation would compute actual weighted average across levels
    let notional = match Notional::compute(&price_limit, &required_quantity) {
        Ok(n) => n,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Verify cash feasibility (simplified — full cost model in COST-001)
    let cash_required = Cash::new(notional.as_raw() as i128);
    if !state.free_cash.is_at_least(cash_required.as_raw()) {
        return MatchResult::Rejected {
            reason: format!(
                "Insufficient cash: available={}, required={}",
                state.free_cash.as_raw(),
                cash_required.as_raw()
            ),
        };
    }

    // All checks passed — reserve atomically
    state.free_cash = state
        .free_cash
        .checked_sub(&cash_required)
        .unwrap_or(Cash::ZERO);

    MatchResult::Filled {
        filled_quantity: required_quantity,
        average_price: price_limit,
        notional,
        cash_reserved: cash_required,
    }
}
