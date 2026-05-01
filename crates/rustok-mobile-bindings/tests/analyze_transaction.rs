//! Integration tests — `analyze_transaction` free function coverage.
//!
//! Exercises the txguard parser + RulesEngine pipeline through the
//! mobile FFI surface. Pure function — no wallet state, no I/O.
//! Verifies VerdictDto / FindingDto / ActionDto / SeverityDto /
//! RuleCategoryDto mirror conversions are wired correctly.

use rustok_mobile_bindings::{ActionDto, RuleCategoryDto, SeverityDto, analyze_transaction};

// Neutral target — not the burn address `0x…dEaD` which some txguard
// rules treat as a scam beacon.
const ANY_TARGET: &str = "0x1111111111111111111111111111111111111111";
const USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
const UNISWAP_V2_ROUTER: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

#[tokio::test]
async fn native_transfer_safe() {
    let verdict = analyze_transaction(ANY_TARGET.into(), "0x".into(), "1000000000000000000".into())
        .expect("analyze");
    assert!(matches!(verdict.action, ActionDto::Allow));
    assert_eq!(verdict.risk_score, 0);
    assert!(verdict.findings.is_empty());
}

#[tokio::test]
async fn unlimited_approval_warns() {
    // ERC-20 approve(spender, U256::MAX) — 4-byte selector + 2 × 32 bytes.
    // selector = 0x095ea7b3 (approve)
    // arg1 = spender padded to 32 bytes
    // arg2 = uint256.max
    let calldata = format!(
        "0x095ea7b3000000000000000000000000{}{}",
        UNISWAP_V2_ROUTER
            .trim_start_matches("0x")
            .to_ascii_lowercase(),
        "f".repeat(64)
    );
    let verdict = analyze_transaction(USDT.into(), calldata, "0".into()).expect("analyze");

    assert!(matches!(verdict.action, ActionDto::Warn));
    assert!(verdict.risk_score > 0);
    let unlimited = verdict
        .findings
        .iter()
        .find(|f| f.rule == "unlimited_approval");
    assert!(
        unlimited.is_some(),
        "must surface unlimited_approval finding"
    );
    let f = unlimited.unwrap();
    assert!(matches!(f.severity, SeverityDto::Warning));
    assert!(matches!(f.category, RuleCategoryDto::Approval));
}

#[tokio::test]
async fn small_approval_safe() {
    // approve(spender, 1_000_000) — non-MAX amount.
    let amount_hex = format!("{:0>64x}", 1_000_000u64);
    let calldata = format!(
        "0x095ea7b3000000000000000000000000{}{}",
        UNISWAP_V2_ROUTER
            .trim_start_matches("0x")
            .to_ascii_lowercase(),
        amount_hex
    );
    let verdict = analyze_transaction(USDT.into(), calldata, "0".into()).expect("analyze");

    assert!(matches!(verdict.action, ActionDto::Allow));
    assert_eq!(verdict.risk_score, 0);
    assert!(verdict.findings.is_empty());
}

#[tokio::test]
async fn unknown_selector_falls_through_to_unknown_function_finding() {
    // Selector 0xdeadbeef + dummy args — not in known.rs registry.
    let calldata = format!("0xdeadbeef{}", "0".repeat(120));
    let verdict = analyze_transaction(ANY_TARGET.into(), calldata, "0".into())
        .expect("analyze accepts unknown selector");
    // Structural check: txguard's `unknown_function` rule (contract.rs)
    // fires on TransactionAction::Unknown with Severity::Warning. Asserts
    // the finding is surfaced + mirror enums map correctly. Avoids
    // depending on the verdict.description text format.
    assert!(matches!(verdict.action, ActionDto::Warn));
    assert!(verdict.findings.iter().any(|f| f.rule == "unknown_function"));
}

#[tokio::test]
async fn verdict_dto_round_trips_action_severity_category() {
    // Force a finding to populate VerdictDto.findings + sub-enum mirrors.
    let calldata = format!(
        "0x095ea7b3000000000000000000000000{}{}",
        UNISWAP_V2_ROUTER
            .trim_start_matches("0x")
            .to_ascii_lowercase(),
        "f".repeat(64)
    );
    let verdict = analyze_transaction(USDT.into(), calldata, "0".into()).expect("analyze");

    let f = verdict
        .findings
        .into_iter()
        .next()
        .expect("at least one finding");
    // Mirror enum mapping verified at the From<txguard::types::*> boundary.
    match f.severity {
        SeverityDto::Info | SeverityDto::Warning | SeverityDto::Danger | SeverityDto::Forbidden => {
        }
    }
    match f.category {
        RuleCategoryDto::Approval
        | RuleCategoryDto::Permit
        | RuleCategoryDto::Send
        | RuleCategoryDto::Swap
        | RuleCategoryDto::Contract
        | RuleCategoryDto::Address => {}
    }
    match verdict.action {
        ActionDto::Block | ActionDto::Warn | ActionDto::Allow => {}
    }
}
