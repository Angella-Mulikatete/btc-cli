use thiserror::Error;

use crate::domain::errors::DomainError;
use crate::infrastructure::rpc::RpcError;

/// The single error type returned by every fallible function in the crate.
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Rpc(#[from] RpcError),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    /// Process exit code. 0 = success (never returned here),
    /// 1 = generic, 2 = user input, 3 = config, 4 = RPC, 5 = I/O, 6 = serialization.
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Domain(e)        => e.exit_code(),
            AppError::Config(_)        => 3,
            AppError::Rpc(e)           => e.exit_code(),
            AppError::Io(_)            => 5,
            AppError::Serialization(_) => 6,
            AppError::Other(_)         => 1,
        }
    }

    /// Short machine-readable kind, used when `--json` is set on error.
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Domain(_)        => "domain",
            AppError::Config(_)        => "config",
            AppError::Rpc(_)           => "rpc",
            AppError::Io(_)            => "io",
            AppError::Serialization(_) => "serialization",
            AppError::Other(_)         => "other",
        }
    }
}