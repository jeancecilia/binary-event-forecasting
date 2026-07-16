-- PostgreSQL research store migrations: initial schema

-- experiments
CREATE TABLE IF NOT EXISTS experiments (
    experiment_id           TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    primary_hypothesis      TEXT,
    status                  TEXT NOT NULL DEFAULT 'draft',
    frozen_at               TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    schema_version          INTEGER NOT NULL DEFAULT 1,
    configuration_hash      TEXT NOT NULL,
    software_build_hash     TEXT NOT NULL
);

-- experiment_manifests
CREATE TABLE IF NOT EXISTS experiment_manifests (
    experiment_id           TEXT PRIMARY KEY REFERENCES experiments(experiment_id),
    manifest_json           JSONB NOT NULL,
    artifact_hash           TEXT NOT NULL,
    frozen_at               TIMESTAMPTZ NOT NULL,
    schema_version          INTEGER NOT NULL DEFAULT 1
);

-- source_documents
CREATE TABLE IF NOT EXISTS source_documents (
    source_id               TEXT PRIMARY KEY,
    source_version          TEXT NOT NULL,
    source_valid_at         TIMESTAMPTZ NOT NULL,
    first_observed_at       TIMESTAMPTZ NOT NULL,
    stored_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revision_id             TEXT NOT NULL,
    content_hash            TEXT NOT NULL,
    schema_version          INTEGER NOT NULL DEFAULT 1
);

-- forecasts
CREATE TABLE IF NOT EXISTS forecasts (
    message_id              TEXT PRIMARY KEY,
    experiment_id           TEXT REFERENCES experiments(experiment_id),
    market_id               TEXT NOT NULL,
    calibrated_probability  BIGINT NOT NULL,
    uncertainty_lower       BIGINT NOT NULL,
    uncertainty_upper       BIGINT NOT NULL,
    emitted_at              TIMESTAMPTZ NOT NULL,
    schema_version          INTEGER NOT NULL DEFAULT 1,
    configuration_hash      TEXT NOT NULL,
    artifact_hash           TEXT NOT NULL,
    source_valid_at         TIMESTAMPTZ NOT NULL,
    first_observed_at       TIMESTAMPTZ NOT NULL,
    stored_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revision_id             TEXT NOT NULL
);

-- simulation_intents
CREATE TABLE IF NOT EXISTS simulation_intents (
    simulation_intent_id    TEXT PRIMARY KEY,
    experiment_id           TEXT REFERENCES experiments(experiment_id),
    source_forecast_message_id TEXT REFERENCES forecasts(message_id),
    market_id               TEXT NOT NULL,
    quantity                BIGINT NOT NULL,
    price_limit             BIGINT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL,
    schema_version          INTEGER NOT NULL DEFAULT 1
);

-- intent_lifecycle_events
CREATE TABLE IF NOT EXISTS intent_lifecycle_events (
    event_id                TEXT PRIMARY KEY,
    intent_id               TEXT REFERENCES simulation_intents(simulation_intent_id),
    event_type              TEXT NOT NULL,
    timestamp               TIMESTAMPTZ NOT NULL,
    detail                  JSONB
);

-- ledger_entries
CREATE TABLE IF NOT EXISTS ledger_entries (
    entry_id                TEXT PRIMARY KEY,
    transition_id           TEXT NOT NULL,
    entry_type              TEXT NOT NULL,
    amount                  BIGINT NOT NULL,
    timestamp               TIMESTAMPTZ NOT NULL,
    schema_version          INTEGER NOT NULL DEFAULT 1
);

-- settlements
CREATE TABLE IF NOT EXISTS settlements (
    settlement_id           TEXT PRIMARY KEY,
    market_id               TEXT NOT NULL,
    resolution_status       TEXT NOT NULL,
    terminal_outcome        TEXT NOT NULL,
    settled_at              TIMESTAMPTZ NOT NULL,
    schema_version          INTEGER NOT NULL DEFAULT 1
);

-- calibration_artifacts
CREATE TABLE IF NOT EXISTS calibration_artifacts (
    artifact_id             TEXT PRIMARY KEY,
    artifact_hash           TEXT NOT NULL,
    model_version           TEXT NOT NULL,
    training_cutoff         TIMESTAMPTZ NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    schema_version          INTEGER NOT NULL DEFAULT 1
);

-- evaluation_runs
CREATE TABLE IF NOT EXISTS evaluation_runs (
    run_id                  TEXT PRIMARY KEY,
    experiment_id           TEXT REFERENCES experiments(experiment_id),
    started_at              TIMESTAMPTZ NOT NULL,
    completed_at            TIMESTAMPTZ,
    status                  TEXT NOT NULL DEFAULT 'running',
    schema_version          INTEGER NOT NULL DEFAULT 1
);

-- metric_results
CREATE TABLE IF NOT EXISTS metric_results (
    result_id               TEXT PRIMARY KEY,
    run_id                  TEXT REFERENCES evaluation_runs(run_id),
    metric_name             TEXT NOT NULL,
    metric_value            DOUBLE PRECISION NOT NULL,
    computed_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- holdout_access_log
CREATE TABLE IF NOT EXISTS holdout_access_log (
    access_id               TEXT PRIMARY KEY,
    experiment_id           TEXT REFERENCES experiments(experiment_id),
    accessor                TEXT NOT NULL,
    resource                TEXT NOT NULL,
    registered              BOOLEAN NOT NULL DEFAULT FALSE,
    accessed_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- verification_runs
CREATE TABLE IF NOT EXISTS verification_runs (
    run_id                  TEXT PRIMARY KEY,
    verif_id                TEXT NOT NULL,
    passed                  BOOLEAN NOT NULL,
    executed_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    detail                  JSONB
);

-- mock_gateway_sessions
CREATE TABLE IF NOT EXISTS mock_gateway_sessions (
    session_id              TEXT PRIMARY KEY,
    environment             TEXT NOT NULL DEFAULT 'LOCAL_MOCK_DEMO',
    scenario_id             TEXT NOT NULL,
    config_hash             TEXT NOT NULL,
    build_hash              TEXT NOT NULL,
    started_at              TIMESTAMPTZ NOT NULL,
    ended_at                TIMESTAMPTZ
);

-- mock_gateway_events
CREATE TABLE IF NOT EXISTS mock_gateway_events (
    event_id                TEXT PRIMARY KEY,
    session_id              TEXT REFERENCES mock_gateway_sessions(session_id),
    event_type              TEXT NOT NULL,
    timestamp               TIMESTAMPTZ NOT NULL,
    request_hash            TEXT NOT NULL,
    response_hash           TEXT NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_forecasts_experiment ON forecasts(experiment_id);
CREATE INDEX IF NOT EXISTS idx_forecasts_market ON forecasts(market_id);
CREATE INDEX IF NOT EXISTS idx_intents_experiment ON simulation_intents(experiment_id);
CREATE INDEX IF NOT EXISTS idx_ledger_transition ON ledger_entries(transition_id);
CREATE INDEX IF NOT EXISTS idx_holdout_experiment ON holdout_access_log(experiment_id);
CREATE INDEX IF NOT EXISTS idx_verification_verif ON verification_runs(verif_id);
