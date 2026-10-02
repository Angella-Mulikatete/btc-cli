// src/cli/args.rs

use clap::{Args, Parser, Subcommand};

use crate::domain::address::AddressType;
use crate::domain::network::Network;

/// Bitcoin CLI — a command-line toolkit for common Bitcoin tasks.
#[derive(Debug, Parser)]
#[command(name = "btc-cli", version, about, long_about = None)]
pub struct Cli {
    /// Network to operate on (testnet4, signet, regtest).
    #[arg(long, global = true, default_value = "regtest")]
    pub network: Network,

    /// Output machine-readable JSON instead of human-readable text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Include secret material (private keys, WIFs) in the output.
    #[arg(long, global = true)]
    pub show_secret: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Key-related commands.
    Key {
        #[command(subcommand)]
        command: KeyCmd,
    },

    /// Mnemonic commands.
    Mnemonic {
        #[command(subcommand)]
        command: MnemonicCmd,
    },

    /// BIP32 key derivation.
    Derive(DeriveArgs),

    /// Address commands.
    Address {
        #[command(subcommand)]
        command: AddressCmd,
    },

    /// Transaction commands.
    Tx {
        #[command(subcommand)]
        command: TxCmd,
    },

    /// Block commands.
    Block {
        #[command(subcommand)]
        command: BlockCmd,
    },

    /// Fee commands.
    Fee {
        #[command(subcommand)]
        command: FeeCmd,
    },
}



#[derive(Debug, Subcommand)]
pub enum KeyCmd {
    /// Generate a fresh secp256k1 keypair.
    Generate,
}


#[derive(Debug, Subcommand)]
pub enum MnemonicCmd {
    /// Generate a fresh BIP39 mnemonic.
    New {
        /// Word count: one of 12, 15, 18, 21, 24.
        #[arg(long, default_value_t = 12)]
        words: usize,
    },
}



#[derive(Debug, Args)]
pub struct DeriveArgs {
    /// Extended key: xprv or xpub.
    pub extended_key: String,

    /// Derivation path, e.g. `m/84'/1'/0'/0/0`.
    pub path: String,

    /// Address type. If omitted, inferred from the path's purpose field.
    #[arg(long = "type")]
    pub address_type: Option<AddressType>,
}


#[derive(Debug, Subcommand)]
pub enum AddressCmd {
    /// Derive an address from a public key.
    FromPubkey {
        /// Public key in hex (compressed or uncompressed).
        public_key: String,

        /// Address type.
        #[arg(long = "type", default_value = "p2wpkh")]
        address_type: AddressType,
    },

    /// Validate an address and report its type and network.
    Validate {
        /// The address to validate.
        address: String,
    },
}


#[derive(Debug, Subcommand)]
pub enum TxCmd {
    /// Decode a raw transaction hex.
    Decode {
        /// Raw transaction hex.
        raw_hex: String,
    },

    /// Build an unsigned transaction from explicit inputs and outputs.
    Create {
        /// Input as `txid:vout`. Repeat for multiple inputs.
        #[arg(long = "input", required = true)]
        inputs: Vec<String>,

        /// Output as `address:value_sats`. Repeat for multiple outputs.
        #[arg(long = "output", required = true)]
        outputs: Vec<String>,

        /// Transaction version (1 or 2).
        #[arg(long, default_value_t = 2)]
        version: i32,

        /// Locktime (block height or unix timestamp).
        #[arg(long, default_value_t = 0)]
        locktime: u32,
    },

    /// Sign an unsigned transaction with one private key.
    Sign {
        /// Raw unsigned transaction hex.
        raw_hex: String,

        /// Previous output as `script_pubkey_hex:value_sats`.
        /// Repeat in the same order as the transaction's inputs.
        #[arg(long = "prevout", required = true)]
        prevouts: Vec<String>,

        /// Private key as WIF or 32-byte hex.
        #[arg(long = "key")]
        key: String,
    },

    /// Broadcast a signed transaction via the connected node.
    Broadcast {
        /// Raw signed transaction hex.
        raw_hex: String,
    },
}



#[derive(Debug, Subcommand)]
pub enum BlockCmd {
    /// Fetch block details by height or hash.
    Info {
        /// Block height (integer) or 64-character hex hash.
        reference: String,
    },
}



#[derive(Debug, Subcommand)]
pub enum FeeCmd {
    /// Get a fee rate estimate from the node.
    Estimate {
        /// Confirmation target in blocks.
        #[arg(long, default_value_t = 6)]
        target: u16,
    },
}