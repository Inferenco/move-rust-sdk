# Move Rust SDK

A chain-agnostic Rust SDK for the [Move language] ecosystem — works with
[Movement], [Aptos], and any Move-based blockchain.

This crate is a thin, unified configuration and naming layer on top of
[`aptos-sdk`]. The underlying cryptography, transaction building, and
network client code is shared; what changes between chains is a
developer-supplied `MoveConfig`.

## Quick start

### Pre-built chain: Aptos

```rust,ignore
use move_rust_sdk::{AptosConfig, MoveClient};

let client = MoveClient::new(AptosConfig::testnet())?;
let balance = client.get_balance(address).await?;
```

### Pre-built chain: Movement (requires the `movement` feature)

```rust,ignore
use move_rust_sdk::{MovementConfig, MoveClient};

let client = MoveClient::new(MovementConfig::testnet())?;
let balance = client.get_balance(address).await?;
```

### Custom Move-based chain (always available)

The `MoveConfig::custom_with_preset` entry point is part of the
chain-agnostic core — it works **without** the `aptos` or `movement`
features enabled. This is the only way to target a chain when the
developer has not opted into a pre-built configuration.

```rust,ignore
use move_rust_sdk::{ChainPreset, MoveClient, MoveConfig};

// Developer-defined fullnode URL + Movement signing domain.
let config = MoveConfig::custom_with_preset(
    "https://my-chain.example.com/v1",
    ChainPreset::Movement,
)?;
let client = MoveClient::new(config)?;
```

## Why a separate `move-rust-sdk` crate?

- **One API, many chains.** Switch from Aptos to Movement by changing
  one line of configuration — no other code needs to change.
- **Pre-built configurations are opt-in.** Enable the `aptos` or
  `movement` feature to get `AptosConfig::testnet()` /
  `MovementConfig::mainnet()` etc. Without a pre-built feature, the
  chain-agnostic core still works — just use `MoveConfig::custom`.
- **No fork required.** Underneath, the same `aptos-sdk` is doing the
  heavy lifting. Transaction building, signing, submission, and
  cryptographic primitives are all the same code.

## Feature flags

The chain-agnostic core — `MoveConfig`, `MoveClient`, the
`ChainPreset` enum, the underlying transaction and cryptography
types — is always available. Pre-built chain configurations are
**opt-in**.

| Feature | Default | Description |
|---------|---------|-------------|
| `aptos` | No | Enable the pre-built Aptos chain configurations (`AptosConfig::mainnet`, `AptosConfig::testnet`, ...) |
| `movement` | No | Enable the pre-built Movement chain configurations (`MovementConfig::mainnet`, `MovementConfig::testnet`, ...) |
| `all-chains` | No | Enable both `aptos` and `movement` |
| `ed25519` | Yes | Ed25519 signature scheme |
| `secp256k1` | Yes | Secp256k1 ECDSA signatures |
| `secp256r1` | Yes | Secp256r1 (P-256) ECDSA signatures |
| `mnemonic` | Yes | BIP-39 mnemonic phrase support |
| `indexer` | Yes | GraphQL indexer client |
| `faucet` | Yes | Faucet integration for testnets |
| `bls` | No | BLS12-381 signatures |
| `keyless` | No | OIDC-based keyless authentication |
| `macros` | No | Proc macros for type-safe contract bindings |

## Signing domains

Different Move-based chains sign transactions with different domain
prefixes. The `MoveConfig` carries the correct domain for the active
chain:

| Chain | Signing domain |
|-------|----------------|
| Aptos | `APTOS::RawTransaction` |
| Movement | `MOVEMENT::RawTransaction` |

The `MoveClient::sign_transaction(raw_txn, account)` helper uses the
active chain's signing domain automatically, so callers do not have
to think about it.

## Examples

```text
# Pre-built chains (require the `aptos` / `movement` features)
cargo run -p move-rust-sdk --example aptos_connect     --features "aptos,ed25519"
cargo run -p move-rust-sdk --example movement_connect  --features "movement,ed25519"

# Custom chain (works with default features)
cargo run -p move-rust-sdk --example custom_chain      --features "ed25519"
cargo run -p move-rust-sdk --example custom_only       --features "ed25519"
```

## Underlying crate

`move-rust-sdk` is a thin facade on top of `aptos-sdk`. Anything
re-exported from `aptos-sdk` (e.g. `account::Ed25519Account`,
`transaction::TransactionBuilder`, `types::AccountAddress`) is
available directly from `move-rust-sdk` so callers do not have to
add `aptos-sdk` as a direct dependency.

The underlying transaction primitives (`RawTransaction::signing_message`,
`sign_transaction`, etc.) still exist on the underlying types — the
chain-agnostic addition is the `*_with_domain` variant, which is what
the `move-rust-sdk` crate calls into for non-Aptos chains.

## License

Apache-2.0.

[Move language]: https://move-language.github.io/move/
[Movement]: https://movementnetwork.xyz
[Aptos]: https://aptosfoundation.org
