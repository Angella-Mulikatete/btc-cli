
use serde::Serialize;

use crate::application::{addresses, blocks, derive, fees, keys, mnemonic, transactions};
use crate::cli::args::{
    AddressCmd, BlockCmd, Command, FeeCmd, KeyCmd, MnemonicCmd, TxCmd,
};
use crate::cli::Context;
use crate::domain::errors::DomainError;
use crate::error::{AppError, Result};
use crate::presentation::json;
use crate::presentation::output::Human;

pub fn dispatch(command: Command, ctx: &Context) -> Result<()> {
    match command {
        Command::Key { command } => match command {
            KeyCmd::Generate => {
                let out = keys::generate(ctx.network, ctx.show_secret)?;
                emit(&out, ctx)
            }
        },

        Command::Mnemonic { command } => match command {
            MnemonicCmd::New { words } => {
                let out = mnemonic::new(words)?;
                emit(&out, ctx)
            }
        },

        Command::Derive(args) => {
            let out = derive::derive(
                &args.extended_key,
                &args.path,
                ctx.network,
                args.address_type,
            )?;
            emit(&out, ctx)
        }

        Command::Address { command } => match command {
            AddressCmd::FromPubkey { public_key, address_type } => {
                let out = addresses::from_pubkey(&public_key, address_type, ctx.network)?;
                emit(&out, ctx)
            }
            AddressCmd::Validate { address } => {
                let out = addresses::validate(&address, ctx.network)?;
                emit(&out, ctx)
            }
        },

        Command::Tx { command } => match command {
            TxCmd::Decode { raw_hex } => {
                let out = transactions::decode(&raw_hex, ctx.network)?;
                emit(&out, ctx)
            }
            TxCmd::Create { inputs, outputs, version, locktime } => {
                let input_specs: Vec<transactions::InputSpec> = parse_all(&inputs)?;
                let output_specs: Vec<transactions::OutputSpec> = parse_all(&outputs)?;
                let out = transactions::create(
                    input_specs,
                    output_specs,
                    ctx.network,
                    version,
                    locktime,
                )?;
                emit(&out, ctx)
            }
            TxCmd::Sign { raw_hex, prevouts, key } => {
                let specs: Vec<transactions::PrevOutSpec> = parse_all(&prevouts)?;
                let out = transactions::sign(&raw_hex, specs, &key, ctx.network)?;
                emit(&out, ctx)
            }
            TxCmd::Broadcast { raw_hex } => {
                let rpc = ctx.rpc_client()?;
                let out = transactions::broadcast(&raw_hex, &rpc, ctx.network)?;
                emit(&out, ctx)
            }
        },

        Command::Block { command } => match command {
            BlockCmd::Info { reference } => {
                let rpc = ctx.rpc_client()?;
                let out = blocks::info(&rpc, &reference)?;
                emit(&out, ctx)
            }
        },

        Command::Fee { command } => match command {
            FeeCmd::Estimate { target } => {
                let rpc = ctx.rpc_client()?;
                let out = fees::estimate(&rpc, target)?;
                emit(&out, ctx)
            }
        },
    }
}

fn parse_all<T>(items: &[String]) -> Result<Vec<T>>
where
    T: std::str::FromStr,
    T::Err: Into<AppError>,
{
    items
        .iter()
        .map(|s| s.parse::<T>().map_err(Into::into))
        .collect::<Result<Vec<T>>>()
}

/// Print in whichever format the user asked for.
fn emit<T>(value: &T, ctx: &Context) -> Result<()>
where
    T: Serialize + Human,
{
    if ctx.json {
        json::print(value)
    } else {
        value.print_human();
        Ok(())
    }
}
#[allow(unused_imports)]
use DomainError as _;