//! Closed enums for receipt status and disposition status (IPC-002).
//!
//! Unknown enum variants fail closed. ReceiptStatus and DispositionStatus
//! are separate closed enums.

use serde::{Deserialize, Serialize};

/// Receipt status returned by the Rust core upon receiving a forecast message.
///
/// Unknown variants cause deserialization to fail (fail-closed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReceiptStatus {
    /// Message valid and queued for processing
    AcceptedQueued,
    /// Previously processed message retried
    DuplicateRetry,
    /// Message expired before processing
    ExpiredOnArrival,
    /// Schema validation failed
    RejectedSchema,
    /// Probability or uncertainty out of bounds
    RejectedBounds,
    /// Queue full, message rejected
    RejectedCapacity,
    /// Target definition version mismatch
    RejectedTargetVersion,
    /// Sender rate limit exceeded
    RejectedRateLimit,
    /// Sequence regression in replay
    ReplaySequenceViolation,
    /// Core operating in degraded mode
    CoreDegraded,
}

/// Lifecycle disposition (terminal state) of a forecast message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DispositionStatus {
    /// Message passed all validation
    Validated,
    /// Forecast policy evaluated
    Evaluated,
    /// Policy chose to abstain
    Abstained,
    /// Intent submitted to matcher
    SimulationSubmitted,
    /// Intent fully simulated
    Simulated,
    /// Intent partially filled
    PartiallyFilled,
    /// Intent rejected by matcher
    SimulationRejected,
    /// Simulation error occurred
    SimulationFailed,
    /// Replaced by newer message
    Superseded,
    /// Removed from queue
    Evicted,
    /// Expired while queued
    ExpiredInQueue,
}

/// Order book side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BookSide {
    Bid,
    Ask,
}

/// Outcome side for a binary event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OutcomeSide {
    Yes,
    No,
}

/// Order class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OrderClass {
    ImmediateAllOrNone,
    Passive,
}

/// Time in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TimeInForce {
    ImmediateOrCancel,
    GoodTillCancelled,
    FillOrKill,
    Day,
}

/// Market feed status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FeedStatus {
    Initializing,
    Fragmented,
    Disconnected,
    Stale,
    Failed,
}

/// Resolution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ResolutionStatus {
    Open,
    Proposed,
    Disputed,
    PendingFinality,
    Final,
}

/// Terminal outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TerminalOutcome {
    Yes,
    No,
    Void,
    Cancelled,
    Invalid,
    DefinitionChanged,
}

/// Valuation status for NAV calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValuationStatus {
    Valued,
    PartiallyValued,
    Unpriceable,
    Stale,
    Fragmented,
}

/// Baseline status for dual-baseline protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BaselineStatus {
    ValidTwoSided,
    OneSided,
    Stale,
    Fragmented,
    Missing,
    SpreadTooWide,
}

/// Execution class for distinguishing local simulation from mock gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExecutionClass {
    LocalConservativeSimulation,
    ExternalizedLocalMockExecution,
}
