//! Configuration handling

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Paths to never delete
    pub whitelist: Vec<PathBuf>,

    /// Directories to scan for dev artifacts
    pub project_paths: Vec<PathBuf>,

    /// Skip files newer than this many days
    pub skip_recent_days: u32,

    /// Maximum journal log size to keep
    pub journal_max_size: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            whitelist: vec![],
            project_paths: vec![
                home.join("Projects"),
                home.join("Development"),
                home.join("dev"),
                home.join("code"),
                home.join("GitHub"),
            ],
            skip_recent_days: 7,
            journal_max_size: "100M".to_string(),
        }
    }
}

impl Config {
    /// Load config from file or return defaults
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Save config to file
    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self).unwrap_or_default();
        std::fs::write(config_path, content)
    }

    /// Get config file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mole-rs")
            .join("config.toml")
    }
}
