use std::str::FromStr;

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, ScriptBuf, Transaction, Txid};
use serde::Serialize;

use crate::domain::errors::DomainError;
use crate::domain::keys;
use crate::domain::network::Network;
use crate::domain::transaction::{self, PrevOut, TxPlan, PlannedInput, PlannedOutput};
use crate::error::Result;


#[derive(Debug, Serialize)]
pub struct DecodedTx {
    pub txid: String,
    pub wtxid: String,
    pub version: i32,
    pub locktime: u32,
    pub size: usize,
    pub vsize: usize,
    pub weight: usize,
    pub is_segwit: bool,
    pub inputs: Vec<DecodedInput>,
    pub outputs: Vec<DecodedOutput>,
}

#[derive(Debug, Serialize)]
pub struct DecodedInput {
    pub index: usize,
    pub txid: String,
    pub vout: u32,
    pub script_sig_hex: String,
    pub sequence: u32,
    pub witness_items: usize,
}

#[derive(Debug, Serialize)]
pub struct DecodedOutput {
    pub index: usize,
    pub value_sats: u64,
    pub script_pubkey_hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Run `tx decode`.
pub fn decode(raw_hex: &str, network: Network) -> Result<DecodedTx> {
    let info = transaction::decode(raw_hex, network)?;

    Ok(DecodedTx {
        txid: info.txid,
        wtxid: info.wtxid,
        version: info.version,
        locktime: info.locktime,
        size: info.size,
        vsize: info.vsize,
        weight: info.weight,
        is_segwit: info.is_segwit,
        inputs: info
            .inputs
            .into_iter()
            .map(|i| DecodedInput {
                index: i.index,
                txid: i.txid,
                vout: i.vout,
                script_sig_hex: i.script_sig_hex,
                sequence: i.sequence,
                witness_items: i.witness_items,
            })
            .collect(),
        outputs: info
            .outputs
            .into_iter()
            .map(|o| DecodedOutput {
                index: o.index,
                value_sats: o.value_sats,
                script_pubkey_hex: o.script_pubkey_hex,
                address: o.address,
            })
            .collect(),
    })
}

// Create (unsigned)
/// A single UTXO the user wants to spend, as supplied on the CLI.

#[derive(Debug, Clone)]
pub struct InputSpec {
    pub txid: Txid,
    pub vout: u32,
}

impl FromStr for InputSpec {
    type Err = DomainError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (txid_str, vout_str) = s.split_once(':').ok_or_else(|| {
            DomainError::InvalidTransaction(format!(
                "input must be `txid:vout`, got `{s}`"
            ))
        })?;
        let txid = Txid::from_str(txid_str.trim())
            .map_err(|e| DomainError::InvalidTransaction(format!("txid: {e}")))?;
        let vout: u32 = vout_str
            .trim()
            .parse()
            .map_err(|e| DomainError::InvalidTransaction(format!("vout: {e}")))?;
        Ok(InputSpec { txid, vout })
    }
}

/// A single output the user wants to create.
#[derive(Debug, Clone)]
pub struct OutputSpec {
    pub address: Address<NetworkUnchecked>,
    pub value_sats: u64,
}

impl FromStr for OutputSpec {
    type Err = DomainError;

    /// Parses `address:value_sats`, e.g. `tb1q...:50000`.
   fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (addr_str, value_str) = s.rsplit_once(':').ok_or_else(|| {
            DomainError::InvalidTransaction(format!(
                "output must be `address:value_sats`, got `{s}`"
            ))
        })?;
        let address = Address::from_str(addr_str.trim())
            .map_err(|e| DomainError::InvalidTransaction(format!("address: {e}")))?;
        let value_sats: u64 = value_str
            .trim()
            .parse()
            .map_err(|e| DomainError::InvalidTransaction(format!("value: {e}")))?;
        Ok(OutputSpec { address, value_sats })
    }
}

#[derive(Debug, Serialize)]
pub struct CreatedTx {
    pub version: i32,
    pub locktime: u32,
    pub input_count: usize,
    pub output_count: usize,
    pub raw_hex: String,
    pub txid: String,
}

/// Run `tx create`.
pub fn create(
    inputs: Vec<InputSpec>,
    outputs: Vec<OutputSpec>,
    network: Network,
    version: i32,
    locktime: u32,
) -> Result<CreatedTx> {
    let planned_inputs: Vec<PlannedInput> = inputs
        .into_iter()
        .map(|i| PlannedInput {
            txid: i.txid,
            vout: i.vout,
            sequence: None,
        })
        .collect();

    let planned_outputs: Vec<PlannedOutput> = outputs
        .into_iter()
        .map(|o| {

            if !o.address.is_valid_for_network(network.to_bitcoin()) {
                return Err(DomainError::NetworkMismatch {
                    expected: network.to_string(),
                    actual: "unknown".into(),
                });
            }
            let checked = o.address.assume_checked();
            Ok(PlannedOutput {
                value: Amount::from_sat(o.value_sats),
                script_pubkey: checked.script_pubkey(),
            })
        })
        .collect::<std::result::Result<Vec<_>, DomainError>>()?;

    let plan = TxPlan {
        version,
        locktime,
        inputs: planned_inputs,
        outputs: planned_outputs,
    };

    let tx = transaction::build(plan)?;
    let raw_hex = bitcoin::consensus::encode::serialize_hex(&tx);
    let txid = tx.compute_txid().to_string();

    Ok(CreatedTx {
        version: tx.version.0,
        locktime: tx.lock_time.to_consensus_u32(),
        input_count: tx.input.len(),
        output_count: tx.output.len(),
        raw_hex,
        txid,
    })
}

// Sign

/// A prevout the user provided on the CLI, as `txid:vout:value_sats:script_hex`.
#[derive(Debug, Clone)]
pub struct PrevOutSpec {
    pub script_pubkey: ScriptBuf,
    pub value: Amount,
}

impl FromStr for PrevOutSpec {
    type Err = DomainError;

    /// Format: `<script_pubkey_hex>:<value_sats>`.
  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (script_hex, value_str) = s.rsplit_once(':').ok_or_else(|| {
            DomainError::InvalidTransaction(format!(
                "prevout must be `script_hex:value_sats`, got `{s}`"
            ))
        })?;
        let script_bytes = hex::decode(script_hex.trim())
            .map_err(|e| DomainError::InvalidTransaction(format!("script hex: {e}")))?;
        let value_sats: u64 = value_str
            .trim()
            .parse()
            .map_err(|e| DomainError::InvalidTransaction(format!("value: {e}")))?;
        Ok(PrevOutSpec {
            script_pubkey: ScriptBuf::from_bytes(script_bytes),
            value: Amount::from_sat(value_sats),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SignedTx {
    pub txid: String,
    pub wtxid: String,
    pub raw_hex: String,
}

/// Run `tx sign`.
pub fn sign(
    raw_hex: &str,
    prevouts: Vec<PrevOutSpec>,
    private_key_str: &str,
    network: Network,
) -> Result<SignedTx> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| DomainError::InvalidTransaction(format!("hex: {e}")))?;
    let tx: Transaction = bitcoin::consensus::deserialize(&bytes)
        .map_err(|e| DomainError::InvalidTransaction(e.to_string()))?;

    let private_key = keys::parse_private_key(private_key_str, network)?;

    let prev = prevouts
        .into_iter()
        .map(|p| PrevOut {
            script_pubkey: p.script_pubkey,
            value: p.value,
        })
        .collect::<Vec<_>>();

    let signed = transaction::sign(tx, &prev, &private_key)?;
    let raw_hex = bitcoin::consensus::encode::serialize_hex(&signed);
    let txid = signed.compute_txid().to_string();
    let wtxid = signed.compute_wtxid().to_string();

    Ok(SignedTx { txid, wtxid, raw_hex })
}

// ---------------------------------------------------------------------------
// Broadcast
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct BroadcastResult {
    pub txid: String,
    pub network: String,
}

/// Run `tx broadcast`. Requires a connected node.
pub fn broadcast(
    raw_hex: &str,
    rpc: &crate::infrastructure::rpc::RpcClient,
    network: Network,
) -> Result<BroadcastResult> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| DomainError::InvalidTransaction(format!("hex: {e}")))?;
    let tx: Transaction = bitcoin::consensus::deserialize(&bytes)
        .map_err(|e| DomainError::InvalidTransaction(e.to_string()))?;

    let txid = rpc.send_raw_transaction(&tx)?;

    Ok(BroadcastResult {
        txid: txid.to_string(),
        network: network.to_string(),
    })
}