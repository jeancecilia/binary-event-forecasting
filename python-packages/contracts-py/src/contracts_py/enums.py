"""Closed enums matching the Rust protocol types."""

from enum import Enum


class ReceiptStatus(str, Enum):
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


class DispositionStatus(str, Enum):
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


class BookSide(str, Enum):
    BID = "Bid"
    ASK = "Ask"


class OutcomeSide(str, Enum):
    YES = "Yes"
    NO = "No"


class OrderClass(str, Enum):
    IMMEDIATE_ALL_OR_NONE = "ImmediateAllOrNone"
    PASSIVE = "Passive"


class TimeInForce(str, Enum):
    IMMEDIATE_OR_CANCEL = "ImmediateOrCancel"
    GOOD_TILL_CANCELLED = "GoodTillCancelled"
    FILL_OR_KILL = "FillOrKill"
    DAY = "Day"


class FeedStatus(str, Enum):
    INITIALIZING = "Initializing"
    FRAGMENTED = "Fragmented"
    DISCONNECTED = "Disconnected"
    STALE = "Stale"
    FAILED = "Failed"


class ResolutionStatus(str, Enum):
    OPEN = "Open"
    PROPOSED = "Proposed"
    DISPUTED = "Disputed"
    PENDING_FINALITY = "PendingFinality"
    FINAL = "Final"


class TerminalOutcome(str, Enum):
    YES = "Yes"
    NO = "No"
    VOID = "Void"
    CANCELLED = "Cancelled"
    INVALID = "Invalid"
    DEFINITION_CHANGED = "DefinitionChanged"
