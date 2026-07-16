# AGENTS.md — Agent Context and Navigation Guide

## System Boundaries

This is a **non-monetary research system**. The following are strictly prohibited and enforced by CI:

- Real trading, betting, or prediction-market adapters
- Private-key handling or signing modules
- Account funding, withdrawal, or payment paths
- Live execution credentials
- External LLM/API clients in the Rust core engine
- Canonical matching/ledger mutation from Python
- Binary floating-point across process boundaries

## Mandatory Reading Order

Before making any changes, read in this order:

1. [`ARCHITECTURE.md`](ARCHITECTURE.md)
2. [`SECURITY_BOUNDARIES.md`](SECURITY_BOUNDARIES.md)
3. [`docs/adr/`](docs/adr/) — All Architecture Decision Records
4. [`contracts/canonicalization.md`](contracts/canonicalization.md)

## Allowed Dependency Directions

```
domain-types  ←  no internal dependencies (leaf crate)
     ↓
protocol      ←  domain-types only
     ↓
market-state  ←  domain-types, protocol
     ↓
forecast-policy ← domain-types, protocol, market-state
     ↓
matching      ←  domain-types, protocol, market-state
     ↓
ledger        ←  domain-types, protocol, matching
     ↓
journal       ←  domain-types, protocol, ledger
replay        ←  domain-types, protocol, journal
     ↓
core-engine   ←  all crates above
```

Lower-level crates **must not** import application-level crates.
Rust **must not** contain LLM/API clients.
Python **must not** mutate canonical matching or ledger state.

## How to Locate Requirements

- All REQ IDs are defined in [`docs/specification/binary_event_forecasting_srs_v1_2.md`](docs/specification/binary_event_forecasting_srs_v1_2.md)
- Verification mappings are in [`verification/verification_matrix_v1_2.csv`](verification/verification_matrix_v1_2.csv)
- Run `just validate-requirements` to check requirement integrity
- Use [`devtools/context-indexer/`](devtools/context-indexer/) to search symbols and requirements

## How to Run Targeted Tests

```bash
# Run verification tests by ID
just test-verif IPC-001-V1

# Run all tests for a requirement
just test-req IPC-001

# Run the full verification suite
just verify-all
```

## How Contracts Are Changed

1. Propose change in a PR
2. Update the JSON Schema in [`contracts/schemas/`](contracts/schemas/)
3. Update Rust types in [`crates/protocol/`](crates/protocol/)
4. Update Python models in [`python-packages/contracts-py/`](python-packages/contracts-py/)
5. Add golden vectors in [`contracts/golden-vectors/`](contracts/golden-vectors/)
6. Run `just validate-contracts` — must pass cross-language hash equivalence

## How Schema Versions Are Handled

- Schema versions are integers, incremented on breaking changes
- Unknown schema versions **fail closed**
- Unknown enum variants **fail closed**
- Each schema version defines its own `ProbabilityScale` constant

## Prohibited Functionality (CI-Enforced)

| Category | Detection Method |
|---|---|
| External HTTP clients in Rust core | `cargo-deny` + custom lint |
| Real-service adapters | Source text scan |
| Private-key imports | Source text scan |
| Payment/deposit code | Source text scan |
| Float in domain types | Clippy `disallowed-types` |
| Python mutating ledger | Import boundary check |

## Definition of Done

A feature is done when:
1. All relevant REQ IDs have passing VERIF tests
2. Cross-language golden vectors match
3. Replay produces identical hashes across repeated runs
4. Forbidden dependency check passes
5. Requirement validator passes
6. Module README is updated
