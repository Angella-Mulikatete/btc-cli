// use clap::{Parser, Subcommand};

// use crate::domain::network::Network;

// #[derive(Debug, Parser)]
// #[command(name = "btc-cli", version, about)]
// pub struct Cli {
//     ///Network to use
//     #[arg(long, global = true, value_enum, default_value_t = Network::Regtest)]
//     pub network: Network,

//     ///print machine-readable JSON instead of text
//     #[arg(long, global = true)]
//     pub json: bool,

//     #[command(subcommand)]
//     pub command: Command,
// }

// #[derive(Debug, Subcommand)]
// pub enum Command {
//     ///work with private/public key pairs
//     #[command(subcommand)]
//     Key(KeyCommand),
// }

// #[derive(Debug, Subcommand)]
// pub enum KeyCommand {
//     ///generate a new private/public key pair
//     Generate,
// }