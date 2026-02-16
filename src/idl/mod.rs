//! IDL fetching and types
//!
//! This module handles fetching Anchor IDLs from on-chain
//! and provides types for working with them.

mod fetcher;
mod legacy;
mod types;

pub use fetcher::*;
pub use types::*;
