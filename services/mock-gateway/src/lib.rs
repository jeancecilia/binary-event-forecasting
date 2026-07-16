//! Local Mock Demo Gateway (DEM-001 through DEM-006).
//!
//! A local-only mock gateway that provides scripted market lifecycle events:
//! acknowledgements, rejections, partial fills, complete fills, cancellations,
//! and settlement events. Runs entirely within the controlled research environment.
//!
//! ## Boundaries
//!
//! - Binds only local interfaces (127.0.0.1 or AF_UNIX)
//! - Rejects configurations with external hostnames, credentials, or private keys
//! - Every scenario carries `environment = LOCAL_MOCK_DEMO`
//! - Produces immutable, versioned scenario traces

pub mod config;
pub mod scenario;
pub mod server;
pub mod trace;

use std::path::PathBuf;

/// Mock gateway configuration.
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// Environment identifier (must be LOCAL_MOCK_DEMO)
    pub environment: String,
    /// Scenario identifier
    pub scenario_id: String,
    /// Configuration hash
    pub config_hash: String,
    /// Gateway build hash
    pub build_hash: String,
    /// Bind address (127.0.0.1:port or AF_UNIX path)
    pub bind_address: String,
    /// Path to scenario definitions
    pub scenarios_path: PathBuf,
    /// Path for trace output
    pub trace_path: PathBuf,
}

impl MockConfig {
    /// Validate that the environment is LOCAL_MOCK_DEMO.
    pub fn validate_environment(&self) -> Result<(), String> {
        if self.environment != "LOCAL_MOCK_DEMO" {
            return Err(format!(
                "Unknown environment '{}'. Only LOCAL_MOCK_DEMO is permitted.",
                self.environment
            ));
        }
        Ok(())
    }
}
