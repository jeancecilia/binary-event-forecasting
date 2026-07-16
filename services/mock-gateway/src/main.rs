//! Binary Event Forecasting — Demo Gateway

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "mock-gateway")]
#[command(about = "Binary Event Forecasting — Demo Gateway")]
struct Cli {
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

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&config_content);
    config.config_hash = hex::encode(hasher.finalize());

    tracing::info!(
        environment = %config.environment,
        scenario_id = %config.scenario_id,
        bind = %config.bind_address,
        config_hash = %config.config_hash,
        "Mock gateway initializing"
    );

    mock_gateway::server::run(&config.bind_address).await?;

    Ok(())
}
