//! Local HTTP server for the mock gateway.
//!
//! Exposes a versioned local interface for testing forecast-message receipt,
//! simulation-intent submission, and lifecycle event retrieval.

use axum::{routing::get, Router};

/// Start the mock gateway HTTP server.
pub async fn run(bind_address: &str) -> anyhow::Result<()> {
    tracing::info!("Mock gateway starting on {bind_address}");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/v1/forecast", axum::routing::post(submit_forecast))
        .route("/v1/intent", axum::routing::post(submit_intent));

    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    tracing::info!("Mock gateway listening on {bind_address}");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "LOCAL_MOCK_DEMO"
}

async fn submit_forecast() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "AcceptedQueued",
        "environment": "LOCAL_MOCK_DEMO"
    }))
}

async fn submit_intent() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "SimulationSubmitted",
        "environment": "LOCAL_MOCK_DEMO"
    }))
}
