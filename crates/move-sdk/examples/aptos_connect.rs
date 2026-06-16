//! Example: Connect to a pre-built chain (Aptos) using the move-rust-sdk.
//!
//! This example shows how to use the chain-agnostic `MoveConfig` / `MoveClient`
//! surface to talk to a pre-built chain (Aptos testnet). The same code,
//! with a one-line change to use `MovementConfig::testnet()`, targets
//! Movement testnet instead — no other code needs to change.
//!
//! Run with:
//!
//! ```text
//! cargo run -p move-rust-sdk --example aptos_connect --features "aptos,ed25519"
//! ```

use move_core_sdk::account::Ed25519Account;
use move_core_sdk::types::AccountAddress;
use move_rust_sdk::{AptosConfig, MoveClient, MoveError, MoveResult};

#[tokio::main]
async fn main() -> MoveResult<()> {
    // 1. Construct a chain-agnostic MoveConfig. The `aptos` feature
    //    enables the `AptosConfig` helper; the `movement` feature
    //    enables `MovementConfig` (see `movement_connect.rs`).
    let config = AptosConfig::testnet();
    println!(
        "Connecting to chain: {} ({})",
        config.kind(),
        config.network()
    );
    println!("Fullnode URL       : {}", config.fullnode_url());
    println!("Chain ID           : {}", config.chain_id().id());
    println!("Signing domain     : {}", config.signing_domain());

    // 2. Build the chain-agnostic MoveClient. This wraps the
    //    underlying move-core-sdk `Aptos` client and adds chain-specific
    //    context (signing domain, framework addresses, native coin).
    let client = MoveClient::new(config)?;

    // 3. Use the client. The active chain is `aptos`, so balances,
    //    transactions, etc. are reported on the Aptos testnet.
    let account = Ed25519Account::generate();
    let _address: AccountAddress = account.address();
    println!("Generated account: {}", account.address());

    // The balance will be 0 (we just created the account) but the
    // call exercises the network plumbing.
    match client.get_balance(account.address()).await {
        Ok(balance) => println!("Balance: {balance} octas"),
        Err(MoveError::Api {
            status_code: 404, ..
        }) => {
            println!("Account not yet on-chain (expected for a fresh account)");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
