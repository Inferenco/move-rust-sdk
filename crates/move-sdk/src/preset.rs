//! [`ChainPreset`] — the chain-specific defaults applied when a
//! developer constructs a [`MoveConfig`](crate::MoveConfig) for a
//! chain that does not have a pre-built configuration (e.g. a private
//! fork).
//!
//! A `ChainPreset` carries the framework addresses, signing domain,
//! and well-known function IDs for a given chain. It is **not** a
//! configuration in itself — the developer still supplies the
//! fullnode URL via [`MoveConfig::custom_with_preset`](crate::MoveConfig::custom_with_preset).

/// A pre-defined set of chain-specific defaults.
///
/// Use this when constructing a [`MoveConfig`](crate::MoveConfig) for a
/// chain that is not in the pre-built list but re-uses the Aptos wire
/// format (e.g. a private Movement fork or a sidechain that has
/// already deployed the `0x1` framework).
///
/// `Default` is [`ChainPreset::Aptos`] — the most common case is "a
/// Move-based chain that looks like Aptos but with a different chain
/// ID".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ChainPreset {
    /// Aptos-compatible chain (default). Signs with
    /// `"APTOS::RawTransaction"`, native coin is `0x1::aptos_coin::AptosCoin`.
    #[default]
    Aptos,
    /// Movement-compatible chain. Signs with
    /// `"MOVEMENT::RawTransaction"`, native coin is `0x1::aptos_coin::AptosCoin`
    /// (re-using the Aptos framework module).
    Movement,
}
