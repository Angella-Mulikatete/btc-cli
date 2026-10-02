use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    #[error("invalid address: {0}")]
    InvalidAddress(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("network mismatch: expected {expected}, got {actual}")]
    NetworkMismatch { expected: String, actual: String },

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),
}

impl DomainError {
    /// All domain errors are user-facing input problems → exit code 2.
    pub fn exit_code(&self) -> i32 {
        2
    }
}