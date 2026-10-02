
use std::path::PathBuf;

use bitcoincore_rpc::json::{EstimateMode, GetBlockHeaderResult};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use thiserror::Error;

use bitcoin::{BlockHash, Transaction, Txid};

use crate::domain::network::Network;

use super::config::{RpcAuth, RpcConfig};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("failed to connect to node at {url}: {source}")]
    Connect {
        url: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("node returned error: {0}")]
    Node(String),

    #[error("RPC response could not be parsed: {0}")]
    Parse(String),

    #[error("RPC not configured (set BTC_RPC_URL and BTC_RPC_USER/PASSWORD or BTC_RPC_COOKIE)")]
    NotConfigured,

    #[error("block not found: {0}")]
    BlockNotFound(String),

    #[error("fee estimate unavailable for target {0} blocks")]
    FeeUnavailable(u16),
}

impl RpcError {
    pub fn exit_code(&self) -> i32 {
        4
    }
}

// ---------------------------------------------------------------------------
// Public data types
// ---------------------------------------------------------------------------

/// How to look up a block.
#[derive(Debug, Clone)]
pub enum BlockRef {
    Height(u64),
    Hash(BlockHash),
}

/// Summary of a block, network-agnostic and serde-ready (the application
/// layer decides the JSON shape).
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub hash: String,
    pub height: u64,
    pub confirmations: i32,
    pub time: u32,
    pub median_time: u32,
    pub version: i32,
    pub merkle_root: String,
    pub previous_block_hash: Option<String>,
    pub next_block_hash: Option<String>,
    pub nonce: u32,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub n_tx: u64,
}

/// Fee estimate returned by the node.
#[derive(Debug, Clone)]
pub struct FeeEstimate {
    /// Fee rate in satoshis per virtual byte.
    pub sat_per_vb: f64,
    /// How many blocks the estimate targets.
    pub blocks: u32,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct RpcClient {
    inner: Client,
    /// Kept for logging and future network-specific behaviour.
    #[allow(dead_code)]
    network: Network,
}

impl RpcClient {
    /// Connect to the node described by `config`.
    pub fn connect(config: &RpcConfig, network: Network) -> Result<Self, RpcError> {
        let auth = match &config.auth {
            RpcAuth::UserPass { user, password } => {
                Auth::UserPass(user.clone(), password.clone())
            }
            RpcAuth::CookieFile(path) => Auth::CookieFile(PathBuf::from(path)),
        };

        let inner = Client::new(&config.url, auth).map_err(|e| RpcError::Connect {
            url: config.url.clone(),
            source: Box::new(e),
        })?;

        Ok(Self { inner, network })
    }

    /// Fetch summary information about a block by height or hash.
    pub fn get_block_info(&self, reference: BlockRef) -> Result<BlockInfo, RpcError> {
        let hash: BlockHash = match reference {
            BlockRef::Hash(h) => h,
            BlockRef::Height(h) => self
                .inner
                .get_block_hash(h)
                .map_err(|e| RpcError::BlockNotFound(format!("height {h}: {e}")))?,
        };

        let header: GetBlockHeaderResult = self
            .inner
            .get_block_header_info(&hash)
            .map_err(|e| RpcError::BlockNotFound(format!("{hash}: {e}")))?;

        Ok(BlockInfo {
            hash: header.hash.to_string(),
            height: header.height as u64,
            confirmations: header.confirmations,
            time: header.time as u32,
            median_time: header.median_time.unwrap_or(0) as u32,
            version: header.version.to_consensus(),
            merkle_root: header.merkle_root.to_string(),
            previous_block_hash: header.previous_block_hash.map(|h| h.to_string()),
            next_block_hash: header.next_block_hash.map(|h| h.to_string()),
            nonce: header.nonce,
            bits: header.bits,
            difficulty: header.difficulty,
            chainwork: hex::encode(&header.chainwork),
            n_tx: header.n_tx as u64,
        })
    }
    /// Ask the node for a fee rate estimate targeting `target` blocks.
    pub fn estimate_fee(&self, target: u16) -> Result<FeeEstimate, RpcError> {
        let result = self
            .inner
            .estimate_smart_fee(target, Some(EstimateMode::Economical))
            .map_err(|e| RpcError::Node(e.to_string()))?;

        let feerate_btc_per_kvb = match result.fee_rate {
            Some(a) => a,
            None => return Err(RpcError::FeeUnavailable(target)),
        };

        // Feerate comes back as BTC per kvB.
        //   sat/vB = BTC/kvB × 100_000_000 sat/BTC ÷ 1000 vB/kvB
        //          = BTC/kvB × 100_000
        // Use `to_btc()` (f64) to keep the fractional part.
        let sat_per_vb = feerate_btc_per_kvb.to_btc() * 100_000.0;

        Ok(FeeEstimate {
            sat_per_vb,
            // `blocks` is the node's actual target — could differ from ours
            // when the node rounds to its internal bucket. It's i64; clamp to u32.
            blocks: result.blocks.max(0) as u32,
        })
    }

    /// Broadcast a signed transaction. Returns its txid.
    pub fn send_raw_transaction(&self, tx: &Transaction) -> Result<Txid, RpcError> {
        self.inner
            .send_raw_transaction(tx)
            .map_err(|e| RpcError::Node(e.to_string()))
    }
}