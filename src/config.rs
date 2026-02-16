//! Configuration management for Periscope CLI
//!
//! Config is stored at ~/.config/periscope/config.toml

use crate::error::{PeriscopeError, PeriscopeResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Default RPC URL (mainnet-beta)
pub const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

/// Config directory name
const CONFIG_DIR: &str = "periscope";

/// Config file name
const CONFIG_FILE: &str = "config.toml";

/// Periscope configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// RPC URL for fetching IDLs
    #[serde(default = "default_rpc_url")]
    pub rpc_url: String,
}

fn default_rpc_url() -> String {
    DEFAULT_RPC_URL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: default_rpc_url(),
        }
    }
}

impl Config {
    /// Get the config directory path (~/.config/periscope/)
    pub fn dir_path() -> PeriscopeResult<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join(CONFIG_DIR))
            .ok_or_else(|| {
                PeriscopeError::ConfigError("Could not determine config directory".into())
            })
    }

    /// Get the config file path (~/.config/periscope/config.toml)
    pub fn file_path() -> PeriscopeResult<PathBuf> {
        Self::dir_path().map(|p| p.join(CONFIG_FILE))
    }

    /// Check if config file exists
    pub fn exists() -> bool {
        Self::file_path().map(|p| p.exists()).unwrap_or(false)
    }

    /// Load config from file, returning defaults if file doesn't exist
    pub fn load() -> PeriscopeResult<Self> {
        let path = Self::file_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path).map_err(PeriscopeError::IoError)?;

        if contents.trim().is_empty() {
            return Ok(Self::default());
        }

        let config: Config = toml::from_str(&contents)
            .map_err(|e| PeriscopeError::ConfigError(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Save config to file, creating directories if needed
    pub fn save(&self) -> PeriscopeResult<()> {
        let dir = Self::dir_path()?;
        let path = Self::file_path()?;

        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(PeriscopeError::IoError)?;
        }

        let contents = toml::to_string_pretty(self).map_err(|e| {
            PeriscopeError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&path, contents).map_err(PeriscopeError::IoError)?;

        Ok(())
    }

    /// Validate the config values
    pub fn validate(&self) -> PeriscopeResult<()> {
        if !self.rpc_url.starts_with("http://") && !self.rpc_url.starts_with("https://") {
            return Err(PeriscopeError::ConfigError(
                "RPC URL must start with http:// or https://".into(),
            ));
        }

        Ok(())
    }
}
