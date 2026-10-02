use serde::Serialize;

use crate::error::Result;
use crate::infrastructure::rpc::RpcClient;

/// JSON-serialisable output of `fee estimate`.
#[derive(Debug, Serialize)]
pub struct FeeEstimateOut {
    /// Fee rate in satoshis per virtual byte.
    pub sat_per_vb: f64,
    pub target_blocks: u32,
}

/// Run `fee estimate`.
pub fn estimate(rpc: &RpcClient, target: u16) -> Result<FeeEstimateOut> {
    let estimate = rpc.estimate_fee(target)?;

    Ok(FeeEstimateOut {
        sat_per_vb: estimate.sat_per_vb,
        target_blocks: estimate.blocks,
    })
}