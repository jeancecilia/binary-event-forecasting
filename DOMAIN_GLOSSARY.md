# DOMAIN_GLOSSARY.md — Terminology and Concepts

## Core Concepts

### Binary Event
A future event with exactly two possible resolutions: `Yes` or `No`. Additional terminal outcomes (`Void`, `Cancelled`, `Invalid`, `DefinitionChanged`) are tracked separately and do not contribute to primary forecasting scores.

### Forecast Message
A versioned IPC message emitted by the Python intelligence plane containing a calibrated probability estimate for a binary event. The forecast message is _not_ a simulation intent; it must be transformed through a deterministic forecast-to-intent policy.

### Simulation Intent
An immutable, deterministically-derived order representation produced by the forecast-to-intent policy. Contains `order_class`, `book_side`, `quantity`, `price_limit`, `time_in_force`, and all provenance references.

### Market Snapshot
An immutable, internally consistent view of an order book at a specific logical timestamp. Contains ordered bid/ask levels, feed metadata, and synchronization status.

### Logical Simulation Clock (`t_simulation`)
A deterministic clock used in Offline Replay Mode. All event ordering, snapshot publication, and state transitions are driven by archived timestamps and deterministic tie-breakers — never by wall-clock time.

## State Machine Concepts

### Receipt Status (Rust → Python acknowledgement)
- `AcceptedQueued` — Message valid and queued for processing
- `DuplicateRetry` — Previously processed message retried
- `ExpiredOnArrival` — Message expired before processing
- `RejectedSchema` — Schema validation failed
- `RejectedBounds` — Probability or uncertainty out of bounds
- `RejectedCapacity` — Queue full, message evicted
- `RejectedTargetVersion` — Target definition version mismatch
- `RejectedRateLimit` — Sender rate limit exceeded
- `ReplaySequenceViolation` — Sequence regression in replay
- `CoreDegraded` — Core operating in degraded mode

### Disposition Status (lifecycle terminal state)
- `Validated` — Message passed all validation
- `Evaluated` — Forecast policy evaluated
- `Abstained` — Policy chose to abstain
- `SimulationSubmitted` — Intent submitted to matcher
- `Simulated` — Intent fully simulated
- `PartiallyFilled` — Intent partially filled
- `SimulationRejected` — Intent rejected by matcher
- `SimulationFailed` — Simulation error
- `Superseded` — Replaced by newer message
- `Evicted` — Removed from queue
- `ExpiredInQueue` — Expired while queued

### Resolution Status
- `Open` — Event not yet resolved
- `Proposed` — Resolution proposed
- `Disputed` — Resolution disputed
- `PendingFinality` — Awaiting finality period
- `Final` — Resolution is final

### Terminal Outcome
- `Yes` — Event occurred
- `No` — Event did not occur
- `Void` — Event voided
- `Cancelled` — Event cancelled
- `Invalid` — Event determined invalid
- `DefinitionChanged` — Event definition changed

### Market Feed Status
- `Initializing` — Feed starting up
- `Fragmented` — Gap, invalid delta, or integrity failure detected
- `Disconnected` — Feed connection lost
- `Stale` — No recent updates
- `Failed` — Feed has failed

No baseline, matching decision, NAV valuation, or policy shall use a market in any of these states.

### Valuation Status
- `Valued` — Position has valid two-sided market
- `PartiallyValued` — One-sided market available
- `Unpriceable` — No usable price data
- `Stale` — Last price exceeds staleness threshold
- `Fragmented` — Market in fragmented state

### Baseline Status
- `ValidTwoSided` — Both bid and ask available within spread limits
- `OneSided` — Only one side available
- `Stale` — Data exceeds staleness threshold
- `Fragmented` — Market integrity issue
- `Missing` — No data available
- `SpreadTooWide` — Bid-ask spread exceeds configured maximum

## Numeric Types

All financial and probability values use **scaled integers**:

| Type | Description |
|---|---|
| `Price` | Scaled integer price (e.g., cents or ticks) |
| `Quantity` | Scaled integer quantity |
| `Notional` | `round(Price × Quantity / PriceScale)` |
| `Cash` | Scaled integer cash balance |
| `ReservedCash` | Cash reserved for open orders |
| `SignedPnl` | Signed profit/loss |
| `ProbabilityScaled` | Probability × `ProbabilityScale` (e.g., 625000 = 0.625 when scale = 1,000,000) |

**Binary floating-point arithmetic is prohibited** in market-state accounting, matching, ledger transitions, fees, or P&L.

## Time Domains

### Bitemporal Data
Every source document and artifact records:
- `source_valid_at` — When the data was valid at the source
- `first_observed_at` — When the system first observed the data
- `stored_at` — When the data was stored
- `revision_id` — Revision identifier
- `content_hash` — Content integrity hash

An artifact may contribute to a forecast only when `first_observed_at ≤ decision_cutoff_at`.

### Cross-Process Latency
Latency is reported as separate components:
- Python enqueue duration
- IPC round-trip time
- Rust receive-to-parse duration
- Rust parse-to-acknowledgement duration
- Rust acknowledgement-to-disposition duration

One-way latency is never estimated by subtracting unrelated language-runtime monotonic timestamps.

## Execution Classes

- `LocalConservativeSimulation` — Results from the local conservative matcher
- `ExternalizedLocalMockExecution` — Results from the local mock gateway

These classes are stored distinctly and never merged silently.

## Experiment Control

- **Frozen chronological holdout:** Training → Calibration → Frozen test (no temporal mixing)
- **Prospective prequential evaluation:** Only already-resolved prior events may update later forecasts
- Protocols are **never mixed** in a single primary result

## Participation Ratio

`ParticipationRatio = Q_simulated / Q_visible`

Results are segmented by preregistered participation-ratio bands. The simulator assumes hypothetical orders do not alter subsequent participant behavior unless an independently versioned impact model is enabled.
