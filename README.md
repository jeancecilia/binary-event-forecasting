# Binary Event Forecasting — Research Monorepo

**Decoupled Binary-Event Forecasting and Non-Monetary Simulation System**

A research platform for measuring whether probabilistic forecasts and simulated execution policies demonstrate reproducible performance under causal, conservative, and preregistered assumptions.

## System Boundaries

This system is **non-monetary by construction**. It contains no production trading adapters, private-key handling, payment paths, or real-service submission routes. All execution is simulated, and all mock interfaces are locally hosted.

## Quick Start

```bash
# Prerequisites: Linux (WSL2 or native), Rust, Python 3.11+, uv, Docker, just
just bootstrap
just build
just test
just replay-verify
```

## Repository Map

See [`docs/generated/repository-map.md`](docs/generated/repository-map.md) for an auto-generated index of all modules, symbols, requirements, and tests.

## Mandatory Reading Order

1. [`ARCHITECTURE.md`](ARCHITECTURE.md) — System design and process boundaries
2. [`SECURITY_BOUNDARIES.md`](SECURITY_BOUNDARIES.md) — Security model and enforcement
3. [`DOMAIN_GLOSSARY.md`](DOMAIN_GLOSSARY.md) — Terminology and concepts
4. [`DATA_DICTIONARY.md`](DATA_DICTIONARY.md) — Schema fields and types
5. [`DEVELOPMENT.md`](DEVELOPMENT.md) — Development workflow
6. [`TESTING.md`](TESTING.md) — Test strategy and verification
7. [`contracts/canonicalization.md`](contracts/canonicalization.md) — How hashes are computed

## Specification

The authoritative specification is [`docs/specification/binary_event_forecasting_srs_v1_2.md`](docs/specification/binary_event_forecasting_srs_v1_2.md).

The verification matrix is at [`verification/verification_matrix_v1_2.csv`](verification/verification_matrix_v1_2.csv).

## Prohibited Functionality

- Real-money accounts, deposits, or withdrawals
- Production order submission to any external service
- Private-key loading, signing, or storage
- External LLM API clients in the Rust core
- Canonical state mutation from Python
- Binary floating-point in market-state accounting
- External network access during offline replay

## License

Research use only. See [`LICENSE`](LICENSE).
