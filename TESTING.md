# TESTING.md — Test Strategy and Verification

## Test Hierarchy

```
verification/
├── verification_matrix_v1_2.csv    # Canonical requirement-to-verification mapping
├── requirement-index.json          # Machine-readable requirement index
├── tests/                          # Test implementations named by VERIF ID
│   ├── ARC-001-V1/
│   ├── ARC-002-V1/
│   ├── SEC-001-V1/
│   ├── IPC-001-V1/
│   ├── AUD-004-V1/
│   └── REP-001-V1/
├── reference-ledgers/              # Pre-computed reference outputs
└── reports/                        # Generated verification reports
```

## Test Naming Convention

Every test must reference its VERIF ID:

### Rust
```rust
#[test]
fn verif_ipc_001_v1_rejects_oversized_frame_before_allocation() { }

#[test]
fn verif_aud_004_v1_sigkill_before_plan_produces_exactly_one_transition() { }
```

### Python
```python
def test_cal_002_v1_temporally_ineligible_rows_are_excluded():
    ...

def test_met_001_v1_controlled_fixture_reproduces_brier_score():
    ...
```

## Test Categories

### Unit Tests
- Located alongside source code (`#[cfg(test)] mod tests`)
- Fast, no I/O, no external dependencies
- Run on every commit

### Property-Based Tests
- Use `proptest` (Rust) or `hypothesis` (Python)
- Verify invariants across random input ranges
- Required for: domain types (TYP-001, TYP-002), matching (SIM-001, SIM-003)

### Integration Tests
- Located in `verification/tests/<VERIF-ID>/`
- May use local SQLite, filesystem, or AF_UNIX sockets
- Run on every PR

### Golden Vector Tests
- Located in `contracts/golden-vectors/`
- Verify identical hashes across Rust and Python
- Run on every PR

### Crash Injection Tests
- Located in `verification/tests/AUD-*-V*/`
- Use process isolation and signal delivery
- Run on PR, may be slower

### Replay Tests
- Located in `verification/tests/REP-*-V*/`
- Verify deterministic replay produces identical hashes
- Run on PR

### Adversarial Tests
- Located in `verification/tests/ROB-001-V*/`
- Feed the fixed attack corpus and verify bounded behavior
- Run on PR

## Running Tests

```bash
# All tests
just test

# Only unit tests (fast)
just test-unit

# Only verification tests
just test-verification

# Specific verification test
just test-verif IPC-001-V1

# All tests for a requirement
just test-req IPC-001

# With coverage
just test-coverage

# Property tests only
just test-proptest
```

## Verification Matrix Compliance

The requirement validator (`scripts/validate_requirements.py`) checks:

1. Every REQ ID in the SRS has at least one VERIF ID in the CSV
2. Every VERIF ID in the CSV references a REQ ID that exists in the SRS
3. No duplicate VERIF IDs
4. No duplicate REQ IDs
5. No invisible/control characters in identifiers
6. Every VERIF ID has a corresponding test directory or explicit reviewed justification

Run with:
```bash
just validate-requirements
```

This check runs in CI and fails the build on violations.

## Reference Ledgers

Pre-computed ledgers for deterministic verification are stored in `verification/reference-ledgers/`. These are generated once from a frozen experiment configuration and checked into the repository. Tests compare against these references within configured tolerances.

## Test Fixtures

Shared test fixtures live in `data/fixtures/`:
- `data/fixtures/order-books/` — Reference order book states
- `data/fixtures/forecasts/` — Sample forecast messages (valid and invalid)
- `data/fixtures/traces/` — Miniature market-event traces
- `data/fixtures/experiments/` — Frozen experiment manifests

## CI Test Matrix

| Workflow | Triggers | Tests |
|---|---|---|
| `rust.yml` | Every push/PR | Rust unit + integration + proptest |
| `python.yml` | Every push/PR | Python unit + integration + type check |
| `contracts.yml` | Every push/PR | Schema validation + golden vectors |
| `architecture.yml` | Every push/PR | Dependency boundary check + requirement validator |
| `replay.yml` | PR only | Deterministic replay + crash injection |
| `verification.yml` | PR only | Full verification suite against reference ledgers |
