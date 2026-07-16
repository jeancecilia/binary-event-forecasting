# ADR-011: Canonical JSON and Hashing

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** All hashing uses SHA-256 of canonical JSON (sorted keys, no whitespace, consistent number formatting).

## Context

Deterministic replay requires identical hashes across Rust and Python. Different JSON serializers produce different outputs (key ordering, whitespace, number formatting).

## Decision

- Sort object keys lexicographically
- No whitespace outside strings
- Consistent number serialization
- SHA-256 for all content hashing
- Golden vectors verify cross-language hash equivalence
- Unordered collections are sorted before hashing

## Consequences

- Requires canonical serialization in both Rust and Python
- Performance overhead of sorting keys (acceptable)
- Golden vectors provide regression detection
- Hash chain in journal enables tamper detection
