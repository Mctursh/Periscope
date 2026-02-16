# Periscope

Fetch and query Anchor program IDLs from Solana.

## Install

```bash
cargo install anchor-periscope
```

## Commands

```bash
# On-chain (requires program ID)
periscope inspect <PROGRAM_ID>
periscope instructions <PROGRAM_ID>
periscope instruction <NAME> <PROGRAM_ID>
periscope errors <PROGRAM_ID>

# From file or URL (program ID not needed)
periscope inspect --idl ./target/idl/program.json
periscope instructions --idl https://github.com/user/repo/blob/main/idl.json
periscope inspect --idl ./idl.json 
```

## Options

```bash
# Custom RPC
periscope --url https://my-rpc.com inspect <PROGRAM_ID>

# Load from file (no program ID needed)
periscope --idl ./idl.json inspect

# Load from URL - GitHub blob URLs auto-convert to raw
periscope --idl https://github.com/user/repo/blob/main/idl.json inspect
```

## Config

Config file location:
- Linux: `~/.config/periscope/config.toml`
- macOS: `~/Library/Application Support/periscope/config.toml`

```bash
periscope config show
periscope config set --url https://api.devnet.solana.com
```

RPC priority: `--url` flag > config file > mainnet-beta default

## Library

```rust
use periscope::{fetch_idl_from_chain, load_idl_from_file};
use solana_sdk::pubkey::Pubkey;

// From chain
let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".parse()?;
let idl = fetch_idl_from_chain(&program_id, "https://api.mainnet-beta.solana.com")?;

// From file
let idl = load_idl_from_file("./target/idl/my_program.json")?;

println!("{}", idl.metadata.name);
```

Functions:
- `fetch_idl_from_chain(program_id, rpc_url)` - Fetch from on-chain IDL account
- `fetch_idl_with_client(client, program_id)` - Fetch with existing RPC client
- `load_idl_from_file(path)` - Load from local JSON file
- `fetch_idl_from_url(url)` - Fetch from URL (async)
- `get_idl_address(program_id)` - Derive IDL account address

## Supported Formats

- Anchor IDL spec 0.1.0+ (Anchor 0.29+)
- Legacy Anchor IDL (pre-0.29)

Format is auto-detected.

## License

MIT
