//! Feed integrity and resynchronization (STA-002).

use protocol::enums::FeedStatus;

/// Track the integrity state of a market data feed.
#[derive(Debug, Clone)]
pub struct FeedIntegrity {
    pub market_id: String,
    pub status: FeedStatus,
    pub last_valid_sequence: Option<u64>,
    pub gap_detected_at: Option<i64>,
}

impl FeedIntegrity {
    /// Mark a feed as fragmented due to a detected gap or integrity failure.
    pub fn mark_fragmented(&mut self, logical_timestamp: i64) {
        self.status = FeedStatus::Fragmented;
        self.gap_detected_at = Some(logical_timestamp);
    }

    /// Returns true if the feed is in a usable state.
    pub fn is_usable(&self) -> bool {
        !matches!(
            self.status,
            FeedStatus::Initializing
                | FeedStatus::Fragmented
                | FeedStatus::Disconnected
                | FeedStatus::Stale
                | FeedStatus::Failed
        )
    }
}
