# ADR-008: Linux AF_UNIX IPC

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** Use Linux AF_UNIX sockets with peer credential authentication for Rust↔Python IPC.

## Context

The SRS specifies authenticated IPC for Linux. Windows named pipes have different security semantics. Cross-platform IPC would add complexity without research benefit.

## Decision

- AF_UNIX socket at `/run/binary-event-research/core.sock`
- SO_PEERCRED for OS-level identity verification
- Strict filesystem permissions (`0600`)
- Python connects as client, Rust binds as server
- Unsupported platforms fail closed

## Consequences

- Linux-only (acceptable per SRS)
- Peer credentials provide transport authentication without JWT/OAuth complexity
- No network exposure — IPC is local only
- Windows development requires WSL2
