"""Smoke test to ensure the pytest test runner functions correctly."""

from contracts_py.forecast import ForecastMessage, SCHEMA_VERSION
from datetime import datetime, timezone

def test_forecast_message_instantiation():
    """Verify that a minimal ForecastMessage can be instantiated."""
    now = datetime.now(timezone.utc)
    
    msg = ForecastMessage(
        schema_version=SCHEMA_VERSION,
        message_id="test_msg_1",
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
        published_at=now,
        first_source_available_at=now,
        ingested_at=now,
        revision_id="rev_1",
        model_artifact_hash="hash_1",
        model_training_cutoff=now,
        ensemble_version="v1",
        component_model_versions={"comp": "v1"},
        prompt_version="v1",
        retrieval_corpus_version="v1",
        calibration_model_version="v1",
        calibration_training_cutoff=now,
        raw_model_probability=500_000,
        calibrated_probability=500_000,
        uncertainty_lower=400_000,
        uncertainty_upper=600_000,
        uncertainty_coverage_level=0.95,
        uncertainty_method="method",
        decision_cutoff_at=now,
        forecast_created_at=now,
        forecast_emitted_at=now,
        expires_at=now,
    )
    
    assert msg.message_id == "test_msg_1"
    assert msg.schema_version == SCHEMA_VERSION
