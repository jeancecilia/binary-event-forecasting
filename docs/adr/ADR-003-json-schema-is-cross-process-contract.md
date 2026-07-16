# ADR-003: JSON Schema Is Cross-Process Contract

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** JSON Schema is the authoritative external contract. Rust uses Serde types. Python uses strict Pydantic models.

## Context

Cross-process messages must be validated identically in both languages. Binary serialization formats (Protobuf, Avro) require schema registry infrastructure. JSON is human-readable, debuggable, and has mature cross-language support.

## Decision

- JSON Schema in `contracts/schemas/` is authoritative
- Rust: `serde` with `serde_json`
- Python: Pydantic with strict validation
- Unknown enum values and schema versions fail closed
- Golden vectors verify cross-language hash equivalence

## Consequences

- JSON is verbose but debuggable
- Performance overhead of JSON serialization is acceptable for research throughput
- Canonical JSON serialization enables deterministic hashing
