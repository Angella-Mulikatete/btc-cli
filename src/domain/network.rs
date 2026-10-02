use std::fmt;

// use clap::ValueEnum;
use serde:: { Serialize, Deserialize};
use std::str::FromStr;

use super::errors::DomainError;

// /// Represents the network to which the Bitcoin client is connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Testnet4,
    Signet,
    Regtest,
}


impl Network {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Network::Testnet4 => "testnet4",
            Network::Signet => "signet",
            Network::Regtest => "regtest",
        }
    }

    pub const fn is_test_family(self) -> bool {
        true
    }

    /// Convert to the `bitcoin` crate's representation.
    pub fn to_bitcoin(self) -> bitcoin::Network {
        match self {
            Network::Testnet4 => bitcoin::Network::Testnet4,
            Network::Signet => bitcoin::Network::Signet,
            Network::Regtest => bitcoin::Network::Regtest,
        }
    }

    pub fn from_bitcoin(n: bitcoin::Network) -> Result<Self, DomainError> {
        match n {
            bitcoin::Network::Testnet4 => Ok(Network::Testnet4),
            bitcoin::Network::Signet => Ok(Network::Signet),
            bitcoin::Network::Regtest => Ok(Network::Regtest),
            other => Err(DomainError::Unsupported(format!(
                 "network `{other:?}` is not supported by this CLI \
                 (supported: testnet4, signet, regtest)"
            )))
        }
    }
}

// impl  From<Network> for bitcoin::Network {
//     fn from(network: Network) -> Self {
//         match network {
//             Network::Testnet4 => bitcoin::Network::Testnet4,
//             Network::Signet => bitcoin::Network::Signet,
//             Network::Regtest => bitcoin::Network::Regtest,
//         }
//     }
// }


impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let network_str = match self {
            Network::Testnet4 => "testnet4",
            Network::Signet => "signet",
            Network::Regtest => "regtest",
        };
        write!(f, "{}", network_str)
    }
}

impl FromStr for Network {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "testnet4" => Ok(Network::Testnet4),
            "signet"   => Ok(Network::Signet),
            "regtest"  => Ok(Network::Regtest),
            other => Err(DomainError::Unsupported(format!(
                "unknown network `{other}` (supported: testnet4, signet, regtest)"
            ))),
        }
    }
}