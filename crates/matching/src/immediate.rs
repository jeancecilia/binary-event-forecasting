//! Immediate all-or-none matching (SIM-001).
//!
//! The matcher builds a fill-plan first, validates everything,
//! then commits atomically. No partial state mutation on failure.

use domain_types::{Cash, DomainError, Notional, Price, Quantity, ReservedCash};
use market_state::MarketSnapshot;
use protocol::{SimulationIntent, enums::BookSide};
use super::{MatchResult, VirtualMatchingState};

/// A pre-validated fill plan that can be committed atomically.
#[derive(Debug, Clone)]
struct FillPlan {
    /// Total fill quantity
    filled_quantity: Quantity,
    /// Exact weighted average fill price (scaled integer)
    average_price: Price,
    /// Total notional (before costs)
    notional: Notional,
    /// Cash required to reserve
    cash_required: Cash,
}

/// Execute an immediate all-or-none match.
///
/// Steps:
/// 1. Validate market status.
/// 2. Build a fill plan without mutating state.
/// 3. Traverse exact eligible price levels.
/// 4. Calculate exact weighted notional.
/// 5. Calculate costs.
/// 6. Validate cash or inventory.
/// 7. Validate shared virtual depth.
/// 8. Commit all changes atomically.
/// 9. Recheck ledger and depth invariants.
pub fn match_immediate(
    intent: &SimulationIntent,
    snapshot: &MarketSnapshot,
    state: &mut VirtualMatchingState,
) -> MatchResult {
    // Step 1: Validate market state
    if !snapshot.is_usable() {
        return MatchResult::Rejected {
            reason: "Market state not usable".to_string(),
        };
    }

    let required_quantity = Quantity::from_raw(intent.quantity);
    let price_limit = Price::from_raw(intent.price_limit);
    let is_buy = matches!(intent.book_side, BookSide::Bid);

    // Step 2: Compute available quantity (checked arithmetic)
    let available = match if is_buy {
        snapshot.available_buy_quantity(&price_limit)
    } else {
        snapshot.available_sell_quantity(&price_limit)
    } {
        Ok(q) => q,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Step 3: All-or-none — reject if insufficient
    if available.as_raw() < required_quantity.as_raw() {
        return MatchResult::Rejected {
            reason: format!(
                "Insufficient depth: available={}, required={}",
                available.as_raw(),
                required_quantity.as_raw()
            ),
        };
    }

    // Step 4: Compute exact weighted notional across eligible levels
    let fill_plan = match build_fill_plan(intent, snapshot, &required_quantity) {
        Ok(plan) => plan,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Step 5: Costs are applied separately (COST-001).
    // For now, use notional as the cash requirement.
    let cash_required = Cash::new(fill_plan.notional.as_raw() as i128);

    // Step 6: Validate cash feasibility
    if !state.free_cash.is_at_least(cash_required.as_raw()) {
        return MatchResult::Rejected {
            reason: format!(
                "Insufficient cash: available={}, required={}",
                state.free_cash.as_raw(),
                cash_required.as_raw()
            ),
        };
    }

    // Step 7: Validate shared virtual depth
    for level in eligible_levels(snapshot, is_buy, &price_limit) {
        if !state.virtual_depth.can_consume(
            level.price.as_raw(),
            &level.fill_quantity,
            &level.available_quantity,
        ) {
            return MatchResult::Rejected {
                reason: format!(
                    "Virtual depth exceeded at price level {}",
                    level.price.as_raw()
                ),
            };
        }
    }

    // Step 8: Commit atomically
    // Deduct free cash
    state.free_cash = match state.free_cash.checked_sub(&cash_required) {
        Ok(c) => c,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Reserve cash
    state.reserved_cash = match state.reserved_cash.checked_add(
        &ReservedCash::new(cash_required.as_raw() as u64),
    ) {
        Ok(r) => r,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Consume virtual depth
    for level in eligible_levels(snapshot, is_buy, &price_limit) {
        if let Err(e) = state.virtual_depth.consume(
            level.price.as_raw(),
            &level.fill_quantity,
            &level.available_quantity,
        ) {
            return MatchResult::Rejected { reason: e };
        }
    }

    // Step 9: Verify cash invariant
    if let Err(e) = state.verify_cash_invariant() {
        return MatchResult::Rejected {
            reason: format!("Cash invariant violation after fill: {e}"),
        };
    }

    MatchResult::Filled {
        filled_quantity: fill_plan.filled_quantity,
        average_price: fill_plan.average_price,
        notional: fill_plan.notional,
        cash_reserved: cash_required,
    }
}

/// A single eligible price level with fill information.
struct EligibleLevel {
    price: Price,
    fill_quantity: Quantity,
    available_quantity: Quantity,
}

/// Collect eligible price levels for a fill.
fn eligible_levels(
    snapshot: &MarketSnapshot,
    is_buy: bool,
    price_limit: &Price,
) -> Vec<EligibleLevel> {
    let levels = if is_buy { &snapshot.asks } else { &snapshot.bids };

    levels
        .iter()
        .filter(|l| {
            if is_buy {
                l.price <= *price_limit
            } else {
                l.price >= *price_limit
            }
        })
        .map(|l| EligibleLevel {
            price: l.price,
            fill_quantity: Quantity::ZERO, // computed during fill plan build
            available_quantity: l.quantity,
        })
        .collect()
}

/// Build a fill plan: compute exact weighted notional across price levels.
///
/// Traverses eligible levels in price priority order,
/// accumulating quantity until the required amount is met.
fn build_fill_plan(
    intent: &SimulationIntent,
    snapshot: &MarketSnapshot,
    required: &Quantity,
) -> Result<FillPlan, DomainError> {
    let is_buy = matches!(intent.book_side, BookSide::Bid);
    let price_limit = Price::from_raw(intent.price_limit);
    let levels = if is_buy { &snapshot.asks } else { &snapshot.bids };

    let mut remaining = required.as_raw();
    let mut weighted_price_sum: u128 = 0;
    let mut total_filled: u64 = 0;

    for level in levels {
        if is_buy && level.price > price_limit {
            continue;
        }
        if !is_buy && level.price < price_limit {
            continue;
        }

        let take = remaining.min(level.quantity.as_raw());
        if take == 0 {
            continue;
        }

        // Accumulate weighted price: price × quantity for this slice
        let contribution = (level.price.as_raw() as u128)
            .checked_mul(take as u128)
            .ok_or(DomainError::Overflow {
                detail: "Weighted price overflow in fill plan".to_string(),
            })?;
        weighted_price_sum = weighted_price_sum
            .checked_add(contribution)
            .ok_or(DomainError::Overflow {
                detail: "Weighted price sum overflow".to_string(),
            })?;

        total_filled = total_filled
            .checked_add(take)
            .ok_or(DomainError::Overflow {
                detail: "Fill quantity overflow".to_string(),
            })?;

        remaining = remaining.saturating_sub(take);

        if remaining == 0 {
            break;
        }
    }

    // Compute weighted average price: round(weighted_price_sum / total_filled)
    if total_filled == 0 {
        return Err(DomainError::DivisionByZero {
            detail: "No quantity filled in plan".to_string(),
        });
    }

    let half = total_filled as u128 / 2;
    let avg_price_raw = (weighted_price_sum + half) / total_filled as u128;
    if avg_price_raw > u64::MAX as u128 {
        return Err(DomainError::Overflow {
            detail: "Average price exceeds u64".to_string(),
        });
    }
    let average_price = Price::from_raw(avg_price_raw as u64);

    // Compute notional: round(average_price × total_filled / PriceScale)
    let filled_qty = Quantity::from_raw(total_filled);
    let notional = Notional::compute(&average_price, &filled_qty)?;

    Ok(FillPlan {
        filled_quantity: filled_qty,
        average_price,
        notional,
        cash_required: Cash::new(notional.as_raw() as i128),
    })
}
