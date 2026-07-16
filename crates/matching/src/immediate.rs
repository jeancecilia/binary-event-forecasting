//! Immediate all-or-none matching (SIM-001).
//!
//! The matcher builds a fill-plan first, validates everything against
//! a candidate state, then commits atomically. No partial state mutation.

use domain_types::{Cash, DomainError, Notional, Price, Quantity, ReservedCash};
use market_state::MarketSnapshot;
use protocol::{SimulationIntent, enums::BookSide};
use super::{MatchResult, VirtualMatchingState};

/// A pre-validated fill plan with per-level quantities.
#[derive(Debug, Clone)]
struct PlannedLevelFill {
    price_raw: u64,
    fill_quantity: Quantity,
    available_quantity: Quantity,
}

/// A complete fill plan that can be committed atomically.
#[derive(Debug, Clone)]
struct FillPlan {
    filled_quantity: Quantity,
    average_price: Price,
    notional: Notional,
    cash_required: Cash,
    level_fills: Vec<PlannedLevelFill>,
}

/// Execute an immediate all-or-none match.
///
/// Steps:
/// 1. Validate market status.
/// 2. Build a fill plan without mutating state.
/// 3. Traverse exact eligible price levels.
/// 4. Calculate exact weighted notional.
/// 5. Validate cash against candidate state.
/// 6. Validate shared virtual depth against candidate.
/// 7. Commit all changes atomically.
/// 8. Recheck ledger and depth invariants.
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

    // Step 3: All-or-none — reject if insufficient total
    if available.as_raw() < required_quantity.as_raw() {
        return MatchResult::Rejected {
            reason: format!(
                "Insufficient depth: available={}, required={}",
                available.as_raw(),
                required_quantity.as_raw()
            ),
        };
    }

    // Step 4: Build fill plan with exact per-level quantities
    let fill_plan = match build_fill_plan(intent, snapshot, &required_quantity) {
        Ok(plan) => plan,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Step 5: Create candidate state and validate everything
    let mut candidate = state.clone();

    // Validate cash
    let cash_required = Cash::new(fill_plan.notional.as_raw() as i128);
    if !candidate.free_cash.is_at_least(cash_required.as_raw()) {
        return MatchResult::Rejected {
            reason: format!(
                "Insufficient cash: available={}, required={}",
                candidate.free_cash.as_raw(),
                cash_required.as_raw()
            ),
        };
    }

    // Validate virtual depth for each level
    for level_fill in &fill_plan.level_fills {
        if !candidate.virtual_depth.can_consume(
            level_fill.price_raw,
            &level_fill.fill_quantity,
            &level_fill.available_quantity,
        ) {
            return MatchResult::Rejected {
                reason: format!(
                    "Virtual depth exceeded at price level {}",
                    level_fill.price_raw
                ),
            };
        }
    }

    // Step 6: Apply all mutations to candidate
    // Deduct free cash
    candidate.free_cash = match candidate.free_cash.checked_sub(&cash_required) {
        Ok(c) => c,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Reserve cash
    candidate.reserved_cash = match candidate.reserved_cash.checked_add(
        &ReservedCash::new(cash_required.as_raw() as u64),
    ) {
        Ok(r) => r,
        Err(e) => return MatchResult::Rejected { reason: e.to_string() },
    };

    // Consume virtual depth
    for level_fill in &fill_plan.level_fills {
        if let Err(e) = candidate.virtual_depth.consume(
            level_fill.price_raw,
            &level_fill.fill_quantity,
            &level_fill.available_quantity,
        ) {
            return MatchResult::Rejected { reason: e };
        }
    }

    // Step 7: Verify cash invariant on candidate
    if let Err(e) = candidate.verify_cash_invariant() {
        return MatchResult::Rejected {
            reason: format!("Cash invariant violation after fill: {e}"),
        };
    }

    // Step 8: All validations passed — commit
    *state = candidate;

    MatchResult::Filled {
        filled_quantity: fill_plan.filled_quantity,
        average_price: fill_plan.average_price,
        notional: fill_plan.notional,
        cash_reserved: Cash::new(fill_plan.notional.as_raw() as i128),
    }
}

/// Build a fill plan: traverse eligible levels in price-priority order
/// and compute exact weighted notional with per-level fill quantities.
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
    let mut level_fills: Vec<PlannedLevelFill> = Vec::new();

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

        // Record per-level fill
        level_fills.push(PlannedLevelFill {
            price_raw: level.price.as_raw(),
            fill_quantity: Quantity::from_raw(take),
            available_quantity: level.quantity,
        });

        // Accumulate weighted price
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

    if total_filled == 0 {
        return Err(DomainError::DivisionByZero {
            detail: "No quantity filled in plan".to_string(),
        });
    }

    // Compute weighted average price
    let half = total_filled as u128 / 2;
    let avg_price_raw = (weighted_price_sum + half) / total_filled as u128;
    if avg_price_raw > u64::MAX as u128 {
        return Err(DomainError::Overflow {
            detail: "Average price exceeds u64".to_string(),
        });
    }
    let average_price = Price::from_raw(avg_price_raw as u64);

    let filled_qty = Quantity::from_raw(total_filled);
    let notional = Notional::compute(&average_price, &filled_qty)?;

    Ok(FillPlan {
        filled_quantity: filled_qty,
        average_price,
        notional,
        cash_required: Cash::new(notional.as_raw() as i128),
        level_fills,
    })
}
