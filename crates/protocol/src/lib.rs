//! IPC Protocol — Versioned Message Types (IPC-001 through IPC-005)
//!
//! This crate defines the canonical Rust types for all cross-process messages.
//! Types must match the JSON Schema contracts exactly.
//!
//! ## Key Types
//!
//! - [`ForecastMessage`] — Incoming forecast from Python intelligence plane
//! - [`ReceiptAcknowledgement`] — Rust acknowledgement of receipt
//! - [`LifecycleDisposition`] — Terminal lifecycle state
//! - [`SimulationIntent`] — Deterministic intent derived from forecast

pub mod canonical;
pub mod disposition;
pub mod enums;
pub mod forecast;
pub mod framing;
pub mod intent;
pub mod market_event;
pub mod mock;
pub mod receipt;
pub mod manifest;

pub use enums::*;
pub use canonical::{canonical_hash, canonical_json};
pub use forecast::ForecastMessage;
pub use intent::SimulationIntent;
pub use receipt::ReceiptAcknowledgement;
pub use disposition::LifecycleDisposition;
pub use market_event::MarketEvent;
pub use mock::MockGatewayMessage;

/// Current schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Maximum signal frame bytes (1 MiB).
pub const MAX_SIGNAL_FRAME_BYTES: usize = 1_048_576;

/// Frame header size: 4-byte big-endian unsigned length.
pub const FRAME_HEADER_SIZE: usize = 4;
