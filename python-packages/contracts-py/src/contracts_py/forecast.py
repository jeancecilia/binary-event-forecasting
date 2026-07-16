"""Forecast message Pydantic model (IPC-003)."""

from __future__ import annotations

from datetime import datetime
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, model_validator

SCHEMA_VERSION: Literal[1] = 1
PROBABILITY_SCALE: int = 1_000_000


class ForecastMessage(BaseModel):
    """A forecast message from the Python intelligence plane.

    Uses strict Pydantic configuration: extra fields forbidden,
    strict types, and immutable after creation.
    """

    model_config = ConfigDict(
        extra="forbid",
        strict=True,
        frozen=True,
    )

    # Protocol identity
    schema_version: Literal[1] = SCHEMA_VERSION
    message_id: str
    sender_instance_id: str
    sender_sequence: int = Field(ge=0)

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
    evidence_set_hash: str = Field(pattern=r"^[a-f0-9]{64}$")
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
    abstention_reason: str | None = None

    # Lifecycle timestamps
    decision_cutoff_at: datetime
    forecast_created_at: datetime
    forecast_emitted_at: datetime
    expires_at: datetime

    @model_validator(mode="after")
    def validate_probability_bounds(self) -> ForecastMessage:
        """Validate the probability invariant automatically."""
        if not (
            0
            <= self.uncertainty_lower
            <= self.calibrated_probability
            <= self.uncertainty_upper
            <= PROBABILITY_SCALE
        ):
            raise ValueError(
                f"Probability invariant violated: "
                f"0 <= {self.uncertainty_lower} <= {self.calibrated_probability} "
                f"<= {self.uncertainty_upper} <= {PROBABILITY_SCALE}"
            )
        return self

    @model_validator(mode="after")
    def validate_timestamp_order(self) -> ForecastMessage:
        """Validate causal timestamp ordering."""
        if self.first_source_available_at > self.decision_cutoff_at:
            raise ValueError(
                f"first_source_available_at ({self.first_source_available_at}) "
                f"must be <= decision_cutoff_at ({self.decision_cutoff_at})"
            )
        if self.decision_cutoff_at >= self.forecast_created_at:
            raise ValueError(
                f"decision_cutoff_at ({self.decision_cutoff_at}) "
                f"must be < forecast_created_at ({self.forecast_created_at})"
            )
        if self.forecast_created_at >= self.forecast_emitted_at:
            raise ValueError(
                f"forecast_created_at ({self.forecast_created_at}) "
                f"must be < forecast_emitted_at ({self.forecast_emitted_at})"
            )
        if self.forecast_emitted_at >= self.expires_at:
            raise ValueError(
                f"forecast_emitted_at ({self.forecast_emitted_at}) "
                f"must be < expires_at ({self.expires_at})"
            )
        if self.model_training_cutoff >= self.decision_cutoff_at:
            raise ValueError(
                f"model_training_cutoff ({self.model_training_cutoff}) "
                f"must be < decision_cutoff_at ({self.decision_cutoff_at})"
            )
        if self.calibration_training_cutoff >= self.decision_cutoff_at:
            raise ValueError(
                f"calibration_training_cutoff ({self.calibration_training_cutoff}) "
                f"must be < decision_cutoff_at ({self.decision_cutoff_at})"
            )
        return self
