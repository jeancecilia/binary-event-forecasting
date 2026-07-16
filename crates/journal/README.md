# Journal

## Responsibility
Crash-recoverable local journal using SQLite WAL. Implements the SRS transition protocol.

## Owns
- Hash-linked journal records
- Message receipt logging
- Lifecycle disposition tracking
- Transition planning and commit
- Sender sequence tracking
- Processed message idempotency
- Ledger checkpoints
- Bounded spool for PostgreSQL reconciliation

## Does not own
- Ledger state (ledger)
- Matching decisions (matching)
- PostgreSQL migration (storage/)

## Transition Protocol
1. Append `DispositionPlanned` durably
2. Apply the ledger transition idempotently
3. Atomically persist `transition_applications` and the canonical ledger checkpoint
4. Append `DispositionCommitted` durably and idempotently

Recovery restores and hash-verifies the latest canonical checkpoint when one
exists. If a crash occurred after planning but before the first checkpoint,
recovery starts from the configured initial ledger and reconstructs state from
the stored transition payload.

## Requirements
- AUD-001, AUD-002, AUD-003, AUD-004

## Verification
- AUD-001-V1, AUD-002-V1, AUD-002-V2, AUD-003-V1, AUD-003-V2, AUD-004-V1
