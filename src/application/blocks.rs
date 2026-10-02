use std::str::FromStr;

use bitcoin::BlockHash;
use serde::Serialize;

use crate::domain::errors::DomainError;
use crate::error::Result;
use crate::infrastructure::rpc::{BlockRef, RpcClient};

/// JSON-serialisable output of `block info`.
#[derive(Debug, Serialize)]
pub struct BlockDetails {
    pub hash: String,
    pub height: u64,
    pub confirmations: i32,
    pub time: u32,
    pub median_time: u32,
    pub version: i32,
    pub merkle_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_block_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_block_hash: Option<String>,
    pub nonce: u32,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub n_tx: u64,
}


pub fn info(rpc: &RpcClient, reference: &str) -> Result<BlockDetails> {
    let block_ref = parse_reference(reference)?;
    let info = rpc.get_block_info(block_ref)?;

    Ok(BlockDetails {
        hash: info.hash,
        height: info.height,
        confirmations: info.confirmations,
        time: info.time,
        median_time: info.median_time,
        version: info.version,
        merkle_root: info.merkle_root,
        previous_block_hash: info.previous_block_hash,
        next_block_hash: info.next_block_hash,
        nonce: info.nonce,
        bits: info.bits,
        difficulty: info.difficulty,
        chainwork: info.chainwork,
        n_tx: info.n_tx,
    })
}

fn parse_reference(s: &str) -> Result<BlockRef> {
    let trimmed = s.trim();

    if trimmed.len() == 64 {
        if let Ok(h) = BlockHash::from_str(trimmed) {
            return Ok(BlockRef::Hash(h));
        }
    }

    if let Ok(height) = trimmed.parse::<u64>() {
        return Ok(BlockRef::Height(height));
    }

    if let Ok(h) = BlockHash::from_str(trimmed) {
        return Ok(BlockRef::Hash(h));
    }

    Err(DomainError::Unsupported(format!(
        "block reference must be a height or 64-char hash, got `{trimmed}`"
    ))
    .into())
}