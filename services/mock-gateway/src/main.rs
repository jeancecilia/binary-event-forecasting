//! Binary Event Forecasting — Local Mock Demo Gateway
//!
//! Entry point for the mock gateway binary.

use clap::Parser;
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

    // Validate environment
    config.validate_environment()?;

    tracing::info!(
        environment = %config.environment,
        scenario_id = %config.scenario_id,
        bind = %config.bind_address,
        "Mock gateway initializing"
    );

    // Verify no external hostnames in configuration
    if config.bind_address.contains("://")
        && !config.bind_address.starts_with("127.0.0.1")
        && !config.bind_address.starts_with("localhost")
    {
        anyhow::bail!(
            "Mock gateway must bind to localhost or AF_UNIX only. Got: {}",
            config.bind_address
        );
    }

    mock_gateway::server::run(&config.bind_address).await?;

    Ok(())
}
