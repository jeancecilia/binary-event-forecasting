//! Mock gateway configuration.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    pub gateway: GatewaySection,
    pub scenarios: ScenariosSection,
    pub trace: TraceSection,
}

#[derive(Debug, Deserialize)]
pub struct GatewaySection {
    pub environment: String,
    pub scenario_id: String,
    pub bind_address: String,
}

#[derive(Debug, Deserialize)]
pub struct ScenariosSection {
    pub path: std::path::PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct TraceSection {
    pub path: std::path::PathBuf,
}

impl TomlConfig {
    pub fn into_mock_config(self) -> super::MockConfig {
        super::MockConfig {
            environment: self.gateway.environment,
            scenario_id: self.gateway.scenario_id,
            config_hash: String::new(), // Computed at startup
            build_hash: env!("CARGO_PKG_VERSION").to_string(),
            bind_address: self.gateway.bind_address,
            scenarios_path: self.scenarios.path,
            trace_path: self.trace.path,
        }
    }
}
