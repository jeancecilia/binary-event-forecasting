//! Telemetry — Cross-Process Latency, Timing, and Metrics Reporting (TEL-001, TEL-002)
//!
//! Reports timing information without subtracting unrelated language-runtime
//! monotonic timestamps. Separates deterministic research artifacts from
//! runtime telemetry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Cross-process latency breakdown (TEL-001).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTelemetry {
    /// Python enqueue duration (Python-side measurement)
    pub python_enqueue_us: Option<u64>,
    /// IPC round-trip time (measured at Rust-side)
    pub ipc_round_trip_us: Option<u64>,
    /// Rust receive-to-parse duration
    pub rust_receive_to_parse_us: Option<u64>,
    /// Rust parse-to-acknowledgement duration
    pub rust_parse_to_ack_us: Option<u64>,
    /// Rust acknowledgement-to-disposition duration
    pub rust_ack_to_disposition_us: Option<u64>,
    /// Directly measured end-to-end duration
    pub end_to_end_us: Option<u64>,
    /// Sum of component measurements (reported separately)
    pub component_sum_us: Option<u64>,
    /// Any estimated one-way latency with stated assumptions
    pub estimated_one_way_us: Option<u64>,
    /// Assumptions for one-way estimate
    pub one_way_assumptions: Option<String>,
}

/// Replay performance telemetry (should not affect deterministic state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayTelemetry {
    /// Wall-clock start time
    pub started_at: DateTime<Utc>,
    /// Wall-clock end time
    pub ended_at: DateTime<Utc>,
    /// Total events processed
    pub events_processed: u64,
    /// Events per second (wall-clock)
    pub events_per_second: f64,
    /// Peak memory usage (bytes)
    pub peak_memory_bytes: Option<u64>,
}

/// Simulation performance metrics (MET-004).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    /// Gross simulated P&L
    pub gross_pnl: i128,
    /// Net simulated P&L (after costs)
    pub net_pnl: i128,
    /// Total costs
    pub total_costs: i128,
    /// Realized P&L
    pub realized_pnl: i128,
    /// Unrealized P&L
    pub unrealized_pnl: i128,
    /// Fill rate
    pub fill_rate: f64,
    /// Partial fill rate
    pub partial_fill_rate: f64,
    /// Rejection rate
    pub rejection_rate: f64,
    /// Unobservable rate
    pub unobservable_rate: f64,
    /// Slippage (weighted)
    pub slippage_bps: f64,
    /// Realized spread
    pub realized_spread_bps: f64,
    /// Post-fill markout
    pub post_fill_markout_bps: Option<f64>,
    /// Maximum drawdown
    pub max_drawdown: i128,
    /// Results by latency scenario
    pub by_latency: serde_json::Value,
    /// Results by participation-ratio segment
    pub by_participation_ratio: serde_json::Value,
}
