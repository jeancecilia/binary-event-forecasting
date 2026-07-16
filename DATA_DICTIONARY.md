# Binary Event Forecasting — Data Dictionary

## Cross-Process Message Fields

### forecast_message (IPC-003)

| Field | Type | Required | Description |
|---|---|---|---|
| `schema_version` | uint32 | Yes | Schema version number |
| `message_id` | string (UUIDv7) | Yes | Unique message identifier |
| `sender_instance_id` | string | Yes | Python worker instance ID |
| `sender_sequence` | uint64 | Yes | Monotonically increasing per-instance sequence |
| `market_id` | string | Yes | Market identifier |
| `contract_or_outcome_id` | string | Yes | Contract/outcome within market |
| `market_definition_version` | string | Yes | Version hash of market definition |
| `event_id` | string | Yes | Event identifier |
| `underlying_event_group_id` | string | Yes | Group for partition-based evaluation |
| `forecast_target` | string | Yes | Human-readable target description |
| `forecast_horizon` | string | Yes | Forecast horizon identifier |
| `source_id` | string | Yes | Source document identifier |
| `source_version` | string | Yes | Source version |
| `evidence_set_hash` | string | Yes | SHA-256 of canonical evidence set |
| `published_at` | datetime | Yes | When the source was published |
| `first_source_available_at` | datetime | Yes | When source first became available |
| `ingested_at` | datetime | Yes | When source was ingested |
| `revision_id` | string | Yes | Source revision identifier |
| `model_artifact_hash` | string | Yes | SHA-256 of model artifact |
| `model_training_cutoff` | datetime | Yes | Training data cutoff |
| `ensemble_version` | string | Yes | Ensemble configuration version |
| `component_model_versions` | object | Yes | Map of component name → version |
| `prompt_version` | string | Yes | Prompt template version |
| `retrieval_corpus_version` | string | Yes | RAG corpus version |
| `calibration_model_version` | string | Yes | Calibration model version |
| `calibration_training_cutoff` | datetime | Yes | Calibration training cutoff |
| `raw_model_probability` | uint64 | Yes | Raw model output (scaled) |
| `calibrated_probability` | uint64 | Yes | Calibrated probability (scaled) |
| `uncertainty_lower` | uint64 | Yes | Lower bound of uncertainty interval |
| `uncertainty_upper` | uint64 | Yes | Upper bound of uncertainty interval |
| `uncertainty_coverage_level` | float64 | Yes | Coverage level (e.g., 0.90) |
| `uncertainty_method` | string | Yes | Method used for uncertainty |
| `abstention_reason` | string | No | Reason if abstaining (null if not) |
| `decision_cutoff_at` | datetime | Yes | Latest time data can influence forecast |
| `forecast_created_at` | datetime | Yes | When forecast was created |
| `forecast_emitted_at` | datetime | Yes | When forecast was sent |
| `expires_at` | datetime | Yes | When forecast expires |

Invariant: `0 ≤ uncertainty_lower ≤ calibrated_probability ≤ uncertainty_upper ≤ ProbabilityScale`

### simulation_intent (FCP-002)

| Field | Type | Required | Description |
|---|---|---|---|
| `simulation_intent_id` | string | Yes | Deterministically derived from inputs |
| `experiment_id` | string | Yes | Experiment identifier |
| `source_forecast_message_id` | string | Yes | Parent forecast message |
| `forecast_policy_version` | string | Yes | Policy version used |
| `configuration_hash` | string | Yes | Hash of policy configuration |
| `market_id` | string | Yes | Target market |
| `contract_or_outcome_id` | string | Yes | Target contract/outcome |
| `forecast_target` | string | Yes | Human-readable target |
| `order_class` | enum | Yes | `ImmediateAllOrNone` or `Passive` |
| `book_side` | enum | Yes | `Bid` or `Ask` |
| `outcome_side` | enum | Yes | `Yes` or `No` |
| `quantity` | uint64 | Yes | Order quantity (scaled integer) |
| `price_limit` | uint64 | Yes | Limit price (scaled integer) |
| `time_in_force` | enum | Yes | `ImmediateOrCancel`, `GoodTillCancelled`, etc. |
| `policy_priority` | uint32 | Yes | Priority for tie-breaking |
| `decision_timestamp` | datetime | Yes | When policy decision was made |
| `simulated_arrival_timestamp` | datetime | Yes | Simulated arrival at matcher |
| `latency_scenario_version` | string | Yes | Latency model version |
| `matching_model_version` | string | Yes | Matching model version |
| `cost_model_version` | string | Yes | Cost model version |
| `acknowledgement_latency_version` | string | Yes | Ack latency model version |
| `cancellation_latency_version` | string | Yes | Cancel latency model version |
| `account_state_version` | string | Yes | Account state version |
| `input_snapshot_version` | string | Yes | Input snapshot version |
| `expires_at` | datetime | Yes | Intent expiry time |

## Database Tables

### Journal (SQLite)

#### journal_records
| Column | Type | Description |
|---|---|---|
| `record_id` | TEXT PK | Unique record identifier |
| `message_id` | TEXT | Associated message ID |
| `intent_id` | TEXT | Associated intent ID (nullable) |
| `lifecycle_state` | TEXT | Current lifecycle state |
| `transition_id` | TEXT | Transition identifier |
| `logical_timestamp` | INTEGER | Logical simulation time |
| `canonical_payload_hash` | TEXT | SHA-256 of canonical payload |
| `previous_record_hash` | TEXT | Hash of previous record |
| `checksum` | TEXT | Record-level checksum |
| `created_at_runtime` | TEXT | Wall-clock timestamp (telemetry only) |

#### message_receipts
Records the initial receipt of every forecast message.

#### message_dispositions
Records the terminal disposition of every forecast message.

#### transition_plans
Records planned ledger transitions.

#### transition_commits
Records committed ledger transitions.

#### ledger_checkpoints
Periodic snapshots of full ledger state for faster recovery.

### PostgreSQL Research Store

Key tables (all append-oriented):

- `experiments` — Experiment registration
- `experiment_manifests` — Frozen experiment manifests
- `source_documents` — Ingested source documents
- `forecasts` — Emitted forecast messages
- `simulation_intents` — Generated simulation intents
- `intent_lifecycle_events` — Lifecycle state transitions
- `ledger_entries` — Ledger mutations
- `settlements` — Event settlements
- `calibration_artifacts` — Calibration model metadata
- `evaluation_runs` — Evaluation run metadata
- `metric_results` — Computed metrics
- `holdout_access_log` — All holdout data access
- `verification_runs` — Verification test results

## Parquet Schemas

### market_events
Raw market events (trades, quote updates, etc.) partitioned by source, year, month, market_id.

### snapshots
Periodic market snapshots for replay and analysis.

### inference_features
Feature vectors used for model inference, partitioned by experiment and date.
