//! Order book implementation.

use domain_types::{Price, Quantity};
use protocol::enums::FeedStatus;
use protocol::market_event::{MarketEvent, MarketEventType};
use serde_json::Value;

pub use super::PriceLevel;
use super::MarketSnapshot;

/// Builds an immutable MarketSnapshot from incremental events.
#[derive(Debug)]
pub struct OrderBookBuilder {
    pub market_id: String,
    pub contract_or_outcome_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub feed_generation: u64,
    pub snapshot_version: u64,
    pub sync_status: FeedStatus,
    pub target_definition_version: String,
    pub source_sequence: Option<u64>,
    pub expected_sequence: Option<u64>,
    pub processed_event_ids: std::collections::HashSet<String>,
}

impl OrderBookBuilder {
    pub fn new(market_id: &str, outcome_id: &str, target_definition: &str) -> Self {
        Self {
            market_id: market_id.to_string(),
            contract_or_outcome_id: outcome_id.to_string(),
            bids: Vec::new(),
            asks: Vec::new(),
            feed_generation: 0,
            snapshot_version: 0,
            target_definition_version: target_definition.to_string(),
            sync_status: FeedStatus::Initializing,
            source_sequence: None,
            expected_sequence: None,
            processed_event_ids: std::collections::HashSet::new(),
        }
    }

    /// Process a deterministic sequence of market events.
    pub fn apply_events(&mut self, events: &[MarketEvent]) -> Result<(), String> {
        let mut sorted_events = events.to_vec();
        // Deterministic sort: logical_timestamp -> source_sequence -> event_type -> event_id
        sorted_events.sort_by(|a, b| {
            a.logical_timestamp.cmp(&b.logical_timestamp)
                .then(a.source_sequence.cmp(&b.source_sequence))
                .then((a.event_type as u8).cmp(&(b.event_type as u8)))
                .then(a.event_id.cmp(&b.event_id))
        });

        for ev in sorted_events {
            self.apply_event(&ev)?;
        }
        Ok(())
    }

    fn apply_event(&mut self, ev: &MarketEvent) -> Result<(), String> {
        if !self.processed_event_ids.insert(ev.event_id.clone()) {
            return Err(format!("Duplicate event ID: {}", ev.event_id));
        }

        if let Some(seq) = ev.source_sequence {
            if let Some(expected) = self.expected_sequence {
                if seq > expected {
                    return Err(format!("Sequence gap detected: expected {}, got {}", expected, seq));
                }
                if seq < expected {
                    return Err(format!("Sequence regression detected: expected {}, got {}", expected, seq));
                }
            }
            self.expected_sequence = Some(seq + 1);
        }

        self.source_sequence = ev.source_sequence;
        self.snapshot_version += 1;

        match ev.event_type {
            MarketEventType::OrderBookSnapshot => {
                self.parse_snapshot(&ev.payload)?;
                self.sync_status = FeedStatus::Synchronized;
            }
            MarketEventType::FeedStatusChange => {
                if let Some(status_str) = ev.payload.get("status").and_then(Value::as_str) {
                    if status_str == "Fragmented" {
                        self.sync_status = FeedStatus::Fragmented;
                    }
                }
            }
            _ => {
                // Future implementation: bid/ask updates
            }
        }
        Ok(())
    }

    fn parse_snapshot(&mut self, payload: &Value) -> Result<(), String> {
        // Parse bids and asks
        let bids = payload.get("bids").and_then(Value::as_array).ok_or("Missing bids")?;
        let asks = payload.get("asks").and_then(Value::as_array).ok_or("Missing asks")?;

        self.bids.clear();
        for b in bids {
            let p = b.get("price").and_then(Value::as_u64).ok_or("Invalid bid price")?;
            let q = b.get("quantity").and_then(Value::as_u64).ok_or("Invalid bid quantity")?;
            if q == 0 {
                return Err("Zero quantity in bid level".to_string());
            }
            self.bids.push(PriceLevel {
                price: Price::from_raw(p),
                quantity: Quantity::from_raw(q),
                order_count: b.get("order_count").and_then(Value::as_u64),
            });
        }
        self.bids.sort_by(|a, b| b.price.cmp(&a.price)); // Descending

        self.asks.clear();
        for a in asks {
            let p = a.get("price").and_then(Value::as_u64).ok_or("Invalid ask price")?;
            let q = a.get("quantity").and_then(Value::as_u64).ok_or("Invalid ask quantity")?;
            if q == 0 {
                return Err("Zero quantity in ask level".to_string());
            }
            self.asks.push(PriceLevel {
                price: Price::from_raw(p),
                quantity: Quantity::from_raw(q),
                order_count: a.get("order_count").and_then(Value::as_u64),
            });
        }
        self.asks.sort_by(|a, b| a.price.cmp(&b.price)); // Ascending

        self.feed_generation += 1;
        Ok(())
    }

    /// Construct the immutable market snapshot, validating invariants.
    pub fn build(
        &self, 
        logical_timestamp: i64, 
        source_timestamp: chrono::DateTime<chrono::Utc>
    ) -> Result<MarketSnapshot, String> {
        if self.sync_status != FeedStatus::Synchronized {
            return Err("Cannot build from non-synchronized state".into());
        }

        // Validate crossed book
        if let (Some(best_bid), Some(best_ask)) = (self.bids.first(), self.asks.first()) {
            if best_bid.price >= best_ask.price {
                return Err("Crossed book".into());
            }
        }

        Ok(MarketSnapshot {
            market_id: self.market_id.clone(),
            contract_or_outcome_id: self.contract_or_outcome_id.clone(),
            snapshot_version: self.snapshot_version,
            feed_generation: self.feed_generation,
            source_sequence: self.source_sequence,
            source_timestamp,
            logical_timestamp,
            sync_status: self.sync_status.clone(),
            bids: self.bids.clone(),
            asks: self.asks.clone(),
            target_definition_version: self.target_definition_version.clone(),
        })
    }
}
