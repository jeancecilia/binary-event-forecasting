"""Lifecycle disposition Pydantic model."""

from __future__ import annotations

from datetime import datetime

from pydantic import BaseModel

from .enums import DispositionStatus


class LifecycleDisposition(BaseModel):
    """Terminal disposition of a forecast message."""

    schema_version: int
    message_id: str
    disposition_status: DispositionStatus
    timestamp: datetime
    transition_id: str
    detail: str | None = None
    previous_status: DispositionStatus | None = None
