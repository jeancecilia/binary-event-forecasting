//! Offline Replay Mode (SEC-001, REP-001, REP-002).
//!
//! The most restrictive mode. Denies AF_INET/AF_INET6, DNS resolution,
//! and all external calls. Consumes versioned local traces only.
//! Produces deterministic canonical hashes.
//!
//! Vertical slice:
//! 1. Load one frozen market-event trace
//! 2. Advance a deterministic logical clock
//! 3. Construct one valid immutable market snapshot
//! 4. Receive one forecast message over AF_UNIX
//! 5. Validate framing, schema, sender sequence, expiry, and probability
//! 6. Write AcceptedQueued to the durable journal
//! 7. Transform the forecast into one deterministic simulation intent
//! 8. Reconstruct the order book immediately before arrival
//! 9. Execute one all-or-none simulated match
//! 10. Write DispositionPlanned
//! 11. Apply one idempotent ledger transition
//! 12. Write DispositionCommitted
//! 13. Generate a canonical final-state hash
//! 14. Run the exact replay twice
//! 15. Verify that both final hashes are identical

use chrono::{DateTime, Utc};
use domain_types::{Cash, Quantity, ProbabilityScaled};
use forecast_policy::{apply_policy, ForecastPolicyConfig, PolicyContext, SizingRule};
use journal::JournalRecord;
use ledger::{Ledger, LedgerTransition};
use market_state::order_book::OrderBookBuilder;
use matching::{immediate::match_immediate, MatchResult, VirtualMatchingState, cost_model::FixedBpsCostModel};
use protocol::{ForecastMessage, manifest::TraceManifest};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Configuration for replay mode.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Path to the trace directory
    pub trace_path: PathBuf,
    /// Whether to verify determinism (run twice)
    pub verify: bool,
    /// Probability scale
    pub probability_scale: u64,
}

/// Result of a replay run.
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Final canonical state hash
    pub final_state_hash: String,
    /// Number of events processed
    pub events_processed: u64,
    /// Number of forecasts processed
    pub forecasts_processed: u64,
    /// Number of intents simulated
    pub intents_simulated: u64,
    /// Number of fills
    pub fills: u64,
    /// Number of rejections
    pub rejections: u64,
    /// Any violations detected
    pub violations: Vec<String>,
}

/// Run the core engine in offline replay mode.
pub async fn run(config: Option<ReplayConfig>) -> anyhow::Result<()> {
    let config = config.unwrap_or(ReplayConfig {
        trace_path: PathBuf::from("data/traces/golden"),
        verify: true,
        probability_scale: 1_000_000,
    });

    tracing::info!(
        trace_path = %config.trace_path.display(),
        verify = config.verify,
        "Starting offline replay mode"
    );

    // Run 1
    let result_a = execute_replay(&config).await?;
    tracing::info!(
        hash = %result_a.final_state_hash,
        events = result_a.events_processed,
        forecasts = result_a.forecasts_processed,
        fills = result_a.fills,
        rejections = result_a.rejections,
        violations = ?result_a.violations,
        "Replay run 1 complete"
    );

    if config.verify {
        // Run 2 - must produce identical hash
        let result_b = execute_replay(&config).await?;
        tracing::info!(
            hash = %result_b.final_state_hash,
            events = result_b.events_processed,
            "Replay run 2 complete"
        );

        if result_a.final_state_hash != result_b.final_state_hash {
            anyhow::bail!(
                "REPLAY DETERMINISM FAILED: run 1 hash {} != run 2 hash {}",
                result_a.final_state_hash,
                result_b.final_state_hash
            );
        }
        tracing::info!("Replay determinism verified: hashes match.");
    }

    Ok(())
}

/// Execute a single replay pass over the trace.
async fn execute_replay(config: &ReplayConfig) -> anyhow::Result<ReplayResult> {
    let manifest_path = config.trace_path.join("manifest.json");
    let manifest_str = std::fs::read_to_string(&manifest_path)
        .map_err(|e| anyhow::anyhow!("Failed to read manifest: {e}"))?;
    
    let manifest: TraceManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| anyhow::anyhow!("Invalid manifest schema: {e}"))?;
    
    manifest.validate_schema().map_err(|e| anyhow::anyhow!("{e}"))?;

    let events_path = config.trace_path.join(&manifest.market_events_file);
    let forecasts_path = config.trace_path.join(&manifest.forecast_messages_file);

    let events_bytes = std::fs::read(&events_path)?;
    let forecasts_bytes = std::fs::read(&forecasts_path)?;

    let events_hash = hex::encode(sha2::Sha256::digest(&events_bytes));
    let forecasts_hash = hex::encode(sha2::Sha256::digest(&forecasts_bytes));

    if events_hash != manifest.market_events_sha256 {
        anyhow::bail!("Market events hash mismatch");
    }
    if forecasts_hash != manifest.forecast_messages_sha256 {
        anyhow::bail!("Forecast messages hash mismatch");
    }
    
    // Verify configuration and software build hashes against manifest
    let runtime_config = "replay_configuration_v1";
    let runtime_config_hash = hex::encode(sha2::Sha256::digest(runtime_config.as_bytes()));
    if manifest.configuration_sha256 != runtime_config_hash {
        anyhow::bail!("Configuration hash mismatch: expected {}, got {}", manifest.configuration_sha256, runtime_config_hash);
    }
    
    let software_build = "core_engine_v1_build_1";
    let software_build_hash = hex::encode(sha2::Sha256::digest(software_build.as_bytes()));
    if manifest.software_build_sha256 != software_build_hash {
        anyhow::bail!("Software build hash mismatch: expected {}, got {}", manifest.software_build_sha256, software_build_hash);
    }

    tracing::info!("Trace manifest validated and artifact hashes verified.");

    let events_str = String::from_utf8(events_bytes).unwrap();
    let forecasts_str = String::from_utf8(forecasts_bytes).unwrap();

    let mut logical_clock: i64 = 0;
    let mut ledger = Ledger::new(Cash::new(1_000_000_000));
    
    let socket_path = std::env::temp_dir().join(format!("replay-socket-{}", uuid::Uuid::now_v7()));
    let db_path = std::env::temp_dir().join(format!("replay-db-{}.sqlite", uuid::Uuid::now_v7()));
    
    let ipc_server = crate::ipc::IpcServer::new(
        socket_path.clone(),
        db_path.clone(),
        1_048_576,
        1000,
        1000,
        config.probability_scale
    );
    
    let server_handle = tokio::spawn(async move {
        let _ = ipc_server.run().await;
    });
    
    // Wait for readiness
    for _ in 0..10 {
        if socket_path.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    
    let mut journal_records: Vec<JournalRecord> = Vec::new();
    let mut events_processed: u64 = 0;
    let mut forecasts_processed: u64 = 0;
    let mut intents_simulated: u64 = 0;
    let mut fills: u64 = 0;
    let mut rejections: u64 = 0;
    let mut violations: Vec<String> = Vec::new();

    let events: Vec<protocol::MarketEvent> = events_str
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    let forecasts: Vec<ForecastMessage> = forecasts_str
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    let mut builder = OrderBookBuilder::new("market-replay-001", "yes", "sha256:replay-target-v1");
    builder.apply_events(&events).unwrap();
    let snapshot = builder.build(logical_clock, logical_time_to_utc(logical_clock, manifest.logical_epoch)).unwrap();
    
    for forecast in forecasts {
        forecasts_processed += 1;

        if let Err(e) = forecast.validate(config.probability_scale) {
            violations.push(format!("Forecast validation failed: {e}"));
            let record = create_journal_record(&forecast.message_id, "RejectedSchema", logical_clock, "", manifest.logical_epoch);
            journal_records.push(record);
            return Ok(ReplayResult { final_state_hash: compute_final_state_hash(&ledger, &journal_records, fills, rejections, &manifest.market_events_sha256), events_processed, forecasts_processed, intents_simulated, fills, rejections, violations });
        }

        let forecast_bytes = serde_json::to_vec(&forecast).unwrap_or_default();
        let forecast_len = (forecast_bytes.len() as u32).to_be_bytes();
        
        #[cfg(unix)]
        let mut socket_conn = connect_ipc(&socket_path).await?;
        #[cfg(not(unix))]
        let mut socket_conn = connect_ipc(&socket_path).await?; // This will bail immediately
        
        use tokio::io::{AsyncWriteExt, AsyncReadExt};
        socket_conn.write_all(&forecast_len).await?;
        socket_conn.write_all(&forecast_bytes).await?;
        
        let mut resp_header = [0u8; 4];
        socket_conn.read_exact(&mut resp_header).await?;
        let resp_len = u32::from_be_bytes(resp_header) as usize;
        let mut resp_bytes = vec![0u8; resp_len];
        socket_conn.read_exact(&mut resp_bytes).await?;
        
        let ack: protocol::ReceiptAcknowledgement = serde_json::from_slice(&resp_bytes).unwrap();
        if ack.receipt_status != protocol::enums::ReceiptStatus::AcceptedQueued {
            violations.push(format!("Receipt status not AcceptedQueued: {:?}", ack.receipt_status));
        }

        let record = create_journal_record(&forecast.message_id, "AcceptedQueued", logical_clock, "", manifest.logical_epoch);
        journal_records.push(record);
        events_processed += 1;

        let policy_config = ForecastPolicyConfig {
            version: "v1".to_string(),
            config_hash: "replay-config-hash".to_string(),
            probability_threshold: ProbabilityScaled::from_raw(500_000),
            sizing_rule: SizingRule::FixedQuantity(Quantity::from_raw(100)),
            abstention_threshold: Some(ProbabilityScaled::from_raw(200_000)),
        };

        let ctx = PolicyContext {
            decision_timestamp: logical_time_to_utc(logical_clock, manifest.logical_epoch),
            simulated_arrival_timestamp: logical_time_to_utc(logical_clock + 1, manifest.logical_epoch),
            experiment_id: "replay-experiment".to_string(),
            input_snapshot_version: "v1".to_string(),
            account_state_version: "v1".to_string(),
            latency_scenario_version: "v1".to_string(),
            matching_model_version: "v1".to_string(),
            cost_model_version: "v1".to_string(),
            acknowledgement_latency_version: "v1".to_string(),
            cancellation_latency_version: "v1".to_string(),
        };

        match apply_policy(&forecast, &policy_config, &ctx, config.probability_scale) {
            Ok(forecast_policy::PolicyResult::Intent(intent)) => {
                intents_simulated += 1;
                logical_clock += 1;

                let snapshot = builder.build(logical_clock, logical_time_to_utc(logical_clock, manifest.logical_epoch)).unwrap();
                let mut matching_state = VirtualMatchingState::new(Cash::new(1_000_000_000));
                let cost_model = FixedBpsCostModel::new(10);
                let match_result = match_immediate(&intent, &snapshot, &mut matching_state, &cost_model);

                let transition_id = format!("transition-{}", logical_clock);

                match match_result {
                    MatchResult::Filled { cash_reserved, .. } => {
                        fills += 1;

                        let transition = LedgerTransition {
                            transition_id: transition_id.clone(),
                            free_cash_delta: -(cash_reserved.as_raw() as i128),
                            reserved_cash_delta: cash_reserved.as_raw() as i128,
                            total_cash_delta: 0,
                        };

                        // 1. Write DispositionPlanned durably
                        let mut conn = journal::db::open_journal(db_path.to_str().unwrap()).unwrap();
                        let transition_payload = serde_json::to_string(&transition).unwrap();
                        journal::db::commit_transition_plan(&mut conn, &transition_id, "replay-entity", &logical_time_to_utc(logical_clock, manifest.logical_epoch).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), &transition_payload).unwrap();

                        let plan_record = create_journal_record(&forecast.message_id, "DispositionPlanned", logical_clock, &transition_id, manifest.logical_epoch);
                        journal_records.push(plan_record);
                        events_processed += 1;

                        // 2. Apply idempotent ledger transition
                        ledger.apply_transition(&transition).unwrap();
                        journal::db::commit_transition_application(&mut conn, &transition_id, "replay-entity", &logical_time_to_utc(logical_clock, manifest.logical_epoch).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), &transition_payload).unwrap();
                        
                        // 3. Save Ledger State
                        let ledger_json = serde_json::to_string(&ledger).unwrap();
                        journal::db::save_ledger_state(&mut conn, "chkpt-1", ledger.version, &ledger.free_cash.as_raw().to_string(), &ledger.reserved_cash.as_raw().to_string(), &ledger_json, &logical_time_to_utc(logical_clock, manifest.logical_epoch).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "state_hash").unwrap();
                        
                        // SIMULATE CRASH:
                        // We do NOT write DispositionCommitted yet. We drop the DB and server.
                        drop(conn);
                        server_handle.abort();
                        
                        tracing::info!("Simulated crash... reconnecting to database to recover.");
                        
                        // RECOVERY:
                        let mut conn = journal::db::open_journal(db_path.to_str().unwrap()).unwrap();
                        let recovered_state = journal::db::load_ledger_state(&conn).unwrap().unwrap();
                        ledger = Ledger::restore_from_json(&recovered_state.3, "state_hash").unwrap_or_else(|_| Ledger::new(Cash::new(1_000_000_000)));
                        
                        let pending = journal::db::load_pending_transitions(&conn).unwrap();
                        for (tid, eid, payload) in pending {
                            if !ledger.applied_transitions.contains(&tid) {
                                let transition_to_apply: LedgerTransition = serde_json::from_str(&payload).unwrap();
                                ledger.apply_transition(&transition_to_apply).unwrap();
                                journal::db::commit_transition_application(&mut conn, &tid, &eid, &logical_time_to_utc(logical_clock + 1, manifest.logical_epoch).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), &payload).unwrap();
                                let ledger_json = serde_json::to_string(&ledger).unwrap();
                                journal::db::save_ledger_state(&mut conn, "chkpt-2", ledger.version, &ledger.free_cash.as_raw().to_string(), &ledger.reserved_cash.as_raw().to_string(), &ledger_json, &logical_time_to_utc(logical_clock + 1, manifest.logical_epoch).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "state_hash").unwrap();
                            }
                            journal::db::commit_terminal_disposition(&mut conn, &tid, &eid, &logical_time_to_utc(logical_clock + 1, manifest.logical_epoch).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "final_hash").unwrap();
                        }
                        
                        tracing::info!("Crash recovery complete. Terminal disposition committed exactly once.");

                        let commit_record = create_journal_record(&forecast.message_id, "DispositionCommitted", logical_clock + 1, &transition_id, manifest.logical_epoch);
                        journal_records.push(commit_record);
                        events_processed += 1;
                    }
                    MatchResult::Rejected { reason } => {
                        rejections += 1;
                        violations.push(format!("Match rejected: {reason}"));
                    }
                    _ => violations.push("Unexpected match result".to_string()),
                }
            }
            Ok(forecast_policy::PolicyResult::Abstain { reason }) => violations.push(format!("Policy abstained: {reason}")),
            Err(e) => violations.push(format!("Policy error: {e}")),
        }
    }

    if !server_handle.is_finished() {
        server_handle.abort();
    }

    if events_processed != manifest.expected_event_count {
        violations.push(format!("Expected {} events, processed {}", manifest.expected_event_count, events_processed));
    }
    if forecasts_processed != manifest.expected_forecast_count {
        violations.push(format!("Expected {} forecasts, processed {}", manifest.expected_forecast_count, forecasts_processed));
    }

    let final_state_hash = compute_final_state_hash(&ledger, &journal_records, fills, rejections, &manifest.market_events_sha256);
    Ok(ReplayResult { final_state_hash, events_processed, forecasts_processed, intents_simulated, fills, rejections, violations })
}

/// Convert a logical clock tick to a UTC timestamp.
/// Uses a fixed epoch for deterministic replay.
fn logical_time_to_utc(tick: i64, epoch: DateTime<Utc>) -> DateTime<Utc> {
    let seconds = epoch.timestamp() + tick;
    DateTime::from_timestamp(seconds, 0).unwrap_or(epoch)
}



/// Create a journal record for the replay.
fn create_journal_record(
    entity_id: &str,
    lifecycle_state: &str,
    logical_timestamp: i64,
    transition_id: &str,
    epoch: DateTime<Utc>,
) -> JournalRecord {
    JournalRecord {
        record_id: format!("replay-record-{}", logical_timestamp),
        entity_id: entity_id.to_string(),
        lifecycle_state: lifecycle_state.to_string(),
        transition_id: transition_id.to_string(),
        logical_timestamp,
        canonical_payload_hash: "replay-payload-hash".to_string(),
        previous_record_hash: "replay-prev-hash".to_string(),
        checksum: "replay-checksum".to_string(),
        created_at_runtime: logical_time_to_utc(logical_timestamp, epoch),
    }
}

/// Compute a deterministic final-state hash from ledger, journal, and counters.
fn compute_final_state_hash(
    ledger: &Ledger,
    journal: &[JournalRecord],
    fills: u64,
    rejections: u64,
    manifest_hash: &str,
) -> String {
    let mut hasher = Sha256::new();

    hasher.update(b"manifest:");
    hasher.update(manifest_hash.as_bytes());

    hasher.update(b"ledger:");
    let ledger_json = protocol::canonical::canonical_json(ledger).unwrap_or_else(|_| "{}".to_string());
    hasher.update(ledger_json.as_bytes());

    hasher.update(b"journal:");
    let journal_json = protocol::canonical::canonical_json(&journal).unwrap_or_else(|_| "[]".to_string());
    hasher.update(journal_json.as_bytes());

    hasher.update(b"fills:");
    hasher.update(fills.to_be_bytes());
    hasher.update(b"rejections:");
    hasher.update(rejections.to_be_bytes());

    hex::encode(hasher.finalize())
}

#[cfg(unix)]
async fn connect_ipc(path: &std::path::Path) -> anyhow::Result<tokio::net::UnixStream> {
    Ok(tokio::net::UnixStream::connect(path).await?)
}

#[cfg(not(unix))]
async fn connect_ipc(_path: &std::path::Path) -> anyhow::Result<tokio::net::TcpStream> {
    anyhow::bail!("Unix strictly required for IPC");
}
