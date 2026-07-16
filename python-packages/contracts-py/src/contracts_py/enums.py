"""Enumerations for the contracts definitions (IPC-003, FCP-002, etc)."""

from enum import StrEnum


class ReceiptStatus(StrEnum):
    ACCEPTED_QUEUED = "AcceptedQueued"
    DUPLICATE_RETRY = "DuplicateRetry"
    VALIDATION_FAILED = "ValidationFailed"
    AUTHENTICATION_FAILED = "AuthenticationFailed"
    AUTHORIZATION_FAILED = "AuthorizationFailed"
    CAPACITY_EXCEEDED = "CapacityExceeded"
    INTERNAL_ERROR = "InternalError"
    PARSING_ERROR = "ParsingError"


class DispositionStatus(StrEnum):
    VALIDATED = "Validated"
    EVALUATED = "Evaluated"
    MATCHED = "Matched"
    SETTLED = "Settled"
    EXPIRED = "Expired"
    CANCELLED = "Cancelled"
    REJECTED_COST = "RejectedCost"
    REJECTED_HORIZON = "RejectedHorizon"
    REJECTED_STATE = "RejectedState"


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
