// use crate::application::keys;
// use crate::error::Result;
// use crate::presentation::output;

// use super::args::{Cli, Command, KeyCommand};

// pub fn run(cli: Cli) -> Result<()> {
//     match cli.command {
//         Command::Key(KeyCommand::Generate) => {
//             let key = keys::generate(cli.network);
//             output::print(&key, cli.json)
//         }
//     }
// }
