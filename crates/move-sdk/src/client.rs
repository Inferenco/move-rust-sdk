//! The chain-agnostic Move client.
//!
//! [`MoveClient`] is a thin wrapper around [`move_core_sdk::Aptos`] that
//! adds the chain-specific context (signing domain, framework
//! addresses, native coin type) from the [`MoveConfig`](crate::MoveConfig).
//!
//! Most of the heavy lifting (network, transaction building,
//! cryptography) is delegated to the underlying `move-core-sdk`. The
//! wrapper exists so that:
//!
//! - the same code can be retargeted at a different chain by swapping
//!   out the [`MoveConfig`](crate::MoveConfig);
//! - the signing prefix is automatically picked from
//!   [`MoveConfig::signing_domain`](crate::MoveConfig::signing_domain) when
//!   the active chain is Movement (or any other non-Aptos chain).

use std::sync::Arc;

use move_core_sdk::api::{FullnodeClient, PendingTransaction};
use move_core_sdk::types::{AccountAddress, ChainId, HashValue};

use crate::MoveError;

use crate::config::MoveConfig;

/// A chain-agnostic Move client.
///
/// The client is a thin wrapper around an underlying
/// [`move_core_sdk::Aptos`] instance. The wrapper is what gives the
/// `move-rust-sdk` crate a chain-agnostic surface: every chain
/// configuration ([`crate::AptosConfig`], [`crate::MovementConfig`],
/// [`crate::MoveConfig::custom`]) goes through this type.
#[derive(Debug, Clone)]
pub struct MoveClient {
    inner: Arc<move_core_sdk::Aptos>,
    config: MoveConfig,
}

impl MoveClient {
    /// Create a new client from a [`MoveConfig`].
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client fails to build.
    pub fn new(config: MoveConfig) -> Result<Self, MoveError> {
        let core_config = config.clone().into_aptos_config();
        let inner = Arc::new(move_core_sdk::Aptos::new(core_config)?);
        Ok(Self { inner, config })
    }

    /// Return the active configuration.
    #[must_use]
    pub fn config(&self) -> &MoveConfig {
        &self.config
    }

    /// Return the chain ID of the active network.
    #[must_use]
    pub fn chain_id(&self) -> ChainId {
        self.inner.chain_id()
    }

    /// Return the framework addresses / function IDs for the active chain.
    #[must_use]
    pub fn framework(&self) -> &crate::config::FrameworkFunctions {
        self.config.framework()
    }

    /// Return the signing domain used by the active chain.
    ///
    /// This is `"APTOS::RawTransaction"` for Aptos-configured chains
    /// and `"MOVEMENT::RawTransaction"` for Movement-configured chains.
    /// It is the prefix hashed with the BCS-encoded transaction bytes
    /// to form the message that account keys sign.
    #[must_use]
    pub fn signing_domain(&self) -> &str {
        self.config.signing_domain()
    }

    /// Get the native-gas balance (in octas) for `address`.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP request fails or the
    /// response cannot be parsed.
    pub async fn get_balance(&self, address: AccountAddress) -> Result<u64, MoveError> {
        self.inner.get_balance(address).await
    }

    /// Submit a signed transaction to the chain and return the
    /// pending-transaction handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the node
    /// rejects the transaction.
    pub async fn submit_transaction(
        &self,
        txn: &move_core_sdk::transaction::SignedTransaction,
    ) -> Result<move_core_sdk::api::AptosResponse<PendingTransaction>, MoveError> {
        self.inner.submit_transaction(txn).await
    }

    /// Sign a [`RawTransaction`](move_core_sdk::transaction::RawTransaction)
    /// with the active chain's signing domain.
    ///
    /// The signing domain is the chain-specific prefix that
    /// determines how transactions are signed (e.g.
    /// `"APTOS::RawTransaction"` for Aptos, `"MOVEMENT::RawTransaction"`
    /// for Movement). The [`MoveConfig::signing_domain`](crate::MoveConfig::signing_domain)
    /// of the active chain is used automatically — callers do not
    /// have to think about it.
    ///
    /// # Errors
    ///
    /// Returns an error if the account fails to sign.
    #[cfg(feature = "ed25519")]
    pub fn sign_transaction<A: move_core_sdk::account::Account>(
        &self,
        raw_txn: &move_core_sdk::transaction::RawTransaction,
        account: &A,
    ) -> Result<move_core_sdk::transaction::SignedTransaction, MoveError> {
        move_core_sdk::transaction::sign_transaction_with_domain(
            raw_txn,
            account,
            self.config.signing_domain().as_bytes(),
        )
    }

    /// Wait for `txn_hash` to be committed and return the JSON
    /// representation of the final committed transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the transaction is
    /// dropped / aborted, or the timeout elapses before confirmation.
    pub async fn wait_for_transaction(
        &self,
        txn_hash: HashValue,
        timeout: Option<std::time::Duration>,
    ) -> Result<move_core_sdk::api::AptosResponse<serde_json::Value>, MoveError> {
        self.inner
            .fullnode()
            .wait_for_transaction(&txn_hash, timeout)
            .await
    }

    /// Submit a signed transaction and wait for it to be committed.
    ///
    /// Convenience wrapper over [`Self::submit_transaction`] +
    /// [`Self::wait_for_transaction`].
    ///
    /// # Errors
    ///
    /// Returns an error if submission or the wait fails.
    pub async fn submit_and_wait(
        &self,
        txn: &move_core_sdk::transaction::SignedTransaction,
        timeout: Option<std::time::Duration>,
    ) -> Result<move_core_sdk::api::AptosResponse<serde_json::Value>, MoveError> {
        self.inner.submit_and_wait(txn, timeout).await
    }

    /// Get a reference to the underlying fullnode client.
    #[must_use]
    pub fn fullnode(&self) -> &FullnodeClient {
        self.inner.fullnode()
    }

    /// Get a reference to the inner `move_core_sdk::Aptos` client.
    ///
    /// This is the escape hatch for any chain-agnostic feature that
    /// the `MoveClient` wrapper does not (yet) re-export. The
    /// returned client is a strict superset of the wrapper.
    #[must_use]
    pub fn inner(&self) -> &move_core_sdk::Aptos {
        &self.inner
    }
}
