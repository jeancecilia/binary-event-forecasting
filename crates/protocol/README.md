# Protocol

## Responsibility
Defines canonical Rust types for all cross-process messages. Types must match JSON Schema contracts exactly.

## Owns
- `ForecastMessage` — incoming forecast from Python
- `SimulationIntent` — deterministic intent
- `ReceiptAcknowledgement` — Rust acknowledgement
- `LifecycleDisposition` — terminal state
- `MarketEvent` — market data event
- Closed enums (ReceiptStatus, DispositionStatus, BookSide, etc.)
- Framing (4-byte big-endian length prefix)
- Canonical JSON hashing

## Does not own
- Domain types (domain-types)
- Market state (market-state)
- Matching decisions (matching)

## Requirements
- IPC-001, IPC-002, IPC-003, IPC-004, IPC-005
- FCP-002

## Verification
- IPC-001-V1, IPC-002-V1, IPC-003-V1, IPC-004-V1, IPC-004-V2, IPC-005-V1
