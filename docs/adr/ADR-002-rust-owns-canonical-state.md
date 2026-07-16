# ADR-002: Rust Owns Canonical State

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** The Rust core engine is the sole owner of canonical simulation state. Python must not mutate matching, ledger, or settlement state.

## Context

Dual-writer architectures lead to divergence. The SRS defines clear responsibility separation (ARC-002). We need one source of truth for matching decisions and ledger state.

## Decision

Rust owns: order books, snapshots, matching, cash/inventory ledger, settlement, journal, replay. Python emits forecast messages and consumes disposition events. All state mutation flows through the Rust core via AF_UNIX IPC.

## Consequences

- Single canonical state, no divergence
- Python is simpler (no ledger logic)
- All state-changing operations are in Rust (deterministic, type-safe, checked arithmetic)
- Python cannot recover from Rust unavailability — acceptable for research
