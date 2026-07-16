//! Market State — Order Books, Snapshots, and Feed Integrity (STA-001 through STA-003)
//!
//! Owns canonical order books, immutable market snapshots, feed integrity tracking,
//! and resynchronization logic. All observable market state flows through this crate.

pub mod order_book;
pub mod snapshot;
pub mod feed;
pub mod depth;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use domain_types::{Price, Quantity};
use protocol::enums::FeedStatus;

/// A price level in the order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price of this level
    pub price: Price,
    /// Total quantity at this level
    pub quantity: Quantity,
    /// Number of orders at this level (if known)
    pub order_count: Option<u64>,
}

/// Immutable market snapshot (STA-001).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    /// Target market identifier
    pub market_id: String,
    /// Contract/outcome identifier
    pub contract_or_outcome_id: String,
    /// Snapshot version (monotonically increasing)
    pub snapshot_version: u64,
    /// Feed connection generation
    pub feed_generation: u64,
    /// Source sequence metadata
    pub source_sequence: Option<u64>,
    /// Source timestamp
    pub source_timestamp: DateTime<Utc>,
    /// Logical observation timestamp
    pub logical_timestamp: i64,
    /// Feed synchronization status
    pub sync_status: FeedStatus,
    /// Ordered bid levels (descending by price)
    pub bids: Vec<PriceLevel>,
    /// Ordered ask levels (ascending by price)
    pub asks: Vec<PriceLevel>,
    /// Target definition version
    pub target_definition_version: String,
}

impl MarketSnapshot {
    /// Returns true if the snapshot is safe to use for matching/baseline/NAV.
    /// Only `Synchronized` feed status is accepted.
    pub fn is_usable(&self) -> bool {
        self.sync_status == FeedStatus::Synchronized
    }

    /// Compute available buy quantity up to a price limit, with checked arithmetic.
    /// Q_available_buy(p_L) = sum_{p ≤ p_L} q_ask(p)
    ///
    /// Returns an error if the sum overflows u64.
    pub fn available_buy_quantity(&self, price_limit: &Price) -> Result<Quantity, domain_types::DomainError> {
        let mut total: u64 = 0;
        for level in &self.asks {
            if level.price <= *price_limit {
                total = total
                    .checked_add(level.quantity.as_raw())
                    .ok_or(domain_types::DomainError::Overflow {
                        detail: "Buy quantity sum overflow".to_string(),
                    })?;
            }
        }
        Ok(Quantity::from_raw(total))
    }

    /// Compute available sell quantity at or above a price limit, with checked arithmetic.
    /// Q_available_sell(p_L) = sum_{p ≥ p_L} q_bid(p)
    ///
    /// Returns an error if the sum overflows u64.
    pub fn available_sell_quantity(&self, price_limit: &Price) -> Result<Quantity, domain_types::DomainError> {
        let mut total: u64 = 0;
        for level in &self.bids {
            if level.price >= *price_limit {
                total = total
                    .checked_add(level.quantity.as_raw())
                    .ok_or(domain_types::DomainError::Overflow {
                        detail: "Sell quantity sum overflow".to_string(),
                    })?;
            }
        }
        Ok(Quantity::from_raw(total))
    }
}
