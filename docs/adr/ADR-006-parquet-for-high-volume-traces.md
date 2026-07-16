# ADR-006: Parquet for High-Volume Traces

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** Store market-event traces and replay datasets as partitioned Parquet files.

## Context

Market-event traces can contain billions of events. Row-oriented databases are inefficient for analytical queries on this scale. The system needs to replay traces deterministically from immutable storage.

## Decision

- Parquet (columnar, compressed) for market events, historical snapshots, inference features, and evaluation results
- Partitioned by source, year, month, market_id
- DuckDB for local analysis and validation
- PostgreSQL rows only for queryable research metadata, not raw events

## Consequences

- Efficient columnar scans for replay and analysis
- Compression reduces storage costs
- Requires DuckDB or Parquet reader for ad-hoc queries
- Partition pruning enables fast time-range queries
