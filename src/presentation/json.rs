
use serde::Serialize;

use crate::error::Result;

/// Print any serialisable value as pretty JSON on stdout.
pub fn print<T: Serialize>(value: &T) -> Result<()> {
    let s = serde_json::to_string_pretty(value)?;
    println!("{s}");
    Ok(())
}


pub fn print_error(kind: &str, message: &str) -> Result<()> {
    let obj = serde_json::json!({
        "error": {
            "kind": kind,
            "message": message,
        }
    });
    println!("{}", serde_json::to_string_pretty(&obj)?);
    Ok(())
}