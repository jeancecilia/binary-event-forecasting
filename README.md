# Binary Event Forecasting

A process-separated forecasting and deterministic execution research engine implemented as a Rust and Python monorepo.

## Start Here

- Agent navigation: [`AGENTS.md`](AGENTS.md)
- Runtime and repository map: [`ARCHITECTURE.md`](ARCHITECTURE.md)
- Requirements: [`docs/specification/binary_event_forecasting_srs_v1_2.md`](docs/specification/binary_event_forecasting_srs_v1_2.md)
- Verification matrix: [`verification/verification_matrix_v1_2.csv`](verification/verification_matrix_v1_2.csv)
- Development workflow: [`DEVELOPMENT.md`](DEVELOPMENT.md)
- Test strategy: [`TESTING.md`](TESTING.md)

## Quick Start

```bash
just bootstrap
just build
just test
just replay-verify
```

## Repository Layout

| Path | Purpose |
|---|---|
| `services/` | Executable Rust and Python services |
| `crates/` | Layered Rust libraries |
| `python-packages/` | Shared Python packages |
| `contracts/` | JSON Schemas and canonicalization rules |
| `verification/` | Requirement mappings and verification artifacts |
| `storage/` | Database schemas and migrations |
| `data/` | Replay traces and fixtures |
| `devtools/` | Repository navigation and validation tools |

