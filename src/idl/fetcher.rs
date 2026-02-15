//! IDL fetching from multiple sources: on-chain, file, or URL

use crate::cli::IdlSource;
use crate::error::{PeriscopeError, PeriscopeResult};
use crate::idl::Idl;
use flate2::read::{DeflateDecoder, ZlibDecoder};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

/// IDL account seed used by Anchor
pub const IDL_SEED: &str = "anchor:idl";

/// HTTP request timeout in seconds
const HTTP_TIMEOUT_SECS: u64 = 30;

/// Byte sizes in IDL account header
const DISCRIMINATOR_SIZE: usize = 8;
const AUTHORITY_SIZE: usize = 32;
const DATA_LEN_SIZE: usize = 4;

/// Offset where data_len field starts (after discriminator + authority)
const DATA_LEN_OFFSET: usize = DISCRIMINATOR_SIZE + AUTHORITY_SIZE; // 40

/// Total header size before compressed data
const HEADER_SIZE: usize = DATA_LEN_OFFSET + DATA_LEN_SIZE; // 44 bytes

// ============================================================================
// Main entry point (CLI usage) - dispatches to appropriate fetcher
// ============================================================================

/// Load IDL from the specified source (CLI entry point)
///
/// # Arguments
/// * `source` - Where to load the IDL from (on-chain, file, or URL)
/// * `program_id` - Program ID (used for on-chain fetch)
/// * `rpc_url` - RPC URL for on-chain fetching
pub async fn load_idl(
    source: IdlSource,
    program_id: &Pubkey,
    rpc_url: &str,
) -> PeriscopeResult<Idl> {
    match source {
        IdlSource::OnChain => fetch_idl_from_chain(program_id, rpc_url),
        IdlSource::File(path) => load_idl_from_file(&path),
        IdlSource::Url(url) => fetch_idl_from_url(&url).await,
    }
}

// ============================================================================
// Public API for library users
// ============================================================================

/// Fetch IDL from on-chain IDL account
///
/// This is the primary function for library users who want to fetch
/// an IDL directly from the Solana blockchain.
///
/// # Arguments
/// * `program_id` - The program ID to fetch the IDL for
/// * `rpc_url` - The RPC endpoint URL
///
/// # Example
/// ```ignore
/// use solana_sdk::pubkey::Pubkey;
/// use periscope::fetch_idl_from_chain;
///
/// let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".parse()?;
/// let idl = fetch_idl_from_chain(&program_id, "https://api.mainnet-beta.solana.com")?;
/// ```
///
/// # Account Layout (44 byte header + compressed data)
/// ```text
/// ┌──────────────┬──────────────┬──────────────┬────────────────────┐
/// │ Discriminator│  Authority   │   data_len   │  Compressed Data   │
/// │   (8 bytes)  │  (32 bytes)  │  (4 bytes)   │    (N bytes)       │
/// └──────────────┴──────────────┴──────────────┴────────────────────┘
/// ```
pub fn fetch_idl_from_chain(program_id: &Pubkey, rpc_url: &str) -> PeriscopeResult<Idl> {
    let client = RpcClient::new(rpc_url.to_string());
    fetch_idl_with_client(&client, program_id)
}

/// Fetch IDL using an existing RPC client
///
/// Use this when you want to reuse an RPC client across multiple calls.
///
/// # Arguments
/// * `client` - An existing RPC client
/// * `program_id` - The program ID to fetch the IDL for
pub fn fetch_idl_with_client(client: &RpcClient, program_id: &Pubkey) -> PeriscopeResult<Idl> {
    // Step 1: Derive IDL account address
    let idl_address = get_idl_address(program_id)?;

    // Step 2: Fetch account data
    let account = client.get_account(&idl_address).map_err(|e| {
        // Check if it's specifically an account not found error
        let error_str = e.to_string();
        if error_str.contains("AccountNotFound") || error_str.contains("could not find account") {
            PeriscopeError::IdlNotFound(program_id.to_string())
        } else {
            // Preserve the original RPC error for other cases (network, timeout, etc.)
            PeriscopeError::RpcError(e)
        }
    })?;

    let data = account.data;

    // Step 3: Validate we have enough data for the header
    if data.len() < HEADER_SIZE {
        return Err(PeriscopeError::DecompressionError(
            "Account data too small for IDL header".to_string(),
        ));
    }

    // Step 4: Extract data_len (bytes 40-44, little-endian u32)
    let data_len_bytes: [u8; 4] = data[DATA_LEN_OFFSET..DATA_LEN_OFFSET + DATA_LEN_SIZE]
        .try_into()
        .map_err(|_| PeriscopeError::DecompressionError("Failed to read data_len".to_string()))?;
    let data_len = u32::from_le_bytes(data_len_bytes) as usize;

    // Step 5: Validate data_len is not zero
    if data_len == 0 {
        return Err(PeriscopeError::DecompressionError(
            "IDL compressed data is empty".to_string(),
        ));
    }

    // Step 6: Validate compressed data length
    if data.len() < HEADER_SIZE + data_len {
        return Err(PeriscopeError::DecompressionError(format!(
            "Compressed data truncated: expected {} bytes, got {}",
            data_len,
            data.len() - HEADER_SIZE
        )));
    }
    let compressed = &data[HEADER_SIZE..HEADER_SIZE + data_len];

    // Step 7: Decompress (try zlib first, fallback to raw deflate)
    let json_bytes = decompress_idl_data(compressed)?;

    // Step 8: Validate UTF-8 before JSON parsing (better error message)
    if std::str::from_utf8(&json_bytes).is_err() {
        return Err(PeriscopeError::DecompressionError(
            "Decompressed data is not valid UTF-8".to_string(),
        ));
    }

    // Step 9: Parse JSON into Idl struct
    let idl: Idl = serde_json::from_slice(&json_bytes)?;

    Ok(idl)
}

/// Load IDL from a local JSON file
///
/// # Arguments
/// * `path` - Path to the IDL JSON file
///
/// # Example
/// ```ignore
/// use periscope::load_idl_from_file;
///
/// let idl = load_idl_from_file("./target/idl/my_program.json")?;
/// ```
pub fn load_idl_from_file(path: &str) -> PeriscopeResult<Idl> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(PeriscopeError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path.display()),
        )));
    }

    let contents = std::fs::read_to_string(path)?;
    let idl: Idl = serde_json::from_str(&contents)?;

    Ok(idl)
}

/// Fetch IDL from a remote URL
///
/// The URL must point to raw JSON content (not an HTML page).
///
/// **Note:** GitHub blob URLs (github.com/.../blob/...) are automatically
/// converted to raw URLs by the CLI. When using this function directly,
/// provide raw.githubusercontent.com URLs.
///
/// # Arguments
/// * `url` - URL to the IDL JSON file
///
/// # Example
/// ```ignore
/// use periscope::fetch_idl_from_url;
///
/// let idl = fetch_idl_from_url(
///     "https://raw.githubusercontent.com/user/repo/main/idl/program.json"
/// ).await?;
/// ```
pub async fn fetch_idl_from_url(url: &str) -> PeriscopeResult<Idl> {
    // Build client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| {
            PeriscopeError::NetworkError(format!("Failed to create HTTP client: {}", e))
        })?;

    // Make request
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| PeriscopeError::NetworkError(format!("HTTP request failed: {}", e)))?;

    // Check status
    if !response.status().is_success() {
        return Err(PeriscopeError::HttpError {
            status: response.status().as_u16(),
            url: url.to_string(),
        });
    }

    // Read body
    let body = response
        .text()
        .await
        .map_err(|e| PeriscopeError::NetworkError(format!("Failed to read response body: {}", e)))?;

    // Parse JSON
    let idl: Idl = serde_json::from_str(&body)?;

    Ok(idl)
}

// ============================================================================
// Helper functions
// ============================================================================

/// Derive the IDL account address for a program
///
/// Address derivation (two-step process):
/// 1. Find program signer: `find_program_address(&[], program_id)`
/// 2. Create with seed: `create_with_seed(signer, "anchor:idl", program_id)`
pub fn get_idl_address(program_id: &Pubkey) -> PeriscopeResult<Pubkey> {
    let (program_signer, _bump) = Pubkey::find_program_address(&[], program_id);

    let idl_address = Pubkey::create_with_seed(&program_signer, IDL_SEED, program_id)
        .map_err(|e| PeriscopeError::InvalidProgramId(e.to_string()))?;

    Ok(idl_address)
}

/// Decompress IDL data, trying zlib first then raw deflate
fn decompress_idl_data(compressed: &[u8]) -> PeriscopeResult<Vec<u8>> {
    // Try zlib (deflate with header) first - most common
    if let Ok(bytes) = decompress_zlib(compressed) {
        return Ok(bytes);
    }

    // Fallback to raw deflate (no header)
    if let Ok(bytes) = decompress_deflate(compressed) {
        return Ok(bytes);
    }

    Err(PeriscopeError::DecompressionError(
        "Failed to decompress IDL data with both zlib and deflate".to_string(),
    ))
}

/// Try to decompress using zlib (deflate with header)
fn decompress_zlib(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Try to decompress using raw deflate (no header)
fn decompress_deflate(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_idl_address() {
        // Jupiter v6 program ID
        let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
            .parse()
            .unwrap();

        let idl_address = get_idl_address(&program_id).unwrap();

        // The IDL address should be deterministic
        println!("IDL address for Jupiter: {}", idl_address);
        assert_ne!(idl_address, program_id);
    }

    #[test]
    fn test_get_idl_address_deterministic() {
        let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
            .parse()
            .unwrap();

        let addr1 = get_idl_address(&program_id).unwrap();
        let addr2 = get_idl_address(&program_id).unwrap();

        assert_eq!(addr1, addr2, "IDL address should be deterministic");
    }

    #[test]
    fn test_header_constants() {
        // Verify our constants are correct
        assert_eq!(DISCRIMINATOR_SIZE, 8);
        assert_eq!(AUTHORITY_SIZE, 32);
        assert_eq!(DATA_LEN_SIZE, 4);
        assert_eq!(DATA_LEN_OFFSET, 40);
        assert_eq!(HEADER_SIZE, 44);
    }
}
