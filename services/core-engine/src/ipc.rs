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
    db_path: std::path::PathBuf,
    max_frame_bytes: usize,
    read_timeout_ms: u64,
    idle_timeout_ms: u64,
    probability_scale: u64,
}

impl IpcServer {
    pub fn new(
        socket_path: std::path::PathBuf,
        db_path: std::path::PathBuf,
        max_frame_bytes: usize,
        read_timeout_ms: u64,
        idle_timeout_ms: u64,
        probability_scale: u64,
    ) -> Self {
        Self {
            socket_path,
            db_path,
            max_frame_bytes,
            read_timeout_ms,
            idle_timeout_ms,
            probability_scale,
        }
    }

    #[cfg(unix)]
    pub async fn run(&self) -> anyhow::Result<()> {
        use tokio::net::UnixListener;
        use std::os::unix::fs::FileTypeExt;
        
        tracing::info!(
            "IPC server starting on {}",
            self.socket_path.display()
        );

        if self.socket_path.exists() {
            let meta = std::fs::symlink_metadata(&self.socket_path)?;
            if !meta.file_type().is_socket() {
                anyhow::bail!("Configured IPC path exists and is not a socket.");
            }
            // In a real system, we might also verify ownership here:
            // use std::os::unix::fs::MetadataExt;
            // if meta.uid() != unsafe { libc::geteuid() } { anyhow::bail!("Socket owned by different user"); }
            
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        
        // Try to set permissions on UNIX
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = std::fs::metadata(&self.socket_path).map(|m| m.permissions()) {
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&self.socket_path, perms);
        }

        loop {
            let (socket, _) = listener.accept().await?;
            Self::handle_connection(
                socket,
                self.idle_timeout_ms,
                self.max_frame_bytes,
                self.read_timeout_ms,
                self.probability_scale,
                self.db_path.clone()
            ).await;
        }
    }

    #[cfg(not(unix))]
    pub async fn run(&self) -> anyhow::Result<()> {
        anyhow::bail!("IPC server requires AF_UNIX. Windows is strictly not supported for this core component.");
    }

    #[cfg(unix)]
    async fn handle_connection(
        mut socket: tokio::net::UnixStream,
        idle_timeout_ms: u64,
        max_frame_bytes: usize,
        read_timeout_ms: u64,
        probability_scale: u64,
        db_path: std::path::PathBuf,
    ) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        // Peer credential authentication
        match socket.peer_cred() {
            Ok(cred) => {
                let current_uid = unsafe { libc::geteuid() };
                if cred.uid() != current_uid {
                    tracing::error!("Unauthorized peer UID: {}. Expected: {}", cred.uid(), current_uid);
                    return;
                }
            }
            Err(e) => {
                tracing::error!("Failed to get peer credentials: {}", e);
                return;
            }
        }

        let mut header = [0u8; FRAME_HEADER_SIZE];
        if let Ok(n) = tokio::time::timeout(
            std::time::Duration::from_millis(idle_timeout_ms),
            socket.read_exact(&mut header)
        ).await {
            match n {
                Ok(_) => {
                    let len = u32::from_be_bytes(header) as usize;
                    if len == 0 || len > max_frame_bytes {
                        tracing::error!("Invalid frame length: {} bytes", len);
                        return;
                    }
                    
                    let mut payload = vec![0u8; len];
                    if let Ok(Ok(_)) = tokio::time::timeout(
                        std::time::Duration::from_millis(read_timeout_ms),
                        socket.read_exact(&mut payload)
                    ).await {
                        // Deserialize & Validate
                        if let Ok(msg) = serde_json::from_slice::<protocol::ForecastMessage>(&payload) {
                            // Validate Message
                            let mut status = protocol::enums::ReceiptStatus::AcceptedQueued;
                            if msg.validate(probability_scale).is_err() {
                                status = protocol::enums::ReceiptStatus::RejectedBounds;
                            }

                            // Open DB and store receipt durably
                            if status == protocol::enums::ReceiptStatus::AcceptedQueued {
                                if let Ok(mut conn) = journal::db::open_journal(db_path.to_str().unwrap()) {
                                    let timestamp = msg.forecast_emitted_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                                    let payload_hash = protocol::canonical::canonical_hash(&msg).unwrap_or_else(|_| "0000".to_string());
                                    if let Ok(db_status) = journal::db::process_forecast_receipt(
                                        &mut conn,
                                        &msg.message_id,
                                        &msg.sender_instance_id,
                                        msg.sender_sequence,
                                        &payload_hash,
                                        &timestamp
                                    ) {
                                        status = db_status;
                                    } else {
                                        status = protocol::enums::ReceiptStatus::CoreDegraded;
                                    }
                                } else {
                                    status = protocol::enums::ReceiptStatus::CoreDegraded;
                                }
                            }
                            
                            let receipt_id = format!("receipt-{}", msg.message_id);
                            let receipt = protocol::ReceiptAcknowledgement {
                                schema_version: 1,
                                message_id: msg.message_id.clone(),
                                receipt_status: status,
                                timestamp: msg.forecast_emitted_at, // deterministic based on message
                                receipt_id,
                                detail: None,
                            };
                            
                            if let Ok(resp_bytes) = serde_json::to_vec(&receipt) {
                                let resp_len = (resp_bytes.len() as u32).to_be_bytes();
                                let _ = socket.write_all(&resp_len).await;
                                let _ = socket.write_all(&resp_bytes).await;
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("Failed to read frame header: {e}"),
            }
        } else {
            tracing::debug!("IPC idle timeout");
        }
    }
}
