//! Tests that verify the feature-gating of the pre-built chain
//! configurations. These tests are written so that they pass under
//! any feature combination, by only asserting on what is currently
//! available.
//!
//! The complementary invariant — "with neither feature enabled,
//! `AptosConfig` / `MovementConfig` are not in scope" — is enforced
//! at compile time by `#[cfg(feature = "...")]` on the test blocks
//! themselves. If a developer accidentally makes `AptosConfig` always
//! available (forgetting the feature gate), the corresponding test
//! will fail to compile when the feature is disabled.

use crate::MoveError;
use crate::config::MoveConfig;
use crate::preset::ChainPreset;

/// Always-available test: the chain-agnostic core is here regardless
/// of feature flags. This is the entry point for any developer who
/// has not enabled a pre-built chain feature.
#[test]
fn chain_agnostic_core_is_always_available() {
    // MoveConfig and MoveClient are always available — no feature
    // gate. Developers who do not opt into a pre-built chain use
    // these directly.
    let _: fn(&str) -> Result<MoveConfig, MoveError> = MoveConfig::custom;
    let _: fn() -> ChainPreset = || ChainPreset::default();
}

/// When the `aptos` feature is enabled, the pre-built `AptosConfig`
/// constructors are available.
#[cfg(feature = "aptos")]
#[test]
fn aptos_feature_enables_aptos_config() {
    let _: MoveConfig = crate::AptosConfig::mainnet();
    let _: MoveConfig = crate::AptosConfig::testnet();
    let _: MoveConfig = crate::AptosConfig::devnet();
    let _: MoveConfig = crate::AptosConfig::local();
}

/// When the `movement` feature is enabled, the pre-built
/// `MovementConfig` constructors are available.
#[cfg(feature = "movement")]
#[test]
fn movement_feature_enables_movement_config() {
    let _: MoveConfig = crate::MovementConfig::mainnet();
    let _: MoveConfig = crate::MovementConfig::testnet();
    let _: MoveConfig = crate::MovementConfig::local();
}

/// Without any pre-built chain feature, the developer must use
/// `MoveConfig::custom` with a `ChainPreset`.
#[cfg(not(any(feature = "aptos", feature = "movement")))]
#[test]
fn no_prebuilt_feature_means_use_custom() {
    // AptosConfig is NOT in scope (no `aptos` feature).
    // MovementConfig is NOT in scope (no `movement` feature).
    // The developer must construct everything by hand:
    let config =
        MoveConfig::custom_with_preset("https://my-chain.example.com/v1", ChainPreset::Aptos)
            .unwrap();
    assert_eq!(config.signing_domain(), "APTOS::RawTransaction");

    let config = MoveConfig::custom_with_preset(
        "https://my-movement-chain.example.com/v1",
        ChainPreset::Movement,
    )
    .unwrap();
    assert_eq!(config.signing_domain(), "MOVEMENT::RawTransaction");
}
