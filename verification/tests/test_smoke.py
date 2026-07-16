"""Smoke test to ensure the pytest test runner functions correctly."""

from datetime import datetime, timedelta, timezone

from contracts_py.forecast import SCHEMA_VERSION, ForecastMessage


def test_forecast_message_instantiation():
    """Verify that a minimal ForecastMessage can be instantiated."""
    now = datetime.now(timezone.utc)
    t_train = now - timedelta(hours=2)
    t_decision = now - timedelta(hours=1)
    t_created = now

    msg = ForecastMessage(
        schema_version=SCHEMA_VERSION,
        message_id="123e4567-e89b-12d3-a456-426614174000",
        sender_instance_id="instance_1",
        sender_sequence=1,
        market_id="mkt_1",
        contract_or_outcome_id="out_1",
        market_definition_version="v1",
        event_id="evt_1",
        underlying_event_group_id="grp_1",
        forecast_target="target_1",
        forecast_horizon="horizon_1",
        source_id="src_1",
        source_version="v1",
        evidence_set_hash="a" * 64,
        published_at=t_train,
        first_source_available_at=t_train,
        ingested_at=t_train,
        revision_id="rev_1",
        model_artifact_hash="hash_1",
        model_training_cutoff=t_train,
        ensemble_version="v1",
        component_model_versions={"comp": "v1"},
        prompt_version="v1",
        retrieval_corpus_version="v1",
        calibration_model_version="v1",
        calibration_training_cutoff=t_train,
        raw_model_probability=500_000,
        calibrated_probability=500_000,
        uncertainty_lower=400_000,
        uncertainty_upper=600_000,
        uncertainty_coverage_level=0.95,
        uncertainty_method="method",
        decision_cutoff_at=t_decision,
        forecast_created_at=t_created,
        forecast_emitted_at=t_created + timedelta(seconds=1),
        expires_at=t_created + timedelta(hours=1),
    )
    
    assert msg.message_id == "123e4567-e89b-12d3-a456-426614174000"
    assert msg.schema_version == SCHEMA_VERSION
