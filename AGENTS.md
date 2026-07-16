# AGENTS.md — Agent Navigation Guide

This file is the shortest path for an agent to understand and modify the repository without loading the entire codebase.

## Read Before Editing

1. Read [`ARCHITECTURE.md`](ARCHITECTURE.md) for the component map and dependency direction.
2. Read the README for the crate, package, or service you will change.
3. Locate the relevant requirement in [`docs/specification/binary_event_forecasting_srs_v1_2.md`](docs/specification/binary_event_forecasting_srs_v1_2.md).
4. Locate its verification mapping in [`verification/verification_matrix_v1_2.csv`](verification/verification_matrix_v1_2.csv).
5. Read [`contracts/canonicalization.md`](contracts/canonicalization.md) before changing contracts, hashes, journals, or replay state.
6. Read the relevant ADR in [`docs/adr/`](docs/adr/) when changing an architectural decision.

## Fast Repository Search

Use exact search before broad reading:

```bash
rg --files
rg -n "SymbolName" crates services python-packages
rg -n "REQ-ID|VERIF-ID" docs verification crates services python-packages
```

Useful task commands:

```bash
just test-verif IPC-001-V1
just test-req IPC-001
just validate-requirements
just validate-contracts
just check-dependencies
just replay-verify
```

## Component Ownership

- Rust owns canonical order books, matching, ledger state, settlement, journaling, and replay.
- Python owns ingestion, preprocessing, inference, calibration, experiments, reporting, and audit export.
- Cross-process changes use the versioned contracts in [`contracts/schemas/`](contracts/schemas/).
- The mock gateway provides scripted integration behavior and trace capture.

## Rust Dependency Direction

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

Exact allowed internal dependencies are checked by [`devtools/dependency-boundary-checker/main.py`](devtools/dependency-boundary-checker/main.py). Lower-level crates should remain independent of application-level crates so agents can reason about modules locally.

## Contract Changes

Update all contract representations together:

1. JSON Schema in [`contracts/schemas/`](contracts/schemas/)
2. Rust types in [`crates/protocol/`](crates/protocol/)
3. Python models in [`python-packages/contracts-py/`](python-packages/contracts-py/)
4. Golden vectors and examples
5. Contract validation and cross-language hash tests

Schema versions are integers. Breaking changes increment the version. Each version defines its own probability scale.

## Definition of Done

A change is complete when:

1. Relevant requirement and verification IDs are identified.
2. Targeted tests pass.
3. Contract vectors match when contracts changed.
4. Replay hashes match when canonical state changed.
5. Dependency and requirement validators pass.
6. The affected module README is current.

