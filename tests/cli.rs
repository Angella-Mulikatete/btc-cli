// use assert_cmd::Command;
// use predicates::str::contains;

// fn btc_cli() -> Command {
//     Command::cargo_bin("btc-cli").unwrap()
// }

// #[test]
// fn key_generate_prints_text() {
//     btc_cli()
//         .args(["key", "generate"])
//         .assert()
//         .success()
//         .stdout(contains("Private key (WIF)"));
// }

// #[test]
// fn key_generate_json_is_valid() {
//     let output = btc_cli()
//         .args(["--network", "signet", "--json", "key", "generate"])
//         .output()
//         .unwrap();
//     assert!(output.status.success());

//     let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
//     assert_eq!(json["network"], "signet");
//     assert_eq!(json["public_key"].as_str().unwrap().len(), 66);
//     assert_eq!(json["private_key_hex"].as_str().unwrap().len(), 64);
// }

// #[test]
// fn mainnet_is_rejected() {
//     btc_cli()
//         .args(["--network", "bitcoin", "key", "generate"])
//         .assert()
//         .failure();
// }
