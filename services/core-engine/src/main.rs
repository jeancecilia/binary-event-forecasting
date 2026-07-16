//! Binary Event Forecasting — Core Simulation Engine
//!
//! Entry point for the Rust core engine binary.

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "core-engine")]
#[command(about = "Binary Event Forecasting — Core Simulation Engine")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/core.toml")]
    config: std::path::PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in replay mode (deterministic offline replay)
    Replay {
        /// Path to trace directory
        #[arg(long)]
        trace: Option<std::path::PathBuf>,
        /// Verify replay determinism
        #[arg(long)]
        verify: bool,
    },
    /// Run database migrations
    Migrate,
    /// Run in prospective observation mode
    Prospective,
    /// Run in mock demo mode
    Mock,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,core_engine=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let cli = Cli::parse();

    // Load configuration
    let config = core_engine::CoreConfig::from_file(&cli.config)?;

    tracing::info!(
        mode = ?config.mode,
        socket = %config.socket_path.display(),
        "Core engine initializing"
    );

    match cli.command.unwrap_or(Commands::Replay {
        trace: None,
        verify: false,
    }) {
        Commands::Replay { trace: _, verify: _ } => {
            core_engine::modes::replay::run().await?;
        }
        Commands::Migrate => {
            tracing::info!("Running database migrations...");
            // TODO: Migration logic
        }
        Commands::Prospective => {
            core_engine::modes::prospective::run().await?;
        }
        Commands::Mock => {
            core_engine::modes::mock::run().await?;
        }
    }

    Ok(())
}
