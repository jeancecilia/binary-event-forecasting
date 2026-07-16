# Domain Types

## Responsibility
Defines explicit scaled integer domain types for all financial and probability values. No binary floating-point arithmetic.

## Owns
- `Price` — scaled integer price
- `Quantity` — scaled integer quantity
- `Notional` — `round(Price × Quantity / PriceScale)`
- `Cash` — signed cash balance (i128)
- `ReservedCash` — cash reserved for open orders
- `SignedPnl` — signed profit/loss
- `ProbabilityScaled` — probability × ProbabilityScale
- `UncertaintyInterval` — bounds with coverage level

## Does not own
- Schema validation (protocol)
- Matching logic (matching)
- Ledger accounting (ledger)

## Requirements
- TYP-001, TYP-002, TYP-003

## Verification
- TYP-001-V1, TYP-002-V1, TYP-003-V1
