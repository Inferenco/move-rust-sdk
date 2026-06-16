# Changelog

All notable changes to `move-rust-sdk` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-16

### Added

- **Initial release of the chain-agnostic `move-rust-sdk` crate.**
- **`MoveConfig`** — chain-agnostic configuration type. Replaces
  `AptosConfig` for callers who want to write code that targets any
  Move-based blockchain. Wraps the underlying
  [`move_core_sdk::config::AptosConfig`].
- **`MoveClient`** — chain-agnostic client. A thin wrapper around
  [`move_core_sdk::Aptos`] that adds chain-specific context (signing
  domain, framework addresses, native coin type) from the
  `MoveConfig`. The wrapper is what gives the `move-rust-sdk` crate
  a chain-agnostic surface: every chain configuration
  (`AptosConfig`, `MovementConfig`, `MoveConfig::custom`) goes
  through this type.
- **`AptosConfig`** (gated by the `aptos` feature) — pre-built
  configurations for the Aptos chain (mainnet, testnet, devnet, local).
  Encodes the official endpoints, chain IDs, and signing domain
  (`"APTOS::RawTransaction"`). When the `aptos` feature is **not**
  enabled, the `AptosConfig` type and its constructors are not
  available — callers must build an equivalent
  `MoveConfig::custom(..., ChainPreset::Aptos)` themselves.
- **`MovementConfig`** (gated by the `movement` feature) — pre-built
  configurations for the Movement chain (mainnet, testnet, local).
  Encodes the official Movement endpoints, chain IDs, and signing
  domain (`"MOVEMENT::RawTransaction"`). When the `movement` feature
  is **not** enabled, the `MovementConfig` type and its constructors
  are not available — callers must build an equivalent
  `MoveConfig::custom(..., ChainPreset::Movement)` themselves.
- **`ChainPreset`** — `Aptos` and `Movement` presets that select the
  signing domain and native coin symbol for `MoveConfig::custom_with_preset`.
- **`ChainKind`** and **`Network`** enums — runtime introspection of
  the active chain and network.
- **`FrameworkFunctions`** — well-known framework function IDs,
  addresses, and the signing domain for a given chain. The
  `aptos()` and `movement()` constructors encode the framework
  layout for each chain.
- **`MoveConfigBuilder`** — an imperative alternative to
  `MoveConfig::custom` and the `with_*` methods.
- **`MoveClient::sign_transaction(raw, account)`** — sign a
  `RawTransaction` with the active chain's signing domain.
- **Examples:**
  - `aptos_connect` — connect to Aptos testnet (requires `aptos` feature)
  - `movement_connect` — connect to Movement testnet (requires `movement` feature)
  - `custom_chain` — connect to a developer-defined Move-based chain

### Changed

- The default `move-rust-sdk` feature set enables the
  **chain-agnostic core** (signing primitives, transaction types,
  network client, `MoveConfig::custom`, `MoveClient`) but **does
  not** enable any pre-built chain configuration. To use
  `AptosConfig::*` constructors, enable the `aptos` feature. To
  use `MovementConfig::*` constructors, enable the `movement`
  feature. To enable both, use `all-chains`. Without any chain
  feature, the developer must use `MoveConfig::custom(..., ChainPreset::*)`
  to target any chain.
- The underlying `aptos-sdk` crate gained new `*_with_domain` methods
  on `RawTransaction`, `RawTransactionOrderless`,
  `MultiAgentRawTransaction`, `FeePayerRawTransaction`, and the
  `sign_*` helpers in `transaction::builder` and
  `transaction::sponsored`. These accept a caller-supplied signing
  domain so that the same BCS-encoded transaction can be signed for
  any Move-based chain. The default (`signing_message()` and
  `sign_transaction()`) is unchanged.
- The `aptos-sdk` `AptosConfig` gained a `set_chain_id(chain_id)`
  builder method so that non-Aptos chain IDs (e.g. Movement's
  `126` for mainnet, `250` for testnet) can be injected into an
  otherwise Aptos-shaped configuration. The default (chain ID
  derived from the `Network` variant) is unchanged.
- The `aptos-sdk` `AptosConfig` gained a `from_parts(fullnode_url,
  indexer_url, faucet_url)` constructor so the `move-rust-sdk` crate
  can bridge a chain-agnostic `MoveConfig` into an `AptosConfig`
  without `pub(crate)` field access.

### Security

- All URL inputs to `MoveConfig::custom*` and `with_*_url` are
  validated with the same scheme allowlist as the underlying
  `aptos-sdk::config::validate_url_scheme` (`http` and `https` only).
  This prevents SSRF attacks via dangerous URL schemes like
  `file://`, `gopher://`, etc.
- The signing domain for each pre-built chain is hard-coded in the
  crate. Developers who construct a custom `MoveConfig` from scratch
  are responsible for picking a signing domain that matches the
  on-chain Move verifier on the target chain. An incorrect signing
  domain will produce signatures that the on-chain verifier rejects
  as `INVALID_SIGNATURE`.
