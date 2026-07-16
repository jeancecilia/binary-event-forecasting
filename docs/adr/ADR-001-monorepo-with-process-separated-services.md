# ADR-001: Monorepo with Process-Separated Services

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** Use a single Git repository containing all services, shared crates, contracts, verification tests, and documentation.

## Context

The system has four logical components (Rust core, Python intelligence plane, mock gateway, durable store) that share versioned contracts and require synchronized changes. Multiple repositories would create contract versioning nightmares and cross-repo synchronization issues.

## Decision

One monorepo: `binary-event-research/`. All contract changes, verification tests, and service changes stay in sync within a single PR.

## Consequences

- Simpler contract versioning and cross-language hash verification
- CI can run contracts, Rust, Python, and architecture checks atomically
- Requires disciplined dependency rules enforced by CI
- Larger checkout but all components are needed for verification
