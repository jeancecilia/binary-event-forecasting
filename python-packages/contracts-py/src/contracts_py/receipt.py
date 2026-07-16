"""Receipt acknowledgement Pydantic model."""

from __future__ import annotations

from datetime import datetime
from typing import Optional
from pydantic import BaseModel

from .enums import ReceiptStatus


class ReceiptAcknowledgement(BaseModel):
    """Acknowledgement of forecast message receipt."""

    schema_version: int
    message_id: str
    receipt_status: ReceiptStatus
    timestamp: datetime
    receipt_id: str
    detail: Optional[str] = None
