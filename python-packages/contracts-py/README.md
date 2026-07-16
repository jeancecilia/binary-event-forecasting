# Contracts-Py — Python Pydantic Models

## Responsibility

Strict Pydantic models matching the JSON Schema contracts for all cross-process messages. Unknown enum values fail closed. Unknown schema versions fail closed.

## Models

- `ForecastMessage`
- `SimulationIntent`
- `ReceiptAcknowledgement`
- `LifecycleDisposition`
- `MarketEvent`
- `MockGatewayMessage`

## Requirements
- IPC-001 through IPC-005
- FCP-002
