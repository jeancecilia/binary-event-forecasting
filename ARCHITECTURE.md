# ARCHITECTURE.md — Repository and Runtime Map

## System Overview

The Binary Event Forecasting project is a process-separated monorepo. Rust owns deterministic canonical simulation state; Python owns forecasting and research workflows; both share versioned JSON contracts.

```text
source data → Python intelligence plane → forecast message
                                         ↓
market data → Rust core engine → policy → matching → ledger
                                         ↓
                                journal and deterministic replay

mock gateway ↔ scripted integration scenarios and traces
```

## Components

### Rust core engine

Location: [`services/core-engine/`](services/core-engine/)

Composes market ingestion, snapshots, forecast validation, policy transformation, matching, ledger transitions, journaling, crash recovery, and replay.

### Python intelligence plane

Location: [`services/intelligence-plane/`](services/intelligence-plane/)

Composes source ingestion, preprocessing, inference, calibration, evidence lineage, experiment registration, reporting, and audit export.

### Mock gateway

Location: [`services/mock-gateway/`](services/mock-gateway/)

Provides a configurable HTTP interface for scripted acknowledgements, fills, cancellations, settlements, and trace recording.

### Storage

| Storage | Purpose | Location |
|---|---|---|
| Parquet | High-volume traces and replay datasets | `data/traces/` |
| SQLite WAL | Canonical local journal and recovery | `var/journal/` |
| SQLite spool | Buffered research-store writes | `var/spool/` |
| PostgreSQL | Searchable experiment and reporting metadata | `storage/postgres/` |
| Content-addressed files | Models, traces, manifests, and reports | `var/artifacts/sha256/` |

## Canonical Data Flow

1. Market events are deterministically ordered and materialized as immutable snapshots.
2. The intelligence plane emits a versioned `ForecastMessage`.
3. The core validates the message independently.
4. A versioned forecast policy derives an immutable `SimulationIntent`.
5. Matching evaluates the intent against the arrival-state snapshot and shared virtual state.
6. Ledger transitions are planned, applied idempotently, checkpointed, and committed through SQLite.
7. Replay recomputes the same canonical state hash from the same frozen inputs.

## Cross-Process Contracts

IPC uses a 4-byte big-endian length prefix followed by UTF-8 JSON. Schema definitions live in [`contracts/schemas/`](contracts/schemas/), with matching Rust and Python models.

Python sends `forecast_message`; Rust returns `receipt_acknowledgement` and later lifecycle dispositions.

## Internal Dependency Direction

```text
domain-types
    ↓
protocol
    ↓
market-state
    ↓
forecast-policy
    ↓
matching
    ↓
ledger
    ↓
journal    replay
    ↓
core-engine
```

This layering is a maintainability rule: lower-level crates expose stable concepts and do not depend on orchestration code. The exact graph is validated by [`devtools/dependency-boundary-checker/main.py`](devtools/dependency-boundary-checker/main.py).

## Primary Entry Points

- Core CLI: [`services/core-engine/src/main.rs`](services/core-engine/src/main.rs)
- Replay orchestration: [`services/core-engine/src/modes/replay.rs`](services/core-engine/src/modes/replay.rs)
- IPC server: [`services/core-engine/src/ipc.rs`](services/core-engine/src/ipc.rs)
- Forecast policy: [`crates/forecast-policy/src/lib.rs`](crates/forecast-policy/src/lib.rs)
- Immediate matching: [`crates/matching/src/immediate.rs`](crates/matching/src/immediate.rs)
- Ledger: [`crates/ledger/src/lib.rs`](crates/ledger/src/lib.rs)
- Journal database: [`crates/journal/src/db.rs`](crates/journal/src/db.rs)
- Python contracts: [`python-packages/contracts-py/src/contracts_py/`](python-packages/contracts-py/src/contracts_py/)

## Architectural Decisions

See [`docs/adr/`](docs/adr/) for decisions about repository layout, canonical ownership, contracts, numeric types, storage, hashing, replay, and agent navigation.

