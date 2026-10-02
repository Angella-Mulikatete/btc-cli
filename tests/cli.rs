
use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("btc-cli").unwrap()
}

#[test]
fn mnemonic_new_default_12_words() {
    cmd()
        .args(["--network", "regtest", "mnemonic", "new"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Word count:    12"));
}

#[test]
fn mnemonic_new_json_has_expected_fields() {
    let out = cmd()
        .args(["--network", "regtest", "--json", "mnemonic", "new", "--words", "24"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["word_count"], 24);
    assert_eq!(v["language"], "english");
    assert!(v["mnemonic"].as_str().unwrap().split_whitespace().count() == 24);
}

#[test]
fn mnemonic_new_rejects_bad_word_count() {
    cmd()
        .args(["--network", "regtest", "mnemonic", "new", "--words", "13"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("word count"));
}

#[test]
fn key_generate_default_hides_secret() {
    cmd()
        .args(["--network", "regtest", "key", "generate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Public key:"))
        .stdout(predicate::str::contains("Private key:").not())
        .stdout(predicate::str::contains("WIF:").not());
}

#[test]
fn key_generate_with_show_secret_includes_wif() {
    cmd()
        .args(["--network", "regtest", "--show-secret", "key", "generate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("WIF:"))
        .stdout(predicate::str::contains("Private key:"));
}

#[test]
fn key_generate_regtest_wif_starts_with_c() {
    let out = cmd()
        .args(["--network", "regtest", "--json", "--show-secret", "key", "generate"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let wif = v["wif"].as_str().unwrap();
    assert!(
        wif.starts_with('c'),
        "regtest WIF should start with 'c', got {wif}"
    );
}

#[test]
fn block_info_without_rpc_exits_4() {
    // Ensure env is clean so we get the "not configured" path.
    cmd()
        .env_remove("BTC_RPC_URL")
        .env_remove("BTC_RPC_USER")
        .env_remove("BTC_RPC_PASSWORD")
        .env_remove("BTC_RPC_COOKIE")
        .args(["--network", "regtest", "block", "info", "0"])
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("RPC not configured"));
}

#[test]
fn block_info_without_rpc_in_json_mode_prints_json_error() {
    let out = cmd()
        .env_remove("BTC_RPC_URL")
        .env_remove("BTC_RPC_USER")
        .env_remove("BTC_RPC_PASSWORD")
        .env_remove("BTC_RPC_COOKIE")
        .args(["--network", "regtest", "--json", "block", "info", "0"])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(4));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["error"]["kind"], "rpc");
}

#[test]
fn address_validate_rejects_mainnet_on_regtest() {
    cmd()
        .args([
            "--network", "regtest",
            "address", "validate",
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn tx_decode_known_transaction() {
    let tx_hex = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";

    cmd()
        .args(["--network", "regtest", "tx", "decode", tx_hex])
        .assert()
        .success()
        .stdout(predicate::str::contains("f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"))
        .stdout(predicate::str::contains("Segwit:        false"));
}