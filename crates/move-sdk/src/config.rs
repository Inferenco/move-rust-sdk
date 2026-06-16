//! Chain-agnostic configuration for the Move Rust SDK.
//!
//! [`MoveConfig`] is the unified configuration type. It wraps the underlying
//! chain-specific configuration (an [`AptosConfig`] or a [`MovementConfig`]
//! at runtime, depending on which feature is enabled) and provides a
//! consistent surface so that the same code can target any Move-based
//! blockchain.
//!
//! ## Choosing a chain
//!
//! Three idiomatic ways to construct a [`MoveConfig`]:
//!
//! 1. **Pre-built chain** — [`AptosConfig::testnet`] or
//!    [`MovementConfig::testnet`] (the latter is only available with the
//!    `movement` feature enabled).
//! 2. **Custom** — [`MoveConfig::custom`] for a developer-supplied
//!    fullnode URL and [`ChainPreset`].
//! 3. **From a pre-built, then overridden** — start with a pre-built
//!    config and call the `with_*` builder methods to customise it.
//!
//! ## Examples
//!
//! ```rust
//! use move_rust_sdk::MoveConfig;
//!
//! // Point at any Move-based chain
//! let cfg = MoveConfig::custom("https://my-chain.example.com/v1").unwrap();
//! ```

use std::time::Duration;
use url::Url;

use move_core_sdk::config::PoolConfig;
use move_core_sdk::error::{AptosError, AptosResult};
use move_core_sdk::retry::RetryConfig;
use move_core_sdk::types::ChainId;

pub use move_core_sdk::config::{read_response_bounded, validate_url_scheme};

use crate::preset::ChainPreset;

// ============================================================================
// Public types
// ============================================================================

/// Which pre-built chain a configuration is for.
///
/// `Aptos` and `Movement` are pre-built chains; `Custom` is anything else
/// (a developer-supplied fullnode URL, a localnet, an in-house fork, ...).
///
/// The presence of the `Aptos` and `Movement` variants in this enum does
/// **not** depend on feature flags. Feature flags only control the
/// constructor methods ([`AptosConfig`], [`MovementConfig`]) — once you
/// have a [`MoveConfig`], it carries its kind as a runtime value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChainKind {
    /// The Aptos blockchain (mainnet, testnet, devnet, local).
    Aptos,
    /// The Movement blockchain (mainnet, testnet, devnet, local).
    Movement,
    /// A developer-defined Move-based chain. See [`MoveConfig::custom`].
    Custom,
}

impl ChainKind {
    /// Human-readable name.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainKind::Aptos => "aptos",
            ChainKind::Movement => "movement",
            ChainKind::Custom => "custom",
        }
    }
}

impl std::fmt::Display for ChainKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pre-defined or developer-defined network of a given chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Network {
    /// Production network.
    Mainnet,
    /// Public test network.
    Testnet,
    /// Development network (resets regularly; chain ID is unknown at
    /// compile time and must be resolved from the node).
    Devnet,
    /// A local development network (e.g. `movement node run-localnet`).
    Local,
    /// A developer-defined network — see [`MoveConfig::custom`].
    Custom,
}

impl Network {
    /// Human-readable name.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
            Network::Devnet => "devnet",
            Network::Local => "local",
            Network::Custom => "custom",
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Well-known framework function IDs and types for a given chain.
///
/// Most Move-based chains share the same framework module layout
/// (`0x1::aptos_account`, `0x1::coin`, `0x1::account`, ...). The
/// [`ChainPreset`] for a given chain encodes these well-known paths so
/// that helpers like [`move_core_sdk::transaction::InputEntryFunctionData::transfer_apt`]
/// resolve to the correct addresses for the active chain.
#[derive(Debug, Clone)]
pub struct FrameworkFunctions {
    /// Address of the core framework (typically `0x1`).
    pub framework_address: move_core_sdk::types::AccountAddress,
    /// `"0x1::aptos_account::transfer"`-style string.
    pub native_transfer: String,
    /// `"0x1::coin::transfer"`-style string.
    pub coin_transfer: String,
    /// `"0x1::aptos_account::create_account"`-style string.
    pub create_account: String,
    /// `"0x1::managed_coin::register"`-style string.
    pub register_coin: String,
    /// `"0x1::code::publish_package_txn"`-style string.
    pub publish_package: String,
    /// `"0x1::account::rotate_authentication_key"`-style string.
    pub rotate_auth_key: String,
    /// Type tag for the native gas token, e.g. `"0x1::aptos_coin::AptosCoin"`.
    pub native_coin_type: String,
    /// Display symbol for the native gas token, e.g. `"APT"` or `"MOVE"`.
    pub native_coin_symbol: String,
    /// Domain separator used in the signing message, e.g.
    /// `"APTOS::RawTransaction"` or `"MOVEMENT::RawTransaction"`.
    pub signing_domain: String,
}

impl FrameworkFunctions {
    /// Default for the Aptos chain.
    #[must_use]
    pub fn aptos() -> Self {
        Self {
            framework_address: move_core_sdk::types::AccountAddress::ONE,
            native_transfer: "0x1::aptos_account::transfer".into(),
            coin_transfer: "0x1::coin::transfer".into(),
            create_account: "0x1::aptos_account::create_account".into(),
            register_coin: "0x1::managed_coin::register".into(),
            publish_package: "0x1::code::publish_package_txn".into(),
            rotate_auth_key: "0x1::account::rotate_authentication_key".into(),
            native_coin_type: "0x1::aptos_coin::AptosCoin".into(),
            native_coin_symbol: "APT".into(),
            signing_domain: "APTOS::RawTransaction".into(),
        }
    }

    /// Default for the Movement chain.
    #[must_use]
    pub fn movement() -> Self {
        Self {
            framework_address: move_core_sdk::types::AccountAddress::ONE,
            native_transfer: "0x1::aptos_account::transfer".into(),
            coin_transfer: "0x1::coin::transfer".into(),
            create_account: "0x1::aptos_account::create_account".into(),
            register_coin: "0x1::managed_coin::register".into(),
            publish_package: "0x1::code::publish_package_txn".into(),
            rotate_auth_key: "0x1::account::rotate_authentication_key".into(),
            native_coin_type: "0x1::aptos_coin::AptosCoin".into(),
            native_coin_symbol: "MOVE".into(),
            signing_domain: "MOVEMENT::RawTransaction".into(),
        }
    }
}

// ============================================================================
// MoveConfig — the unified configuration type
// ============================================================================

/// Chain-agnostic configuration for the Move Rust SDK.
///
/// `MoveConfig` is the entry point for both pre-built chains (Aptos and
/// Movement) and developer-defined Move-based chains. The runtime
/// representation carries:
///
/// - a [`ChainKind`] so the SDK can pick the right signing prefix,
///   framework addresses, and well-known function IDs;
/// - a [`Network`] (mainnet / testnet / devnet / local / custom) for
///   chain-ID resolution defaults;
/// - a [`FrameworkFunctions`] block;
/// - the developer-supplied endpoints and client settings.
#[derive(Debug, Clone)]
pub struct MoveConfig {
    pub(crate) kind: ChainKind,
    pub(crate) network: Network,
    pub(crate) fullnode_url: Url,
    pub(crate) indexer_url: Option<Url>,
    pub(crate) faucet_url: Option<Url>,
    pub(crate) chain_id: u8,
    pub(crate) framework: FrameworkFunctions,
    pub(crate) timeout: Duration,
    pub(crate) retry_config: RetryConfig,
    pub(crate) pool_config: PoolConfig,
    pub(crate) api_key: Option<String>,
    /// The signing domain prefix for the active chain. Most callers do
    /// not need this; the SDK injects it automatically when signing
    /// transactions. It is exposed so that callers building transactions
    /// off-chain (e.g. hardware wallets) can reconstruct the signing
    /// message deterministically.
    pub signing_domain: String,
}

impl MoveConfig {
    /// Construct a configuration for a developer-defined Move-based
    /// chain.
    ///
    /// # Security
    ///
    /// Only `http://` and `https://` URL schemes are allowed. HTTPS is
    /// strongly recommended for production. HTTP is acceptable only for
    /// localhost development environments.
    ///
    /// # Errors
    ///
    /// Returns an error if the `fullnode_url` cannot be parsed as a
    /// valid URL or uses an unsupported scheme (e.g. `file://`,
    /// `ftp://`).
    pub fn custom(fullnode_url: &str) -> AptosResult<Self> {
        Self::custom_with_preset(fullnode_url, ChainPreset::default())
    }

    /// Like [`MoveConfig::custom`] but with a developer-supplied
    /// [`ChainPreset`] so the SDK can pick the right signing domain and
    /// framework addresses for the chain.
    ///
    /// # Errors
    ///
    /// Returns an error if the `fullnode_url` cannot be parsed as a
    /// valid URL or uses an unsupported scheme.
    pub fn custom_with_preset(fullnode_url: &str, preset: ChainPreset) -> AptosResult<Self> {
        let url = Url::parse(fullnode_url)?;
        validate_url_scheme(&url)?;

        let framework = match preset {
            ChainPreset::Aptos => FrameworkFunctions::aptos(),
            ChainPreset::Movement => FrameworkFunctions::movement(),
        };
        let signing_domain = framework.signing_domain.clone();

        Ok(Self {
            kind: ChainKind::Custom,
            network: Network::Custom,
            fullnode_url: url,
            indexer_url: None,
            faucet_url: None,
            chain_id: 0, // resolved from the node on first use
            framework,
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            pool_config: PoolConfig::default(),
            api_key: None,
            signing_domain,
        })
    }

    /// The [`ChainKind`] of this configuration.
    #[must_use]
    pub fn kind(&self) -> ChainKind {
        self.kind
    }

    /// The [`Network`] of this configuration.
    #[must_use]
    pub fn network(&self) -> Network {
        self.network
    }

    /// The framework functions and addresses for the active chain.
    #[must_use]
    pub fn framework(&self) -> &FrameworkFunctions {
        &self.framework
    }

    /// The fullnode URL.
    #[must_use]
    pub fn fullnode_url(&self) -> &Url {
        &self.fullnode_url
    }

    /// The indexer URL, if configured.
    #[must_use]
    pub fn indexer_url(&self) -> Option<&Url> {
        self.indexer_url.as_ref()
    }

    /// The faucet URL, if configured.
    #[must_use]
    pub fn faucet_url(&self) -> Option<&Url> {
        self.faucet_url.as_ref()
    }

    /// The configured chain ID. For devnet and custom networks, this
    /// may be `0` (unknown) until the SDK resolves it from the node.
    #[must_use]
    pub fn chain_id(&self) -> ChainId {
        ChainId::new(self.chain_id)
    }

    /// The request timeout.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// The retry configuration.
    #[must_use]
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// The connection-pool configuration.
    #[must_use]
    pub fn pool_config(&self) -> &PoolConfig {
        &self.pool_config
    }

    /// The configured API key, if any.
    #[must_use]
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// The signing domain used for the active chain (e.g.
    /// `"APTOS::RawTransaction"` or `"MOVEMENT::RawTransaction"`).
    #[must_use]
    pub fn signing_domain(&self) -> &str {
        &self.signing_domain
    }

    // ---- builder methods ---------------------------------------------------

    /// Set the request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the retry configuration.
    #[must_use]
    pub fn with_retry(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Disable automatic retry. Equivalent to
    /// `with_retry(RetryConfig::no_retry())`.
    #[must_use]
    pub fn without_retry(mut self) -> Self {
        self.retry_config = RetryConfig::no_retry();
        self
    }

    /// Set the connection-pool configuration.
    #[must_use]
    pub fn with_pool(mut self, pool_config: PoolConfig) -> Self {
        self.pool_config = pool_config;
        self
    }

    /// Set an API key.
    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set a custom indexer URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed or uses an
    /// unsupported scheme.
    pub fn with_indexer_url(mut self, url: &str) -> AptosResult<Self> {
        let parsed = Url::parse(url)?;
        validate_url_scheme(&parsed)?;
        self.indexer_url = Some(parsed);
        Ok(self)
    }

    /// Set a custom faucet URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed or uses an
    /// unsupported scheme.
    pub fn with_faucet_url(mut self, url: &str) -> AptosResult<Self> {
        let parsed = Url::parse(url)?;
        validate_url_scheme(&parsed)?;
        self.faucet_url = Some(parsed);
        Ok(self)
    }

    /// Convert this `MoveConfig` into the underlying
    /// [`move_core_sdk::config::AptosConfig`] used by the implementation.
    ///
    /// This is the bridge between the chain-agnostic `MoveConfig`
    /// surface and the underlying `move-core-sdk` configuration. The
    /// returned `AptosConfig` carries the same URLs, chain ID, and
    /// framework settings.
    ///
    /// # Panics
    ///
    /// Panics if any of the URLs use an unsupported scheme. This
    /// cannot happen for `MoveConfig` values constructed via
    /// [`MoveConfig::custom`], [`MoveConfig::custom_with_preset`],
    /// [`MoveConfigBuilder`], or any of the pre-built chain
    /// constructors — all of those validate URLs eagerly. It is
    /// only reachable if the caller bypasses the safe constructor
    /// (e.g. by mutating the struct fields directly).
    #[must_use]
    pub fn into_aptos_config(self) -> move_core_sdk::config::AptosConfig {
        let mut out = move_core_sdk::config::AptosConfig::from_parts(
            self.fullnode_url,
            self.indexer_url,
            self.faucet_url,
        )
        .expect("URLs already validated by MoveConfig constructor")
        .with_timeout(self.timeout)
        .with_retry(self.retry_config)
        .with_pool(self.pool_config);

        if let Some(key) = self.api_key {
            out = out.with_api_key(key);
        }
        if self.chain_id != 0 {
            out = out.set_chain_id(ChainId::new(self.chain_id));
        }
        out
    }
}

// ============================================================================
// Pre-built chain configurations
// ============================================================================

/// Pre-built configurations for the Aptos chain.
#[cfg(feature = "aptos")]
#[derive(Debug)]
pub struct AptosConfig;

#[cfg(feature = "aptos")]
impl AptosConfig {
    /// Configuration for the Aptos mainnet.
    ///
    /// # Panics
    ///
    /// Panics if any of the hard-coded Aptos mainnet URLs cannot
    /// be parsed. This is a compile-time bug — the URLs are
    /// constants checked into the source tree.
    #[must_use]
    pub fn mainnet() -> MoveConfig {
        MoveConfig {
            kind: ChainKind::Aptos,
            network: Network::Mainnet,
            fullnode_url: Url::parse("https://fullnode.mainnet.aptoslabs.com/v1")
                .expect("valid mainnet URL"),
            indexer_url: Some(
                Url::parse("https://indexer.mainnet.aptoslabs.com/v1/graphql")
                    .expect("valid indexer URL"),
            ),
            faucet_url: None,
            chain_id: 1,
            framework: FrameworkFunctions::aptos(),
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::conservative(),
            pool_config: PoolConfig::default(),
            api_key: None,
            signing_domain: FrameworkFunctions::aptos().signing_domain,
        }
    }

    /// Configuration for the Aptos testnet.
    ///
    /// # Panics
    ///
    /// Panics if any of the hard-coded Aptos testnet URLs cannot
    /// be parsed. This is a compile-time bug — the URLs are
    /// constants checked into the source tree.
    #[must_use]
    pub fn testnet() -> MoveConfig {
        MoveConfig {
            kind: ChainKind::Aptos,
            network: Network::Testnet,
            fullnode_url: Url::parse("https://fullnode.testnet.aptoslabs.com/v1")
                .expect("valid testnet URL"),
            indexer_url: Some(
                Url::parse("https://indexer.testnet.aptoslabs.com/v1/graphql")
                    .expect("valid indexer URL"),
            ),
            faucet_url: Some(
                Url::parse("https://faucet.testnet.aptoslabs.com").expect("valid faucet URL"),
            ),
            chain_id: 2,
            framework: FrameworkFunctions::aptos(),
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            pool_config: PoolConfig::default(),
            api_key: None,
            signing_domain: FrameworkFunctions::aptos().signing_domain,
        }
    }

    /// Configuration for the Aptos devnet.
    ///
    /// The devnet chain ID is resolved on first use — the SDK fetches
    /// the live value from the fullnode.
    ///
    /// # Panics
    ///
    /// Panics if any of the hard-coded Aptos devnet URLs cannot
    /// be parsed. This is a compile-time bug — the URLs are
    /// constants checked into the source tree.
    #[must_use]
    pub fn devnet() -> MoveConfig {
        MoveConfig {
            kind: ChainKind::Aptos,
            network: Network::Devnet,
            fullnode_url: Url::parse("https://fullnode.devnet.aptoslabs.com/v1")
                .expect("valid devnet URL"),
            indexer_url: Some(
                Url::parse("https://indexer.devnet.aptoslabs.com/v1/graphql")
                    .expect("valid indexer URL"),
            ),
            faucet_url: Some(
                Url::parse("https://faucet.devnet.aptoslabs.com").expect("valid faucet URL"),
            ),
            chain_id: 0,
            framework: FrameworkFunctions::aptos(),
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            pool_config: PoolConfig::default(),
            api_key: None,
            signing_domain: FrameworkFunctions::aptos().signing_domain,
        }
    }

    /// Configuration for a local Aptos node.
    ///
    /// # Panics
    ///
    /// Panics if the hard-coded local URL cannot be parsed. This is
    /// a compile-time bug — the URL is a constant checked into the
    /// source tree.
    #[must_use]
    pub fn local() -> MoveConfig {
        MoveConfig {
            kind: ChainKind::Aptos,
            network: Network::Local,
            fullnode_url: Url::parse("http://127.0.0.1:8080/v1").expect("valid local URL"),
            indexer_url: None,
            faucet_url: Some(Url::parse("http://127.0.0.1:8081").expect("valid local faucet URL")),
            chain_id: 4,
            framework: FrameworkFunctions::aptos(),
            timeout: Duration::from_secs(10),
            retry_config: RetryConfig::aggressive(),
            pool_config: PoolConfig::low_latency(),
            api_key: None,
            signing_domain: FrameworkFunctions::aptos().signing_domain,
        }
    }
}

/// Pre-built configurations for the Movement chain.
///
/// Endpoints come from the official Movement documentation
/// (<https://docs.movementnetwork.xyz/devs/networkEndpoints>).
#[cfg(feature = "movement")]
#[derive(Debug)]
pub struct MovementConfig;

#[cfg(feature = "movement")]
impl MovementConfig {
    /// Configuration for the Movement mainnet (chain ID `126`).
    ///
    /// # Panics
    ///
    /// Panics if any of the hard-coded Movement mainnet URLs cannot
    /// be parsed. This is a compile-time bug — the URLs are
    /// constants checked into the source tree.
    #[must_use]
    pub fn mainnet() -> MoveConfig {
        MoveConfig {
            kind: ChainKind::Movement,
            network: Network::Mainnet,
            fullnode_url: Url::parse("https://mainnet.movementnetwork.xyz/v1")
                .expect("valid mainnet URL"),
            indexer_url: Some(
                Url::parse("https://indexer.mainnet.movementnetwork.xyz/v1/graphql")
                    .expect("valid indexer URL"),
            ),
            faucet_url: None,
            chain_id: 126,
            framework: FrameworkFunctions::movement(),
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::conservative(),
            pool_config: PoolConfig::default(),
            api_key: None,
            signing_domain: FrameworkFunctions::movement().signing_domain,
        }
    }

    /// Configuration for the Movement testnet (chain ID `250`).
    ///
    /// # Panics
    ///
    /// Panics if any of the hard-coded Movement testnet URLs cannot
    /// be parsed. This is a compile-time bug — the URLs are
    /// constants checked into the source tree.
    #[must_use]
    pub fn testnet() -> MoveConfig {
        MoveConfig {
            kind: ChainKind::Movement,
            network: Network::Testnet,
            fullnode_url: Url::parse("https://testnet.movementnetwork.xyz/v1")
                .expect("valid testnet URL"),
            indexer_url: Some(
                Url::parse("https://hasura.testnet.movementnetwork.xyz/v1/graphql")
                    .expect("valid indexer URL"),
            ),
            faucet_url: Some(
                Url::parse("https://faucet.testnet.movementnetwork.xyz/")
                    .expect("valid faucet URL"),
            ),
            chain_id: 250,
            framework: FrameworkFunctions::movement(),
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            pool_config: PoolConfig::default(),
            api_key: None,
            signing_domain: FrameworkFunctions::movement().signing_domain,
        }
    }

    /// Configuration for a local Movement node. The default ports
    /// match the official `movement node run-localnet` command.
    ///
    /// # Panics
    ///
    /// Panics if the hard-coded local URL cannot be parsed. This is
    /// a compile-time bug — the URL is a constant checked into the
    /// source tree.
    #[must_use]
    pub fn local() -> MoveConfig {
        MoveConfig {
            kind: ChainKind::Movement,
            network: Network::Local,
            fullnode_url: Url::parse("http://127.0.0.1:8080/v1").expect("valid local URL"),
            indexer_url: None,
            faucet_url: Some(Url::parse("http://127.0.0.1:8081").expect("valid local faucet URL")),
            chain_id: 0,
            framework: FrameworkFunctions::movement(),
            timeout: Duration::from_secs(10),
            retry_config: RetryConfig::aggressive(),
            pool_config: PoolConfig::low_latency(),
            api_key: None,
            signing_domain: FrameworkFunctions::movement().signing_domain,
        }
    }
}

// ============================================================================
// Builder
// ============================================================================

/// A more imperative alternative to [`MoveConfig::custom`] / the
/// `with_*` builder methods.
#[derive(Debug, Clone, Default)]
pub struct MoveConfigBuilder {
    fullnode_url: Option<Url>,
    indexer_url: Option<Url>,
    faucet_url: Option<Url>,
    chain_id: Option<u8>,
    framework: Option<FrameworkFunctions>,
    timeout: Option<Duration>,
    retry_config: Option<RetryConfig>,
    pool_config: Option<PoolConfig>,
    api_key: Option<String>,
}

impl MoveConfigBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the fullnode URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed or uses an
    /// unsupported scheme.
    pub fn fullnode_url(mut self, url: &str) -> AptosResult<Self> {
        let parsed = Url::parse(url)?;
        validate_url_scheme(&parsed)?;
        self.fullnode_url = Some(parsed);
        Ok(self)
    }

    /// Set the indexer URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed or uses an
    /// unsupported scheme.
    pub fn indexer_url(mut self, url: &str) -> AptosResult<Self> {
        let parsed = Url::parse(url)?;
        validate_url_scheme(&parsed)?;
        self.indexer_url = Some(parsed);
        Ok(self)
    }

    /// Set the faucet URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed or uses an
    /// unsupported scheme.
    pub fn faucet_url(mut self, url: &str) -> AptosResult<Self> {
        let parsed = Url::parse(url)?;
        validate_url_scheme(&parsed)?;
        self.faucet_url = Some(parsed);
        Ok(self)
    }

    /// Set the chain ID. If unset, defaults to `0` (resolved from the
    /// node).
    #[must_use]
    pub fn chain_id(mut self, id: u8) -> Self {
        self.chain_id = Some(id);
        self
    }

    /// Set the framework functions for the chain.
    #[must_use]
    pub fn framework(mut self, framework: FrameworkFunctions) -> Self {
        self.framework = Some(framework);
        self
    }

    /// Set the request timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the retry configuration.
    #[must_use]
    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.retry_config = Some(retry);
        self
    }

    /// Set the connection-pool configuration.
    #[must_use]
    pub fn pool(mut self, pool: PoolConfig) -> Self {
        self.pool_config = Some(pool);
        self
    }

    /// Set an API key.
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Build the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if `fullnode_url` is not set or uses an
    /// unsupported scheme.
    pub fn build(self) -> AptosResult<MoveConfig> {
        let fullnode_url = self
            .fullnode_url
            .ok_or_else(|| AptosError::Config("fullnode_url is required".into()))?;
        let framework = self.framework.unwrap_or_else(FrameworkFunctions::aptos);
        let chain_id = self.chain_id.unwrap_or(0);

        Ok(MoveConfig {
            kind: ChainKind::Custom,
            network: Network::Custom,
            fullnode_url,
            indexer_url: self.indexer_url,
            faucet_url: self.faucet_url,
            chain_id,
            signing_domain: framework.signing_domain.clone(),
            framework,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            retry_config: self.retry_config.unwrap_or_default(),
            pool_config: self.pool_config.unwrap_or_default(),
            api_key: self.api_key,
        })
    }
}
