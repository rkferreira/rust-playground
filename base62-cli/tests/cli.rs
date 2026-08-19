// tests/cli.rs
use assert_cmd::Command;
use predicates::prelude::*;
use base62_generator::encode;

#[test]
fn test_cli_encode_decode_roundtrip() {
    // Prepare a hex input
    let hex_input = "deadbeef";
    // Expected Base62 using library
    let bytes = hex::decode(hex_input).unwrap();
    let expected_base62 = encode(&bytes);

    // Run the CLI encode command
    let mut cmd = Command::cargo_bin("base62-cli").unwrap();
    cmd.arg("encode").arg(hex_input);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&expected_base62));

    // Run the CLI decode command on the output
    let mut decode_cmd = Command::cargo_bin("base62-cli").unwrap();
    decode_cmd.arg("decode").arg(&expected_base62);
    decode_cmd.assert()
        .success()
        .stdout(predicate::str::contains(hex_input));
}
