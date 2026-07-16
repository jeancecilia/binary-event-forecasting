//! Forecast-to-Simulation Policy (FCP-001, FCP-002)
//!
//! Transforms a forecast message into a deterministic simulation intent.
//! The forecast message itself does NOT constitute a simulation intent.
//! The transformation uses a versioned, deterministic policy.

use domain_types::{Price, Quantity, ProbabilityScaled};
use protocol::{ForecastMessage, SimulationIntent, enums::*};

/// Configuration for the forecast-to-intent policy.
#[derive(Debug, Clone)]
pub struct ForecastPolicyConfig {
    /// Policy version
    pub version: String,
    /// Configuration hash
    pub config_hash: String,
    /// Probability threshold for generating intents
    pub probability_threshold: ProbabilityScaled,
    /// Sizing rule configuration
    pub sizing_rule: SizingRule,
    /// Whether to abstain when uncertainty is too wide
    pub abstention_threshold: Option<ProbabilityScaled>,
}

/// Sizing rule for translating probability to quantity.
#[derive(Debug, Clone)]
pub enum SizingRule {
    /// Fixed quantity regardless of probability
    FixedQuantity(Quantity),
    /// Quantity proportional to edge over threshold
    EdgeProportional { max_quantity: Quantity, scale: f64 },
}

/// Result of applying the forecast policy.
#[derive(Debug, Clone)]
pub enum PolicyResult {
    /// Generate this simulation intent
    Intent(SimulationIntent),
    /// Abstain from trading
    Abstain { reason: String },
}

/// Apply the forecast-to-intent policy.
pub fn apply_policy(
    forecast: &ForecastMessage,
    config: &ForecastPolicyConfig,
    probability_scale: u64,
) -> Result<PolicyResult, String> {
    let calibrated = ProbabilityScaled::new(
        forecast.calibrated_probability,
        probability_scale,
    )
    .map_err(|e| format!("Invalid probability: {e}"))?;

    // Check probability bounds invariant
    forecast.validate_probability_bounds(probability_scale)?;

    // Check abstention
    if let Some(ref reason) = forecast.abstention_reason {
        return Ok(PolicyResult::Abstain {
            reason: reason.clone(),
        });
    }

    // Check uncertainty width if configured
    if let Some(threshold) = config.abstention_threshold {
        let uncertainty_width = forecast.uncertainty_upper - forecast.uncertainty_lower;
        if uncertainty_width > threshold.as_raw() {
            return Ok(PolicyResult::Abstain {
                reason: format!(
                    "Uncertainty width {uncertainty_width} exceeds threshold {threshold}",
                    threshold = threshold.as_raw()
                ),
            });
        }
    }

    // Determine book side and outcome side from probability
    let (book_side, outcome_side, price_limit_raw) = if calibrated.as_raw()
        > config.probability_threshold.as_raw()
    {
        // High probability → buy YES
        (BookSide::Bid, OutcomeSide::Yes, calibrated.as_raw())
    } else if calibrated.as_raw() < probability_scale - config.probability_threshold.as_raw() {
        // Low probability → buy NO (sell YES)
        (BookSide::Ask, OutcomeSide::No, probability_scale - calibrated.as_raw())
    } else {
        return Ok(PolicyResult::Abstain {
            reason: "Probability within no-trade zone".to_string(),
        });
    };

    let quantity = match &config.sizing_rule {
        SizingRule::FixedQuantity(q) => *q,
        SizingRule::EdgeProportional { max_quantity, scale } => {
            let edge = (calibrated.as_raw() as f64
                - config.probability_threshold.as_raw() as f64)
                .abs();
            let proportion = (edge * scale).min(max_quantity.as_raw() as f64) as u64;
            Quantity::from_raw(proportion)
        }
    };

    let now = chrono::Utc::now();
    let intent_id = compute_intent_id(forecast, config)?;

    Ok(PolicyResult::Intent(SimulationIntent {
        simulation_intent_id: intent_id,
        experiment_id: "default".to_string(), // TODO: from config
        source_forecast_message_id: forecast.message_id.clone(),
        forecast_policy_version: config.version.clone(),
        configuration_hash: config.config_hash.clone(),
        market_id: forecast.market_id.clone(),
        contract_or_outcome_id: forecast.contract_or_outcome_id.clone(),
        forecast_target: forecast.forecast_target.clone(),
        order_class: OrderClass::ImmediateAllOrNone,
        book_side,
        outcome_side,
        quantity: quantity.as_raw(),
        price_limit: price_limit_raw,
        time_in_force: TimeInForce::ImmediateOrCancel,
        policy_priority: 0,
        decision_timestamp: now,
        simulated_arrival_timestamp: now,
        latency_scenario_version: "v1".to_string(),
        matching_model_version: "v1".to_string(),
        cost_model_version: "v1".to_string(),
        acknowledgement_latency_version: "v1".to_string(),
        cancellation_latency_version: "v1".to_string(),
        account_state_version: "v1".to_string(),
        input_snapshot_version: "v1".to_string(),
        expires_at: forecast.expires_at,
    }))
}

/// Compute a deterministic simulation intent ID from canonical inputs.
fn compute_intent_id(
    forecast: &ForecastMessage,
    config: &ForecastPolicyConfig,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(forecast.message_id.as_bytes());
    hasher.update(config.version.as_bytes());
    hasher.update(config.config_hash.as_bytes());
    hasher.update(forecast.calibrated_probability.to_be_bytes());
    let hash = hex::encode(hasher.finalize());
    Ok(format!("intent-{hash}"))
}
