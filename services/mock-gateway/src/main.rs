//! Binary Event Forecasting — Local Mock Demo Gateway
//!
//! Entry point for the mock gateway binary.

use clap::Parser;
use std::net::ToSocketAddrs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "mock-gateway")]
#[command(about = "Binary Event Forecasting — Local Mock Demo Gateway")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/mock-gateway.toml")]
    config: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,mock_gateway=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let cli = Cli::parse();

    let config_content = std::fs::read_to_string(&cli.config)?;
    let toml_config: mock_gateway::config::TomlConfig = toml::from_str(&config_content)?;
    let mut config = toml_config.into_mock_config();

    // Compute the real configuration hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&config_content);
    config.config_hash = hex::encode(hasher.finalize());

    // Validate environment
    config.validate_environment()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    tracing::info!(
        environment = %config.environment,
        scenario_id = %config.scenario_id,
        bind = %config.bind_address,
        config_hash = %config.config_hash,
        "Mock gateway initializing"
    );

    // Enforce local-only binding using proper socket address parsing
    let bind_addr = config.bind_address.clone();
    validate_local_bind(&bind_addr)?;

    mock_gateway::server::run(&config.bind_address).await?;

    Ok(())
}

/// Validate that the bind address is strictly local.
///
/// Accepts only loopback addresses (127.0.0.1, ::1) or localhost.
/// Rejects 0.0.0.0, external IPs, and any address that resolves externally.
fn validate_local_bind(address: &str) -> anyhow::Result<()> {
    // Handle AF_UNIX paths
    if address.starts_with('/') || address.starts_with("./") || address.starts_with("../") {
        return Ok(());
    }

    // Parse as socket address
    let addrs: Vec<_> = match address.to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(_) => {
            // If we can't parse it and it's not a Unix path, reject
            anyhow::bail!(
                "Cannot parse bind address '{}'. Must be a loopback address or AF_UNIX path.",
                address
            );
        }
    };

    if addrs.is_empty() {
        anyhow::bail!("Bind address '{}' resolved to no addresses.", address);
    }

    for addr in &addrs {
        let ip = addr.ip();
        if !ip.is_loopback() {
            anyhow::bail!(
                "Mock gateway must bind to loopback only. Got: {} (resolved: {}). \
                 External binding is prohibited (DEM-001, DEM-002).",
                address,
                addr
            );
        }
    }

    tracing::info!(
        "Local bind validated: {} resolves to {:?}",
        address,
        addrs.iter().map(|a| a.to_string()).collect::<Vec<_>>()
    );

    Ok(())
}
