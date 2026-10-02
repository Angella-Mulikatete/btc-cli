// src/cli/mod.rs

pub mod args;
pub mod commands;

use std::process::ExitCode;
use clap::Parser;

use crate::domain::network::Network;
use crate::error::{AppError, Result};
use crate::infrastructure::config::RpcConfig;
use crate::infrastructure::rpc::RpcClient;
use crate::presentation::json;


/// Per-invocation state shared by every command handler.
pub struct Context {
    pub network: Network,
    pub json: bool,
    pub show_secret: bool,
}

impl Context {
    /// Connect to the node. Fails cleanly if RPC isn't configured.
    pub fn rpc_client(&self) -> Result<RpcClient> {
        let config = RpcConfig::from_env(self.network)?;
        let client = RpcClient::connect(&config, self.network)?;
        Ok(client)
    }
}

pub fn run() -> ExitCode {
    let cli = args::Cli::parse();
    let ctx = Context {
        network: cli.network,
        json: cli.json,
        show_secret: cli.show_secret,
    };

    match commands::dispatch(cli.command, &ctx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if ctx.json {
                // Best-effort: if even JSON printing fails, fall back to stderr.
                let _ = json::print_error(err.kind(), &err.to_string());
            } else {
                eprintln!("error: {err}");
            }
            ExitCode::from(err.exit_code() as u8)
        }
    }
}

#[allow(unused_imports)]
use AppError as _;