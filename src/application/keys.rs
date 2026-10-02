
use serde::Serialize;

use crate::domain::keys;
use crate::domain::network::Network;
use crate::error::{ Result};

/// JSON-serialisable output of `key generate`.
#[derive(Debug, Serialize)]
pub struct GeneratedKey {
    pub network: String,
    /// WIF (secret). Only present when the caller asked for it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wif: Option<String>,
    /// Raw hex of the private scalar (secret). Only present when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_hex: Option<String>,
    pub public_hex: String,
}

/// Run `key generate`.
///
/// `show_secret` controls whether private material is included. The
/// presentation layer will normally require an explicit flag to set it.
pub fn generate(network: Network, show_secret: bool) -> Result<GeneratedKey> {
    let pair = keys::generate(network);

    Ok(GeneratedKey {
        network: network.to_string(),
        wif: show_secret.then(|| pair.wif()),
        private_hex: show_secret.then(|| pair.private_hex()),
        public_hex: pair.public_hex(),
    })
}