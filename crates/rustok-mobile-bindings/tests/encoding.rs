//! Integration tests — hex/decimal parser edge cases at the FFI boundary.
//!
//! Validates that parser helpers (`parse_address`, `parse_u256`,
//! `parse_bytes`, `parse_b256`, `parse_hex_bytes` used internally by
//! `analyze_transaction` and signing methods) handle:
//! - Empty strings
//! - Missing `0x` prefix
//! - Mixed-case Address (EIP-55 checksum)
//! - All-lowercase Address
//! - Decimal U256 boundary values

use rustok_mobile_bindings::{BindingsError, EncodingErrorKind, analyze_transaction};

#[tokio::test]
async fn analyze_accepts_eip55_mixed_case_address() {
    let verdict = analyze_transaction(
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".into(),
        "0x".into(),
        "0".into(),
    )
    .expect("EIP-55 mixed case must parse");
    assert_eq!(verdict.findings.len(), 0);
}

#[tokio::test]
async fn analyze_accepts_lowercase_address() {
    let verdict = analyze_transaction(
        "0xd8da6bf26964af9d7eed9e03e53415d37aa96045".into(),
        "0x".into(),
        "0".into(),
    )
    .expect("lowercase address must parse");
    assert_eq!(verdict.findings.len(), 0);
}

#[tokio::test]
async fn analyze_accepts_address_without_0x_prefix() {
    // alloy's `Address::FromStr` accepts both `0x`-prefixed and bare hex.
    // Document this permissive parsing — `parse_address` does not enforce
    // prefix presence, only valid 20-byte hex.
    let verdict = analyze_transaction(
        "d8da6bf26964af9d7eed9e03e53415d37aa96045".into(),
        "0x".into(),
        "0".into(),
    )
    .expect("bare 20-byte hex must parse");
    assert_eq!(verdict.findings.len(), 0);
}

#[tokio::test]
async fn analyze_rejects_address_wrong_length() {
    let err = analyze_transaction("0xdeadbeef".into(), "0x".into(), "0".into()).unwrap_err();
    assert!(matches!(
        err,
        BindingsError::Encoding {
            kind: EncodingErrorKind::Address
        }
    ));
}

#[tokio::test]
async fn analyze_rejects_empty_address() {
    let err = analyze_transaction("".into(), "0x".into(), "0".into()).unwrap_err();
    assert!(matches!(
        err,
        BindingsError::Encoding {
            kind: EncodingErrorKind::Address
        }
    ));
}

#[tokio::test]
async fn analyze_accepts_zero_value() {
    let verdict = analyze_transaction(
        "0x0000000000000000000000000000000000000001".into(),
        "0x".into(),
        "0".into(),
    )
    .expect("zero value must parse");
    let _ = verdict.action;
}

#[tokio::test]
async fn analyze_accepts_max_u256_value() {
    let max_u256 = "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        .to_string();
    let verdict = analyze_transaction(
        "0x0000000000000000000000000000000000000001".into(),
        "0x".into(),
        max_u256,
    )
    .expect("max U256 decimal must parse");
    let _ = verdict.action;
}

#[tokio::test]
async fn analyze_rejects_u256_overflow() {
    // One above U256::MAX.
    let overflow = "115792089237316195423570985008687907853269984665640564039457584007913129639936"
        .to_string();
    let err = analyze_transaction(
        "0x0000000000000000000000000000000000000001".into(),
        "0x".into(),
        overflow,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        BindingsError::Encoding {
            kind: EncodingErrorKind::Amount
        }
    ));
}

#[tokio::test]
async fn analyze_rejects_negative_amount() {
    let err = analyze_transaction(
        "0x0000000000000000000000000000000000000001".into(),
        "0x".into(),
        "-1".into(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        BindingsError::Encoding {
            kind: EncodingErrorKind::Amount
        }
    ));
}

#[tokio::test]
async fn analyze_rejects_hex_amount() {
    // Plan: amounts are decimal strings only. Hex amount must fail.
    let err = analyze_transaction(
        "0x0000000000000000000000000000000000000001".into(),
        "0x".into(),
        "0xff".into(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        BindingsError::Encoding {
            kind: EncodingErrorKind::Amount
        }
    ));
}

#[tokio::test]
async fn analyze_accepts_empty_calldata_as_native_transfer() {
    let verdict = analyze_transaction(
        "0x0000000000000000000000000000000000000001".into(),
        "0x".into(),
        "1".into(),
    )
    .expect("empty calldata = native transfer");
    let _ = verdict.action;
}

#[tokio::test]
async fn analyze_rejects_odd_length_hex_calldata() {
    let err = analyze_transaction(
        "0x0000000000000000000000000000000000000001".into(),
        "0xabc".into(),
        "0".into(),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        BindingsError::Encoding {
            kind: EncodingErrorKind::Calldata
        }
    ));
}
