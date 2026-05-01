//! Swap module — [`SwapProvider`] trait, 0x client, 1inch stub.
//!
//! # Trust boundary
//!
//! External HTTP API responses (0x.org, 1inch.dev) are parsed into
//! provider-specific DTOs and converted to a normalised [`SwapQuote`].
//! Unknown JSON fields are silently discarded for forward-compatibility
//! with vendor schema additions.
//!
//! # No broadcast
//!
//! [`SwapProvider::get_quote`] returns calldata only. Callers must invoke
//! [`crate::sign::sign_and_send_transaction`] to broadcast;
//! [`preview_swap`] runs the quote through `txguard` first and returns
//! a [`SwapPreview`] for UI presentation.
//!
//! # Quote cache
//!
//! `ZeroXProvider` caches successful quotes for 30 seconds to smooth UI
//! re-renders and respect free-tier rate limits. The cache is
//! per-instance — share at most across one user's session.
//!
//! # `chain_id` flow
//!
//! `QuoteParams.chain_id` selects the 0x base URL and is propagated into
//! `SwapQuote.chain_id`. Downstream
//! [`crate::sign::sign_and_send_transaction`] cross-checks this against
//! its own `chain_id` parameter; mismatch is rejected.
//!
//! # Trait dispatch
//!
//! [`SwapProvider`] is designed for static dispatch (`impl SwapProvider`).
//! Dynamic dispatch (`Box<dyn SwapProvider>`) requires
//! `#[trait_variant::make]` or `async-trait` and is out of scope here.

pub mod one_inch;
pub mod types;
pub mod zero_x;

pub use types::{LiquiditySource, QuoteParams, SwapError, SwapPreview, SwapQuote};

use alloy_network::TransactionBuilder;
use alloy_primitives::Address;
use alloy_rpc_types_eth::TransactionRequest;

use crate::provider::MultiProvider;
use crate::sign;

/// Trait for swap quote providers (0x, 1inch, ...).
pub trait SwapProvider: Send + Sync {
    /// Fetch a swap quote for the given parameters.
    ///
    /// # Errors
    ///
    /// See [`SwapError`]. Use [`SwapError::is_rate_limited`] to
    /// distinguish HTTP 429.
    fn get_quote(
        &self,
        params: QuoteParams,
    ) -> impl std::future::Future<Output = Result<SwapQuote, SwapError>> + Send;

    /// Provider display name (e.g. `"0x"`).
    fn name(&self) -> &str;

    /// Chains supported by this provider.
    fn supported_chains(&self) -> &[u64];
}

/// Build a [`TransactionRequest`] from a swap quote, validating that
/// `signer` matches the taker baked into calldata.
///
/// # Errors
///
/// Returns [`SwapError::Invalid`] if `signer != quote.taker_address` —
/// 0x calldata is taker-specific and signing with a different address
/// would likely revert on-chain.
pub fn quote_to_transaction(
    quote: &SwapQuote,
    signer: Address,
) -> Result<TransactionRequest, SwapError> {
    if quote.taker_address != signer {
        return Err(SwapError::Invalid(format!(
            "signer {signer} does not match quote taker_address {}",
            quote.taker_address
        )));
    }
    Ok(TransactionRequest::default()
        .with_to(quote.to)
        .with_value(quote.value)
        .with_input(quote.data.clone())
        .with_chain_id(quote.chain_id))
}

/// Run `txguard` analysis + gas estimate for a swap quote.
///
/// Wraps [`crate::sign::preview_transaction`] using `quote.taker_address`
/// as the `from` address. Always returns `Ok` even when the verdict
/// blocks; UI must inspect `verdict.action` before signing.
///
/// # Errors
///
/// - [`SwapError::Preview`] if the underlying preview pipeline fails
///   (calldata parse, RPC).
pub async fn preview_swap(
    provider: &MultiProvider,
    quote: &SwapQuote,
) -> Result<SwapPreview, SwapError> {
    let tx = TransactionRequest::default()
        .with_to(quote.to)
        .with_value(quote.value)
        .with_input(quote.data.clone())
        .with_chain_id(quote.chain_id);
    let preview = sign::preview_transaction(provider, &tx, quote.taker_address, quote.chain_id)
        .await
        .map_err(|e| SwapError::Preview(format!("{e}")))?;

    let warnings: Vec<String> = preview
        .verdict
        .findings
        .iter()
        .map(|f| f.description.clone())
        .collect();

    Ok(SwapPreview {
        quote: quote.clone(),
        verdict: preview.verdict,
        warnings,
        gas_cost_eth: preview.estimated_gas_cost_wei,
        total_cost_eth: preview.total_cost_wei,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Bytes, TxKind, U256, address};

    fn sample_quote() -> SwapQuote {
        SwapQuote {
            provider: "0x".to_string(),
            chain_id: 1,
            taker_address: address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"),
            sell_token: address!("EeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
            buy_token: address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            sell_amount: U256::from(100u128),
            buy_amount: U256::from(99u128),
            minimum_buy_amount: U256::from(98u128),
            to: address!("Def1C0ded9bec7F1a1670819833240f027b25EfF"),
            data: Bytes::from(vec![0xd9, 0x62, 0x7a, 0xa4]),
            value: U256::from(100u128),
            gas_estimate: 200_000,
            price: 1.0,
            allowance_target: None,
            sources: vec![],
        }
    }

    #[test]
    fn quote_to_transaction_basic() {
        let quote = sample_quote();
        let tx = quote_to_transaction(&quote, quote.taker_address).expect("ok");
        assert_eq!(tx.to.as_ref().and_then(TxKind::to).copied(), Some(quote.to));
        assert_eq!(tx.value, Some(quote.value));
        assert_eq!(tx.chain_id, Some(quote.chain_id));
        assert_eq!(tx.input.input().cloned().unwrap_or_default(), quote.data);
    }

    #[test]
    fn quote_to_transaction_signer_mismatch_rejected() {
        let quote = sample_quote();
        let other = address!("0000000000000000000000000000000000000001");
        let err = quote_to_transaction(&quote, other).unwrap_err();
        assert!(matches!(err, SwapError::Invalid(_)));
    }
}
