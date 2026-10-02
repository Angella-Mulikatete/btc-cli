// src/presentation/output.rs

use crate::application::addresses::{DerivedAddress, ValidatedAddress};
use crate::application::blocks::BlockDetails;
use crate::application::derive::Derived;
use crate::application::fees::FeeEstimateOut;
use crate::application::keys::GeneratedKey;
use crate::application::mnemonic::NewMnemonic;
use crate::application::transactions::{
    BroadcastResult, CreatedTx, DecodedTx, SignedTx,
};

/// Which output format to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

/// Anything the CLI can print in human-readable form.
pub trait Human {
    fn print_human(&self);
}

/// Helper: print an aligned `label: value` line.
fn kv(label: &str, value: impl std::fmt::Display) {
    println!("{:<14} {}", format!("{label}:"), value);
}

/// Separator line for grouped output.
fn rule() {
    println!("{}", "-".repeat(40));
}

// ---------------------------------------------------------------------------
// key generate
// ---------------------------------------------------------------------------

impl Human for GeneratedKey {
    fn print_human(&self) {
        kv("Network", &self.network);
        kv("Public key", &self.public_hex);
        if let Some(h) = &self.private_hex {
            kv("Private key", h);
        }
        if let Some(w) = &self.wif {
            kv("WIF", w);
        }
    }
}

// ---------------------------------------------------------------------------
// mnemonic new
// ---------------------------------------------------------------------------

impl Human for NewMnemonic {
    fn print_human(&self) {
        kv("Language", &self.language);
        kv("Word count", self.word_count);
        println!();
        println!("{}", self.mnemonic);
    }
}

// ---------------------------------------------------------------------------
// derive
// ---------------------------------------------------------------------------

impl Human for Derived {
    fn print_human(&self) {
        kv("Network", &self.network);
        kv("Source", &self.source_kind);
        kv("Path", &self.path);
        kv("Address type", &self.address_type);
        kv("Address", &self.address);
        kv("Public key", &self.public_hex);
        if let Some(h) = &self.private_hex {
            kv("Private key", h);
        }
        if let Some(w) = &self.wif {
            kv("WIF", w);
        }
    }
}

// ---------------------------------------------------------------------------
// address from-pubkey
// ---------------------------------------------------------------------------

impl Human for DerivedAddress {
    fn print_human(&self) {
        kv("Network", &self.network);
        kv("Address type", &self.address_type);
        kv("Address", &self.address);
        kv("Public key", &self.public_hex);
        kv("Script", &self.script_pubkey_hex);
    }
}

// ---------------------------------------------------------------------------
// address validate
// ---------------------------------------------------------------------------

impl Human for ValidatedAddress {
    fn print_human(&self) {
        kv("Address", &self.address);
        kv("Type", &self.address_type);
        kv("Network", &self.network);
        kv("Script", &self.script_pubkey_hex);
        println!("OK");
    }
}

// ---------------------------------------------------------------------------
// tx decode
// ---------------------------------------------------------------------------

impl Human for DecodedTx {
    fn print_human(&self) {
        kv("txid", &self.txid);
        kv("wtxid", &self.wtxid);
        kv("Version", self.version);
        kv("Locktime", self.locktime);
        kv("Size", format!("{} bytes", self.size));
        kv("Vsize", format!("{} vB", self.vsize));
        kv("Weight", format!("{} WU", self.weight));
        kv("Segwit", self.is_segwit);

        println!();
        println!("Inputs ({}):", self.inputs.len());
        for i in &self.inputs {
            println!(
                "  [{}] {}:{} seq=0x{:08x} witness_items={}",
                i.index, i.txid, i.vout, i.sequence, i.witness_items
            );
        }

        println!();
        println!("Outputs ({}):", self.outputs.len());
        for o in &self.outputs {
            let addr = o.address.as_deref().unwrap_or("(non-standard)");
            println!(
                "  [{}] {:>12} sats  {}",
                o.index, o.value_sats, addr
            );
        }
    }
}

// ---------------------------------------------------------------------------
// tx create
// ---------------------------------------------------------------------------

impl Human for CreatedTx {
    fn print_human(&self) {
        kv("Version", self.version);
        kv("Locktime", self.locktime);
        kv("Inputs", self.input_count);
        kv("Outputs", self.output_count);
        kv("txid", &self.txid);
        println!();
        println!("Unsigned transaction (hex):");
        println!("{}", self.raw_hex);
    }
}

// ---------------------------------------------------------------------------
// tx sign
// ---------------------------------------------------------------------------

impl Human for SignedTx {
    fn print_human(&self) {
        kv("txid", &self.txid);
        kv("wtxid", &self.wtxid);
        println!();
        println!("Signed transaction (hex):");
        println!("{}", self.raw_hex);
    }
}

// ---------------------------------------------------------------------------
// tx broadcast
// ---------------------------------------------------------------------------

impl Human for BroadcastResult {
    fn print_human(&self) {
        kv("Network", &self.network);
        kv("txid", &self.txid);
        println!("Broadcast accepted.");
    }
}

// ---------------------------------------------------------------------------
// block info
// ---------------------------------------------------------------------------

impl Human for BlockDetails {
    fn print_human(&self) {
        kv("Hash", &self.hash);
        kv("Height", self.height);
        kv("Confirmations", self.confirmations);
        kv("Time", self.time);
        kv("Median time", self.median_time);
        kv("Version", self.version);
        kv("Merkle root", &self.merkle_root);
        if let Some(p) = &self.previous_block_hash {
            kv("Previous block", p);
        }
        if let Some(n) = &self.next_block_hash {
            kv("Next block", n);
        }
        kv("Nonce", self.nonce);
        kv("Bits", &self.bits);
        kv("Difficulty", self.difficulty);
        kv("Chainwork", &self.chainwork);
        kv("Transactions", self.n_tx);
    }
}

// ---------------------------------------------------------------------------
// fee estimate
// ---------------------------------------------------------------------------

impl Human for FeeEstimateOut {
    fn print_human(&self) {
        rule();
        kv("Target", format!("{} blocks", self.target_blocks));
        kv("Fee rate", format!("{:.2} sat/vB", self.sat_per_vb));
    }
}