
use btc_cli::domain::network::Network;
use btc_cli::domain::transaction;

/// The first-ever Bitcoin transaction (Satoshi → Hal Finney, block 170).
const SATOSHI_TX: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";

#[test]
fn decode_known_mainnet_transaction() {
    let info = transaction::decode(SATOSHI_TX, Network::Regtest).unwrap();

    assert_eq!(
        info.txid,
        "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"
    );
    assert_eq!(info.wtxid, info.txid); // not segwit, so txid == wtxid
    assert_eq!(info.version, 1);
    assert_eq!(info.locktime, 0);
    assert_eq!(info.size, 275);
    assert_eq!(info.vsize, 275);
    assert_eq!(info.weight, 1100);
    assert!(!info.is_segwit);
    assert_eq!(info.inputs.len(), 1);
    assert_eq!(info.outputs.len(), 2);
    assert_eq!(info.outputs[0].value_sats, 1_000_000_000);
    assert_eq!(info.outputs[1].value_sats, 4_000_000_000);
}

#[test]
fn decode_rejects_garbage() {
    let err = transaction::decode("zzzz", Network::Regtest).unwrap_err();
    let s = format!("{err}");
    assert!(s.contains("hex"), "got: {s}");
}

#[test]
fn decode_rejects_truncated_bytes() {
    // Valid hex, but not a valid transaction.
    let err = transaction::decode("0100", Network::Regtest).unwrap_err();
    let s = format!("{err}");
    assert!(s.len() > 0, "expected an error message");
}