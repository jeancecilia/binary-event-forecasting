//! Immediate all-or-none matching (SIM-001).
//!
//! The matcher builds a fill-plan first, validates everything against
//! a candidate state, then commits atomically. No partial state mutation.

use super::{cost_model::CostModel, virtual_depth::DepthKey, MatchResult, VirtualMatchingState};
use domain_types::{Cash, DomainError, Notional, Price, Quantity, ReservedCash};
use market_state::MarketSnapshot;
use protocol::{enums::BookSide, SimulationIntent};

/// A pre-validated fill plan with per-level quantities.
#[derive(Debug, Clone)]
struct PlannedLevelFill {
    depth_key: DepthKey,
    fill_quantity: Quantity,
    available_quantity: Quantity,
}

/// A complete fill plan that can be committed atomically.
#[derive(Debug, Clone)]
struct FillPlan {
    filled_quantity: Quantity,
    average_price: Price,
    notional: Notional,
    level_fills: Vec<PlannedLevelFill>,
}

/// Execute an immediate all-or-none match.
///
/// Steps:
/// 1. Validate market status.
/// 2. Build a fill plan without mutating state.
/// 3. Calculate exact weighted notional.
/// 4. Validate cash against candidate state.
/// 5. Validate shared virtual depth against candidate.
/// 6. Commit all changes atomically.
/// 7. Recheck ledger and depth invariants.
pub fn match_immediate(
    intent: &SimulationIntent,
    snapshot: &MarketSnapshot,
    state: &mut VirtualMatchingState,
    cost_model: &dyn CostModel,
) -> MatchResult {
    if !snapshot.is_usable() {
        return MatchResult::Rejected {
            reason: "Market state not usable".to_string(),
        };
    }

    let required_quantity = Quantity::from_raw(intent.quantity);
    let is_buy = matches!(intent.book_side, BookSide::Bid);

    let fill_plan =
        match build_fill_plan(intent, snapshot, &required_quantity, &state.virtual_depth) {
            Ok(plan) => plan,
            Err(e) => {
                return MatchResult::Rejected {
                    reason: e.to_string(),
                }
            }
        };

    let mut candidate = state.clone();

    let cash_required = match cost_model.calculate_total_cash(fill_plan.notional) {
        Ok(c) => c,
        Err(e) => {
            return MatchResult::Rejected {
                reason: format!("Cost model error: {e}"),
            }
        }
    };

    let inv_key = crate::inventory::InventoryKey {
        market_id: intent.market_id.clone(),
        outcome_id: intent.contract_or_outcome_id.clone(),
        side: intent.outcome_side,
    };
    let mut inv_line = candidate.inventory.get_line(&inv_key);

    if is_buy {
        if !candidate.free_cash.is_at_least(cash_required.as_raw()) {
            return MatchResult::Rejected {
                reason: format!(
                    "Insufficient cash: available={}, required={}",
                    candidate.free_cash.as_raw(),
                    cash_required.as_raw()
                ),
            };
        }
    } else if !inv_line.free.is_at_least(&fill_plan.filled_quantity) {
        return MatchResult::Rejected {
            reason: format!(
                "Insufficient inventory: available={}, required={}",
                inv_line.free.as_raw(),
                fill_plan.filled_quantity.as_raw()
            ),
        };
    }

    for level in &fill_plan.level_fills {
        if !candidate.virtual_depth.can_consume(
            &level.depth_key,
            &level.fill_quantity,
            &level.available_quantity,
        ) {
            return MatchResult::Rejected {
                reason: format!("Virtual depth exceeded at {:?}", level.depth_key),
            };
        }
    }

    if is_buy {
        candidate.free_cash = match candidate.free_cash.checked_sub(&cash_required) {
            Ok(c) => c,
            Err(e) => {
                return MatchResult::Rejected {
                    reason: e.to_string(),
                }
            }
        };
        candidate.reserved_cash = match candidate
            .reserved_cash
            .checked_add(&ReservedCash::new(cash_required.as_raw() as u64))
        {
            Ok(r) => r,
            Err(e) => {
                return MatchResult::Rejected {
                    reason: e.to_string(),
                }
            }
        };
    } else {
        inv_line.free = match inv_line.free.checked_sub(&fill_plan.filled_quantity) {
            Ok(c) => c,
            Err(e) => {
                return MatchResult::Rejected {
                    reason: e.to_string(),
                }
            }
        };
        inv_line.reserved = match inv_line.reserved.checked_add(&fill_plan.filled_quantity) {
            Ok(r) => r,
            Err(e) => {
                return MatchResult::Rejected {
                    reason: e.to_string(),
                }
            }
        };

        if let Err(e) = candidate.inventory.insert_line(inv_key, inv_line) {
            return MatchResult::Rejected { reason: e };
        }
    }

    for level in &fill_plan.level_fills {
        if let Err(e) = candidate.virtual_depth.consume(
            &level.depth_key,
            &level.fill_quantity,
            &level.available_quantity,
        ) {
            return MatchResult::Rejected { reason: e };
        }
    }

    if let Err(e) = candidate.verify_cash_invariant() {
        return MatchResult::Rejected {
            reason: format!("Cash invariant violation after fill: {e}"),
        };
    }

    *state = candidate;

    MatchResult::Filled {
        filled_quantity: fill_plan.filled_quantity,
        average_price: fill_plan.average_price,
        notional: fill_plan.notional,
        cash_reserved: if is_buy { cash_required } else { Cash::new(0) },
        inventory_reserved: if is_buy {
            Quantity::ZERO
        } else {
            fill_plan.filled_quantity
        },
    }
}

fn build_fill_plan(
    intent: &SimulationIntent,
    snapshot: &MarketSnapshot,
    required: &Quantity,
    virtual_depth: &super::virtual_depth::VirtualDepth,
) -> Result<FillPlan, DomainError> {
    let is_buy = matches!(intent.book_side, BookSide::Bid);
    let price_limit = Price::from_raw(intent.price_limit);
    let levels = if is_buy {
        &snapshot.asks
    } else {
        &snapshot.bids
    };
    let book_side = if is_buy { BookSide::Ask } else { BookSide::Bid };

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

        let depth_key = DepthKey {
            market_id: snapshot.market_id.clone(),
            contract_or_outcome_id: snapshot.contract_or_outcome_id.clone(),
            book_side,
            price_raw: level.price.as_raw(),
            feed_generation: snapshot.feed_generation,
        };

        let already = virtual_depth.consumed_quantity(&depth_key);
        let actual_available = level.quantity.as_raw().saturating_sub(already);
        let take = remaining.min(actual_available);
        if take == 0 {
            continue;
        }

        level_fills.push(PlannedLevelFill {
            depth_key,
            fill_quantity: Quantity::from_raw(take),
            available_quantity: level.quantity,
        });

        let contribution = (level.price.as_raw() as u128)
            .checked_mul(take as u128)
            .ok_or(DomainError::Overflow {
                detail: "Weighted price overflow".to_string(),
            })?;
        weighted_price_sum =
            weighted_price_sum
                .checked_add(contribution)
                .ok_or(DomainError::Overflow {
                    detail: "Weighted sum overflow".to_string(),
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
            detail: "No quantity filled".to_string(),
        });
    }

    if remaining != 0 {
        return Err(DomainError::InsufficientDepth {
            available: total_filled,
            required: required.as_raw(),
        });
    }

    let half_scale = domain_types::PRICE_SCALE as u128 / 2;
    let notional_raw = (weighted_price_sum + half_scale) / (domain_types::PRICE_SCALE as u128);
    if notional_raw > u64::MAX as u128 {
        return Err(DomainError::Overflow {
            detail: "Notional exceeds u64".to_string(),
        });
    }
    let notional = Notional::from_raw(notional_raw as u64);

    let half = total_filled as u128 / 2;
    let avg_price_raw = (weighted_price_sum + half) / total_filled as u128;
    if avg_price_raw > u64::MAX as u128 {
        return Err(DomainError::Overflow {
            detail: "Avg price exceeds u64".to_string(),
        });
    }
    let average_price = Price::from_raw(avg_price_raw as u64);
    let filled_qty = Quantity::from_raw(total_filled);

    Ok(FillPlan {
        filled_quantity: filled_qty,
        average_price,
        notional,
        level_fills,
    })
}
