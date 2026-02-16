//! IDL fetching from multiple sources: on-chain, file, or URL

use crate::cli::IdlSource;
use crate::error::{PeriscopeError, PeriscopeResult};
use crate::idl::legacy::LegacyIdl;
use crate::idl::Idl;
use flate2::read::{DeflateDecoder, ZlibDecoder};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

pub const IDL_SEED: &str = "anchor:idl";

const HTTP_TIMEOUT_SECS: u64 = 30;
const DISCRIMINATOR_SIZE: usize = 8;
const AUTHORITY_SIZE: usize = 32;
const DATA_LEN_SIZE: usize = 4;
const DATA_LEN_OFFSET: usize = DISCRIMINATOR_SIZE + AUTHORITY_SIZE;
const HEADER_SIZE: usize = DATA_LEN_OFFSET + DATA_LEN_SIZE;

/// Load IDL from the specified source.
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

/// Fetch IDL from on-chain IDL account.
pub fn fetch_idl_from_chain(program_id: &Pubkey, rpc_url: &str) -> PeriscopeResult<Idl> {
    let client = RpcClient::new(rpc_url.to_string());
    fetch_idl_with_client(&client, program_id)
}

/// Fetch IDL using an existing RPC client.
pub fn fetch_idl_with_client(client: &RpcClient, program_id: &Pubkey) -> PeriscopeResult<Idl> {
    let idl_address = get_idl_address(program_id)?;

    let account = client.get_account(&idl_address).map_err(|e| {
        let error_str = e.to_string();
        if error_str.contains("AccountNotFound") || error_str.contains("could not find account") {
            PeriscopeError::IdlNotFound(program_id.to_string())
        } else {
            PeriscopeError::RpcError(e)
        }
    })?;

    let data = account.data;

    if data.len() < HEADER_SIZE {
        return Err(PeriscopeError::DecompressionError(
            "Account data too small for IDL header".to_string(),
        ));
    }

    let data_len_bytes: [u8; 4] = data[DATA_LEN_OFFSET..DATA_LEN_OFFSET + DATA_LEN_SIZE]
        .try_into()
        .map_err(|_| PeriscopeError::DecompressionError("Failed to read data_len".to_string()))?;
    let data_len = u32::from_le_bytes(data_len_bytes) as usize;

    if data_len == 0 {
        return Err(PeriscopeError::DecompressionError(
            "IDL compressed data is empty".to_string(),
        ));
    }

    if data.len() < HEADER_SIZE + data_len {
        return Err(PeriscopeError::DecompressionError(format!(
            "Compressed data truncated: expected {} bytes, got {}",
            data_len,
            data.len() - HEADER_SIZE
        )));
    }

    let compressed = &data[HEADER_SIZE..HEADER_SIZE + data_len];
    let json_bytes = decompress_idl_data(compressed)?;

    let json_str = std::str::from_utf8(&json_bytes)
        .map_err(|_| PeriscopeError::DecompressionError("Invalid UTF-8".to_string()))?;

    parse_idl_json(json_str)
}

/// Load IDL from a local JSON file.
pub fn load_idl_from_file(path: &str) -> PeriscopeResult<Idl> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(PeriscopeError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path.display()),
        )));
    }

    let contents = std::fs::read_to_string(path)?;
    parse_idl_json(&contents)
}

/// Fetch IDL from a remote URL.
pub async fn fetch_idl_from_url(url: &str) -> PeriscopeResult<Idl> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| {
            PeriscopeError::NetworkError(format!("Failed to create HTTP client: {}", e))
        })?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| PeriscopeError::NetworkError(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(PeriscopeError::HttpError {
            status: response.status().as_u16(),
            url: url.to_string(),
        });
    }

    let body = response.text().await.map_err(|e| {
        PeriscopeError::NetworkError(format!("Failed to read response body: {}", e))
    })?;

    parse_idl_json(&body)
}

/// Parse IDL JSON, auto-detecting format (new 0.1.0 spec vs legacy).
fn parse_idl_json(json_str: &str) -> PeriscopeResult<Idl> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;

    let is_new_format = value.get("address").map(|v| v.is_string()).unwrap_or(false);

    if is_new_format {
        return serde_json::from_str(json_str).map_err(PeriscopeError::ParseError);
    }

    if value.get("name").is_some() {
        if let Ok(legacy) = serde_json::from_str::<LegacyIdl>(json_str) {
            return Ok(legacy.into());
        }
    }

    if let Ok(idl) = serde_json::from_str::<Idl>(json_str) {
        return Ok(idl);
    }

    if let Ok(legacy) = serde_json::from_str::<LegacyIdl>(json_str) {
        return Ok(legacy.into());
    }

    Err(PeriscopeError::ParseError(
        serde_json::from_str::<Idl>(json_str).unwrap_err(),
    ))
}

/// Derive the IDL account address for a program.
pub fn get_idl_address(program_id: &Pubkey) -> PeriscopeResult<Pubkey> {
    let (program_signer, _bump) = Pubkey::find_program_address(&[], program_id);

    let idl_address = Pubkey::create_with_seed(&program_signer, IDL_SEED, program_id)
        .map_err(|e| PeriscopeError::InvalidProgramId(e.to_string()))?;

    Ok(idl_address)
}

fn decompress_idl_data(compressed: &[u8]) -> PeriscopeResult<Vec<u8>> {
    if let Ok(bytes) = decompress_zlib(compressed) {
        return Ok(bytes);
    }

    if let Ok(bytes) = decompress_deflate(compressed) {
        return Ok(bytes);
    }

    Err(PeriscopeError::DecompressionError(
        "Failed to decompress IDL data".to_string(),
    ))
}

fn decompress_zlib(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn decompress_deflate(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_idl_address() {
        let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
            .parse()
            .unwrap();

        let idl_address = get_idl_address(&program_id).unwrap();
        assert_ne!(idl_address, program_id);
    }

    #[test]
    fn test_get_idl_address_deterministic() {
        let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
            .parse()
            .unwrap();

        let addr1 = get_idl_address(&program_id).unwrap();
        let addr2 = get_idl_address(&program_id).unwrap();

        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_header_constants() {
        assert_eq!(DISCRIMINATOR_SIZE, 8);
        assert_eq!(AUTHORITY_SIZE, 32);
        assert_eq!(DATA_LEN_SIZE, 4);
        assert_eq!(DATA_LEN_OFFSET, 40);
        assert_eq!(HEADER_SIZE, 44);
    }
}
