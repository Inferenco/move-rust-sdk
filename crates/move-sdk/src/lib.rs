//! # Move Rust SDK
//!
//! A chain-agnostic Rust SDK for the [Move language] ecosystem — works with
//! [Movement], [Aptos], and any Move-based blockchain.
//!
//! This crate is a thin, unified configuration and naming layer on top of
//! [`move_core_sdk`]. The underlying cryptography, transaction building, and
//! network client code is shared; what changes between chains is a
//! developer-supplied [`MoveConfig`].
//!
//! ## Quick start
//!
//! ### Connecting to a pre-built chain (Aptos)
//!
//! Requires the `aptos` feature. The example is shown compiled
//! but not executed (`no_run`) because it makes a real network
//! request.
//!
//! ```rust,no_run
//! # #[cfg(feature = "aptos")]
//! # {
//! use move_rust_sdk::{AptosConfig, MoveClient};
//! use move_core_sdk::types::AccountAddress;
//!
//! #[tokio::main]
//! async fn main() -> move_rust_sdk::MoveResult<()> {
//!     // AptosTestnet is one of the pre-built chains
//!     let client = MoveClient::new(AptosConfig::testnet())?;
//!     let address = AccountAddress::from_hex("0x1")?;
//!     let balance = client.get_balance(address).await?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### Connecting to a pre-built chain (Movement)
//!
//! Requires the `movement` feature. The example is shown compiled
//! but not executed (`no_run`) because it makes a real network
//! request.
//!
//! ```rust,no_run
//! # #[cfg(feature = "movement")]
//! # {
//! use move_rust_sdk::{MoveClient, MovementConfig};
//! use move_core_sdk::types::AccountAddress;
//!
//! #[tokio::main]
//! async fn main() -> move_rust_sdk::MoveResult<()> {
//!     // MovementTestnet is a pre-built chain
//!     let client = MoveClient::new(MovementConfig::testnet())?;
//!     let address = AccountAddress::from_hex("0x1")?;
//!     let balance = client.get_balance(address).await?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### Connecting to a custom Move chain
//!
//! ```rust
//! use move_rust_sdk::{ChainPreset, MoveClient, MoveConfig};
//!
//! let config = MoveConfig::custom_with_preset(
//!     "https://my-chain.example.com/v1",
//!     ChainPreset::Aptos,
//! ).unwrap();
//! let client = MoveClient::new(config).unwrap();
//! ```
//!
//! ## Feature flags
//!
//! The chain-agnostic core — `MoveConfig`, `MoveClient`, the
//! `ChainPreset` enum, the underlying transaction and cryptography
//! types — is always available.
//!
//! Pre-built chain configurations are **opt-in**. Without any
//! pre-built chain feature, the developer must construct a
//! `MoveConfig::custom(url, ChainPreset::*)` to target any chain.
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `aptos` | No | Enable the pre-built Aptos chain configurations (`AptosConfig::mainnet`, `AptosConfig::testnet`, ...) |
//! | `movement` | No | Enable the pre-built Movement chain configurations (`MovementConfig::mainnet`, `MovementConfig::testnet`, ...) |
//! | `all-chains` | No | Enable both `aptos` and `movement` |
//! | `ed25519` | Yes | Ed25519 signature scheme |
//! | `secp256k1` | Yes | Secp256k1 ECDSA signatures |
//! | `secp256r1` | Yes | Secp256r1 (P-256) ECDSA signatures |
//! | `mnemonic` | Yes | BIP-39 mnemonic phrase support |
//! | `indexer` | Yes | GraphQL indexer client |
//! | `faucet` | Yes | Faucet integration for testnets |
//! | `bls` | No | BLS12-381 signatures |
//! | `keyless` | No | OIDC-based keyless authentication |
//! | `macros` | No | Proc macros for type-safe contract bindings |
//!
//! ### Targeting a chain without a pre-built feature
//!
//! If neither `aptos` nor `movement` is enabled, the developer
//! constructs a chain configuration explicitly:
//!
//! ```rust
//! use move_rust_sdk::{ChainPreset, MoveConfig};
//!
//! // Target any Aptos-compatible chain (signs with
//! // "APTOS::RawTransaction").
//! let config = MoveConfig::custom_with_preset(
//!     "https://my-aptos-compatible-chain.example.com/v1",
//!     ChainPreset::Aptos,
//! ).unwrap();
//! assert_eq!(config.signing_domain(), "APTOS::RawTransaction");
//! ```
//!
//! Wrap the configuration in a `MoveClient` to make network calls:
//!
//! ```rust
//! use move_rust_sdk::{ChainPreset, MoveClient, MoveConfig};
//!
//! let config = MoveConfig::custom_with_preset(
//!     "https://my-aptos-compatible-chain.example.com/v1",
//!     ChainPreset::Aptos,
//! ).unwrap();
//! let client = MoveClient::new(config).unwrap();
//! ```
//!
//! [Move language]: https://move-language.github.io/move/
//! [Movement]: https://movementnetwork.xyz
//! [Aptos]: https://aptosfoundation.org

#![cfg_attr(docsrs, feature(doc_cfg))]

// === Re-exports from move-core-sdk ===
//
// The underlying chain-agnostic implementation lives in move-core-sdk.
// The move-rust-sdk crate re-exports it and adds a unified configuration
// layer so the same code works against any Move-based chain.
//
// move_core_sdk::* items are also re-exported directly so downstream
// code can `use move_rust_sdk::account::Ed25519Account` without
// reaching into the underlying crate.

pub use move_core_sdk;

// === Chain-agnostic types ===

mod client;
mod config;
mod preset;

pub use client::MoveClient;
pub use config::{
    ChainKind, FrameworkFunctions, MoveConfig, MoveConfigBuilder, Network, read_response_bounded,
    validate_url_scheme,
};
pub use preset::ChainPreset;

// === Pre-built chain configurations (gated by feature flags) ===

#[cfg(feature = "aptos")]
pub use config::AptosConfig;

#[cfg(feature = "movement")]
pub use config::MovementConfig;

// === Common re-exports ===

pub use move_core_sdk::account;
pub use move_core_sdk::api;
pub use move_core_sdk::crypto;
pub use move_core_sdk::error::AptosError as MoveError;
pub use move_core_sdk::transaction;
pub use move_core_sdk::types;
pub use move_core_sdk::types::{AccountAddress, ChainId, HashValue};

/// A chain-agnostic `Result` type for the move-rust-sdk.
///
/// This is a type alias for the underlying [`MoveError`] error type.
pub type MoveResult<T> = ::std::result::Result<T, MoveError>;

// Re-export proc macros when the feature is enabled
#[cfg(feature = "macros")]
pub use move_core_sdk_macros::{MoveStruct, aptos_contract, aptos_contract_file};

#[cfg(test)]
mod tests;
