# ADR-004: Scaled Integer Numeric Types

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** All financial and probability values use explicit scaled integer domain types. Binary floating-point is prohibited in market-state accounting.

## Context

Floating-point arithmetic introduces non-determinism across platforms, rounding inconsistencies, and silent precision loss. The SRS mandates TYP-001 through TYP-003.

## Decision

`Price`, `Quantity`, `Notional`, `Cash`, `ReservedCash`, `SignedPnl`, and `ProbabilityScaled` are all scaled integers with checked arithmetic. `PRICE_SCALE = 100_000_000`, `PROBABILITY_SCALE_V1 = 1_000_000`.

## Consequences

- Deterministic across all platforms
- No silent precision loss
- Requires wide intermediate types (u128, i128) for notional calculations
- Overflow returns errors, never panics or silently wraps
- Floating-point allowed only in telemetry/reporting, never in state
