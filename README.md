# Periscope

Fetch and query Anchor program IDLs from Solana.

## Install

```bash
cargo install --path .
```

## Commands

```bash
# Program overview
periscope inspect <PROGRAM_ID>

# List all instructions
periscope instructions <PROGRAM_ID>

# Instruction details (accounts, args, discriminator)
periscope instruction <PROGRAM_ID> <NAME>

# List error codes
periscope errors <PROGRAM_ID>
```

## Options

```bash
# Custom RPC
periscope --url https://my-rpc.com inspect <PROGRAM_ID>

# Load from file instead of chain
periscope --idl ./target/idl/program.json inspect <PROGRAM_ID>

# Load from URL (GitHub blob URLs auto-convert to raw)
periscope --idl https://github.com/user/repo/blob/main/idl.json inspect <PROGRAM_ID>
```

## Config

Config file location depends on OS:
- Linux: `~/.config/periscope/config.toml`
- macOS: `~/Library/Application Support/periscope/config.toml`

```bash
# Show current config
periscope config show

# Set default RPC
periscope config set --url https://api.devnet.solana.com
```

RPC priority: `--url` flag > config file > mainnet-beta default

## Library

```rust
use periscope::fetch_idl_from_chain;
use solana_sdk::pubkey::Pubkey;

let program_id: Pubkey = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".parse()?;
let idl = fetch_idl_from_chain(&program_id, "https://api.mainnet-beta.solana.com")?;

println!("{}", idl.metadata.name);
for ix in &idl.instructions {
    println!("  {}", ix.name);
}
```

Functions:
- `fetch_idl_from_chain(program_id, rpc_url)` - Fetch from on-chain IDL account
- `fetch_idl_with_client(client, program_id)` - Fetch with existing RPC client
- `load_idl_from_file(path)` - Load from local JSON file
- `fetch_idl_from_url(url)` - Fetch from URL (async)
- `get_idl_address(program_id)` - Derive IDL account address

## Requirements

- Anchor programs only
- IDL must be published on-chain (`anchor idl init` / `anchor idl upgrade`)
- Anchor IDL spec 0.1.0+ (Anchor 0.29+)

<!-- ## TODO

- [ ] IDL caching
- [ ] Older Anchor IDL formats
- [ ] Interactive mode -->

## License

MIT
