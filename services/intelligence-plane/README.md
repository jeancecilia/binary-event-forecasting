# Intelligence Plane

## Responsibility

The Python Intelligence and Audit Plane performs:
- Source document ingestion
- Untrusted-text preprocessing
- Feature generation and storage
- Model inference (ensemble)
- Probability calibration
- Evidence-set lineage and hashing
- Experiment registration
- Research reporting and audit export

## Boundaries

**Must not:**
- Mutate canonical order-book or matching state
- Create fills, change balances, or update settlement
- Write directly to the Rust journal
- Execute code from untrusted model output

## Architecture

```
source-ingestion → preprocessing → inference → calibration
                                              ↓
                                        forecast_message
                                              ↓
                                    (AF_UNIX → Rust core)
```

## Requirements

- CAL-001 through CAL-004
- FCP-001, FCP-002 (emits forecast_message, does not create intents)
- EXP-001, EXP-002
- ROB-001 (adversarial input robustness)

## Verification

- CAL-001-V1, CAL-002-V1, CAL-002-V2, CAL-003-V1, CAL-004-V1
- FCP-001-V1, FCP-002-V1
- EXP-001-V1, EXP-002-V1
- ROB-001-V1
