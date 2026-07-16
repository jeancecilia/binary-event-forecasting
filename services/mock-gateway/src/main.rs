//! Binary Event Forecasting — Local Mock Demo Gateway
//!
//! Entry point for the mock gateway binary.

use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(name = "mock-gateway")]
#[command(about = "Binary Event Forecasting — Local Mock Demo Gateway")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/mock-gateway.toml")]
    config: std::path::PathBuf,
}

/// Validated local bind target.
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

    // Compute the real configuration hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&config_content);
    config.config_hash = hex::encode(hasher.finalize());

    // Validate environment
    config.validate_environment()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Validate local binding
    let bind = validate_and_resolve_bind(&config.bind_address)?;

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
            tracing::info!("Binding Unix socket: {}", path.display());
            anyhow::bail!("Unix socket binding not yet implemented for mock gateway");
        }
    }

    Ok(())
}

/// Validate that the bind address is strictly local and does not perform DNS resolution.
///
/// Accepts:
/// - Literal "127.0.0.1:PORT" or "[::1]:PORT"
/// - Literal "localhost:PORT" (resolved without DNS to 127.0.0.1)
/// - Unix socket paths starting with '/' or './'
///
/// Rejects:
/// - "0.0.0.0:PORT"
/// - External IP addresses
/// - Hostnames that require DNS resolution
fn validate_and_resolve_bind(address: &str) -> anyhow::Result<LocalBind> {
    // Unix socket path
    if address.starts_with('/') || address.starts_with("./") || address.starts_with("../") {
        return Ok(LocalBind::Unix(PathBuf::from(address)));
    }

    // "localhost" is the only allowed hostname
    let (host_part, port_str) = address
        .rsplit_once(':')
        .ok_or_else(|| anyhow::anyhow!("Invalid bind address format: '{}'. Expected host:port.", address))?;

    let ip: IpAddr = if host_part == "localhost" {
        // Resolve localhost to 127.0.0.1 without DNS
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    } else {
        // Try parsing as a literal IP
        match host_part.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(_) => {
                // Reject any hostname that isn't "localhost"
                anyhow::bail!(
                    "Mock gateway must bind to localhost or a loopback IP only. \
                     Got hostname '{}' which is not 'localhost'. \
                     DNS resolution of arbitrary hostnames is prohibited (DEM-001, DEM-002).",
                    host_part
                );
            }
        }
    };

    if !ip.is_loopback() {
        anyhow::bail!(
            "Mock gateway must bind to loopback only. Got IP {}, which is not a loopback address.",
            ip
        );
    }

    let port: u16 = port_str.parse().map_err(|_| {
        anyhow::anyhow!("Invalid port: '{}'", port_str)
    })?;

    Ok(LocalBind::Tcp(SocketAddr::new(ip, port)))
}
