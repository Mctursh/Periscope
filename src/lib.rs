//! Periscope - Explore and query Anchor program IDLs on-chain
//!
//! This library provides functionality to fetch and query Anchor IDLs
//! from Solana programs. It can be used standalone or via the CLI.
//!
//! # Quick Start
//!
//! ```ignore
//! use periscope::{fetch_idl_from_chain, Idl};
//! use solana_sdk::pubkey::Pubkey;
//!
//! // Fetch IDL from chain
//! let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".parse()?;
//! let idl = fetch_idl_from_chain(&program_id, "https://api.mainnet-beta.solana.com")?;
//!
//! // Query the IDL
//! println!("Program: {}", idl.metadata.name);
//! for instruction in &idl.instructions {
//!     println!("  - {}", instruction.name);
//! }
//! ```
//!
//! # Loading IDL from Different Sources
//!
//! ```ignore
//! use periscope::{fetch_idl_from_chain, load_idl_from_file, fetch_idl_from_url};
//!
//! // From on-chain
//! let idl = fetch_idl_from_chain(&program_id, rpc_url)?;
//!
//! // From local file
//! let idl = load_idl_from_file("./target/idl/my_program.json")?;
//!
//! // From URL
//! let idl = fetch_idl_from_url("https://raw.githubusercontent.com/...").await?;
//! ```

pub mod cache;
pub mod cli;
pub mod config;
pub mod display;
pub mod error;
pub mod idl;

// Public re-exports for library users
pub use error::{PeriscopeError, PeriscopeResult};
pub use idl::{
    // Fetching functions
    fetch_idl_from_chain,
    fetch_idl_from_url,
    fetch_idl_with_client,
    get_idl_address,
    load_idl_from_file,
    // Types
    Idl,
    IdlAccount,
    IdlAccountItem,
    IdlAccountRef,
    IdlError as IdlErrorDef,
    IdlEventRef,
    IdlField,
    IdlInstruction,
    IdlMetadata,
    IdlType,
    IdlTypeDef,
};
