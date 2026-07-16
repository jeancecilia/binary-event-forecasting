# ADR-009: No Real-Service Adapters

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** The research build shall not contain production trading adapters, private-key handling, payment code, or real-service submission routes.

## Context

The system is non-monetary by construction. Real-service adapters create legal, security, and compliance risks. The research focus is on forecast accuracy and simulation policy, not production trading.

## Decision

CI enforces absence of: real exchange/ broker adapters, private-key imports, payment/deposit code, live credentials, and external order submission routes. Violations fail the build.

## Consequences

- Clear research boundary
- No regulatory burden
- Cannot accidentally interact with real markets
- Mock gateway provides sufficient testing capability
