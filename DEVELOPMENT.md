# DEVELOPMENT.md — Development Workflow

## Prerequisites

- **Linux** (WSL2 with Ubuntu 24.04+, native Linux, or dev container)
- **Rust** (see [`rust-toolchain.toml`](rust-toolchain.toml) for exact version)
- **Python 3.11+** with [`uv`](https://github.com/astral-sh/uv)
- **Docker** or **Podman**
- **PostgreSQL** client tools (`psql`, `pg_isready`)
- **SQLite** (`sqlite3`)
- **DuckDB** CLI
- **just** command runner
- **ripgrep** (`rg`)
- **rust-analyzer** (editor integration)
- **Pyright** (Python type checker)

## Bootstrap

```bash
# Clone and enter the repository
git clone <repo-url>
cd binary-event-research

# Install all dependencies and set up development environment
just bootstrap
```

## Project Structure

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full layout.

### Quick Reference

| Directory | Purpose |
|---|---|
| `services/` | Executable services (core-engine, mock-gateway, intelligence-plane) |
| `crates/` | Shared Rust libraries |
| `python-packages/` | Shared Python packages |
| `contracts/` | Cross-language JSON schemas and golden vectors |
| `storage/` | Database migrations and schemas |
| `verification/` | Verification tests, matrix, reference ledgers |
| `experiments/` | Experiment manifests and frozen configurations |
| `data/` | Fixtures, traces, adversarial corpus, local test data |
| `docs/` | ADRs, module docs, diagrams, generated docs |
| `devtools/` | Developer tooling (context indexer, validators) |
| `deploy/` | Container, seccomp, systemd, network configurations |
| `scripts/` | Automation scripts |

## Common Commands

```bash
# Build everything
just build

# Run all tests
just test

# Run specific verification test
just test-verif IPC-001-V1

# Run requirement validator
just validate-requirements

# Run contract validator (cross-language golden vectors)
just validate-contracts

# Run dependency boundary checker
just check-dependencies

# Run offline replay verification
just replay-verify

# Generate repository map
just generate-repo-map

# Lint all code
just lint

# Type check Python
just typecheck
```

## Development Workflow

### 1. Pick a task
Find the relevant REQ ID from the SRS and VERIF ID from the verification matrix.

### 2. Create a branch
```bash
git checkout -b feat/REQ-ID-description
```

### 3. Implement
- Write code in the appropriate crate/package
- Add verification tests named after VERIF IDs
- Update module README if public API changes

### 4. Validate locally
```bash
just lint
just test
just validate-requirements
just check-dependencies
```

### 5. Create PR
CI will run the full pipeline. All gates must pass.

## Coding Standards

### Rust
- `#[deny(unsafe_code)]` in all crates except the IPC server (peer credentials)
- `#[deny(clippy::disallowed_types)]` for floating-point in domain crates
- All public types must implement `Debug`, `Clone`, `Serialize`, `Deserialize`
- Checked arithmetic only (`checked_add`, `checked_mul`, etc.)
- No `unwrap()` or `expect()` in production code paths

### Python
- Strict Pydantic models for all cross-process messages
- Type hints required on all public functions
- Pyright in strict mode
- No `Any` types in contract models
- `ruff` for linting and formatting

### Contracts
- JSON Schema is the authoritative external contract
- All changes require updates to Rust types, Python models, and golden vectors
- Schema version must be incremented on breaking changes

## Adding a New Crate

1. Create `crates/<name>/Cargo.toml` with appropriate dependencies
2. Add to workspace `[members]` in root `Cargo.toml`
3. Add dependency rules to `devtools/dependency-boundary-checker/rules.toml`
4. Create `crates/<name>/README.md`
5. Run `just check-dependencies` to verify no forbidden edges

## Adding a New Python Package

1. Create `python-packages/<name>/pyproject.toml`
2. Add to workspace in root `pyproject.toml` (if using workspace)
3. Add dependency rules
4. Create `python-packages/<name>/README.md`

## Changing a Contract

1. Update the JSON Schema in `contracts/schemas/`
2. Update Rust types in `crates/protocol/src/`
3. Update Python models in `python-packages/contracts-py/src/`
4. Add/update golden vectors in `contracts/golden-vectors/`
5. Add invalid examples in `contracts/invalid-examples/`
6. Run `just validate-contracts`
7. Increment schema version if breaking

## Debugging

### Core engine
```bash
# Run with debug logging
RUST_LOG=debug just run-core

# Run under rr for deterministic debugging
just debug-replay
```

### IPC
```bash
# Monitor the Unix socket
socat -v UNIX-LISTEN:/tmp/debug.sock,mode=700,reuseaddr,fork UNIX-CONNECT:/run/binary-event-research/core.sock
```

### Journal
```bash
# Inspect the SQLite journal
sqlite3 var/journal/core-journal.sqlite "SELECT * FROM journal_records ORDER BY logical_timestamp DESC LIMIT 10;"
```
