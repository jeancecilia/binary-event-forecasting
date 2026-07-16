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

    match cli.command.unwrap_or(Commands::Replay {
        trace: None,
        verify: true,
    }) {
        Commands::Replay { trace, verify } => {
            let replay_config = core_engine::modes::replay::ReplayConfig {
                trace_path: trace.unwrap_or_else(|| std::path::PathBuf::from("data/traces/golden")),
                verify,
                probability_scale: 1_000_000,
            };
            core_engine::modes::replay::run(Some(replay_config)).await?;
        }
        Commands::Migrate => {
            tracing::info!("Running database migrations...");
            // TODO: Migration logic
        }
        Commands::Prospective => {
            tracing::info!("Starting prospective observation mode");
            // TODO: Implementation
        }
        Commands::Mock => {
            tracing::info!("Starting mock demo mode");
            // TODO: Implementation
        }
    }

    Ok(())
}
