# ADR-007: Content-Addressed Artifact Storage

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** Store models, calibration files, traces, manifests, and reports by SHA-256 hash.

## Context

Overwriting artifacts silently breaks reproducibility. Multiple experiments may reference the same artifact. Disk deduplication and integrity verification are needed.

## Decision

Store all artifacts at `var/artifacts/sha256/<AB>/<CD>/<full-hash>`. The database stores metadata and hash. The filesystem stores bytes.

## Consequences

- Immutable by construction — overwrites are impossible
- Automatic deduplication
- Integrity verifiable by recomputing hash
- Requires hash computation at write time
- Garbage collection is a future concern
