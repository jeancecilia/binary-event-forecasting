// Allow float arithmetic for EdgeProportional sizing rule in research policy.
// Float is prohibited in domain-types, matching, ledger, and market-state accounting.
#![allow(clippy::float_arithmetic)]

//! Forecast-to-Simulation Policy (FCP-001, FCP-002)
//!
//! Transforms a forecast message into a deterministic simulation intent.
//! The forecast message itself does NOT constitute a simulation intent.
//! The transformation uses a versioned, deterministic policy.
//!
//! ## Determinism
//!
//! Timestamps come from `PolicyContext`, never from `Utc::now()`.
//! Identical inputs produce bit-identical intents.

use chrono::{DateTime, Utc};
use domain_types::{ProbabilityScaled, Quantity};
use protocol::{enums::*, ForecastMessage, SimulationIntent};

/// Context provided by the caller (logical clock or latency model).
/// The policy must never create timestamps using `Utc::now()`.
#[derive(Debug, Clone)]
pub struct PolicyContext {
    /// When the policy decision was made (logical time)
    pub decision_timestamp: DateTime<Utc>,
    /// Simulated arrival at the matcher (logical time)
    pub simulated_arrival_timestamp: DateTime<Utc>,
    /// Experiment identifier
    pub experiment_id: String,
    /// Input snapshot version
    pub input_snapshot_version: String,
    /// Account state version
    pub account_state_version: String,
    /// Latency scenario version
    pub latency_scenario_version: String,
    /// Matching model version
    pub matching_model_version: String,
    /// Cost model version
    pub cost_model_version: String,
    /// Acknowledgement latency model version
    pub acknowledgement_latency_version: String,
    /// Cancellation latency model version
    pub cancellation_latency_version: String,
}

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
#[allow(clippy::large_enum_variant)]
pub enum PolicyResult {
    /// Generate this simulation intent
    Intent(SimulationIntent),
    /// Abstain from trading
    Abstain { reason: String },
}

/// Apply the forecast-to-intent policy.
///
/// All timestamps come from `ctx`, ensuring deterministic output
/// given identical forecast, config, and context.
pub fn apply_policy(
    forecast: &ForecastMessage,
    config: &ForecastPolicyConfig,
    ctx: &PolicyContext,
    probability_scale: u64,
) -> Result<PolicyResult, String> {
    let calibrated = ProbabilityScaled::new(forecast.calibrated_probability, probability_scale)
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
                    "Uncertainty width {uncertainty_width} exceeds threshold {}",
                    threshold.as_raw()
                ),
            });
        }
    }

    // Determine book side and outcome side from probability
    let (book_side, outcome_side, price_limit_raw) =
        if calibrated.as_raw() > config.probability_threshold.as_raw() {
            // High probability → buy YES
            let price_raw = (calibrated.as_raw() as u128 * domain_types::PRICE_SCALE as u128
                / probability_scale as u128) as u64;
            (BookSide::Bid, OutcomeSide::Yes, price_raw)
        } else if calibrated.as_raw() < probability_scale - config.probability_threshold.as_raw() {
            // Low probability → buy NO (sell YES)
            let price_raw = ((probability_scale - calibrated.as_raw()) as u128
                * domain_types::PRICE_SCALE as u128
                / probability_scale as u128) as u64;
            (BookSide::Ask, OutcomeSide::No, price_raw)
        } else {
            return Ok(PolicyResult::Abstain {
                reason: "Probability within no-trade zone".to_string(),
            });
        };

    let quantity = match &config.sizing_rule {
        SizingRule::FixedQuantity(q) => *q,
        SizingRule::EdgeProportional {
            max_quantity,
            scale,
        } => {
            let edge =
                (calibrated.as_raw() as f64 - config.probability_threshold.as_raw() as f64).abs();
            let proportion = (edge * scale).min(max_quantity.as_raw() as f64) as u64;
            Quantity::from_raw(proportion)
        }
    };

    let intent_id = compute_intent_id(forecast, config, ctx)?;

    Ok(PolicyResult::Intent(SimulationIntent {
        simulation_intent_id: intent_id,
        experiment_id: ctx.experiment_id.clone(),
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
        decision_timestamp: ctx.decision_timestamp,
        simulated_arrival_timestamp: ctx.simulated_arrival_timestamp,
        latency_scenario_version: ctx.latency_scenario_version.clone(),
        matching_model_version: ctx.matching_model_version.clone(),
        cost_model_version: ctx.cost_model_version.clone(),
        acknowledgement_latency_version: ctx.acknowledgement_latency_version.clone(),
        cancellation_latency_version: ctx.cancellation_latency_version.clone(),
        account_state_version: ctx.account_state_version.clone(),
        input_snapshot_version: ctx.input_snapshot_version.clone(),
        expires_at: forecast.expires_at,
    }))
}

/// Compute a deterministic simulation intent ID from canonical inputs.
/// Includes all context timestamps to ensure true determinism.
fn compute_intent_id(
    forecast: &ForecastMessage,
    config: &ForecastPolicyConfig,
    ctx: &PolicyContext,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(forecast.message_id.as_bytes());
    hasher.update(config.version.as_bytes());
    hasher.update(config.config_hash.as_bytes());
    hasher.update(forecast.calibrated_probability.to_be_bytes());
    // Include context to make intent ID fully deterministic
    hasher.update(ctx.decision_timestamp.to_rfc3339().as_bytes());
    hasher.update(ctx.simulated_arrival_timestamp.to_rfc3339().as_bytes());
    hasher.update(ctx.experiment_id.as_bytes());
    let hash = hex::encode(hasher.finalize());
    Ok(format!("intent-{hash}"))
}
