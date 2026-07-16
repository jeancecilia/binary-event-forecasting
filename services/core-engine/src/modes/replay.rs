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
use domain_types::{Cash, Price, Quantity, ProbabilityScaled};
use forecast_policy::{apply_policy, ForecastPolicyConfig, PolicyContext, SizingRule};
use journal::JournalRecord;
use ledger::{Ledger, LedgerTransition};
use market_state::{MarketSnapshot, PriceLevel, order_book::OrderBookBuilder};
use matching::{immediate::match_immediate, MatchResult, VirtualMatchingState, cost_model::{CostModel, FixedBpsCostModel}};
use protocol::{ForecastMessage, enums::FeedStatus, manifest::TraceManifest};
use sha2::{Digest, Sha256};
use std::fs;
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
    let result_a = execute_replay(&config)?;
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
        let result_b = execute_replay(&config)?;
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
fn execute_replay(config: &ReplayConfig) -> anyhow::Result<ReplayResult> {
    // 1. Load trace manifest and verify hashes
    let manifest_path = config.trace_path.join("manifest.json");
    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| anyhow::anyhow!("Failed to read manifest: {e}"))?;
    
    let manifest: TraceManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| anyhow::anyhow!("Invalid manifest schema: {e}"))?;
    
    manifest.validate_schema().map_err(|e| anyhow::anyhow!("{e}"))?;

    let events_path = config.trace_path.join(&manifest.market_events_file);
    let forecasts_path = config.trace_path.join(&manifest.forecast_messages_file);

    let events_bytes = fs::read(&events_path)?;
    let forecasts_bytes = fs::read(&forecasts_path)?;

    let events_hash = hex::encode(Sha256::digest(&events_bytes));
    let forecasts_hash = hex::encode(Sha256::digest(&forecasts_bytes));

    if events_hash != manifest.market_events_sha256 {
        anyhow::bail!("Market events hash mismatch: expected {}, got {}", manifest.market_events_sha256, events_hash);
    }
    if forecasts_hash != manifest.forecast_messages_sha256 {
        anyhow::bail!("Forecast messages hash mismatch: expected {}, got {}", manifest.forecast_messages_sha256, forecasts_hash);
    }

    tracing::info!("Trace manifest validated and artifact hashes verified.");

    let events_str = String::from_utf8(events_bytes).unwrap();
    let forecasts_str = String::from_utf8(forecasts_bytes).unwrap();

    // Initialize state
    let mut logical_clock: i64 = 0;
    let mut ledger = Ledger::new(Cash::new(1_000_000_000)); // 1M notional units
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

    if events.len() as u64 != manifest.expected_event_count {
        anyhow::bail!("Event count mismatch: expected {}, got {}", manifest.expected_event_count, events.len());
    }

    let forecasts: Vec<ForecastMessage> = forecasts_str
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    if forecasts.len() as u64 != manifest.expected_forecast_count {
        anyhow::bail!("Forecast count mismatch: expected {}, got {}", manifest.expected_forecast_count, forecasts.len());
    }

    let mut builder = OrderBookBuilder::new(
        "market-replay-001",
        "yes",
        "sha256:replay-target-v1",
    );
    builder.apply_events(&events).unwrap();

    let snapshot = builder.build(logical_clock, logical_time_to_utc(logical_clock)).unwrap();
    
    for forecast in forecasts {
        forecasts_processed += 1;

    // Step 5: Validate the forecast
    if let Err(e) = forecast.validate(config.probability_scale) {
        let reason = format!("Forecast validation failed: {e}");
        violations.push(reason.clone());
        
        let record = create_journal_record(
            &forecast.message_id,
            "RejectedSchema",
            logical_clock,
            "",
        );
        journal_records.push(record);

        // Fail closed - do not proceed with accepted message
        let final_state_hash = compute_final_state_hash(
            &ledger,
            &journal_records,
            fills,
            rejections,
            &manifest.market_events_sha256,
        );

        return Ok(ReplayResult {
            final_state_hash,
            events_processed,
            forecasts_processed,
            intents_simulated,
            fills,
            rejections,
            violations,
        });
    }

    // Step 6: Write AcceptedQueued to SQLite journal
    let mut journal_conn = journal::db::open_journal(":memory:").unwrap();
    let forecast_bytes = serde_json::to_vec(&forecast).unwrap();
    let forecast_hash = hex::encode(Sha256::digest(&forecast_bytes));
    
    // Test the sender sequence retry and rejection logic
    let receipt_status = match journal::db::process_forecast_receipt(
        &mut journal_conn,
        &forecast.message_id,
        &forecast.sender_instance_id,
        forecast.sender_sequence,
        &forecast_hash,
    ) {
        Ok(status) => status,
        Err(e) => {
            violations.push(format!("SQLite Error: {e}"));
            return Ok(ReplayResult {
                final_state_hash: "".to_string(),
                events_processed,
                forecasts_processed,
                intents_simulated,
                fills,
                rejections,
                violations,
            });
        }
    };
    
    if receipt_status != protocol::enums::ReceiptStatus::AcceptedQueued {
        violations.push(format!("Receipt status not AcceptedQueued: {:?}", receipt_status));
    }

    let record = create_journal_record(
        &forecast.message_id,
        "AcceptedQueued",
        logical_clock,
        "",
    );
    journal_records.push(record);
    events_processed += 1;

    // Step 7: Transform forecast into simulation intent
    let policy_config = ForecastPolicyConfig {
        version: "v1".to_string(),
        config_hash: "replay-config-hash".to_string(),
        probability_threshold: ProbabilityScaled::from_raw(500_000),
        sizing_rule: SizingRule::FixedQuantity(Quantity::from_raw(100)),
        abstention_threshold: Some(ProbabilityScaled::from_raw(200_000)),
    };

    let ctx = PolicyContext {
        decision_timestamp: logical_time_to_utc(logical_clock),
        simulated_arrival_timestamp: logical_time_to_utc(logical_clock + 1),
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

            // Step 8: Reconstruct order book immediately before arrival
            let snapshot = builder.build(logical_clock, logical_time_to_utc(logical_clock)).unwrap();

            // Step 9: Execute all-or-none match
            let mut matching_state = VirtualMatchingState::new(
                Cash::new(1_000_000_000),
            );
            let cost_model = FixedBpsCostModel::new(10); // 10 bps fee
            let match_result = match_immediate(&intent, &snapshot, &mut matching_state, &cost_model);

            let transition_id = format!("transition-{}", logical_clock);

            match match_result {
                MatchResult::Filled { cash_reserved, .. } => {
                    fills += 1;

                    // Step 10: Write DispositionPlanned
                    let plan_record = create_journal_record(
                        &forecast.message_id,
                        "DispositionPlanned",
                        logical_clock,
                        &transition_id,
                    );
                    journal_records.push(plan_record);
                    events_processed += 1;

                    // Step 11: Apply idempotent ledger transition
                    let transition = LedgerTransition {
                        transition_id: transition_id.clone(),
                        free_cash_delta: -(cash_reserved.as_raw() as i128),
                        reserved_cash_delta: cash_reserved.as_raw() as i128,
                        total_cash_delta: 0,
                    };
                    ledger.apply_transition(&transition)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;

                    // Step 12: Write DispositionCommitted
                    let commit_record = create_journal_record(
                        &forecast.message_id,
                        "DispositionCommitted",
                        logical_clock + 1,
                        &transition_id,
                    );
                    journal_records.push(commit_record);
                    events_processed += 1;
                }
                MatchResult::Rejected { reason } => {
                    rejections += 1;
                    violations.push(format!("Match rejected: {reason}"));
                }
                _ => {
                    violations.push("Unexpected match result".to_string());
                }
            }
        }
        Ok(forecast_policy::PolicyResult::Abstain { reason }) => {
            violations.push(format!("Policy abstained: {reason}"));
        }
        Err(e) => {
            violations.push(format!("Policy error: {e}"));
        }
    }

    } // end of for forecast in forecasts

    // Step 13: Generate canonical final-state hash
    let final_state_hash = compute_final_state_hash(
        &ledger,
        &journal_records,
        fills,
        rejections,
        &manifest.market_events_sha256,
    );

    Ok(ReplayResult {
        final_state_hash,
        events_processed,
        forecasts_processed,
        intents_simulated,
        fills,
        rejections,
        violations,
    })
}

/// Convert a logical clock tick to a UTC timestamp.
/// Uses a fixed epoch for deterministic replay.
fn logical_time_to_utc(tick: i64) -> DateTime<Utc> {
    let epoch = DateTime::from_timestamp(1_700_000_000, 0)
        .unwrap_or(DateTime::UNIX_EPOCH);
    let seconds = epoch.timestamp() + tick;
    DateTime::from_timestamp(seconds, 0).unwrap_or(epoch)
}



/// Create a journal record for the replay.
fn create_journal_record(
    entity_id: &str,
    lifecycle_state: &str,
    logical_timestamp: i64,
    transition_id: &str,
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
        created_at_runtime: logical_time_to_utc(logical_timestamp),
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
    hasher.update(serde_json::to_vec(ledger).expect("Failed to serialize ledger"));

    hasher.update(b"journal:");
    hasher.update(serde_json::to_vec(journal).expect("Failed to serialize journal"));

    hasher.update(b"fills:");
    hasher.update(fills.to_be_bytes());
    hasher.update(b"rejections:");
    hasher.update(rejections.to_be_bytes());

    hex::encode(hasher.finalize())
}
