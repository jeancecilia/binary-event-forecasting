"""Forecast message Pydantic model (IPC-003)."""

from __future__ import annotations

from datetime import datetime
from typing import Any, Optional
from pydantic import BaseModel, Field


SCHEMA_VERSION: int = 1
PROBABILITY_SCALE: int = 1_000_000


class ForecastMessage(BaseModel):
    """A forecast message from the Python intelligence plane."""

    # Protocol identity
    schema_version: int = Field(default=SCHEMA_VERSION)
    message_id: str
    sender_instance_id: str
    sender_sequence: int

    # Target identity
    market_id: str
    contract_or_outcome_id: str
    market_definition_version: str
    event_id: str
    underlying_event_group_id: str
    forecast_target: str
    forecast_horizon: str

    # Source provenance
    source_id: str
    source_version: str
    evidence_set_hash: str
    published_at: datetime
    first_source_available_at: datetime
    ingested_at: datetime
    revision_id: str

    # Model provenance
    model_artifact_hash: str
    model_training_cutoff: datetime
    ensemble_version: str
    component_model_versions: dict[str, str] = Field(default_factory=dict)
    prompt_version: str
    retrieval_corpus_version: str
    calibration_model_version: str
    calibration_training_cutoff: datetime

    # Forecast values (scaled integers)
    raw_model_probability: int = Field(ge=0, le=PROBABILITY_SCALE)
    calibrated_probability: int = Field(ge=0, le=PROBABILITY_SCALE)
    uncertainty_lower: int = Field(ge=0, le=PROBABILITY_SCALE)
    uncertainty_upper: int = Field(ge=0, le=PROBABILITY_SCALE)
    uncertainty_coverage_level: float = Field(ge=0.0, le=1.0)
    uncertainty_method: str
    abstention_reason: Optional[str] = None

    # Lifecycle timestamps
    decision_cutoff_at: datetime
    forecast_created_at: datetime
    forecast_emitted_at: datetime
    expires_at: datetime

    def validate_probability_bounds(self) -> None:
        """Validate the probability invariant."""
        if not (0 <= self.uncertainty_lower <= self.calibrated_probability <= self.uncertainty_upper <= PROBABILITY_SCALE):
            raise ValueError(
                f"Probability invariant violated: "
                f"0 <= {self.uncertainty_lower} <= {self.calibrated_probability} "
                f"<= {self.uncertainty_upper} <= {PROBABILITY_SCALE}"
            )
