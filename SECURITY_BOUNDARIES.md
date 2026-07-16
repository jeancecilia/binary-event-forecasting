# SECURITY_BOUNDARIES.md — Security Model and Enforcement

## Core Security Principle

This system is **non-monetary by construction**. It contains no path to real money, real trading, or real prediction markets. Every security control exists to protect research integrity, not financial assets.

## Operating Modes

### Offline Replay Mode (SEC-001)

The most restrictive mode. Enforced at the system-call level:

- **Denied:** `AF_INET`, `AF_INET6` socket creation
- **Denied:** DNS resolution (`getaddrinfo`, etc.)
- **Allowed:** Configured `AF_UNIX` IPC sockets only
- **Allowed:** Local filesystem access (traces, configs, artifacts)
- **No external data or model calls permitted**

Violation behavior:
1. Abort the replay immediately
2. Durably record the violation
3. Mark the run invalid
4. Produce no valid final research artifact

Implementation: seccomp BPF filter applied at process start. See [`deploy/seccomp/`](deploy/seccomp/).

### Prospective Observation Mode (SEC-002)

Allows **read-only** access to approved research data sources through a strict egress allowlist:

- Only configured allowlisted data routes succeed
- Unknown destinations are rejected
- All denials are durably recorded
- Every simulation intent routes to the local simulator or mock gateway
- No remote execution, account, payment, or credential paths exist

### Non-Monetary Enforcement (SEC-003)

CI scans enforce the absence of:

| Prohibited Category | Detection |
|---|---|
| Production trading adapters | Source text scan for known library patterns |
| Private-key handling (`ed25519-dalek`, `secp256k1`, etc.) | `cargo-deny` + custom rules |
| Account funding/withdrawal code | Source text scan |
| Payment processing | Source text scan |
| Live execution credentials | Secret scanning (trufflehog, git-secrets) |
| Real-service submission routes | URL/hostname pattern check |
| External order submission | Dependency graph analysis |

## IPC Security (SEC-004)

### Authentication
- On Linux: peer credentials (`SO_PEERCRED`) verify connecting OS identity (PID, UID, GID)
- Strict filesystem permissions on the Unix-domain socket (`0600`)
- Peer identity establishes **transport authentication only**, not semantic trust

### Secondary Validation (Rust-side, independent of Python)
Every incoming message is validated for:
1. Schema version
2. Sender identity and sequence
3. Message freshness and expiry
4. Replay status
5. Target allowlist
6. Target-definition version
7. Probability and uncertainty bounds
8. Size and rate limits
9. Experiment and policy versions

A message that passes Python-side validation but fails Rust-side validation is rejected.
Python is treated as an **untrusted client** for validation purposes.

### Connection Limits
- Maximum frame size (`MAX_SIGNAL_FRAME_BYTES`)
- If declared length exceeds limit, connection is closed **before** allocating payload buffer
- Read timeout and idle timeout enforced

## Platform Security (PLT-001)

- Authenticated IPC requires Linux (`SO_PEERCRED`, `AF_UNIX` with filesystem permissions)
- Deployment verification inspects effective PID, UID, GID under actual container user-namespace
- Unsupported platforms **fail closed** rather than silently disabling peer verification

## Mock Gateway Security (DEM-001, DEM-002)

- Binds only local interfaces (`127.0.0.1` or `AF_UNIX`)
- Rejects configurations containing external hostnames
- Rejects configurations containing remote credentials
- Rejects configurations containing real-service submission routes
- Rejects configurations containing private keys
- Rejects any `environment` value other than `LOCAL_MOCK_DEMO`
- Unknown or mixed environment identifiers fail closed

## Data Integrity

- All cross-process messages are checksummed
- Journal records are hash-linked (each record references previous record hash)
- Content-addressed artifact store prevents silent overwrites
- Canonical JSON serialization is used before all hashing
- Unordered collections are canonicalized before hashing

## CI Enforcement

All security boundaries are enforced in CI:
- Dependency graph analysis (forbidden edges)
- Source text scanning (prohibited patterns)
- Secret scanning (credentials)
- Container image scanning
- Network policy validation
- seccomp profile validation
