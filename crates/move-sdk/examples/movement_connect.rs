//! Example: Connect to Movement using the move-rust-sdk.
//!
//! This example shows how to use the chain-agnostic `MoveConfig` / `MoveClient`
//! surface to talk to the Movement blockchain. The `movement` feature
//! must be enabled.
//!
//! Run with:
//!
//! ```text
//! cargo run -p move-rust-sdk --example movement_connect --features "movement,ed25519"
//! ```

use move_core_sdk::account::Ed25519Account;
use move_core_sdk::types::AccountAddress;
#[cfg(feature = "movement")]
use {
    move_rust_sdk::MoveClient, move_rust_sdk::MoveError, move_rust_sdk::MoveResult,
    move_rust_sdk::MovementConfig,
};

#[cfg(feature = "movement")]
#[tokio::main]
async fn main() -> MoveResult<()> {
    // 1. Construct a chain-agnostic MoveConfig targeting Movement testnet.
    let config = MovementConfig::testnet();
    println!(
        "Connecting to chain: {} ({})",
        config.kind(),
        config.network()
    );
    println!("Fullnode URL       : {}", config.fullnode_url());
    println!("Chain ID           : {}", config.chain_id().id());
    println!("Signing domain     : {}", config.signing_domain());
    println!(
        "Native coin symbol : {}",
        config.framework().native_coin_symbol
    );

    // 2. Build the chain-agnostic MoveClient.
    let client = MoveClient::new(config)?;

    // 3. Use the client. The active chain is `movement`, so balances,
    //    transactions, etc. are reported on the Movement testnet.
    let account = Ed25519Account::generate();
    let _address: AccountAddress = account.address();
    println!("Generated account: {}", account.address());

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

// If the `movement` feature is not enabled, print a helpful message
// and exit successfully so the rest of the workspace can still be
// built.
#[cfg(not(feature = "movement"))]
fn main() {
    eprintln!(
        "This example requires the `movement` feature.\n\
         Run with: cargo run -p move-rust-sdk --example movement_connect --features \"movement,ed25519\""
    );
}
