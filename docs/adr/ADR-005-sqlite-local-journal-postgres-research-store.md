# ADR-005: SQLite Local Journal + PostgreSQL Research Store

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** Use SQLite WAL for the crash-safe local journal and PostgreSQL for searchable research metadata.

## Context

The journal must survive crashes and produce exactly-one terminal dispositions. PostgreSQL is unavailable in offline replay mode. High query flexibility is needed for research metadata.

## Decision

- SQLite WAL (`synchronous=FULL`) for local journal — zero-dependency, crash-safe
- Separate SQLite file for bounded spool
- PostgreSQL for experiments, forecasts, intents, metrics, and holdout access logs
- Content-addressed artifact store for models and traces

## Consequences

- Two different SQL dialects (acceptable for separate concerns)
- Journal does not require network connectivity
- PostgreSQL is optional for offline replay
- Spool provides graceful degradation when PostgreSQL is unavailable
