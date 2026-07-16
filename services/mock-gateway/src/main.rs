//! Binary Event Forecasting — Local Mock Demo Gateway

use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "mock-gateway")]
#[command(about = "Binary Event Forecasting — Local Mock Demo Gateway")]
struct Cli {
    #[arg(short, long, default_value = "config/mock-gateway.toml")]
    config: std::path::PathBuf,
}

enum LocalBind {
    Tcp(SocketAddr),
    Unix(PathBuf),
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

    config.validate_environment()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let bind = validate_local_bind(&config.bind_address)?;

    tracing::info!(
        environment = %config.environment,
        scenario_id = %config.scenario_id,
        bind = %config.bind_address,
        config_hash = %config.config_hash,
        "Mock gateway initializing"
    );

    match bind {
        LocalBind::Tcp(addr) => {
            tracing::info!("Binding TCP on loopback: {}", addr);
            mock_gateway::server::run(&addr.to_string()).await?;
        }
        LocalBind::Unix(path) => {
            tracing::info!("Unix socket path: {}", path.display());
            anyhow::bail!("Unix socket binding not yet implemented for mock gateway");
        }
    }

    Ok(())
}

fn validate_local_bind(address: &str) -> anyhow::Result<LocalBind> {
    // Unix socket path
    if address.starts_with('/') || address.starts_with("./") || address.starts_with("../") {
        return Ok(LocalBind::Unix(PathBuf::from(address)));
    }

    // Try parsing as SocketAddr directly (handles 127.0.0.1:8080 and [::1]:8080)
    if let Ok(addr) = SocketAddr::from_str(address) {
        let ip = addr.ip();
        if !ip.is_loopback() {
            anyhow::bail!(
                "Mock gateway must bind to loopback only. Got {}, which is not a loopback address.",
                ip
            );
        }
        return Ok(LocalBind::Tcp(addr));
    }

    // Try "localhost:PORT" — resolve to 127.0.0.1 without DNS
    if let Some(port_str) = address.strip_prefix("localhost:") {
        let port: u16 = port_str.parse().map_err(|_| {
            anyhow::anyhow!("Invalid port in '{}'", address)
        })?;
        return Ok(LocalBind::Tcp(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port,
        )));
    }

    // Try "ipv6-localhost:PORT" (::1 representation)
    if let Some(port_str) = address.strip_prefix("[::1]:") {
        let port: u16 = port_str.parse().map_err(|_| {
            anyhow::anyhow!("Invalid port in '{}'", address)
        })?;
        return Ok(LocalBind::Tcp(SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            port,
        )));
    }

    anyhow::bail!(
        "Mock gateway must bind to localhost or a loopback IP only. \
         Got '{}'. External binding and DNS resolution are prohibited (DEM-001, DEM-002).",
        address
    )
}
