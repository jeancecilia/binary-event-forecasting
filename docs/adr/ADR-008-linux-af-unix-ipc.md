# ADR-008: Linux AF_UNIX IPC

**Status:** Accepted  
**Date:** 2026-07-15  
**Decision:** Use Linux AF_UNIX sockets for Rust↔Python IPC.

## Context

The core engine and intelligence plane are separate local processes. They need a simple streaming transport for versioned JSON messages without adding a separate broker to the development and replay workflow.

## Decision

- Rust binds the configured AF_UNIX socket.
- Python connects as a client.
- Messages use the framing and schema rules defined by the protocol crate.
- Windows development uses WSL2 for the current implementation.

## Consequences

- The transport is simple to inspect and test locally.
- Replay and integration tests can use temporary socket paths.
- Native Windows support would require an additional transport implementation.

