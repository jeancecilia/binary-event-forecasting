# ADR-010: Requirements Matrix Enforced by CI

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** The verification matrix (CSV) is the source of truth for requirement-to-verification mapping. CI validates bidirectional traceability.

## Context

Without automated enforcement, requirement documents drift from implementation. The SRS has 58 requirements and 62 verification artifacts.

## Decision

- `scripts/validate_requirements.py` checks bidirectional traceability
- CI rejects undefined IDs, duplicate IDs, orphan requirements, and orphan verifications
- Tests are named after VERIF IDs
- Control characters in identifiers are rejected

## Consequences

- Guaranteed requirement coverage visibility
- Cannot ship a requirement without verification
- Test naming convention enforces traceability
- Requirement changes require verification matrix updates
