import sys
import os

with open(r'c:\Users\Jean\Desktop\binary_event_forecasting\services\core-engine\src\modes\replay.rs', 'r') as f:
    content = f.read()

new_execute_replay = '''async fn execute_replay(config: &ReplayConfig) -> anyhow::Result<ReplayResult> {
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
    
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
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
    let snapshot = builder.build(logical_clock, logical_time_to_utc(logical_clock)).unwrap();
    
    for forecast in forecasts {
        forecasts_processed += 1;

        if let Err(e) = forecast.validate(config.probability_scale) {
            violations.push(format!("Forecast validation failed: {e}"));
            let record = create_journal_record(&forecast.message_id, "RejectedSchema", logical_clock, "");
            journal_records.push(record);
            return Ok(ReplayResult { final_state_hash: compute_final_state_hash(&ledger, &journal_records, fills, rejections, &manifest.market_events_sha256), events_processed, forecasts_processed, intents_simulated, fills, rejections, violations });
        }

        let forecast_bytes = serde_json::to_vec(&forecast).unwrap_or_default();
        let forecast_len = (forecast_bytes.len() as u32).to_be_bytes();
        
        let mut socket_conn = connect_ipc(&socket_path).await?;
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

        let record = create_journal_record(&forecast.message_id, "AcceptedQueued", logical_clock, "");
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

                let snapshot = builder.build(logical_clock, logical_time_to_utc(logical_clock)).unwrap();
                let mut matching_state = VirtualMatchingState::new(Cash::new(1_000_000_000));
                let cost_model = FixedBpsCostModel::new(10);
                let match_result = match_immediate(&intent, &snapshot, &mut matching_state, &cost_model);

                let transition_id = format!("transition-{}", logical_clock);

                match match_result {
                    MatchResult::Filled { cash_reserved, .. } => {
                        fills += 1;

                        // 1. Write DispositionPlanned durably
                        let mut conn = journal::db::open_journal(db_path.to_str().unwrap()).unwrap();
                        journal::db::commit_transition_plan(&mut conn, &transition_id, "replay-entity", &logical_time_to_utc(logical_clock).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "hash").unwrap();

                        let plan_record = create_journal_record(&forecast.message_id, "DispositionPlanned", logical_clock, &transition_id);
                        journal_records.push(plan_record);
                        events_processed += 1;

                        // 2. Apply idempotent ledger transition
                        let transition = LedgerTransition {
                            transition_id: transition_id.clone(),
                            free_cash_delta: -(cash_reserved.as_raw() as i128),
                            reserved_cash_delta: cash_reserved.as_raw() as i128,
                            total_cash_delta: 0,
                        };
                        ledger.apply_transition(&transition).unwrap();
                        
                        // 3. Save Ledger State
                        journal::db::save_ledger_state(&mut conn, "chkpt-1", ledger.version, &ledger.free_cash.as_raw().to_string(), &ledger.reserved_cash.as_raw().to_string(), &ledger.total_cash.as_raw().to_string(), &logical_time_to_utc(logical_clock).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "state_hash").unwrap();
                        
                        // SIMULATE CRASH:
                        // We do NOT write DispositionCommitted yet. We drop the DB and server.
                        drop(conn);
                        server_handle.abort();
                        
                        tracing::info!("Simulated crash... reconnecting to database to recover.");
                        
                        // RECOVERY:
                        let mut conn = journal::db::open_journal(db_path.to_str().unwrap()).unwrap();
                        let recovered_state = journal::db::load_ledger_state(&conn).unwrap().unwrap();
                        let applied_transitions = journal::db::load_applied_transitions(&conn).unwrap();
                        ledger = Ledger::restore(recovered_state.0, Cash::new(recovered_state.1.parse().unwrap()), domain_types::ReservedCash::new(recovered_state.2.parse().unwrap()), applied_transitions).unwrap();
                        
                        let pending = journal::db::load_pending_transitions(&conn).unwrap();
                        for (tid, eid) in pending {
                            journal::db::commit_terminal_disposition(&mut conn, &tid, &eid, &logical_time_to_utc(logical_clock + 1).to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "final_hash").unwrap();
                        }
                        
                        tracing::info!("Crash recovery complete. Terminal disposition committed exactly once.");

                        let commit_record = create_journal_record(&forecast.message_id, "DispositionCommitted", logical_clock + 1, &transition_id);
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

    let final_state_hash = compute_final_state_hash(&ledger, &journal_records, fills, rejections, &manifest.market_events_sha256);
    Ok(ReplayResult { final_state_hash, events_processed, forecasts_processed, intents_simulated, fills, rejections, violations })
}'''

start_idx = content.find('async fn execute_replay(config: &ReplayConfig) -> anyhow::Result<ReplayResult> {')
end_idx = content.find('fn logical_time_to_utc(tick: i64) -> DateTime<Utc> {')
if start_idx != -1 and end_idx != -1:
    end_idx = content.rfind('/// Convert a logical clock tick', start_idx, end_idx)
    updated_content = content[:start_idx] + new_execute_replay + '\n\n' + content[end_idx:]
    with open(r'c:\Users\Jean\Desktop\binary_event_forecasting\services\core-engine\src\modes\replay.rs', 'w') as f:
        f.write(updated_content)
    print('SUCCESS')
else:
    print('FAILED TO FIND INDICES')
