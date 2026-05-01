//! Swap-specific security rules — router whitelist, slippage analysis,
//! approval-to-DEX detection.
//!
//! # API surface
//!
//! - [`analyze_swap`] is generic over [`ParsedTransaction`]. The current
//!   integration point is `rustok-core::swap::preview_swap`, which calls
//!   it for swap calldata (where `parsed.action` is typically
//!   [`TransactionAction::Unknown`] because router selectors are not
//!   in the parser's known table). The `approval_to_known_router` rule
//!   is dormant in that flow — it fires only when
//!   `parsed.action == TokenApproval`, reachable via a future
//!   `preview_approval` orchestrator (commit 9+).
//! - [`analyze_swap_extras`] merges swap findings into a baseline
//!   [`Verdict`] from [`super::RulesEngine::analyze`] and recomputes
//!   `risk_score` / `action`. Use this when calling site already has a
//!   baseline verdict.
//!
//! # Trust boundary
//!
//! Router addresses are hardcoded per chain in [`KNOWN_ROUTERS_BY_CHAIN`].
//! Each address was verified against vendor primary sources (URLs in
//! the const definition). Adding a chain or address requires the same
//! verification step.

use alloy_primitives::{Address, address};

use crate::parser::{ParsedTransaction, TransactionAction};
use crate::types::{
    Action, Finding, RuleCategory, Severity, Verdict, action_from_score, risk_score,
};

/// Inputs for swap-specific rule analysis. Provided by the caller from
/// the [`SwapQuote`](https://docs.rs/rustok-core) used to build the
/// transaction.
#[derive(Debug, Clone)]
pub struct SwapAnalysisContext {
    /// Target chain id. Used to look up the chain-specific router list.
    pub chain_id: u64,
    /// Slippage tolerance in basis points (50 = 0.5%).
    pub slippage_bps: u16,
}

/// Per-chain registry of well-known DEX router addresses.
///
/// Each entry is verified against primary vendor documentation. When
/// adding a new chain or address, cite the source and verify the
/// address on the chain's block explorer.
///
/// Sources (verified 2026-05-01):
/// - Uniswap V2/V3 / SwapRouter / SwapRouter02:
///   `https://github.com/Uniswap/contracts/tree/main/deployments`
/// - 0x v1 ExchangeProxy:
///   `https://github.com/0xProject/protocol/blob/main/packages/contract-addresses/addresses.json`
/// - 1inch AggregationRouter V5 (mainnet only) / V6 (unified):
///   Etherscan-verified contract names; V6 unified across major EVM
///   chains per `https://github.com/1inch/limit-order-protocol`.
const KNOWN_ROUTERS_BY_CHAIN: &[(u64, &[Address])] = &[
    // Ethereum mainnet
    (
        1,
        &[
            // 0x v1 ExchangeProxy
            address!("0xdef1c0ded9bec7f1a1670819833240f027b25eff"),
            // Uniswap V3 SwapRouter
            address!("0xE592427A0AEce92De3Edee1F18E0157C05861564"),
            // Uniswap V3 SwapRouter02
            address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"),
            // Uniswap V2 Router02
            address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"),
            // 1inch AggregationRouter V5
            address!("0x1111111254EEB25477B68fb85Ed929f73A960582"),
            // 1inch AggregationRouter V6
            address!("0x111111125421ca6dc452d289314280a0f8842a65"),
        ],
    ),
    // Arbitrum One
    (
        42161,
        &[
            address!("0xdef1c0ded9bec7f1a1670819833240f027b25eff"), // 0x v1
            address!("0xE592427A0AEce92De3Edee1F18E0157C05861564"), // UniV3 SwapRouter
            address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), // UniV3 SwapRouter02
            address!("0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24"), // UniV2 Router02
            address!("0x111111125421ca6dc452d289314280a0f8842a65"), // 1inch V6
        ],
    ),
    // Base
    (
        8453,
        &[
            address!("0xdef1c0ded9bec7f1a1670819833240f027b25eff"), // 0x v1
            // Uniswap V3 SwapRouter (no "02" suffix variant) is not
            // deployed on Base — only SwapRouter02.
            address!("0x2626664c2603336E57B271c5C0b26F421741e481"), // UniV3 SwapRouter02 (Base)
            address!("0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24"), // UniV2 Router02
            address!("0x111111125421ca6dc452d289314280a0f8842a65"), // 1inch V6
        ],
    ),
    // Optimism
    (
        10,
        &[
            // 0x v1 ExchangeProxy on Optimism uses a DIFFERENT address
            // than the unified `0xdef1c0...` on Ethereum/Arbitrum/Base/Polygon.
            address!("0xdef1abe32c034e558cdd535791643c58a13acc10"),
            address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), // UniV3 SwapRouter02
            address!("0x4A7b5Da61326A6379179b40d00F57E5bbDC962c2"), // UniV2 Router02 (OP)
            address!("0x111111125421ca6dc452d289314280a0f8842a65"), // 1inch V6
        ],
    ),
    // Polygon
    (
        137,
        &[
            address!("0xdef1c0ded9bec7f1a1670819833240f027b25eff"), // 0x v1
            address!("0xE592427A0AEce92De3Edee1F18E0157C05861564"), // UniV3 SwapRouter
            address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), // UniV3 SwapRouter02
            address!("0xedf6066a2b290C185783862C7F4776A2C8077AD1"), // UniV2 Router02 (Polygon)
            address!("0x111111125421ca6dc452d289314280a0f8842a65"), // 1inch V6
        ],
    ),
];

/// Slippage threshold (in basis points) above which a `Warning` finding
/// is emitted. 300 bps = 3% — matches MetaMask / 1inch defaults.
const EXCESSIVE_SLIPPAGE_BPS: u16 = 300;

/// Compute swap-specific findings against a parsed transaction.
///
/// Pure function — does not modify any caller state. Returns an empty
/// `Vec` if no rule fires.
#[must_use]
pub fn analyze_swap(parsed: &ParsedTransaction, ctx: &SwapAnalysisContext) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_unknown_router(parsed, ctx, &mut findings);
    check_excessive_slippage(ctx, &mut findings);
    check_approval_to_known_router(parsed, ctx, &mut findings);
    findings
}

/// Caller helper: take a baseline [`Verdict`] from
/// [`super::RulesEngine::analyze`] and merge swap-specific findings,
/// recomputing `risk_score`, `action`, and `description`.
///
/// Returns `base` unchanged if no swap findings apply.
#[must_use]
pub fn analyze_swap_extras(
    base: Verdict,
    parsed: &ParsedTransaction,
    ctx: &SwapAnalysisContext,
) -> Verdict {
    let extras = analyze_swap(parsed, ctx);
    if extras.is_empty() {
        return base;
    }

    let warnings_summary: Vec<&str> = extras.iter().map(|f| f.description.as_str()).collect();
    let new_description = format!(
        "{}. Swap warnings: {}",
        base.description,
        warnings_summary.join("; ")
    );

    let mut findings = base.findings;
    findings.extend(extras);
    let score = risk_score(&findings);
    let action = if findings.iter().any(|f| f.severity == Severity::Forbidden) {
        Action::Block
    } else {
        action_from_score(score)
    };

    Verdict {
        action,
        risk_score: score,
        findings,
        description: new_description,
        simulation: base.simulation,
    }
}

/// Look up the router whitelist for a chain. Empty slice if the chain
/// is not registered.
fn known_routers_for_chain(chain_id: u64) -> &'static [Address] {
    KNOWN_ROUTERS_BY_CHAIN
        .iter()
        .find(|(c, _)| *c == chain_id)
        .map_or(&[][..], |(_, r)| *r)
}

/// Flag swap routed through an address not in the chain whitelist.
///
/// Restricted to swap-style calldata (`TransactionAction::Unknown`).
/// Recognised actions (transfer, approve, ...) have their own rules
/// and `parsed.to` for them is the token contract, not a DEX router —
/// firing `unknown_router` on those would emit a misleading "swap
/// routed through" message.
fn check_unknown_router(
    parsed: &ParsedTransaction,
    ctx: &SwapAnalysisContext,
    findings: &mut Vec<Finding>,
) {
    if !matches!(parsed.action, TransactionAction::Unknown { .. }) {
        return;
    }
    let routers = known_routers_for_chain(ctx.chain_id);
    if !routers.contains(&parsed.to) {
        findings.push(Finding {
            rule: "unknown_router",
            severity: Severity::Warning,
            category: RuleCategory::Swap,
            description: format!(
                "Swap routed through unknown contract {}. Only well-known DEX routers should be used. Verify the address before signing.",
                parsed.to
            ),
        });
    }
}

/// Flag slippage tolerance above the safe threshold. Boundary
/// is `> EXCESSIVE_SLIPPAGE_BPS` — exactly equal is allowed.
fn check_excessive_slippage(ctx: &SwapAnalysisContext, findings: &mut Vec<Finding>) {
    if ctx.slippage_bps > EXCESSIVE_SLIPPAGE_BPS {
        findings.push(Finding {
            rule: "excessive_slippage",
            severity: Severity::Warning,
            category: RuleCategory::Swap,
            description: format!(
                "Slippage tolerance {}.{:02}% exceeds the safe threshold of {}.{:02}%. High slippage exposes you to MEV and price manipulation.",
                ctx.slippage_bps / 100,
                ctx.slippage_bps % 100,
                EXCESSIVE_SLIPPAGE_BPS / 100,
                EXCESSIVE_SLIPPAGE_BPS % 100
            ),
        });
    }
}

/// Mark `approve(known_router, _)` calls as expected swap-workflow
/// approvals (`Severity::Info`). The existing
/// `unlimited_approval` rule (in [`super::approval`]) still fires its
/// `Warning` independently when `amount == U256::MAX`; this rule adds
/// orthogonal context, not a downgrade.
///
/// Dormant in the current `preview_swap` flow (calldata is a swap, not
/// an `approve`). Activated when callers pass an `approve` parsed
/// transaction directly through [`analyze_swap`].
fn check_approval_to_known_router(
    parsed: &ParsedTransaction,
    ctx: &SwapAnalysisContext,
    findings: &mut Vec<Finding>,
) {
    let TransactionAction::TokenApproval { spender, .. } = &parsed.action else {
        return;
    };
    let routers = known_routers_for_chain(ctx.chain_id);
    if routers.contains(spender) {
        findings.push(Finding {
            rule: "approval_to_known_router",
            severity: Severity::Info,
            category: RuleCategory::Swap,
            description: format!(
                "Approving known DEX router {spender} — expected for swap workflow."
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;

    const RANDOM: Address = address!("0x1111111111111111111111111111111111111111");
    const UNISWAP_V3_ROUTER: Address = address!("0xE592427A0AEce92De3Edee1F18E0157C05861564");
    const ZEROX_OPTIMISM: Address = address!("0xdef1abe32c034e558cdd535791643c58a13acc10");
    const ZEROX_MAINNET: Address = address!("0xdef1c0ded9bec7f1a1670819833240f027b25eff");
    const USDC: Address = address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

    fn unknown_call_to(target: Address) -> ParsedTransaction {
        ParsedTransaction {
            to: target,
            value: U256::ZERO,
            action: TransactionAction::Unknown {
                selector: "deadbeef".into(),
                calldata_len: 100,
            },
            function_name: None,
            function_selector: Some([0xde, 0xad, 0xbe, 0xef]),
        }
    }

    fn approve_to(spender: Address) -> ParsedTransaction {
        ParsedTransaction {
            to: USDC,
            value: U256::ZERO,
            action: TransactionAction::TokenApproval {
                spender,
                amount: U256::from(1_000u64),
            },
            function_name: Some("approve".into()),
            function_selector: Some([0x09, 0x5e, 0xa7, 0xb3]),
        }
    }

    fn ctx(chain_id: u64, slippage_bps: u16) -> SwapAnalysisContext {
        SwapAnalysisContext {
            chain_id,
            slippage_bps,
        }
    }

    #[test]
    fn unknown_router_warns_on_mainnet() {
        let parsed = unknown_call_to(RANDOM);
        let findings = analyze_swap(&parsed, &ctx(1, 50));
        assert!(findings.iter().any(|f| f.rule == "unknown_router"));
    }

    #[test]
    fn known_uniswap_v3_router_no_unknown_router() {
        let parsed = unknown_call_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(1, 50));
        assert!(findings.iter().all(|f| f.rule != "unknown_router"));
    }

    #[test]
    fn arbitrum_chain_specific_router_match() {
        // Uniswap V3 SwapRouter is at the same address on Arbitrum.
        let parsed = unknown_call_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(42161, 50));
        assert!(findings.iter().all(|f| f.rule != "unknown_router"));
    }

    #[test]
    fn optimism_uses_distinct_0x_proxy() {
        // Mainnet 0x proxy must NOT be considered a router on Optimism.
        let parsed = unknown_call_to(ZEROX_MAINNET);
        let findings = analyze_swap(&parsed, &ctx(10, 50));
        assert!(findings.iter().any(|f| f.rule == "unknown_router"));
        // The Optimism-specific 0x proxy MUST be recognised.
        let parsed_op = unknown_call_to(ZEROX_OPTIMISM);
        let findings_op = analyze_swap(&parsed_op, &ctx(10, 50));
        assert!(findings_op.iter().all(|f| f.rule != "unknown_router"));
    }

    #[test]
    fn unknown_chain_treats_all_addresses_unknown() {
        let parsed = unknown_call_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(99_999, 50));
        assert!(findings.iter().any(|f| f.rule == "unknown_router"));
    }

    #[test]
    fn does_not_fire_unknown_router_for_native_transfer() {
        let parsed = ParsedTransaction {
            to: RANDOM,
            value: U256::from(1u64),
            action: TransactionAction::NativeTransfer,
            function_name: None,
            function_selector: None,
        };
        let findings = analyze_swap(&parsed, &ctx(1, 50));
        assert!(findings.iter().all(|f| f.rule != "unknown_router"));
    }

    #[test]
    fn slippage_at_threshold_no_finding() {
        let parsed = unknown_call_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(1, 300));
        assert!(findings.iter().all(|f| f.rule != "excessive_slippage"));
    }

    #[test]
    fn slippage_just_above_threshold_warns() {
        let parsed = unknown_call_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(1, 301));
        assert!(findings.iter().any(|f| f.rule == "excessive_slippage"));
    }

    #[test]
    fn excessive_slippage_warns_at_500_bps() {
        let parsed = unknown_call_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(1, 500));
        let f = findings
            .iter()
            .find(|f| f.rule == "excessive_slippage")
            .expect("excessive_slippage finding");
        assert_eq!(f.severity, Severity::Warning);
        assert_eq!(f.category, RuleCategory::Swap);
    }

    #[test]
    fn approval_to_known_router_info() {
        let parsed = approve_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(1, 50));
        let f = findings
            .iter()
            .find(|f| f.rule == "approval_to_known_router")
            .expect("approval_to_known_router finding");
        assert_eq!(f.severity, Severity::Info);
    }

    #[test]
    fn approval_to_unknown_target_no_router_finding() {
        let parsed = approve_to(RANDOM);
        let findings = analyze_swap(&parsed, &ctx(1, 50));
        assert!(
            findings
                .iter()
                .all(|f| f.rule != "approval_to_known_router")
        );
    }

    #[test]
    fn approve_target_does_not_fire_unknown_router() {
        // For an `approve(spender, amount)` parsed action, `parsed.to` is
        // the token contract (USDC here) — never a DEX router. The
        // `unknown_router` rule must NOT fire, as it would emit a
        // misleading "Swap routed through unknown contract" message.
        let parsed = approve_to(UNISWAP_V3_ROUTER);
        let findings = analyze_swap(&parsed, &ctx(1, 50));
        assert!(findings.iter().all(|f| f.rule != "unknown_router"));
    }

    #[test]
    fn analyze_swap_extras_returns_base_unchanged_when_no_extras() {
        let base = Verdict {
            action: Action::Allow,
            risk_score: 0,
            findings: vec![],
            description: "baseline".into(),
            simulation: None,
        };
        let parsed = unknown_call_to(UNISWAP_V3_ROUTER);
        let merged = analyze_swap_extras(base, &parsed, &ctx(1, 50));
        assert_eq!(merged.action, Action::Allow);
        assert_eq!(merged.risk_score, 0);
        assert!(merged.findings.is_empty());
        assert_eq!(merged.description, "baseline");
    }

    #[test]
    fn analyze_swap_extras_merges_findings_and_recomputes_score() {
        let base = Verdict {
            action: Action::Allow,
            risk_score: 0,
            findings: vec![],
            description: "baseline".into(),
            simulation: None,
        };
        let parsed = unknown_call_to(RANDOM);
        let merged = analyze_swap_extras(base, &parsed, &ctx(1, 500));

        // Two extras: unknown_router (Warning) + excessive_slippage (Warning).
        assert_eq!(merged.findings.len(), 2);
        assert!(merged.risk_score > 0);
        assert_eq!(merged.action, Action::Warn);
        // Description is augmented with swap warnings list.
        assert!(merged.description.contains("baseline"));
        assert!(merged.description.contains("Swap warnings"));
    }
}
