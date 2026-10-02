
use std::str::FromStr;

use bitcoin::key::PrivateKey;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::Network as BtcNetwork;

use btc_cli::domain::address::{self, AddressType};
use btc_cli::domain::errors::DomainError;
use btc_cli::domain::keys;
use btc_cli::domain::network::Network;

fn fixed_key() -> PrivateKey {
    // Deterministic 32-byte secret so tests don't depend on RNG.
    let mut bytes = [0u8; 32];
    bytes[31] = 1; // scalar = 1
    let secret = SecretKey::from_slice(&bytes).unwrap();
    PrivateKey::new(secret, BtcNetwork::Regtest)
}

#[test]
fn p2wpkh_address_starts_with_bcrt1q_on_regtest() {
    let key = fixed_key();
    let secp = Secp256k1::new();
    let pubkey = key.public_key(&secp);

    let addr = address::from_pubkey(&pubkey, AddressType::P2wpkh, Network::Regtest).unwrap();
    assert!(addr.to_string().starts_with("bcrt1q"));
}

#[test]
fn p2tr_address_starts_with_bcrt1p_on_regtest() {
    let key = fixed_key();
    let secp = Secp256k1::new();
    let pubkey = key.public_key(&secp);

    let addr = address::from_pubkey(&pubkey, AddressType::P2tr, Network::Regtest).unwrap();
    assert!(addr.to_string().starts_with("bcrt1p"));
}

#[test]
fn p2pkh_address_starts_with_letter_on_regtest() {
    let key = fixed_key();
    let secp = Secp256k1::new();
    let pubkey = key.public_key(&secp);

    let addr = address::from_pubkey(&pubkey, AddressType::P2pkh, Network::Regtest).unwrap();
    // Regtest/testnet P2PKH addresses start with 'm' or 'n'.
    let s = addr.to_string();
    assert!(
        s.starts_with('m') || s.starts_with('n'),
        "unexpected P2PKH prefix: {s}"
    );
}

#[test]
fn validate_round_trip_matches_derived_script() {
    let key = fixed_key();
    let secp = Secp256k1::new();
    let pubkey = key.public_key(&secp);

    for kind in [
        AddressType::P2pkh,
        AddressType::P2shP2wpkh,
        AddressType::P2wpkh,
        AddressType::P2tr,
    ] {
        let addr = address::from_pubkey(&pubkey, kind, Network::Regtest).unwrap();
        let derived_script = hex::encode(addr.script_pubkey().as_bytes());

        let info = address::validate(&addr.to_string(), Network::Regtest).unwrap();
        assert_eq!(info.kind, kind);
        assert_eq!(info.network, Network::Regtest);
        assert_eq!(info.script_pubkey_hex, derived_script);
    }
}

#[test]
fn mainnet_bech32_rejected_on_regtest() {
    // BIP173 test vector — a valid mainnet bech32 address.
    let mainnet_addr = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let err = address::validate(mainnet_addr, Network::Regtest).unwrap_err();
    assert!(
        matches!(err, DomainError::NetworkMismatch { .. }),
        "expected NetworkMismatch, got {err:?}"
    );
}

#[test]
fn garbage_address_rejected() {
    let err = address::validate("not-an-address", Network::Regtest).unwrap_err();
    assert!(matches!(err, DomainError::InvalidAddress(_)));
}

#[test]
fn address_type_from_str_accepts_aliases() {
    assert_eq!("p2wpkh".parse::<AddressType>().unwrap(), AddressType::P2wpkh);
    assert_eq!("segwit".parse::<AddressType>().unwrap(), AddressType::P2wpkh);
    assert_eq!("p2tr".parse::<AddressType>().unwrap(), AddressType::P2tr);
    assert_eq!("taproot".parse::<AddressType>().unwrap(), AddressType::P2tr);
    assert!("quantum".parse::<AddressType>().is_err());
}

#[test]
fn parse_public_key_round_trip() {
    let key = fixed_key();
    let secp = Secp256k1::new();
    let pubkey = key.public_key(&secp);
    let hex_form = pubkey.to_string();

    let parsed = keys::parse_public_key(&hex_form).unwrap();
    assert_eq!(parsed, pubkey);
}

#[test]
fn parse_public_key_rejects_garbage() {
    let err = keys::parse_public_key("nonsense").unwrap_err();
    assert!(matches!(err, DomainError::InvalidPublicKey(_)));
}

// Silence a warning if FromStr isn't needed elsewhere in this file.
#[allow(unused_imports)]
use std::str::FromStr as _;