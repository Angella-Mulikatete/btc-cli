
use bitcoin::bip32::Xpriv;
use bitcoin::Network as BtcNetwork;

use btc_cli::application::derive;
use btc_cli::domain::errors::DomainError;
use btc_cli::domain::network::Network;


const SEED: [u8; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
];

fn regtest_root() -> Xpriv {
    Xpriv::new_master(BtcNetwork::Regtest, &SEED).unwrap()
}

#[test]
fn derive_at_root_returns_the_master_key() {
    let root = regtest_root();
    let out = derive::derive(
        &root.to_string(),
        "m",
        Network::Regtest,
        None,
    )
    .unwrap();

    // The public key at `m` is the master key's own public key.
    assert_eq!(out.path, "m");
    assert_eq!(out.source_kind, "xprv");
    assert!(out.private_hex.is_some());
    assert!(out.wif.is_some());
}

#[test]
fn derive_is_deterministic() {
    let root = regtest_root();
    let a = derive::derive(&root.to_string(), "m/84'/1'/0'/0/0", Network::Regtest, None).unwrap();
    let b = derive::derive(&root.to_string(), "m/84'/1'/0'/0/0", Network::Regtest, None).unwrap();

    assert_eq!(a.address, b.address);
    assert_eq!(a.public_hex, b.public_hex);
    assert_eq!(a.private_hex, b.private_hex);
}

#[test]
fn different_paths_produce_different_addresses() {
    let root = regtest_root();
    let a = derive::derive(&root.to_string(), "m/84'/1'/0'/0/0", Network::Regtest, None).unwrap();
    let b = derive::derive(&root.to_string(), "m/84'/1'/0'/0/1", Network::Regtest, None).unwrap();

    assert_ne!(a.address, b.address);
}

#[test]
fn address_type_is_inferred_from_purpose() {
    let root = regtest_root();

    let p2wpkh = derive::derive(&root.to_string(), "m/84'/1'/0'/0/0", Network::Regtest, None).unwrap();
    assert_eq!(p2wpkh.address_type, "p2wpkh");

    let p2tr = derive::derive(&root.to_string(), "m/86'/1'/0'/0/0", Network::Regtest, None).unwrap();
    assert_eq!(p2tr.address_type, "p2tr");

    let p2pkh = derive::derive(&root.to_string(), "m/44'/1'/0'/0/0", Network::Regtest, None).unwrap();
    assert_eq!(p2pkh.address_type, "p2pkh");
}

#[test]
fn explicit_address_type_overrides_inference() {
    let root = regtest_root();
    let out = derive::derive(
        &root.to_string(),
        "m/84'/1'/0'/0/0",
        Network::Regtest,
        Some(btc_cli::domain::address::AddressType::P2tr),
    )
    .unwrap();
    assert_eq!(out.address_type, "p2tr");
}

#[test]
fn hardened_path_from_xpub_is_rejected() {
    let root = regtest_root();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let xpub = bitcoin::bip32::Xpub::from_priv(&secp, &root);

    let err = derive::derive(
        &xpub.to_string(),
        "m/84'/1'/0'/0/0", // contains hardened steps
        Network::Regtest,
        None,
    )
    .unwrap_err();

    let s = format!("{err}");
    assert!(
        s.contains("hardened"),
        "expected hardened-path error, got: {s}"
    );
}

#[test]
fn mainnet_xprv_is_rejected() {
    let mainnet_root = Xpriv::new_master(BtcNetwork::Bitcoin, &SEED).unwrap();
    let err = derive::derive(&mainnet_root.to_string(), "m/0", Network::Regtest, None).unwrap_err();
    let s = format!("{err}");
    assert!(s.contains("mainnet"), "expected mainnet rejection, got: {s}");
}

#[test]
fn garbage_extended_key_rejected() {
    let err = derive::derive("not-an-xprv", "m/0", Network::Regtest, None).unwrap_err();
    let s = format!("{err}");
    assert!(s.contains("xprv") || s.contains("xpub"), "got: {s}");
}

// Keep this import used.
#[allow(unused_imports)]
use DomainError as _;