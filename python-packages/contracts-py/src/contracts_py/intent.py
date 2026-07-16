"""Simulation intent Pydantic model (FCP-002)."""

from __future__ import annotations

from datetime import datetime

from pydantic import BaseModel

from .enums import BookSide, OrderClass, OutcomeSide, TimeInForce


class SimulationIntent(BaseModel):
    """An immutable simulation intent derived from a forecast message."""

    simulation_intent_id: str
    experiment_id: str
    source_forecast_message_id: str
    forecast_policy_version: str
    configuration_hash: str
    market_id: str
    contract_or_outcome_id: str
    forecast_target: str
    order_class: OrderClass
    book_side: BookSide
    outcome_side: OutcomeSide
    quantity: int  # scaled integer
    price_limit: int  # scaled integer
    time_in_force: TimeInForce
    policy_priority: int
    decision_timestamp: datetime
    simulated_arrival_timestamp: datetime
    latency_scenario_version: str
    matching_model_version: str
    cost_model_version: str
    acknowledgement_latency_version: str
    cancellation_latency_version: str
    account_state_version: str
    input_snapshot_version: str
    expires_at: datetime
