//! Deterministic Offline Replay (REP-001)
//!
//! Reads frozen market-event traces from Parquet, advances a deterministic
//! logical clock, and replays all simulation steps. Produces canonical
//! final-state hashes that must be identical across repeated runs.

use sha2::{Digest, Sha256};

/// Configuration for an offline replay run.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Path to the trace directory
    pub trace_path: String,
    /// Path to model artifacts
    pub artifact_path: String,
    /// Whether to verify determinism (run twice and compare)
    pub verify: bool,
    /// Configuration hash for this replay
    pub config_hash: String,
}

/// Result of a replay run.
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Final canonical state hash
    pub final_state_hash: String,
    /// Number of events replayed
    pub events_processed: u64,
    /// Number of intents simulated
    pub intents_simulated: u64,
    /// Whether the run was deterministic (only set when verify=true)
    pub deterministic: Option<bool>,
    /// Any violations detected
    pub violations: Vec<String>,
}

/// Compute a SHA-256 hash of the full state for determinism verification.
pub fn compute_state_hash<T: serde::Serialize>(state: &T) -> Result<String, String> {
    let json = serde_json::to_string(state).map_err(|e| e.to_string())?;
    let hash = Sha256::digest(json.as_bytes());
    Ok(hex::encode(hash))
}

/// Verify that two replay results produce identical final-state hashes.
pub fn verify_determinism(run_a: &ReplayResult, run_b: &ReplayResult) -> bool {
    run_a.final_state_hash == run_b.final_state_hash
}
