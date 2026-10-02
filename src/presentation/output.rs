// use std::fmt;
// use serde::Serialize;

// use crate::application::keys::GeneratedKey;
// use crate::error::Result;

// pub fn print<T>(value:&T, json: bool) -> Result<()>
// where
//     T: Serialize + fmt::Display,
// {
//     if json {
//         let json_string = serde_json::to_string_pretty(value)?;
//         println!("{}", json_string);
//     } else {
//           println!("{value}");
//     }
//     Ok(())
// }

// impl fmt::Display for GeneratedKey {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "Network: {}\nPrivate Key (WIF): {}\nPublic Key (Hex): {}\nPublic Key: {}",
//             self.network, self.private_key_wif, self.public_key_hex, self.public_key
//         )
//     }
// }