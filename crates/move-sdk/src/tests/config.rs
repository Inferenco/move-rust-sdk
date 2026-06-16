use crate::config::{ChainKind, FrameworkFunctions, MoveConfig, Network};

#[test]
fn custom_config_parses_https() {
    let cfg = MoveConfig::custom("https://my-chain.example.com/v1").unwrap();
    assert_eq!(cfg.kind(), ChainKind::Custom);
    assert_eq!(cfg.network(), Network::Custom);
    assert_eq!(
        cfg.fullnode_url().as_str(),
        "https://my-chain.example.com/v1"
    );
    assert_eq!(cfg.indexer_url(), None);
    assert_eq!(cfg.faucet_url(), None);
    assert_eq!(cfg.chain_id().id(), 0);
}

#[test]
fn custom_config_rejects_invalid_scheme() {
    let result = MoveConfig::custom("ftp://my-chain.example.com");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("URL scheme") || err.contains("url"));
}

#[test]
fn custom_config_accepts_http_for_local_dev() {
    let cfg = MoveConfig::custom("http://127.0.0.1:8080/v1").unwrap();
    assert_eq!(cfg.fullnode_url().as_str(), "http://127.0.0.1:8080/v1");
}

#[test]
fn custom_config_rejects_invalid_url() {
    assert!(MoveConfig::custom("not a url").is_err());
}

#[test]
fn with_timeout_is_applied() {
    let cfg = MoveConfig::custom("https://x.example.com/v1")
        .unwrap()
        .with_timeout(std::time::Duration::from_secs(5));
    assert_eq!(cfg.timeout(), std::time::Duration::from_secs(5));
}

#[test]
fn with_indexer_url_is_applied() {
    let cfg = MoveConfig::custom("https://x.example.com/v1")
        .unwrap()
        .with_indexer_url("https://x.example.com/graphql")
        .unwrap();
    assert_eq!(
        cfg.indexer_url().unwrap().as_str(),
        "https://x.example.com/graphql"
    );
}

#[test]
fn with_faucet_url_is_applied() {
    let cfg = MoveConfig::custom("https://x.example.com/v1")
        .unwrap()
        .with_faucet_url("https://x.example.com/faucet")
        .unwrap();
    assert_eq!(
        cfg.faucet_url().unwrap().as_str(),
        "https://x.example.com/faucet"
    );
}

#[test]
fn chain_kind_as_str() {
    assert_eq!(ChainKind::Aptos.as_str(), "aptos");
    assert_eq!(ChainKind::Movement.as_str(), "movement");
    assert_eq!(ChainKind::Custom.as_str(), "custom");
}

#[test]
fn network_as_str() {
    assert_eq!(Network::Mainnet.as_str(), "mainnet");
    assert_eq!(Network::Testnet.as_str(), "testnet");
    assert_eq!(Network::Devnet.as_str(), "devnet");
    assert_eq!(Network::Local.as_str(), "local");
    assert_eq!(Network::Custom.as_str(), "custom");
}

#[test]
fn aptos_framework_uses_aptos_signing_domain() {
    let fw = FrameworkFunctions::aptos();
    assert_eq!(fw.signing_domain, "APTOS::RawTransaction");
    assert_eq!(fw.native_coin_symbol, "APT");
    assert_eq!(fw.native_coin_type, "0x1::aptos_coin::AptosCoin");
}

#[test]
fn movement_framework_uses_movement_signing_domain() {
    let fw = FrameworkFunctions::movement();
    assert_eq!(fw.signing_domain, "MOVEMENT::RawTransaction");
    assert_eq!(fw.native_coin_symbol, "MOVE");
    // Movement re-uses the Aptos framework modules for now.
    assert_eq!(fw.native_transfer, "0x1::aptos_account::transfer");
}

#[test]
fn framework_address_is_one() {
    // Both Aptos and Movement use 0x1 as the core framework address.
    assert_eq!(
        FrameworkFunctions::aptos().framework_address,
        move_core_sdk::types::AccountAddress::ONE
    );
    assert_eq!(
        FrameworkFunctions::movement().framework_address,
        move_core_sdk::types::AccountAddress::ONE
    );
}

#[test]
fn into_aptos_config_preserves_urls() {
    let cfg = MoveConfig::custom("https://x.example.com/v1").unwrap();
    let aptos = cfg.into_aptos_config();
    assert_eq!(aptos.fullnode_url().as_str(), "https://x.example.com/v1");
}

#[test]
fn into_aptos_config_preserves_chain_id() {
    let mut cfg = MoveConfig::custom("https://x.example.com/v1").unwrap();
    cfg.chain_id = 42;
    let aptos = cfg.into_aptos_config();
    assert_eq!(aptos.chain_id().id(), 42);
}

#[test]
fn into_aptos_config_zero_chain_id_falls_back() {
    let cfg = MoveConfig::custom("https://x.example.com/v1").unwrap();
    let aptos = cfg.into_aptos_config();
    // chain_id 0 means "resolved from the node" -- the underlying
    // AptosConfig will report 0 from Custom, which the SDK then
    // resolves lazily.
    assert_eq!(aptos.chain_id().id(), 0);
}

#[cfg(feature = "aptos")]
#[test]
fn aptos_mainnet_uses_correct_endpoints() {
    let cfg = crate::AptosConfig::mainnet();
    assert_eq!(cfg.kind(), ChainKind::Aptos);
    assert_eq!(cfg.network(), Network::Mainnet);
    assert_eq!(cfg.chain_id().id(), 1);
    assert_eq!(
        cfg.fullnode_url().as_str(),
        "https://fullnode.mainnet.aptoslabs.com/v1"
    );
    assert!(cfg.faucet_url().is_none(), "mainnet has no faucet");
    assert_eq!(cfg.signing_domain(), "APTOS::RawTransaction");
}

#[cfg(feature = "aptos")]
#[test]
fn aptos_testnet_uses_correct_endpoints() {
    let cfg = crate::AptosConfig::testnet();
    assert_eq!(cfg.chain_id().id(), 2);
    assert!(cfg.faucet_url().is_some());
    assert_eq!(cfg.signing_domain(), "APTOS::RawTransaction");
}

#[cfg(feature = "aptos")]
#[test]
fn aptos_devnet_uses_correct_endpoints() {
    let cfg = crate::AptosConfig::devnet();
    assert_eq!(cfg.network(), Network::Devnet);
    // Devnet chain ID is resolved from the node.
    assert_eq!(cfg.chain_id().id(), 0);
    assert!(cfg.faucet_url().is_some());
}

#[cfg(feature = "aptos")]
#[test]
fn aptos_local_uses_localhost() {
    let cfg = crate::AptosConfig::local();
    assert_eq!(cfg.network(), Network::Local);
    assert!(cfg.fullnode_url().as_str().contains("127.0.0.1"));
    assert!(cfg.faucet_url().is_some());
}

#[cfg(feature = "movement")]
#[test]
fn movement_mainnet_uses_correct_endpoints() {
    let cfg = crate::MovementConfig::mainnet();
    assert_eq!(cfg.kind(), ChainKind::Movement);
    assert_eq!(cfg.network(), Network::Mainnet);
    assert_eq!(cfg.chain_id().id(), 126);
    assert_eq!(
        cfg.fullnode_url().as_str(),
        "https://mainnet.movementnetwork.xyz/v1"
    );
    assert!(cfg.faucet_url().is_none(), "mainnet has no faucet");
    assert_eq!(cfg.signing_domain(), "MOVEMENT::RawTransaction");
}

#[cfg(feature = "movement")]
#[test]
fn movement_testnet_uses_correct_endpoints() {
    let cfg = crate::MovementConfig::testnet();
    assert_eq!(cfg.chain_id().id(), 250);
    assert!(cfg.faucet_url().is_some());
    assert_eq!(cfg.signing_domain(), "MOVEMENT::RawTransaction");
    assert_eq!(
        cfg.fullnode_url().as_str(),
        "https://testnet.movementnetwork.xyz/v1"
    );
}

#[cfg(feature = "movement")]
#[test]
fn movement_local_uses_localhost() {
    let cfg = crate::MovementConfig::local();
    assert_eq!(cfg.network(), Network::Local);
    assert!(cfg.fullnode_url().as_str().contains("127.0.0.1"));
}

/// Aptos and Movement must use different signing domains or the
/// resulting on-chain signatures will be invalid on the wrong
/// network. This test pins the contract: do not change these
/// strings without also updating the on-chain Move verifier.
#[test]
fn signing_domains_differ_for_aptos_vs_movement() {
    let aptos = FrameworkFunctions::aptos();
    let movement = FrameworkFunctions::movement();
    assert_ne!(aptos.signing_domain, movement.signing_domain);
    assert_eq!(aptos.signing_domain, "APTOS::RawTransaction");
    assert_eq!(movement.signing_domain, "MOVEMENT::RawTransaction");
}

#[cfg(feature = "ed25519")]
#[test]
fn aptos_signing_message_uses_aptos_prefix() {
    use move_core_sdk::transaction::RawTransaction;
    use move_core_sdk::types::ChainId;

    let raw = RawTransaction::new(
        move_core_sdk::types::AccountAddress::ONE,
        0,
        move_core_sdk::transaction::TransactionPayload::Script(
            move_core_sdk::transaction::Script::new(vec![], vec![], vec![]),
        ),
        1_000,
        100,
        1_000_000_000,
        ChainId::testnet(),
    );

    let message = raw.signing_message().unwrap();
    let bcs_len = raw.to_bcs().unwrap().len();
    // The signing message is sha3_256("APTOS::RawTransaction") || BCS(raw).
    assert_eq!(message.len(), 32 + bcs_len);
}

#[cfg(all(feature = "ed25519", feature = "movement"))]
#[test]
fn movement_signing_message_uses_movement_prefix() {
    use move_core_sdk::transaction::RawTransaction;
    use move_core_sdk::types::ChainId;

    let raw = RawTransaction::new(
        move_core_sdk::types::AccountAddress::ONE,
        0,
        move_core_sdk::transaction::TransactionPayload::Script(
            move_core_sdk::transaction::Script::new(vec![], vec![], vec![]),
        ),
        1_000,
        100,
        1_000_000_000,
        ChainId::new(250), // Movement testnet
    );

    let aptos_msg = raw
        .signing_message_with_domain(b"APTOS::RawTransaction")
        .unwrap();
    let movement_msg = raw
        .signing_message_with_domain(b"MOVEMENT::RawTransaction")
        .unwrap();

    // Same transaction, different signing domain -> different message.
    assert_ne!(aptos_msg, movement_msg);
    // But the BCS body (everything after the 32-byte SHA3-256 prefix) is identical.
    assert_eq!(&aptos_msg[32..], &movement_msg[32..]);
}
