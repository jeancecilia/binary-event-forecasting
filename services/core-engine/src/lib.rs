//! Binary Event Forecasting — Core Simulation Engine
//!
//! The Rust core is the sole owner of canonical simulation state.
//! It handles market-event ingestion, snapshot construction, forecast
//! validation, matching, ledger transitions, and deterministic replay.
//!
//! ## Architecture
//!
//! The core engine composes internal crates in a strict layered dependency:
//!
//! ```text
//! domain-types → protocol → market-state → forecast-policy
//!     → matching → ledger → journal/replay → core-engine
//! ```
//!
//! ## Operating Modes
//!
//! - **Replay**: Deterministic offline replay from frozen traces
//! - **Prospective**: Configured research data consumption
//! - **Mock**: Demo gateway integration

pub mod config;
pub mod ipc;
pub mod lifecycle;
pub mod modes;

use std::path::PathBuf;

/// Core engine configuration loaded from TOML.
#[derive(Debug, Clone)]
pub struct CoreConfig {
    /// Path to the IPC socket
    pub socket_path: PathBuf,
    /// Path to the SQLite journal
    pub journal_path: PathBuf,
    /// Path to the spool
    pub spool_path: PathBuf,
    /// PostgreSQL connection string (optional)
    pub postgres_url: Option<String>,
    /// Operating mode
    pub mode: OperatingMode,
    /// Maximum signal frame bytes
    pub max_signal_frame_bytes: usize,
    /// Read timeout in milliseconds
    pub read_timeout_ms: u64,
    /// Idle timeout in milliseconds
    pub idle_timeout_ms: u64,
}

/// Operating modes of the core engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatingMode {
    /// Offline replay from frozen inputs
    Replay,
    /// Prospective observation through configured data sources
    Prospective,
    /// Demo gateway integration
    Mock,
}

impl CoreConfig {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: config::TomlConfig = toml::from_str(&content)?;
        config.into_core_config()
    }

    /// Returns true if the engine is in replay mode.
    pub fn is_replay(&self) -> bool {
        self.mode == OperatingMode::Replay
    }

    /// Returns true if the engine is in prospective mode.
    pub fn is_prospective(&self) -> bool {
        self.mode == OperatingMode::Prospective
    }
}
