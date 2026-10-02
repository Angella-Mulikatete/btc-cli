use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use serde::Serialize;

use crate::domain::address::{self, AddressType};
use crate::domain::errors::DomainError;
use crate::domain::network::Network;
use crate::error::Result;

/// JSON-serialisable output of `derive`.
#[derive(Debug, Serialize)]
pub struct Derived {
    pub network: String,
    pub path: String,
    pub source_kind: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wif: Option<String>,
    pub public_hex: String,
    pub address: String,
    pub address_type: String,
}

/// Run `derive`.
///
/// * `extended_key` — xprv or xpub string, network-checked against `network`.
/// * `path` — BIP32 path like `m/84'/1'/0'/0/0`. `m` alone means "the key itself".
/// * `address_type` — if `None`, inferred from the purpose field of `path`.
pub fn derive(
    extended_key: &str,
    path: &str,
    network: Network,
    address_type: Option<AddressType>,
) -> Result<Derived> {
    let trimmed_key = extended_key.trim();
    let derivation_path = parse_path(path)?;

    // Try xprv first; fall back to xpub.
    if let Ok(root) = Xpriv::from_str(trimmed_key) {
        derive_from_xprv(root, &derivation_path, network, address_type)
    } else if let Ok(root) = Xpub::from_str(trimmed_key) {
        derive_from_xpub(root, &derivation_path, network, address_type)
    } else {
        Err(DomainError::Unsupported(
            "extended key is neither a valid xprv nor xpub".into(),
        )
        .into())
    }
}

fn parse_path(path: &str) -> Result<DerivationPath> {
    let trimmed = path.trim();
    DerivationPath::from_str(trimmed)
        .map_err(|e| DomainError::InvalidDerivationPath(format!("`{trimmed}`: {e}")).into())
}

/// Check that the extended key's encoded network matches what the user asked for.
fn check_network(kind: NetworkKind, expected: Network) -> Result<()> {
    match (kind, expected) {
    
        (NetworkKind::Test, _) => Ok(()),
        (NetworkKind::Main, _) => Err(DomainError::Unsupported(
            "mainnet extended key is not supported by this CLI".into(),
        )
        .into()),
    }
}

fn derive_from_xprv(
    root: Xpriv,
    path: &DerivationPath,
    network: Network,
    address_type: Option<AddressType>,
) -> Result<Derived> {
    check_network(root.network, network)?;

    let secp = Secp256k1::new();
    let child = root
        .derive_priv(&secp, path)
        .map_err(|e| DomainError::InvalidDerivationPath(e.to_string()))?;

    let private_key = bitcoin::PrivateKey::new(child.private_key, network.to_bitcoin());
    let public_key = private_key.public_key(&secp);

    let kind = address_type.unwrap_or_else(|| infer_address_type(path));
    let addr = address::from_pubkey(&public_key, kind, network)?;

    Ok(Derived {
        network: network.to_string(),
        path: path.to_string(),
        source_kind: "xprv".into(),
        private_hex: Some(hex::encode(child.private_key.secret_bytes())),
        wif: Some(private_key.to_wif()),
        public_hex: public_key.to_string(),
        address: addr.to_string(),
        address_type: kind.to_string(),
    })
}

fn derive_from_xpub(
    root: Xpub,
    path: &DerivationPath,
    network: Network,
    address_type: Option<AddressType>,
) -> Result<Derived> {
    check_network(root.network, network)?;

    // Guard against hardened steps — cannot be derived from an xpub.
    if path.into_iter().any(|c| c.is_hardened()) {
        return Err(DomainError::InvalidDerivationPath(
            "cannot derive a hardened child from an xpub; \
             hardened steps require the private key"
                .into(),
        )
        .into());
    }

    let secp = Secp256k1::new();
    let child = root
        .derive_pub(&secp, path)
        .map_err(|e| DomainError::InvalidDerivationPath(e.to_string()))?;

    let public_key = bitcoin::PublicKey::new(child.public_key);
    let kind = address_type.unwrap_or_else(|| infer_address_type(path));
    let addr = address::from_pubkey(&public_key, kind, network)?;

    Ok(Derived {
        network: network.to_string(),
        path: path.to_string(),
        source_kind: "xpub".into(),
        private_hex: None,
        wif: None,
        public_hex: public_key.to_string(),
        address: addr.to_string(),
        address_type: kind.to_string(),
    })
}

/// Infer address type from the purpose field of a BIP44/49/84/86 path.
///
/// Purpose 44 → P2PKH, 49 → P2SH-P2WPKH, 84 → P2WPKH, 86 → P2TR.
/// Anything else defaults to P2WPKH, which is the modern default.
fn infer_address_type(path: &DerivationPath) -> AddressType {
    let purpose = match path.into_iter().next() {
        Some(ChildNumber::Normal { index }) => Some(*index),
        Some(ChildNumber::Hardened { index }) => Some(*index),
        None => None,
    };

    match purpose {
        Some(44) => AddressType::P2pkh,
        Some(49) => AddressType::P2shP2wpkh,
        Some(86) => AddressType::P2tr,
        _ => AddressType::P2wpkh, // 84 and anything else
    }
}