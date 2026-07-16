# Matching

## Responsibility
Evaluates immutable simulation intents against arrival-state market books.

## Owns
- Immediate all-or-none matching
- Passive queue lifecycle simulation
- Shared virtual depth tracking

## Does not own
- Forecast generation
- Calibration
- Market ingestion
- Experiment registration

## Public API
- `match_immediate(intent, snapshot, state) -> MatchResult`
- `acknowledge_passive(intent, snapshot, state) -> MatchResult`
- `VirtualDepth::can_consume(price, requested, available) -> bool`
- `VirtualDepth::consume(price, quantity, available) -> Result`

## Important Invariants
- Depth is never consumed twice
- All-or-none intents never partially fill
- Matching uses state immediately before arrival
- Cash invariant: FreeCash + ReservedCash = TotalCash

## Requirements
- TIM-002, SIM-001, SIM-002, SIM-003, SIM-004

## Verification
- TIM-002-V1, SIM-001-V1, SIM-002-V1, SIM-003-V1, SIM-004-V1
