//! Demo Gateway (DEM-001 through DEM-006).
//!
//! A configurable gateway that provides scripted market lifecycle events:
//! acknowledgements, rejections, partial fills, complete fills, cancellations,
//! and settlement events, with immutable versioned scenario traces.

pub mod config;
pub mod scenario;
pub mod server;
pub mod trace;

use std::path::PathBuf;

/// Mock gateway configuration.
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// Environment identifier
    pub environment: String,
    /// Scenario identifier
    pub scenario_id: String,
    /// Configuration hash
    pub config_hash: String,
    /// Gateway build hash
    pub build_hash: String,
    /// TCP bind address
    pub bind_address: String,
    /// Path to scenario definitions
    pub scenarios_path: PathBuf,
    /// Path for trace output
    pub trace_path: PathBuf,
}
