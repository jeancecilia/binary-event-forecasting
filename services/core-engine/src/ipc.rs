//! IPC server module.
//!
//! Implements the AF_UNIX IPC server with:
//! - Peer credential authentication (SO_PEERCRED on Linux)
//! - 4-byte big-endian length-prefixed framing
//! - Strict validation of all incoming messages
//! - Receipt acknowledgements
//! - Lifecycle disposition tracking

/// Maximum signal frame bytes (default 1 MiB).
pub const DEFAULT_MAX_SIGNAL_FRAME_BYTES: usize = 1_048_576;

/// IPC frame header: 4-byte big-endian unsigned length.
pub const FRAME_HEADER_SIZE: usize = 4;

/// IpcServer manages the AF_UNIX socket and message dispatch.
#[allow(dead_code)]
pub struct IpcServer {
    socket_path: std::path::PathBuf,
    max_frame_bytes: usize,
    read_timeout_ms: u64,
    idle_timeout_ms: u64,
}

impl IpcServer {
    /// Create a new IPC server.
    pub fn new(
        socket_path: std::path::PathBuf,
        max_frame_bytes: usize,
        read_timeout_ms: u64,
        idle_timeout_ms: u64,
    ) -> Self {
        Self {
            socket_path,
            max_frame_bytes,
            read_timeout_ms,
            idle_timeout_ms,
        }
    }

    /// Start the IPC server. Blocks until shutdown.
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!(
            "IPC server starting on {}",
            self.socket_path.display()
        );
        // TODO: Implementation in Milestone 2
        Ok(())
    }
}
