//! Core engine configuration types.

use serde::Deserialize;
use std::path::PathBuf;

/// TOML-deserializable configuration.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TomlConfig {
    pub engine: EngineSection,
    pub journal: Option<JournalSection>,
    pub spool: Option<SpoolSection>,
    pub postgres: Option<PostgresSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineSection {
    pub socket_path: PathBuf,
    pub mode: String,
    pub max_signal_frame_bytes: Option<usize>,
    pub read_timeout_ms: Option<u64>,
    pub idle_timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalSection {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpoolSection {
    pub path: PathBuf,
    pub max_bytes: Option<u64>,
    pub max_records: Option<u64>,
    pub on_exhaustion: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresSection {
    pub url: String,
}

impl TomlConfig {
    /// Convert TOML config into the runtime CoreConfig.
    pub fn into_core_config(self) -> anyhow::Result<super::CoreConfig> {
        let mode = match self.engine.mode.as_str() {
            "replay" => super::OperatingMode::Replay,
            "prospective" => super::OperatingMode::Prospective,
            "mock" => super::OperatingMode::Mock,
            other => anyhow::bail!("Unknown operating mode: {other}"),
        };

        Ok(super::CoreConfig {
            socket_path: self.engine.socket_path,
            journal_path: self
                .journal
                .map(|j| j.path)
                .unwrap_or_else(|| PathBuf::from("var/journal/core-journal.sqlite")),
            spool_path: self
                .spool
                .map(|s| s.path)
                .unwrap_or_else(|| PathBuf::from("var/spool/research-store-spool.sqlite")),
            postgres_url: self.postgres.map(|p| p.url),
            mode,
            max_signal_frame_bytes: self.engine.max_signal_frame_bytes.unwrap_or(1_048_576),
            read_timeout_ms: self.engine.read_timeout_ms.unwrap_or(30_000),
            idle_timeout_ms: self.engine.idle_timeout_ms.unwrap_or(300_000),
        })
    }
}
