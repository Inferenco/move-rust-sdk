//! Example: Connect to a custom Move-based chain without any
//! pre-built chain feature.
//!
//! This example works **without** the `aptos` or `movement` features
//! enabled. It demonstrates the chain-agnostic core: the developer
//! supplies everything (fullnode URL, signing domain, native coin
//! symbol) via `MoveConfig::custom_with_preset`.
//!
//! Run with:
//!
//! ```text
//! cargo run -p move-rust-sdk --example custom_only --features "ed25519"
//! ```

use move_core_sdk::account::Ed25519Account;
use move_rust_sdk::{ChainPreset, MoveClient, MoveConfig, MoveError, MoveResult};

#[tokio::main]
async fn main() -> MoveResult<()> {
    // Without any pre-built chain feature enabled, the developer
    // picks a `ChainPreset` to select the signing domain and the
    // native coin symbol.
    let preset = ChainPreset::Aptos;

    // Then points `MoveConfig` at any fullnode URL.
    let config = MoveConfig::custom_with_preset("http://127.0.0.1:8080/v1", preset)?;
    println!(
        "Connecting to chain: {} ({})",
        config.kind(),
        config.network()
    );
    println!("Signing domain     : {}", config.signing_domain());
    println!(
        "Native coin symbol : {}",
        config.framework().native_coin_symbol
    );

    let client = MoveClient::new(config)?;
    let account = Ed25519Account::generate();
    println!("Generated account: {}", account.address());

    match client.get_balance(account.address()).await {
        Ok(balance) => println!("Balance: {balance} octas"),
        Err(MoveError::Api {
            status_code: 404, ..
        }) => {
            println!("Account not yet on-chain (expected for a fresh account)");
        }
        Err(MoveError::Http(e)) => {
            println!("HTTP error (expected for a localnet that is not running): {e}");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
