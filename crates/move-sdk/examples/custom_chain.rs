//! Example: Connect to a custom Move-based chain.
//!
//! This example shows how to use `MoveConfig::custom` (or
//! `MoveConfig::custom_with_preset`) to talk to a developer-defined
//! Move-based chain. The `ChainPreset` selects the signing domain
//! and the native coin symbol.
//!
//! Run with:
//!
//! ```text
//! cargo run -p move-rust-sdk --example custom_chain --features "ed25519"
//! ```

use move_core_sdk::account::Ed25519Account;
use move_rust_sdk::{ChainPreset, MoveClient, MoveConfig, MoveError, MoveResult};

#[tokio::main]
async fn main() -> MoveResult<()> {
    // 1. Pick a `ChainPreset`. For chains that re-use the Aptos
    //    wire format, `ChainPreset::Aptos` is the right default.
    //    For chains that re-use the Movement wire format, use
    //    `ChainPreset::Movement`. (The framework function IDs and
    //    framework addresses are the same for both today; the
    //    difference is the signing domain and the native coin
    //    symbol.)
    let preset = ChainPreset::Movement;

    // 2. Construct a custom MoveConfig pointing at any fullnode URL.
    let config = MoveConfig::custom_with_preset("http://127.0.0.1:8080/v1", preset)?;
    println!(
        "Connecting to chain: {} ({})",
        config.kind(),
        config.network()
    );
    println!("Fullnode URL       : {}", config.fullnode_url());
    println!(
        "Chain ID           : {} (resolved from the node on first use)",
        config.chain_id().id()
    );
    println!("Signing domain     : {}", config.signing_domain());
    println!(
        "Native coin symbol : {}",
        config.framework().native_coin_symbol
    );

    // 3. Build the client. This will fail at request time (we are
    //    not running a real localnet), but the construction succeeds
    //    and the signing domain is correctly set.
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
