# Dependency Boundary Checker

Checks that crate and package dependencies respect the allowed dependency directions defined in `ARCHITECTURE.md`.

## Usage

```bash
uv run python devtools/dependency-boundary-checker/main.py
```

## Rules

- Lower-level crates must not import application crates
- Rust must not contain LLM/API clients
- Python must not import Rust ledger/journal crates
- core-engine may import all crates

See `rules.toml` for the full dependency graph.
