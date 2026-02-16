use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use periscope::cli::{Cli, Commands, ConfigCommands, IdlSource};
use periscope::config::Config;
use periscope::display::{
    display_error, display_idl_overview, display_instruction_detail,
    display_instruction_not_found, display_instructions_list, display_errors_list,
};
use periscope::idl::{load_idl_from_file, fetch_idl_from_url, fetch_idl_from_chain, Idl};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    let result = run(cli).await;

    if let Err(e) = &result {
        display_error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

async fn run(cli: Cli) -> Result<()> {
    match &cli.command {
        Commands::Inspect { program_id } => {
            cmd_inspect(&cli, program_id.as_deref()).await
        }
        Commands::Instructions { program_id } => {
            cmd_instructions(&cli, program_id.as_deref()).await
        }
        Commands::Instruction { name, program_id } => {
            cmd_instruction(&cli, program_id.as_deref(), name).await
        }
        Commands::Errors { program_id } => {
            cmd_errors(&cli, program_id.as_deref()).await
        }
        Commands::Config { action } => {
            cmd_config(action.clone())
        }
    }
}

// ============================================================================
// Command handlers
// ============================================================================

/// Handle `inspect` command - show full IDL overview
async fn cmd_inspect(cli: &Cli, program_id: Option<&str>) -> Result<()> {
    let idl = fetch_idl(cli, program_id).await?;
    display_idl_overview(&idl);
    Ok(())
}

/// Handle `instructions` command - list all instructions
async fn cmd_instructions(cli: &Cli, program_id: Option<&str>) -> Result<()> {
    let idl = fetch_idl(cli, program_id).await?;
    display_instructions_list(&idl);
    Ok(())
}

/// Handle `instruction` command - show specific instruction details
async fn cmd_instruction(cli: &Cli, program_id: Option<&str>, name: &str) -> Result<()> {
    let idl = fetch_idl(cli, program_id).await?;

    // Find the instruction by name (case-insensitive)
    let instruction = idl.instructions.iter().find(|ix| {
        ix.name.eq_ignore_ascii_case(name)
    });

    match instruction {
        Some(ix) => {
            display_instruction_detail(ix);
            Ok(())
        }
        None => {
            let available: Vec<&str> = idl.instructions.iter().map(|ix| ix.name.as_str()).collect();
            display_instruction_not_found(name, &available);
            Err(anyhow!("Instruction '{}' not found", name))
        }
    }
}

/// Handle `errors` command - list all error codes
async fn cmd_errors(cli: &Cli, program_id: Option<&str>) -> Result<()> {
    let idl = fetch_idl(cli, program_id).await?;
    display_errors_list(&idl);
    Ok(())
}

/// Handle `config` subcommands
fn cmd_config(action: ConfigCommands) -> Result<()> {
    match action {
        ConfigCommands::Show => {
            let config = Config::load()?;
            let config_path = Config::file_path()?;
            let exists = Config::exists();

            println!();
            println!("Periscope Configuration:");
            println!("  Config file: {}", config_path.display());
            println!("  File exists: {}", if exists { "yes" } else { "no (using defaults)" });
            println!();
            println!("  RPC URL: {}", config.rpc_url);
            println!();
            Ok(())
        }
        ConfigCommands::Set { url } => {
            if let Some(url) = url {
                // Load existing config or defaults
                let mut config = Config::load()?;

                // Update the RPC URL
                config.rpc_url = url.clone();

                // Validate before saving
                config.validate()?;

                // Save to file
                config.save()?;

                let config_path = Config::file_path()?;
                println!("Saved RPC URL to {}", config_path.display());
                println!("  rpc_url = \"{}\"", url);
            } else {
                println!("No value provided to set.");
                println!("Usage: periscope config set --url <RPC_URL>");
            }
            Ok(())
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Fetch IDL from the appropriate source
///
/// - If --idl is provided: load from file/URL (program_id not required)
/// - If --idl is not provided: fetch on-chain (program_id required)
async fn fetch_idl(cli: &Cli, program_id: Option<&str>) -> Result<Idl> {
    let source = cli.idl_source();

    match source {
        IdlSource::File(path) => {
            // Load from local file - program_id not needed
            let idl = load_idl_from_file(&path)?;
            Ok(idl)
        }
        IdlSource::Url(url) => {
            // Fetch from URL - program_id not needed
            let idl = fetch_idl_from_url(&url).await?;
            Ok(idl)
        }
        IdlSource::OnChain => {
            // Fetch on-chain - program_id required
            let program_id_str = program_id.ok_or_else(|| {
                anyhow!("Program ID is required when fetching on-chain. Use --idl to load from file/URL instead.")
            })?;

            let pubkey = Pubkey::from_str(program_id_str)
                .map_err(|_| anyhow!("Invalid program ID: {}", program_id_str))?;

            let rpc_url = get_rpc_url(cli);
            let idl = fetch_idl_from_chain(&pubkey, &rpc_url)?;
            Ok(idl)
        }
    }
}

/// Get RPC URL from --url flag or config
fn get_rpc_url(cli: &Cli) -> String {
    match &cli.url {
        Some(url) => url.clone(),
        None => {
            let config = Config::load().unwrap_or_default();
            config.rpc_url
        }
    }
}
