use std::str::FromStr;

use bitcoin::key::{PrivateKey, PublicKey};
use bitcoin::secp256k1::Secp256k1;

use super::errors::DomainError;
use super::network::Network;

/// A freshly generated or parsed secp256k1 keypair.
///
/// Contains secret material when `private_key` is `Some`. Callers must
/// not log this struct or write it to disk without an explicit user request.
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
}

impl KeyPair {
    /// WIF (Wallet Import Format) string for the private key.
    pub fn wif(&self) -> String {
        self.private_key.to_wif()
    }

    /// Hex of the raw 32-byte secret scalar.
    pub fn private_hex(&self) -> String {
        hex::encode(self.private_key.inner.secret_bytes())
    }

    /// Compressed public key as hex.
    pub fn public_hex(&self) -> String {
        self.public_key.to_string()
    }
}

/// Generate a fresh secp256k1 keypair for the given network.
pub fn generate(network: Network) -> KeyPair {
    let secp = Secp256k1::new();
    let private_key = PrivateKey::generate(network.to_bitcoin());
    let public_key = private_key.public_key(&secp);
    KeyPair { private_key, public_key }
}

/// Parse a private key from either WIF or raw 32-byte hex.
///
/// - If the input is valid WIF, its embedded network must match `expected`.
///   Mismatches return `DomainError::NetworkMismatch`.
/// - If the input is raw hex, it is assumed to belong to `expected`.
pub fn parse_private_key(input: &str, expected: Network) -> Result<PrivateKey, DomainError> {
    let trimmed = input.trim();

    // Try WIF first — it carries a network prefix and a checksum.

    if let Ok(key) = PrivateKey::from_wif(trimmed) {
    match key.network {
        bitcoin::NetworkKind::Main => {
            return Err(DomainError::Unsupported(
                "mainnet WIF is not supported by this CLI".into()
            ));
        }
        bitcoin::NetworkKind::Test => {
            return Ok(key);
        }
    }
}

    // Fall back to raw hex.
    let bytes = hex::decode(trimmed).map_err(|e| {
        DomainError::InvalidPrivateKey(format!("not valid WIF or hex: {e}"))
    })?;

    if bytes.len() != 32 {
        return Err(DomainError::InvalidPrivateKey(format!(
            "hex private key must be 32 bytes, got {}",
            bytes.len()
        )));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);

    let secret = bitcoin::secp256k1::SecretKey::from_slice(&arr)
        .map_err(|e| DomainError::InvalidPrivateKey(e.to_string()))?;

    Ok(PrivateKey::new(secret, expected.to_bitcoin()))
}

/// Parse a public key from hex, accepting both compressed (33-byte) and
/// uncompressed (65-byte) encodings.
pub fn parse_public_key(input: &str) -> Result<PublicKey, DomainError> {
    PublicKey::from_str(input.trim())
        .map_err(|e| DomainError::InvalidPublicKey(e.to_string()))
}


impl Network {
    pub fn from_network_kind(kind: bitcoin::NetworkKind) -> Result<Self, DomainError> {
        match kind {
            bitcoin::NetworkKind::Test => Ok(Network::Testnet4),
            bitcoin::NetworkKind::Main => Err(DomainError::Unsupported("Mainnet is not supported".into())),
        }
    }
}