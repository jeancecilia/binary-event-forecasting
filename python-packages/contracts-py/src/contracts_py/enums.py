"""Enumerations for the contracts definitions (IPC-003, FCP-002, etc)."""

from enum import StrEnum


class ReceiptStatus(StrEnum):
    ACCEPTED_QUEUED = "AcceptedQueued"
    DUPLICATE_RETRY = "DuplicateRetry"
    EXPIRED_ON_ARRIVAL = "ExpiredOnArrival"
    REJECTED_SCHEMA = "RejectedSchema"
    REJECTED_BOUNDS = "RejectedBounds"
    REJECTED_CAPACITY = "RejectedCapacity"
    REJECTED_TARGET_VERSION = "RejectedTargetVersion"
    REJECTED_RATE_LIMIT = "RejectedRateLimit"
    REPLAY_SEQUENCE_VIOLATION = "ReplaySequenceViolation"
    CORE_DEGRADED = "CoreDegraded"


class DispositionStatus(StrEnum):
    VALIDATED = "Validated"
    EVALUATED = "Evaluated"
    ABSTAINED = "Abstained"
    SIMULATION_SUBMITTED = "SimulationSubmitted"
    SIMULATED = "Simulated"
    PARTIALLY_FILLED = "PartiallyFilled"
    SIMULATION_REJECTED = "SimulationRejected"
    SIMULATION_FAILED = "SimulationFailed"
    SUPERSEDED = "Superseded"
    EVICTED = "Evicted"
    EXPIRED_IN_QUEUE = "ExpiredInQueue"


class BookSide(StrEnum):
    BID = "Bid"
    ASK = "Ask"


class OutcomeSide(StrEnum):
    YES = "Yes"
    NO = "No"


class OrderClass(StrEnum):
    IMMEDIATE_ALL_OR_NONE = "ImmediateAllOrNone"
    PASSIVE = "Passive"


class TimeInForce(StrEnum):
    IMMEDIATE_OR_CANCEL = "ImmediateOrCancel"
    GOOD_TILL_CANCELLED = "GoodTillCancelled"
    GOOD_TILL_DATE = "GoodTillDate"


class FeedStatus(StrEnum):
    INITIALIZING = "Initializing"
    SYNCHRONIZED = "Synchronized"
    FRAGMENTED = "Fragmented"
    STALE = "Stale"
    HALTED = "Halted"


class ResolutionStatus(StrEnum):
    OPEN = "Open"
    PROPOSED = "Proposed"
    DISPUTED = "Disputed"
    RESOLVED = "Resolved"
    CANCELLED = "Cancelled"


class TerminalOutcome(StrEnum):
    YES = "Yes"
    NO = "No"
    VOID = "Void"
    CANCELLED = "Cancelled"
    INVALID = "Invalid"
