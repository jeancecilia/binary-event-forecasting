# Software Requirements Specification (SRS)

**System:** Decoupled Binary-Event Forecasting and Non-Monetary Simulation System  
**Version:** 1.2 — Research-Ready Candidate  
**Status:** Normative engineering specification  
**Scope:** Offline replay, prospective observation, model calibration, conservative paper simulation, and a local mock demo environment  
**Out of Scope:** Real-money accounts, production order submission, private-key signing, deposits, withdrawals, or connectivity to a real prediction-market service

---

## 0. Document Control

### 0.1 Normative language

The terms **shall** and **shall not** express mandatory requirements. **Should** expresses a recommendation. **May** expresses an allowed option.

### 0.2 Requirement and verification identity

Each normative requirement shall have one stable `REQ ID`. Each verification artifact shall have one unique `VERIF ID`. A requirement may have multiple verification artifacts.

### 0.3 Research objective

The system shall measure whether probabilistic forecasts and simulated execution policies demonstrate reproducible performance under causal, conservative, and preregistered assumptions. The system shall permit negative results and shall not treat profitability as a software acceptance criterion.

### 0.4 Safety boundary

The research build shall remain non-monetary by construction. The external demo capability defined in this document is a **locally hosted mock gateway**, not an adapter to a real trading, betting, or prediction-market service.

---

# 1. Architecture and Process Boundaries

## ARC-001 — Process Boundaries

The system shall consist of the following process-separated components:

1. **Rust Core Simulation Engine**
   - market-event ingestion;
   - canonical market-state construction;
   - immutable snapshot publication;
   - forecast-message validation;
   - deterministic forecast-to-intent invocation;
   - local matching and ledger transitions;
   - deterministic offline replay;
   - lifecycle and timing telemetry.

2. **Python Intelligence and Audit Plane**
   - source ingestion;
   - untrusted-text preprocessing;
   - model-ensemble inference;
   - probability calibration;
   - evidence and feature lineage;
   - experiment registration;
   - audit export and research reporting.

3. **Local Mock Demo Gateway**
   - local REST/WebSocket-like test interface;
   - mock acknowledgements, rejections, partial fills, cancellations, and settlement events;
   - deterministic scenario scripting;
   - immutable trace recording.

4. **Durable Research Store**
   - append-oriented journal;
   - bounded local spool;
   - immutable experiment manifests;
   - versioned input traces and recorded inference artifacts.

All cross-process state changes shall occur through versioned message contracts.

## ARC-002 — Responsibility Separation

The Rust process shall not contain external LLM or model-service clients. The Python process shall not mutate canonical order-book, matching, cash, inventory, or settlement state directly. The local mock gateway shall not modify forecast artifacts.

A component shall communicate only through its declared interface. Forbidden dependency edges shall fail CI.

---

# 2. Operating Modes and Security Boundaries

## SEC-001 — Offline Replay Mode

Offline Replay Mode shall:

- deny `AF_INET` and `AF_INET6` socket creation;
- deny DNS resolution;
- allow only explicitly configured local `AF_UNIX` IPC;
- consume versioned local traces, configurations, and recorded model/calibration artifacts;
- perform no external data or model calls;
- fail closed if any prohibited network access is attempted;
- produce no valid final research artifact after a network-isolation violation.

## SEC-002 — Prospective Observation Mode

Prospective Observation Mode may consume approved **non-transactional research data sources** through a strict egress allowlist. It shall:

- allow only configured read-only data routes;
- reject unknown destinations;
- contain no remote execution, account, payment, or credential path;
- route every simulation intent to the local simulator or local mock gateway;
- record all network denials.

## SEC-003 — Non-Monetary Enforcement

The research build shall not contain:

- production or real-service submission adapters;
- private-key loading or signing modules;
- account-funding, withdrawal, or payment code;
- live execution credentials;
- runtime routes capable of sending orders to a real service.

CI shall inspect source code, dependency graphs, container manifests, configuration schemas, and network policies. A violation shall fail the build.

## SEC-004 — IPC Authentication and Secondary Validation

On Linux, the Rust IPC server shall authenticate the connecting OS identity with peer credentials and strict socket filesystem permissions. Peer identity shall not establish semantic trust.

The Rust core shall independently validate:

- schema version;
- sender identity and sequence;
- message freshness and expiry;
- replay status;
- target allowlist;
- target-definition version;
- probability and uncertainty bounds;
- size and rate limits;
- experiment and policy versions.

## PLT-001 — Supported Platform

Authenticated IPC is specified for Linux. Deployment verification shall inspect effective PID, UID, and GID under the actual container user-namespace configuration. Unsupported platforms shall fail closed rather than silently disabling peer verification.

---

# 3. Local Mock Demo Environment

## DEM-001 — Local Mock Gateway Boundary

The local mock gateway shall run entirely within the controlled research environment. It shall not resolve or connect to any external trading, betting, or prediction-market host.

It shall expose a versioned local interface sufficient to test:

- market snapshots and deltas;
- forecast-message receipt;
- simulation-intent submission;
- acknowledgements;
- rejections;
- partial and complete mock fills;
- cancellation requests and effective cancellation;
- mock settlement and resolution status changes.

## DEM-002 — Environment Pinning

Every mock-gateway session shall carry:

- `environment = LOCAL_MOCK_DEMO`;
- a versioned scenario identifier;
- a configuration hash;
- a gateway build hash.

Unknown or mixed environment identifiers shall fail closed. The gateway shall reject configuration containing external hostnames, remote credentials, or external submission routes.

## DEM-003 — Execution-Class Separation

The system shall store local matching results and mock-gateway results as distinct classes:

- `LocalConservativeSimulation`;
- `ExternalizedLocalMockExecution`.

A mock-gateway result shall not be labeled deterministic unless it is replayed from a frozen scenario trace.

## DEM-004 — Canonical Mock Audit

Every mock request and lifecycle response shall record:

- experiment ID;
- intent ID;
- gateway scenario ID;
- environment;
- request, acknowledgement, and disposition timestamps;
- requested and filled quantity;
- requested and average fill price;
- fee/cost model version;
- state, rejection, or cancellation reason;
- market snapshot version;
- canonical request and response hashes.

## DEM-005 — Trace Capture and Replay

All mock-gateway market events and lifecycle responses used in a research experiment shall be captured in an immutable, versioned trace. Subsequent deterministic replay shall consume the trace locally and shall make no gateway or network call.

## DEM-006 — Mock-to-Local Comparison

The evaluation suite shall compare the local conservative matcher and local mock gateway using:

- acknowledgement-latency distribution;
- rejection-rate disagreement;
- fill-status disagreement;
- filled-quantity disagreement;
- average-price disagreement;
- lifecycle divergence;
- settlement divergence.

Agreement shall be reported as integration evidence and shall not be treated as evidence of real-world execution quality or profitability.

---

# 4. Time Domains and Data Lineage

## CLK-001 — Logical Simulation Clock

Offline Replay Mode shall use a deterministic logical simulation clock, `t_simulation`.

Market-event ordering, snapshot publication, intent arrival, acknowledgements, cancellations, fills, and settlement shall be driven only by:

- archived event timestamps;
- documented upstream sequences where available;
- deterministic tie-breakers.

Runtime monotonic clocks shall measure replay performance only and shall not affect simulated state transitions or canonical hashes.

## DAT-001 — Bitemporal Data Availability

Every source document, feature, retrieval item, market event, target-definition artifact, and correction shall record:

- `source_valid_at`;
- `first_observed_at`;
- `stored_at`;
- `revision_id`;
- `content_hash`.

An artifact may contribute to a forecast only when:

\[
first\_observed\_at \leq decision\_cutoff\_at
\]

Backfilled or corrected data shall not retroactively alter a previously materialized forecast artifact.

## TIM-001 — Cross-Market Snapshot Coherence

A rule comparing multiple markets shall read a version vector containing each market's:

- snapshot version;
- logical observation timestamp;
- synchronization status.

The system shall compute:

\[
\Delta_{skew}=\max_i(t_i)-\min_i(t_i)
\]

The evaluation shall be rejected as incoherent when:

\[
\Delta_{skew}>\Delta_{skew,max}
\]

`DeltaSkewMax` shall be versioned and preregistered before holdout evaluation.

## TEL-001 — Cross-Process Latency

The system shall not calculate one-way cross-process latency by subtracting unrelated language-runtime monotonic timestamps.

It shall report separately:

- Python enqueue duration;
- IPC round-trip time;
- Rust receive-to-parse duration;
- Rust parse-to-acknowledgement duration;
- Rust acknowledgement-to-disposition duration.

A directly measured end-to-end duration and the sum of component measurements shall be reported separately. Any estimated one-way latency shall state its assumptions.

---

# 5. Market State, Order Book, and Numeric Integrity

## STA-001 — Atomic Market Snapshot

Every evaluation shall observe an immutable, internally consistent snapshot containing:

- target ID;
- snapshot version;
- feed connection generation;
- source sequence metadata;
- source timestamp;
- logical observation timestamp;
- synchronization status;
- ordered bid levels;
- ordered ask levels;
- target-definition version.

Snapshot publication shall be atomic.

## STA-002 — Feed Integrity and Resynchronization

A detected gap, invalid delta, reconnect, malformed event, failed checksum, or failed integrity check shall mark the affected market `Fragmented`.

No baseline, matching decision, NAV valuation, or forecast-to-intent policy shall use a state marked:

- `Initializing`;
- `Fragmented`;
- `Disconnected`;
- `Stale`;
- `Failed`.

Simulation may resume only after an authoritative snapshot and all eligible subsequent deltas have been applied and validated.

## STA-003 — Order-Book Depth and Validity

The canonical order book shall consist of discrete price levels with exact scaled prices and quantities.

For a simulated buy intent with limit \(p_L\):

\[
Q_{available,buy}(p_L)=\sum_{p\le p_L}q_{ask}(p)
\]

For a simulated sell intent:

\[
Q_{available,sell}(p_L)=\sum_{p\ge p_L}q_{bid}(p)
\]

The system shall distinguish:

- invalid or stale market state;
- valid state with insufficient depth;
- valid state with sufficient depth;
- unobservable queue progression.

## TYP-001 — Explicit Integer Domain Types

The following shall use explicit scaled integer domain types:

- `Price`;
- `Quantity`;
- `Notional`;
- `Cash`;
- `ReservedCash`;
- `SignedPnl`;
- `ProbabilityScaled`.

Binary floating-point arithmetic shall not be used in market-state accounting, matching, ledger transitions, fees, or P&L.

## TYP-002 — Checked Arithmetic

Arithmetic shall use checked, sufficiently wide intermediates. Notional shall be calculated as:

\[
Notional=\operatorname{round}\left(\frac{Price\times Quantity}{PriceScale}\right)
\]

Each conversion shall define:

- rounding direction;
- maximum rounding error in scaled units;
- overflow behavior;
- state-mutation policy on error.

An arithmetic failure shall not partially mutate state.

## TYP-003 — Cross-Boundary Probabilities

Probabilities crossing a process boundary shall use:

\[
0\le ProbabilityScaled\le ProbabilityScale
\]

The protocol shall define one `ProbabilityScale` constant per schema version. Bounds, uncertainty intervals, and quantization shall be validated before the message enters the forecast policy.

## COST-001 — Versioned Cost Model

Every simulation intent shall reference a cost-model version effective at `t_arrival`.

The cost model shall distinguish:

- simulated maker cost or rebate;
- simulated taker cost;
- settlement cost where applicable;
- explicitly modeled operational cost.

Execution feasibility and cash feasibility shall be evaluated separately:

\[
Q_{available}\ge Q_{requested}
\]

\[
Cash_{available}\ge Notional+Costs
\]

Costs shall not alter observable order-book depth.

---

# 6. IPC Protocol and Queueing

## IPC-001 — Deterministic Framing

IPC shall use:

- a four-byte big-endian unsigned length header;
- a UTF-8 JSON payload;
- an explicit schema version;
- a maximum frame length;
- a read timeout;
- an idle timeout.

If the declared length exceeds `MAX_SIGNAL_FRAME_BYTES`, the receiver shall close the connection before allocating a payload buffer proportional to the declared size.

## IPC-002 — Closed Status Enums

Receipt acknowledgements and lifecycle dispositions shall be separate closed enums.

**ReceiptStatus:**

- `AcceptedQueued`;
- `DuplicateRetry`;
- `ExpiredOnArrival`;
- `RejectedSchema`;
- `RejectedBounds`;
- `RejectedCapacity`;
- `RejectedTargetVersion`;
- `RejectedRateLimit`;
- `ReplaySequenceViolation`;
- `CoreDegraded`.

**DispositionStatus:**

- `Validated`;
- `Evaluated`;
- `Abstained`;
- `SimulationSubmitted`;
- `Simulated`;
- `PartiallyFilled`;
- `SimulationRejected`;
- `SimulationFailed`;
- `Superseded`;
- `Evicted`;
- `ExpiredInQueue`.

Authentication failure may terminate the connection before receipt parsing and shall be audited locally.

## IPC-003 — Complete Forecast Schema

A `forecast_message` shall contain:

### Protocol identity
- `schema_version`;
- `message_id`;
- `sender_instance_id`;
- `sender_sequence`.

### Target identity
- `market_id`;
- `contract_or_outcome_id`;
- `market_definition_version`;
- `event_id`;
- `underlying_event_group_id`;
- `forecast_target`;
- `forecast_horizon`.

### Source provenance
- `source_id`;
- `source_version`;
- `evidence_set_hash`;
- `published_at`;
- `first_source_available_at`;
- `ingested_at`;
- `revision_id`.

### Model provenance
- `model_artifact_hash`;
- `model_training_cutoff`;
- `ensemble_version`;
- `component_model_versions`;
- `prompt_version`;
- `retrieval_corpus_version`;
- `calibration_model_version`;
- `calibration_training_cutoff`.

### Forecast values
- `raw_model_probability`;
- `calibrated_probability`;
- `uncertainty_lower`;
- `uncertainty_upper`;
- `uncertainty_coverage_level`;
- `uncertainty_method`;
- `abstention_reason`.

### Lifecycle timestamps
- `decision_cutoff_at`;
- `forecast_created_at`;
- `forecast_emitted_at`;
- `expires_at`.

The following invariant shall hold:

\[
0\le uncertainty_{lower}\le calibratedProbability
\le uncertainty_{upper}\le ProbabilityScale
\]

## IPC-004 — Deterministic Admission and Eviction

When the bounded queue is full, the admission controller shall evaluate the union of queued messages and the incoming message.

The retained set shall be chosen deterministically according to versioned configuration using:

1. validity and expiry;
2. per-event fairness budget;
3. source revision within the same `source_id`;
4. earliest `expires_at`;
5. lowest semantic priority;
6. oldest `ingested_at`;
7. lexicographic `message_id`.

The incoming message may itself receive `RejectedCapacity`. Every eviction shall emit an `Evicted` disposition referencing the removed `message_id`.

## IPC-005 — Sender Sequence Semantics

Replay identity shall be:

\[
(sender\_instance\_id,\ sender\_sequence)
\]

A retry shall preserve `message_id`, sender instance, and sender sequence. A genuine process restart shall create a new sender instance ID. Sender sequence shall be strictly increasing within an instance.

The durable journal shall retain processed message IDs and sender-sequence state for the configured retention period.

---

# 7. Durable Audit and Recovery

## AUD-001 — Durable Event Journal

Before a message may produce a state-changing simulation disposition, the core shall append its validation result and lifecycle state to a crash-recoverable local journal.

Journal records shall contain:

- record ID;
- message or intent ID;
- lifecycle state;
- transition ID;
- logical timestamp;
- canonical payload hash;
- previous record hash or equivalent integrity linkage;
- checksum.

## AUD-002 — Idempotency Recovery

After restart, the core shall reconstruct:

- processed message IDs;
- sender sequence state;
- intent lifecycle state;
- planned and committed transitions;
- cash, inventory, reserved resources, and settlement state.

A retry shall result in exactly one terminal disposition.

## AUD-003 — Database Spooling

Database unavailability shall not silently discard audit records.

When database connectivity is unavailable and spool capacity remains, records shall be:

- written to a bounded local spool;
- checksummed;
- replayed in canonical order;
- reconciled exactly once.

When spool capacity is exhausted, the system shall stop accepting state-changing work and enter the preregistered `Degraded` or `Halting` state.

## AUD-004 — Crash-Consistent Transition Commit

Every state-changing transition shall use a unique `transition_id` and the following protocol:

1. append `DispositionPlanned` durably;
2. apply the ledger transition idempotently;
3. append `DispositionCommitted` durably.

Recovery shall distinguish planned, applied, and committed transitions. Reapplying the same transition ID shall not change final state more than once.

---

# 8. Forecasting, Calibration, and Experiment Control

## FCP-001 — Forecast-to-Simulation Policy

A forecast message shall not itself constitute a simulation intent.

The transformation shall use a deterministic, versioned policy. Changes to any of the following after holdout access shall require a new experiment ID:

- forecast policy;
- target mapping;
- threshold or abstention rule;
- sizing rule;
- latency scenario;
- matching model;
- cost model;
- exclusion rule.

## FCP-002 — Simulation Intent Schema

A `simulation_intent` shall be immutable and contain:

- `simulation_intent_id`;
- `experiment_id`;
- `source_forecast_message_id`;
- `forecast_policy_version`;
- `configuration_hash`;
- `market_id`;
- `contract_or_outcome_id`;
- `forecast_target`;
- `order_class`;
- `book_side`;
- `outcome_side`;
- `quantity`;
- `price_limit`;
- `time_in_force`;
- `policy_priority`;
- `decision_timestamp`;
- `simulated_arrival_timestamp`;
- `latency_scenario_version`;
- `matching_model_version`;
- `cost_model_version`;
- `acknowledgement_latency_version`;
- `cancellation_latency_version`;
- `account_state_version`;
- `input_snapshot_version`;
- `expires_at`.

`simulation_intent_id` shall be derived deterministically from the canonical inputs.

## EXP-001 — Experiment Registration

Before holdout evaluation, the experiment manifest shall register:

- experiment ID;
- primary hypothesis;
- primary metrics;
- model, prompt, ensemble, and calibration versions;
- forecast selection rule;
- baseline definitions;
- matching, latency, cost, and settlement models;
- event-group weighting;
- exclusion rules;
- planned comparisons and statistical correction;
- software and configuration hashes.

## EXP-002 — Holdout Access Audit

Every request to access holdout outcomes or aggregate holdout metrics shall be durably logged. An unregistered request shall be rejected and audited.

Any design change after holdout access shall require a new experiment and a new untouched or prospective evaluation period.

## CAL-001 — Temporal Calibration Protocol

Observations sharing an `underlying_event_group_id` shall remain in the same partition.

The supported evaluation protocols shall be explicitly declared as either:

- **Frozen chronological holdout:** training, calibration, then frozen test;
- **Prospective prequential evaluation:** only already resolved prior events may update later forecasts.

The protocols shall not be mixed in one primary result.

## CAL-002 — Probability Calibration

Raw language-model outputs, token likelihoods, and self-reported confidence shall not be treated as calibrated event probabilities.

The calibrated probability shall be produced by a separately versioned calibration artifact trained only on temporally eligible resolved events. If minimum sample or data-quality requirements are not met, the system shall abstain or use the preregistered fallback.

## CAL-003 — Model Temporal Eligibility

For a causal historical forecast, the model artifact shall have a documented training-data cutoff strictly preceding the forecast's publication cutoff.

A model without a verifiable knowledge boundary shall be excluded from causal historical performance claims. It may be evaluated only prospectively or in a separately labeled non-causal extraction experiment.

## CAL-004 — Dual Baseline Protocol

The system shall report skill against two baselines.

1. **Information baseline**
   \[
   p_{information}=M(T_{published}^{-})
   \]

2. **Post-latency baseline**
   \[
   p_{post-latency}=M(t_{arrival}^{-})
   \]

The baseline function \(M(t)\) and its staleness, spread, and depth requirements shall be preregistered.

Statuses shall include:

- `ValidTwoSided`;
- `OneSided`;
- `Stale`;
- `Fragmented`;
- `Missing`;
- `SpreadTooWide`.

Fallback or exclusion behavior shall be deterministic and preregistered.

---

# 9. Causal Matching and Ledger Simulation

## TIM-002 — Arrival-State Reconstruction

The matcher shall evaluate an intent against the canonical state immediately before arrival:

\[
B(t_{arrival}^{-})
\]

All exogenous events ordered before the intent shall be applied first. For identical timestamps:

1. documented upstream sequence shall control when available;
2. otherwise exogenous market events shall precede the simulated intent;
3. remaining ties shall use a deterministic canonical key.

A snapshot observed after arrival shall never be used retroactively.

## SIM-001 — Immediate Execution Matching

An immediate all-or-none intent shall:

- traverse all eligible price levels up to the price limit;
- verify total available quantity;
- reject when full quantity is unavailable;
- compute exact weighted notional;
- evaluate costs separately;
- reserve cash or inventory atomically;
- consume shared virtual depth only after all checks pass.

Partial execution shall not occur for an all-or-none intent.

## SIM-002 — Passive Queue Lifecycle

A passive intent shall enter the queue only at simulated acknowledgement time.

Initial quantity ahead shall equal observable resting quantity at that price before insertion. Under the conservative model:

- only confirmed aggressive trade volume shall reduce quantity ahead;
- unclassified size reductions shall not improve queue position;
- price traversal without matching evidence shall not imply a fill;
- partial fills shall be represented explicitly;
- cancellation becomes effective only after configured latency;
- unidentifiable queue progression shall produce `Unobservable` or bounded results rather than an exact fill.

## SIM-003 — Shared Virtual Matching State

All strategies and policies shall submit intents to one shared matching adapter.

Observable depth, cash, reserved cash, inventory, and margin shall not be consumed more than once.

After every transition, the engine shall verify:

\[
FreeCash+ReservedCash=TotalCash
\]

\[
Position_{after}=Position_{before}+SignedFillQuantity
\]

\[
CombinedFilledQuantity\le AvailableVirtualQuantity
\]

## SIM-004 — Concurrent Intent Tie-Breaker

Intents with identical arrival timestamps shall be ordered using:

\[
OrderKey=(t_{arrival},policyPriority,simulationIntentId)
\]

All keys and priorities shall be preregistered and versioned.

## SIM-005 — Settlement and Ledger Finalization

Resolution status and terminal outcome shall be separate.

**ResolutionStatus:**
- `Open`;
- `Proposed`;
- `Disputed`;
- `PendingFinality`;
- `Final`.

**TerminalOutcome:**
- `Yes`;
- `No`;
- `Void`;
- `Cancelled`;
- `Invalid`;
- `DefinitionChanged`.

Primary binary forecasting scores shall be generated only for final, eligible `Yes` or `No` outcomes. All other terminal outcomes shall follow a preregistered ledger and scoring policy.

## SIM-006 — Counterfactual Limitation

The simulator assumes hypothetical orders do not alter subsequent participant behavior unless an independently versioned impact model is enabled.

Results shall be segmented by:

\[
ParticipationRatio=\frac{Q_{simulated}}{Q_{visible}}
\]

Participation-ratio thresholds shall be preregistered.

---

# 10. Evaluation and Telemetry

## MET-001 — Primary Forecast Evaluation

The primary report shall include:

- Brier Score;
- clipped Log Loss;
- skill against information baseline;
- skill against post-latency baseline;
- calibration diagnostics;
- coverage;
- abstention rate.

The report shall specify the Log Loss clipping constant, calibration method, and weighting rule.

## MET-002 — Forecast Selection Rule

For each:

\[
underlyingEventGroupId\times ForecastHorizon
\]

the primary forecast shall be selected using one preregistered deterministic rule, such as:

- first eligible forecast after horizon boundary;
- last eligible forecast before cutoff;
- predefined aggregation of all eligible forecasts.

Selection shall not use eventual score or simulated result.

## MET-003 — Statistical Uncertainty

Metric uncertainty shall be estimated at the `underlying_event_group_id` level using a preregistered clustered method.

Configuration shall specify:

- confidence level;
- resampling or inference method;
- number of resamples;
- random seed;
- numerical-library version;
- metric tolerance.

## MET-004 — Simulation Performance

Against independently calculated reference ledgers, the system shall report:

- gross and net simulated P&L;
- costs;
- realized and unrealized P&L;
- fill, partial-fill, rejection, and unobservable rates;
- slippage;
- realized spread;
- post-fill markout;
- inventory exposure;
- maximum drawdown;
- results by latency scenario;
- results by participation-ratio segment.

## MET-005 — Abstention Reporting

The system shall report both:

1. **Covered-subset score**, with coverage;
2. **All-event score**, substituting the preregistered baseline when the model abstains.

## TEL-002 — NAV Edge-Case Handling

The system shall calculate:

- `NAV_mid`;
- `NAV_conservative`.

Valuation status shall be:

- `Valued`;
- `PartiallyValued`;
- `Unpriceable`;
- `Stale`;
- `Fragmented`.

No unavailable or insufficiently liquid position shall receive a silent optimistic valuation.

---

# 11. Robustness, Replay, and Documentation

## REP-001 — Deterministic Offline Replay

A valid replay shall:

- use recorded model and calibration artifacts;
- make no external call;
- canonicalize unordered collections before hashing;
- record input trace, configuration, software, container, model, calibration, and schema hashes;
- separate deterministic research artifacts from runtime telemetry.

Given identical inputs and supported execution environment, canonical ledgers, selected forecasts, dispositions, and final state hashes shall be identical.

## REP-002 — Replay Network Isolation

An attempted prohibited external call during replay shall:

- abort the replay;
- durably record the violation;
- mark the run invalid;
- publish no valid final research artifact.

## ROB-001 — Adversarial Input Robustness

The system shall execute a versioned adversarial corpus containing:

- schema violations;
- oversized content;
- prompt injection;
- fake quotations;
- contradictory sources;
- satire;
- delayed corrections;
- machine-generated rumors.

It shall report:

- parser survival;
- schema rejection rate;
- model direction-change rate;
- abstention behavior;
- calibration degradation;
- invalid-output rate.

Untrusted model output shall never be executed as code or control logic.

## DOC-001 — Requirement Integrity

Every normative requirement ID shall be unique and defined in the main body. Every verification ID shall be unique. Every normative requirement shall map to at least one verification artifact or an explicit, reviewed justification.

The document validator shall reject:

- undefined IDs;
- duplicate IDs;
- mismatched requirement names;
- invisible formatting characters in controlled identifiers;
- unverified normative requirements.

---

# 12. Verification Matrix

The canonical verification matrix is delivered as a separate CSV file. A requirement may have multiple verification rows, each with a unique `Verif ID`.

---

# 13. Implementation Backlog

## EPIC-01 — Architecture and Build Boundaries
**Requirements:** ARC-001, ARC-002, SEC-001 through SEC-004, PLT-001  
**Exit condition:** Process graph, dependency rules, and network policies pass CI.

## EPIC-02 — Local Mock Demo Gateway
**Requirements:** DEM-001 through DEM-006  
**Exit condition:** Deterministic local lifecycle scenarios and trace replay pass.

## EPIC-03 — Time, Lineage, and Market State
**Requirements:** CLK-001, DAT-001, TIM-001, TEL-001, STA-001 through STA-003  
**Exit condition:** Logical-time replay, bitemporal lineage, gap recovery, and snapshot invariants pass.

## EPIC-04 — Domain Types and Cost Accounting
**Requirements:** TYP-001 through TYP-003, COST-001  
**Exit condition:** Property tests, overflow behavior, and reference cost ledgers pass.

## EPIC-05 — IPC and Recovery
**Requirements:** IPC-001 through IPC-005, AUD-001 through AUD-004  
**Exit condition:** Framing, authentication, replay, crash injection, and spool recovery pass.

## EPIC-06 — Forecast and Experiment Control
**Requirements:** FCP-001, FCP-002, EXP-001, EXP-002, CAL-001 through CAL-004  
**Exit condition:** Preregistration, temporal eligibility, calibration, and holdout controls pass.

## EPIC-07 — Matching and Settlement
**Requirements:** TIM-002, SIM-001 through SIM-006  
**Exit condition:** Reference-order-book fixtures, shared liquidity, queue bounds, and settlement pass.

## EPIC-08 — Metrics and Research Reporting
**Requirements:** MET-001 through MET-005, TEL-002  
**Exit condition:** Forecast, abstention, simulation, uncertainty, and NAV fixtures reproduce reference values.

## EPIC-09 — Reproducibility and Robustness
**Requirements:** REP-001, REP-002, ROB-001, DOC-001  
**Exit condition:** Deterministic replay, attack corpus, and bidirectional traceability pass.

---

# 14. Final Release Gate

The system may be labeled **Research-Ready** only when:

1. every normative requirement has at least one passing verification artifact;
2. no `Blocking` or `Failing` verification remains;
3. the exact experiment manifest is frozen;
4. the holdout-access log shows no unregistered access;
5. replay artifacts are deterministic;
6. crash recovery produces exactly one terminal ledger transition;
7. all demo integration remains local and non-monetary;
8. the report clearly separates forecast skill, simulated performance, and integration evidence.

A positive forecast or P&L result is not required for software acceptance.

---

## Appendix A — Traceability Summary

- Normative requirements: **58**
- Verification artifacts: **62**
- Requirements without a verification row: **0**
- Verification rows without a normative requirement: **0**
- Duplicate verification IDs: **0**

The machine-readable verification matrix is stored in `verification_matrix_v1_2.csv`.
