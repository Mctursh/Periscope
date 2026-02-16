//! IDL caching layer for Periscope
//!
//! Caches fetched IDLs at ~/.config/periscope/cache/

use crate::idl::Idl;
use std::path::PathBuf;

/// Cache directory name
pub const CACHE_DIR: &str = "cache";

/// IDL cache manager
pub struct IdlCache;

impl IdlCache {
    /// Get the cache directory path (~/.config/periscope/cache/)
    pub fn cache_dir() -> Option<PathBuf> {
        crate::config::Config::dir_path()
            .ok()
            .map(|p| p.join(CACHE_DIR))
    }

    /// Get cached IDL for a program, if it exists
    pub fn get(_program_id: &str) -> Option<Idl> {
        // TODO: Implement cache retrieval
        // 1. Build path: cache_dir/{program_id}.json
        // 2. Read and parse if exists
        // 3. Return None if not cached
        todo!("Implement cache get")
    }

    /// Store IDL in cache
    pub fn set(_program_id: &str, _idl: &Idl) -> anyhow::Result<()> {
        // TODO: Implement cache storage
        // 1. Serialize IDL to JSON
        // 2. Write to cache_dir/{program_id}.json
        todo!("Implement cache set")
    }

    /// Clear cached IDL for a specific program
    pub fn clear(_program_id: &str) -> anyhow::Result<()> {
        // TODO: Implement cache clearing
        todo!("Implement cache clear")
    }

    /// Clear entire cache
    pub fn clear_all() -> anyhow::Result<()> {
        // TODO: Implement full cache clear
        todo!("Implement cache clear all")
    }
}
