//! CLI command definitions using clap

use clap::{Parser, Subcommand};

/// Periscope - Explore and query Anchor program IDLs on-chain
#[derive(Debug, Parser)]
#[command(name = "periscope")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// RPC URL (overrides config)
    #[arg(short, long, global = true)]
    pub url: Option<String>,

    /// Bypass cache and fetch fresh from chain
    #[arg(short, long, global = true)]
    pub refresh: bool,

    /// Load IDL from file path or URL instead of fetching from chain
    /// Accepts: local file path (./idl.json) or URL (https://...)
    /// GitHub URLs are auto-converted to raw URLs
    #[arg(short, long, global = true)]
    pub idl: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Represents where the IDL should be loaded from
#[derive(Debug, Clone)]
pub enum IdlSource {
    /// Fetch from on-chain IDL account
    OnChain,
    /// Load from local file
    File(String),
    /// Fetch from URL
    Url(String),
}


/// Convert GitHub blob URLs to raw.githubusercontent.com URLs
fn normalize_github_url(url: &str) -> String {
    if url.contains("github.com") && url.contains("/blob/") {
        url.replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/")
    } else {
        url.to_string()
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show full IDL overview for a program
    Inspect {
        /// Program ID (base58)
        program_id: String,
    },

    /// List all instructions in the program
    Instructions {
        /// Program ID (base58)
        program_id: String,
    },

    /// Show details for a specific instruction
    Instruction {
        /// Program ID (base58)
        program_id: String,

        /// Instruction name
        name: String,
    },

    /// List all error codes defined by the program
    Errors {
        /// Program ID (base58)
        program_id: String,
    },

    /// Manage Periscope configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set configuration value
    Set {
        /// RPC URL to use
        #[arg(long)]
        url: Option<String>,
    },
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Determine the IDL source based on --idl flag
    pub fn idl_source(&self) -> IdlSource {
        match &self.idl {
            None => IdlSource::OnChain,
            Some(path) => {
                if path.starts_with("http://") || path.starts_with("https://") {
                    IdlSource::Url(normalize_github_url(path))
                } else {
                    IdlSource::File(path.clone())
                }
            }
        }
    }
}
