// src/application/mnemonic.rs

use bip39::{Language, Mnemonic, WordCount};
use serde::Serialize;

use crate::domain::errors::DomainError;
use crate::error::Result;

/// JSON-serialisable output of `mnemonic new`.
#[derive(Debug, Serialize)]
pub struct NewMnemonic {
    pub mnemonic: String,
    pub word_count: usize,
    pub language: String,
}

/// Run `mnemonic new`.
///
/// `word_count` must be one of the BIP39 standard sizes:
/// 12, 15, 18, 21, or 24.
pub fn new(word_count: usize) -> Result<NewMnemonic> {
    let wc = match word_count {
        12 => WordCount::Words12,
        15 => WordCount::Words15,
        18 => WordCount::Words18,
        21 => WordCount::Words21,
        24 => WordCount::Words24,
        other => {
            return Err(DomainError::InvalidMnemonic(format!(
                "word count must be one of 12, 15, 18, 21, 24 (got {other})"
            ))
            .into());
        }
    };

    let mnemonic = Mnemonic::generate_in(Language::English, wc)
        .map_err(|e| DomainError::InvalidMnemonic(e.to_string()))?;

    Ok(NewMnemonic {
        mnemonic: mnemonic.to_string(),
        word_count,
        language: "english".to_string(),
    })
}