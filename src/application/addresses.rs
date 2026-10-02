// src/application/addresses.rs

use serde::Serialize;

use crate::domain::address::{self, AddressType};
use crate::domain::keys;
use crate::domain::network::Network;
use crate::error::Result;

/// JSON-serialisable output of `address from-pubkey`.
#[derive(Debug, Serialize)]
pub struct DerivedAddress {
    pub network: String,
    pub address_type: String,
    pub address: String,
    pub public_hex: String,
    pub script_pubkey_hex: String,
}

/// Run `address from-pubkey`.
pub fn from_pubkey(
    public_key_hex: &str,
    kind: AddressType,
    network: Network,
) -> Result<DerivedAddress> {
    let public_key = keys::parse_public_key(public_key_hex)?;
    let addr = address::from_pubkey(&public_key, kind, network)?;

    Ok(DerivedAddress {
        network: network.to_string(),
        address_type: kind.to_string(),
        address: addr.to_string(),
        public_hex: public_key.to_string(),
        script_pubkey_hex: hex::encode(addr.script_pubkey().as_bytes()),
    })
}

/// JSON-serialisable output of `address validate`.
#[derive(Debug, Serialize)]
pub struct ValidatedAddress {
    pub address: String,
    pub address_type: String,
    pub network: String,
    pub script_pubkey_hex: String,
}

/// Run `address validate`.
pub fn validate(addr: &str, expected: Network) -> Result<ValidatedAddress> {
    let info = address::validate(addr, expected)?;

    Ok(ValidatedAddress {
        address: info.address.to_string(),
        address_type: info.kind.to_string(),
        network: info.network.to_string(),
        script_pubkey_hex: info.script_pubkey_hex,
    })
}