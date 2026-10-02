
use bitcoin::absolute::LockTime;
use bitcoin::transaction::Version;
use bitcoin::{Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid};
use bitcoin::psbt::Psbt;
use bitcoin::key::PrivateKey;
use bitcoin::secp256k1::Secp256k1;


use super::errors::DomainError;
use super::network::Network;

// ---------------------------------------------------------------------------
// Decode
// ---------------------------------------------------------------------------


#[derive(Debug, Clone)]
pub struct PrevOut {
    pub script_pubkey: ScriptBuf,
    pub value: Amount,
}

/// Decoded summary of a raw transaction.
#[derive(Debug, Clone)]
pub struct TxInfo {
    pub txid: String,
    pub wtxid: String,
    pub version: i32,
    pub locktime: u32,
    pub size: usize,
    pub vsize: usize,
    pub weight: usize,
    pub is_segwit: bool,
    pub inputs: Vec<TxInputInfo>,
    pub outputs: Vec<TxOutputInfo>,
}

#[derive(Debug, Clone)]
pub struct TxInputInfo {
    pub index: usize,
    pub txid: String,
    pub vout: u32,
    pub script_sig_hex: String,
    pub sequence: u32,
    pub witness_items: usize,
}

#[derive(Debug, Clone)]
pub struct TxOutputInfo {
    pub index: usize,
    pub value_sats: u64,
    pub script_pubkey_hex: String,
    pub address: Option<String>,
}

/// Decode a raw transaction hex string.
pub fn decode(hex_str: &str, network: Network) -> Result<TxInfo, DomainError> {
    let bytes = hex::decode(hex_str.trim())
        .map_err(|e| DomainError::InvalidTransaction(format!("hex decode: {e}")))?;

    let tx: Transaction = bitcoin::consensus::deserialize(&bytes)
        .map_err(|e| DomainError::InvalidTransaction(e.to_string()))?;

    let txid = tx.compute_txid().to_string();
    let wtxid = tx.compute_wtxid().to_string();
    let size = tx.total_size();
    let vsize = tx.vsize();
    let weight: usize = tx.weight().to_wu() as usize;
    let is_segwit = tx.input.iter().any(|i| !i.witness.is_empty());

    let inputs = tx
        .input
        .iter()
        .enumerate()
        .map(|(index, inp)| TxInputInfo {
            index,
            txid: inp.previous_output.txid.to_string(),
            vout: inp.previous_output.vout,
            script_sig_hex: hex::encode(inp.script_sig.as_bytes()),
            sequence: inp.sequence.0,
            witness_items: inp.witness.len(),
        })
        .collect();

    let outputs = tx
        .output
        .iter()
        .enumerate()
        .map(|(index, out)| {
            let address = bitcoin::Address::from_script(
                &out.script_pubkey,
                network.to_bitcoin(),
            )
            .ok()
            .map(|a| a.to_string());

            TxOutputInfo {
                index,
                value_sats: out.value.to_sat(),
                script_pubkey_hex: hex::encode(out.script_pubkey.as_bytes()),
                address,
            }
        })
        .collect();

    Ok(TxInfo {
        txid,
        wtxid,
        version: tx.version.0,
        locktime: tx.lock_time.to_consensus_u32(),
        size,
        vsize,
        weight,
        is_segwit,
        inputs,
        outputs,
    })
}

// ---------------------------------------------------------------------------
// Build (unsigned)
// ---------------------------------------------------------------------------

/// Everything needed to construct an *unsigned* transaction.
///
/// Signing is a separate step and requires additional inputs (the previous
/// outputs' scriptPubKeys, and the private keys). That is deliberately not
/// modelled here.
#[derive(Debug, Clone)]
pub struct TxPlan {
    pub version: i32,
    pub locktime: u32,
    pub inputs: Vec<PlannedInput>,
    pub outputs: Vec<PlannedOutput>,
}

#[derive(Debug, Clone)]
pub struct PlannedInput {
    pub txid: Txid,
    pub vout: u32,
    /// `None` means use the default (RBF-enabled).
    pub sequence: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct PlannedOutput {
    pub value: Amount,
    pub script_pubkey: ScriptBuf,
}

/// Build an unsigned transaction from a plan.
pub fn build(plan: TxPlan) -> Result<Transaction, DomainError> {
    if plan.inputs.is_empty() {
        return Err(DomainError::InvalidTransaction(
            "transaction must have at least one input".into(),
        ));
    }
    if plan.outputs.is_empty() {
        return Err(DomainError::InvalidTransaction(
            "transaction must have at least one output".into(),
        ));
    }

    let version = match plan.version {
        1 => Version::ONE,
        2 => Version::TWO,
        other => {
            return Err(DomainError::InvalidTransaction(format!(
                "unsupported transaction version {other} (use 1 or 2)"
            )));
        }
    };

    let lock_time = LockTime::from_consensus(plan.locktime);

    let input: Vec<TxIn> = plan
        .inputs
        .into_iter()
        .map(|i| TxIn {
            previous_output: OutPoint {
                txid: i.txid,
                vout: i.vout,
            },
            script_sig: ScriptBuf::new(),
            sequence: i
                .sequence
                .map(Sequence)
                .unwrap_or(Sequence::ENABLE_RBF_NO_LOCKTIME),
            witness: bitcoin::Witness::new(),
        })
        .collect();

    let output: Vec<TxOut> = plan
        .outputs
        .into_iter()
        .map(|o| TxOut {
            value: o.value,
            script_pubkey: o.script_pubkey,
        })
        .collect();

    Ok(Transaction {
        version,
        lock_time,
        input,
        output,
    })
}

pub fn sign(
    tx: Transaction,
    prevouts: &[PrevOut],
    private_key: &PrivateKey,
) -> Result<Transaction, DomainError> {
    if prevouts.len() != tx.input.len() {
        return Err(DomainError::InvalidTransaction(format!(
            "need one prevout per input: got {} prevouts for {} inputs",
            prevouts.len(),
            tx.input.len()
        )));
    }

    let mut psbt = Psbt::from_unsigned_tx(tx)
        .map_err(|e| DomainError::InvalidTransaction(format!("psbt: {e}")))?;

    for (i, prevout) in prevouts.iter().enumerate() {
        psbt.inputs[i].witness_utxo = Some(TxOut {
            value: prevout.value,
            script_pubkey: prevout.script_pubkey.clone(),
        });
    }

    let secp = Secp256k1::new();
    let pubkey = private_key.public_key(&secp);

    use std::collections::HashMap;
    let mut keys: HashMap<bitcoin::PublicKey, PrivateKey> = HashMap::new();
    keys.insert(pubkey, *private_key);

    let successes = match psbt.sign(&keys, &secp) {
        Ok(map) => map,
        Err((_partial, failures)) => {
            let failed: Vec<usize> = failures.keys().copied().collect();
            return Err(DomainError::InvalidTransaction(format!(
                "could not sign inputs {failed:?} \
                 (do the prevouts match the key, and are the input types supported?)"
            )));
        }
    };

    if successes.len() != psbt.inputs.len() {
        return Err(DomainError::InvalidTransaction(format!(
            "signed {} of {} inputs",
            successes.len(),
            psbt.inputs.len()
        )));
    }

    psbt.extract_tx()
        .map_err(|e| DomainError::InvalidTransaction(format!("extract: {e}")))
}