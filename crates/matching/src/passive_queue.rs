//! Passive queue lifecycle simulation (SIM-002).

use domain_types::Quantity;
use super::{MatchResult, VirtualMatchingState};
use market_state::MarketSnapshot;
use protocol::SimulationIntent;

/// Acknowledge a passive intent and insert into the virtual queue.
///
/// Under the conservative model:
/// - Only confirmed aggressive trade volume reduces quantity ahead
/// - Unclassified size reductions do not improve queue position
/// - Price traversal without matching evidence does not imply a fill
/// - Partial fills are represented explicitly
/// - Cancellation becomes effective only after configured latency
/// - Unidentifiable queue progression produces `Unobservable`
pub fn acknowledge_passive(
    intent: &SimulationIntent,
    snapshot: &MarketSnapshot,
    _state: &VirtualMatchingState,
) -> MatchResult {
    if !snapshot.is_usable() {
        return MatchResult::Rejected {
            reason: "Market state not usable for passive order".to_string(),
        };
    }

    let price = intent.price_limit;
    let is_buy = matches!(intent.book_side, protocol::enums::BookSide::Bid);

    // Find quantity resting at this price level before insertion
    let resting_qty: u64 = if is_buy {
        snapshot
            .bids
            .iter()
            .filter(|l| l.price.as_raw() == price)
            .map(|l| l.quantity.as_raw())
            .sum()
    } else {
        snapshot
            .asks
            .iter()
            .filter(|l| l.price.as_raw() == price)
            .map(|l| l.quantity.as_raw())
            .sum()
    };

    MatchResult::Queued {
        quantity_ahead: Quantity::from_raw(resting_qty),
        queue_position: 0, // Computed after full queue model
    }
}
