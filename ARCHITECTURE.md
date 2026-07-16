# ARCHITECTURE.md — System Design and Process Boundaries

## Overview

The Binary Event Forecasting system is a **decoupled, process-separated research platform** with four logical components:

```
┌─────────────────────────────────────────────────────────────┐
│                    Research Monorepo                         │
│                                                              │
│  ┌──────────────────┐    AF_UNIX IPC    ┌────────────────┐  │
│  │  Rust Core Engine │◄────────────────►│ Python Intel.  │  │
│  │  (canonical state)│   versioned msgs  │ Plane           │  │
│  │                   │                   │ (inference)     │  │
│  └────────┬──────────┘                   └────────────────┘  │
│           │                                                   │
│           │ SQLite journal + PostgreSQL store                 │
│           ▼                                                   │
│  ┌──────────────────┐                                        │
│  │  Durable Research │                                        │
│  │  Store            │                                        │
│  └──────────────────┘                                        │
│                                                              │
│  ┌──────────────────┐                                        │
│  │  Local Mock       │  ← localhost/AF_UNIX only             │
│  │  Gateway          │                                        │
│  └──────────────────┘                                        │
└─────────────────────────────────────────────────────────────┘
```

## Process Boundaries

### 1. Rust Core Simulation Engine (`services/core-engine/`)

**Sole owner of:**
- Market-event ingestion and ordering
- Canonical order books and snapshots
- Logical simulation clock (`t_simulation`)
- Forecast-message validation (secondary, independent of Python)
- Forecast-to-intent policy transformation
- Matching engine (immediate + passive queue)
- Cash and inventory ledger
- Settlement
- Durable journal (SQLite WAL)
- Crash recovery and idempotency
- Canonical artifact hashing
- Offline replay

**Must not contain:**
- LLM API clients
- External model-service clients
- Production trading adapters
- Private-key handling
- Real market credentials
- External order submission routes

### 2. Python Intelligence and Audit Plane (`services/intelligence-plane/`)

**Owns:**
- Source document ingestion
- Untrusted-text preprocessing
- Feature generation and storage
- Model inference (ensemble)
- Probability calibration
- Evidence-set lineage and hashing
- Experiment registration
- Research reporting and audit export

**Must not:**
- Mutate canonical order-book or matching state
- Create fills, change balances, or update settlement
- Write directly to the Rust journal
- Execute code from untrusted model output

### 3. Local Mock Demo Gateway (`services/mock-gateway/`)

**Owns:**
- Local REST/WebSocket-like test interface
- Scripted mock lifecycle events (acks, fills, cancellations, settlements)
- Deterministic scenario scripting
- Immutable trace recording

**Must not:**
- Connect to external trading, betting, or prediction-market hosts
- Accept configurations with external hostnames or credentials
- Modify forecast artifacts

### 4. Durable Research Store

**Comprises four storage mechanisms:**

| Storage | Purpose | Location |
|---|---|---|
| Parquet | High-volume immutable traces, market events, replay datasets | `data/traces/` |
| SQLite (WAL) | Crash-safe local journal | `var/journal/core-journal.sqlite` |
| SQLite (spool) | Bounded PostgreSQL spool | `var/spool/research-store-spool.sqlite` |
| PostgreSQL | Searchable research metadata and reporting | External database |
| Content-addressed (SHA-256) | Models, calibration files, traces, manifests | `var/artifacts/sha256/` |

## Cross-Process Communication

All cross-process state changes use **versioned message contracts** over Linux `AF_UNIX` sockets.

IPC framing:
- 4-byte big-endian unsigned length prefix
- UTF-8 JSON payload
- Explicit schema version
- Maximum frame size (`MAX_SIGNAL_FRAME_BYTES`)
- Read timeout and idle timeout

Python sends `forecast_message` → Rust validates independently → Rust produces `receipt_acknowledgement` → Rust produces `lifecycle_disposition`.

## Dependency Rules (CI-Enforced)

```
domain-types
    ↓
protocol        ← domain-types only
    ↓
market-state    ← domain-types, protocol
    ↓
forecast-policy ← domain-types, protocol, market-state
    ↓
matching        ← domain-types, protocol, market-state
    ↓
ledger          ← domain-types, protocol, matching
    ↓
journal         ← domain-types, protocol, ledger
replay          ← domain-types, protocol, journal
    ↓
core-engine     ← all crates above
```

Forbidden edges:
- Lower crate → higher crate
- Rust → LLM/API client libraries
- Python → Rust ledger/journal crates
- mock-gateway → external network

## Operating Modes

### Offline Replay Mode
- Denies `AF_INET`/`AF_INET6` socket creation
- Denies DNS resolution
- Allows only configured `AF_UNIX` IPC
- Consumes versioned local traces only
- Produces deterministic canonical hashes

### Prospective Observation Mode
- Allows only configured read-only data routes
- Rejects unknown destinations
- Records all network denials
- Routes all intents to local simulator or mock gateway

## Key Design Decisions

See [`docs/adr/`](docs/adr/) for all Architecture Decision Records.
