use std::fmt;
use std::str::FromStr;

use bitcoin::address::{Address, KnownHrp, NetworkChecked, NetworkUnchecked};
use bitcoin::key::TweakedPublicKey;
use bitcoin::secp256k1::XOnlyPublicKey;
use bitcoin::{CompressedPublicKey, NetworkKind, PublicKey};

use super::errors::DomainError;
use super::network::Network;

/// The address types this CLI can derive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressType {
    P2pkh,
    P2shP2wpkh,
    P2wpkh,
    P2tr,
}

impl AddressType {
    pub const fn as_str(self) -> &'static str {
        match self {
            AddressType::P2pkh      => "p2pkh",
            AddressType::P2shP2wpkh => "p2sh-p2wpkh",
            AddressType::P2wpkh     => "p2wpkh",
            AddressType::P2tr       => "p2tr",
        }
    }
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AddressType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "p2pkh"       | "legacy" => Ok(AddressType::P2pkh),
            "p2sh-p2wpkh" | "nested" => Ok(AddressType::P2shP2wpkh),
            "p2wpkh"      | "segwit" | "bech32" => Ok(AddressType::P2wpkh),
            "p2tr"        | "taproot" => Ok(AddressType::P2tr),
            other => Err(DomainError::Unsupported(format!(
                "unknown address type `{other}` \
                 (supported: p2pkh, p2sh-p2wpkh, p2wpkh, p2tr)"
            ))),
        }
    }
}

/// Everything a caller might want to know after validating an address.
#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub address: Address,
    pub kind: AddressType,
    pub network: Network,
    pub script_pubkey_hex: String,
}

/// Derive an address from a public key.
pub fn from_pubkey(
    public_key: &PublicKey,
    kind: AddressType,
    network: Network,
) -> Result<Address, DomainError> {
    let btc_network = network.to_bitcoin();
    let network_kind: NetworkKind = btc_network.into();
    let known_hrp: KnownHrp = match network {
        Network::Testnet4 | Network::Signet => KnownHrp::Testnets,
        Network::Regtest => KnownHrp::Regtest,
    };

    // Only compressed pubkeys are allowed for segwit/taproot.
    let compressed = CompressedPublicKey::try_from(*public_key).map_err(|e| {
        DomainError::InvalidPublicKey(format!(
            "public key is not compressed or not valid for address derivation: {e}"
        ))
    })?;

    let addr = match kind {
        AddressType::P2pkh => Address::p2pkh(public_key.pubkey_hash(), network_kind),
        AddressType::P2shP2wpkh => Address::p2shwpkh(&compressed, network_kind),
        AddressType::P2wpkh => Address::p2wpkh(&compressed, known_hrp),
        AddressType::P2tr => {
            let xonly = XOnlyPublicKey::from_slice(&compressed.to_bytes()[1..33])
                .map_err(|e| DomainError::InvalidPublicKey(e.to_string()))?;
            let internal = TweakedPublicKey::dangerous_assume_tweaked(xonly);
            Address::p2tr_tweaked(internal, btc_network)
        }
    };

    Ok(addr)
}

/// Validate an address string, reporting its type and network.
pub fn validate(input: &str, expected: Network) -> Result<AddressInfo, DomainError> {
    let unchecked: Address<NetworkUnchecked> = Address::from_str(input.trim())
        .map_err(|e| DomainError::InvalidAddress(e.to_string()))?;

    if !unchecked.is_valid_for_network(expected.to_bitcoin()) {

        let actual = [Network::Testnet4, Network::Signet, Network::Regtest]
            .into_iter()
            .find(|n| unchecked.is_valid_for_network(n.to_bitcoin()));

        return Err(DomainError::NetworkMismatch {
            expected: expected.to_string(),
            actual: actual
                .map(|n| n.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        });
    }

    let checked: Address<NetworkChecked> = unchecked.assume_checked();
    let kind = classify(&checked)?;

    Ok(AddressInfo {
        script_pubkey_hex: hex::encode(checked.script_pubkey().as_bytes()),
        address: checked,
        kind,
        network: expected,
    })
}


fn classify(addr: &Address<NetworkChecked>) -> Result<AddressType, DomainError> {
    match addr.address_type() {
        Some(bitcoin::AddressType::P2pkh)  => Ok(AddressType::P2pkh),
        Some(bitcoin::AddressType::P2sh)   => Ok(AddressType::P2shP2wpkh),
        Some(bitcoin::AddressType::P2wpkh) => Ok(AddressType::P2wpkh),
        Some(bitcoin::AddressType::P2tr)   => Ok(AddressType::P2tr),
        Some(other) => Err(DomainError::Unsupported(format!(
            "address type `{other:?}` is not supported \
             (only p2pkh, p2sh-p2wpkh, p2wpkh, p2tr)"
        ))),
        None => Err(DomainError::Unsupported(
            "address type could not be determined".into(),
        )),
    }
}