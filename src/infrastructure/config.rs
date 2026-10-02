use std::env;
use std::path::PathBuf;

use crate::domain::network::Network;

use super::rpc::RpcError;

/// How to authenticate to the Bitcoin Core RPC endpoint.
#[derive(Debug, Clone)]
pub enum RpcAuth {
    /// `rpcuser` / `rpcpassword` from `bitcoin.conf`.
    UserPass { user: String, password: String },

    /// Path to a `.cookie` file written by bitcoind.
    CookieFile(PathBuf),
}

/// Everything needed to talk to a Bitcoin Core node.
#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub url: String,
    pub auth: RpcAuth,
}

impl RpcConfig {
    /// Read RPC configuration from the process environment.
    ///
    /// - `BTC_RPC_URL`        — full URL, defaults to `http://127.0.0.1:<port>`.
    /// - `BTC_RPC_COOKIE`     — path to a cookie file. Takes precedence.
    /// - `BTC_RPC_USER`       — RPC username.
    /// - `BTC_RPC_PASSWORD`   — RPC password.
    ///
    /// Returns `RpcError::NotConfigured` if no auth is available.
    pub fn from_env(network: Network) -> Result<Self, RpcError> {
        let url = env::var("BTC_RPC_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_url(network));

        // Cookie file wins if present.
        if let Some(path) = env::var("BTC_RPC_COOKIE").ok().filter(|s| !s.is_empty()) {
            return Ok(Self {
                url,
                auth: RpcAuth::CookieFile(PathBuf::from(path)),
            });
        }

        let user = env::var("BTC_RPC_USER").unwrap_or_default();
        let password = env::var("BTC_RPC_PASSWORD").unwrap_or_default();
        if !user.is_empty() && !password.is_empty() {
            return Ok(Self {
                url,
                auth: RpcAuth::UserPass { user, password },
            });
        }

        Err(RpcError::NotConfigured)
    }
}

/// Default RPC URL for the given network. These ports match Bitcoin Core's
/// built-in defaults for `-testnet4`, `-signet`, and `-regtest`.
pub fn default_url(network: Network) -> String {
    let port = match network {
        Network::Testnet4 => 48332,
        Network::Signet   => 38332,
        Network::Regtest  => 18443,
    };
    format!("http://127.0.0.1:{port}")
}