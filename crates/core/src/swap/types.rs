//! Swap module — public types and error taxonomy.
//!
//! `SwapQuote` is the canonical cross-provider shape produced from
//! external API responses (0x, 1inch). Untrusted JSON is parsed in
//! provider-specific DTOs and converted here; the rest of the crate
//! only sees the normalised form.

use alloy_primitives::{Address, Bytes, U256};
use thiserror::Error;
use txguard::types::Verdict;

/// Inputs for a swap quote request.
#[derive(Debug, Clone)]
pub struct QuoteParams {
    /// Source token. Native ETH = `0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE`.
    pub sell_token: Address,
    /// Destination token.
    pub buy_token: Address,
    /// Amount of `sell_token` in token base units (wei).
    pub sell_amount: U256,
    /// Target chain id. Must be in `SwapProvider::supported_chains()`.
    pub chain_id: u64,
    /// Maximum acceptable slippage in basis points (50 = 0.5%).
    pub slippage_bps: u16,
    /// Wallet address — calldata is bound to this taker.
    pub taker_address: Address,
}

/// Normalised swap quote returned by any [`super::SwapProvider`].
///
/// `data` is router calldata bound to `taker_address`. Submitting the
/// quote from a different signer will likely revert on-chain;
/// [`super::quote_to_transaction`] enforces this invariant.
#[derive(Debug, Clone)]
pub struct SwapQuote {
    /// Provider display name (e.g. `"0x"`).
    pub provider: String,
    /// Chain id this quote is bound to.
    pub chain_id: u64,
    /// Slippage tolerance in basis points (50 = 0.5%). Propagated from
    /// `QuoteParams` so downstream `txguard` swap rules can analyse it
    /// without re-deriving from `buy_amount` / `minimum_buy_amount`
    /// (which loses precision through integer division).
    pub slippage_bps: u16,
    /// Taker baked into calldata.
    pub taker_address: Address,
    /// Source token.
    pub sell_token: Address,
    /// Destination token.
    pub buy_token: Address,
    /// Source amount (wei).
    pub sell_amount: U256,
    /// Expected output amount (wei).
    pub buy_amount: U256,
    /// Lower bound on output after slippage. Informational — actual
    /// on-chain protection is enforced by router calldata itself.
    pub minimum_buy_amount: U256,
    /// Router contract.
    pub to: Address,
    /// Calldata to submit to `to`.
    pub data: Bytes,
    /// ETH value to attach (non-zero only for ETH→token swaps).
    pub value: U256,
    /// Provider-supplied gas estimate.
    pub gas_estimate: u64,
    /// Sell/buy price ratio. UI display only.
    pub price: f64,
    /// ERC-20 allowance target. `Some` requires `approve` before swap.
    pub allowance_target: Option<Address>,
    /// Liquidity sources used in the route.
    pub sources: Vec<LiquiditySource>,
}

/// Single liquidity source contributing to a quote.
#[derive(Debug, Clone)]
pub struct LiquiditySource {
    /// DEX or aggregator name (e.g. `"Uniswap_V3"`).
    pub name: String,
    /// Fraction of the order routed through this source. 0.0..=1.0.
    pub proportion: f64,
}

/// Combined swap quote + txguard analysis + cost estimate.
#[derive(Debug, Clone)]
pub struct SwapPreview {
    /// Original quote.
    pub quote: SwapQuote,
    /// txguard verdict.
    pub verdict: Verdict,
    /// Human-readable warnings. Currently mirrors `verdict.findings`;
    /// reserved for swap-specific synthesis once swap rules land.
    pub warnings: Vec<String>,
    /// `gas_estimate * max_fee_per_gas` (wei, saturating).
    pub gas_cost_eth: U256,
    /// `value + gas_cost_eth` (wei, saturating).
    pub total_cost_eth: U256,
}

/// Errors from swap operations.
#[derive(Debug, Error)]
pub enum SwapError {
    /// Provider does not support the requested chain.
    #[error("unsupported chain id {chain_id}")]
    UnsupportedChain {
        /// Requested chain id.
        chain_id: u64,
    },
    /// Network or transport error from `reqwest`.
    #[error("provider HTTP error: {0}")]
    Http(String),
    /// Provider returned a non-2xx HTTP status.
    #[error("provider returned status {status}")]
    ProviderStatus {
        /// HTTP status code.
        status: u16,
        /// First N ASCII-printable chars of the response body.
        body_excerpt: String,
        /// `Retry-After` header value (seconds), when present and parseable.
        retry_after_secs: Option<u64>,
    },
    /// JSON deserialisation or numeric parsing failed.
    #[error("response parse failed: {0}")]
    Parse(String),
    /// Provider not yet implemented (e.g. 1inch stub).
    #[error("provider unavailable: {0}")]
    ProviderUnavailable(String),
    /// Preview pipeline failure (calldata parse, RPC). Never carries a
    /// `txguard` block verdict — `super::preview_swap` returns `Ok` even
    /// when blocked, mirroring `crate::sign::preview_transaction`. UI
    /// must inspect `SwapPreview::verdict::action` for block detection.
    #[error("preview failed: {0}")]
    Preview(String),
    /// Caller violated an invariant (signer mismatch, etc.).
    #[error("invalid: {0}")]
    Invalid(String),
}

impl SwapError {
    /// Returns `true` if the error represents an HTTP 429 rate-limit response.
    #[must_use]
    pub const fn is_rate_limited(&self) -> bool {
        matches!(self, Self::ProviderStatus { status: 429, .. })
    }

    /// Returns the HTTP status code if this error originated from a non-2xx
    /// provider response, otherwise `None`.
    #[must_use]
    pub const fn http_status(&self) -> Option<u16> {
        if let Self::ProviderStatus { status, .. } = self {
            Some(*status)
        } else {
            None
        }
    }

    /// Returns the `Retry-After` value (seconds) for HTTP 429 responses
    /// when the header was present and parseable.
    #[must_use]
    pub const fn retry_after_secs(&self) -> Option<u64> {
        if let Self::ProviderStatus {
            retry_after_secs, ..
        } = self
        {
            *retry_after_secs
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_error_429_is_rate_limited() {
        let err = SwapError::ProviderStatus {
            status: 429,
            body_excerpt: "rate limit".into(),
            retry_after_secs: Some(30),
        };
        assert!(err.is_rate_limited());
        assert_eq!(err.http_status(), Some(429));
        assert_eq!(err.retry_after_secs(), Some(30));
    }

    #[test]
    fn swap_error_500_not_rate_limited() {
        let err = SwapError::ProviderStatus {
            status: 500,
            body_excerpt: "internal".into(),
            retry_after_secs: None,
        };
        assert!(!err.is_rate_limited());
        assert_eq!(err.http_status(), Some(500));
        assert_eq!(err.retry_after_secs(), None);
    }

    #[test]
    fn swap_error_non_status_has_no_http() {
        let err = SwapError::ProviderUnavailable("stub".into());
        assert!(!err.is_rate_limited());
        assert_eq!(err.http_status(), None);
        assert_eq!(err.retry_after_secs(), None);
    }
}
