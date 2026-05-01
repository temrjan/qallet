//! 0x Swap API v1 client.
//!
//! Implements [`SwapProvider`] against `https://{chain}.api.0x.org`.
//! Successful quotes are cached for 30 seconds to smooth UI re-renders
//! and respect free-tier rate limits.
//!
//! Logging discipline: never log API key, taker address, sell amount, or
//! response body. `tracing::debug` covers chain id and token pair only;
//! `tracing::warn` covers HTTP status code only on failure.
//!
//! Trust boundary: response JSON is parsed into [`ZeroXQuoteResponse`]
//! and converted to [`SwapQuote`]. Unknown JSON fields are ignored
//! (forward-compatibility with vendor schema additions).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use alloy_primitives::{Address, Bytes, U256};
use serde::{Deserialize, Deserializer};

use crate::swap::SwapProvider;
use crate::swap::types::{LiquiditySource, QuoteParams, SwapError, SwapQuote};

const ZEROX_CHAINS: &[(u64, &str)] = &[
    (1, "https://api.0x.org"),
    (42161, "https://arbitrum.api.0x.org"),
    (8453, "https://base.api.0x.org"),
    (10, "https://optimism.api.0x.org"),
    (137, "https://polygon.api.0x.org"),
];

const SUPPORTED_CHAIN_IDS: &[u64] = &[1, 42161, 8453, 10, 137];
const CACHE_TTL: Duration = Duration::from_secs(30);
const PROVIDER_NAME: &str = "0x";
const BODY_EXCERPT_MAX: usize = 256;
const BPS_TOTAL: u16 = 10_000;

/// 0x Swap API client.
pub struct ZeroXProvider {
    client: reqwest::Client,
    api_key: Option<String>,
    cache: Arc<Mutex<HashMap<CacheKey, CacheEntry>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    chain_id: u64,
    sell: Address,
    buy: Address,
    amount: U256,
    taker: Address,
    slippage_bps: u16,
}

impl CacheKey {
    const fn from_params(p: &QuoteParams) -> Self {
        Self {
            chain_id: p.chain_id,
            sell: p.sell_token,
            buy: p.buy_token,
            amount: p.sell_amount,
            taker: p.taker_address,
            slippage_bps: p.slippage_bps,
        }
    }
}

struct CacheEntry {
    quote: SwapQuote,
    expires_at: Instant,
}

impl ZeroXProvider {
    /// Construct using the given HTTP client. The API key is read at
    /// compile time from the `ZERO_X_API_KEY` environment variable
    /// (free tier works without).
    #[must_use]
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client,
            api_key: option_env!("ZERO_X_API_KEY").map(String::from),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Construct with an explicit API key (for tests or runtime injection).
    #[must_use]
    pub fn with_api_key(client: reqwest::Client, api_key: String) -> Self {
        Self {
            client,
            api_key: Some(api_key),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Resolve the chain-specific 0x base URL.
    ///
    /// # Errors
    ///
    /// Returns [`SwapError::UnsupportedChain`] for chains not in
    /// `ZEROX_CHAINS`.
    fn base_url(chain_id: u64) -> Result<&'static str, SwapError> {
        ZEROX_CHAINS
            .iter()
            .find(|(id, _)| *id == chain_id)
            .map(|(_, url)| *url)
            .ok_or(SwapError::UnsupportedChain { chain_id })
    }

    /// Lock the cache, recovering from `PoisonError`.
    ///
    /// The cache holds plain `HashMap` data with no invariants that a
    /// panic could leave inconsistent, so recovery is safe.
    fn cache_lock(&self) -> std::sync::MutexGuard<'_, HashMap<CacheKey, CacheEntry>> {
        self.cache.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Read a cached quote, evicting expired entries. The returned
    /// `Option<SwapQuote>` owns its data — the lock is released before
    /// any caller `.await` point.
    fn cache_get(&self, key: &CacheKey, now: Instant) -> Option<SwapQuote> {
        let mut guard = self.cache_lock();
        guard.retain(|_, e| e.expires_at > now);
        guard.get(key).map(|e| e.quote.clone())
    }

    /// Insert a fresh quote, evicting expired entries first.
    fn cache_put(&self, key: CacheKey, quote: SwapQuote, now: Instant) {
        let mut guard = self.cache_lock();
        guard.retain(|_, e| e.expires_at > now);
        guard.insert(
            key,
            CacheEntry {
                quote,
                expires_at: now + CACHE_TTL,
            },
        );
    }

    #[cfg(test)]
    fn cache_put_with_expiry(&self, key: CacheKey, quote: SwapQuote, expires_at: Instant) {
        let mut guard = self.cache_lock();
        guard.insert(key, CacheEntry { quote, expires_at });
    }
}

impl SwapProvider for ZeroXProvider {
    async fn get_quote(&self, params: QuoteParams) -> Result<SwapQuote, SwapError> {
        let key = CacheKey::from_params(&params);
        let now = Instant::now();
        if let Some(cached) = self.cache_get(&key, now) {
            tracing::debug!(
                chain_id = params.chain_id,
                sell = %params.sell_token,
                buy = %params.buy_token,
                "0x cache hit"
            );
            return Ok(cached);
        }

        let base = Self::base_url(params.chain_id)?;
        let slippage_pct = f64::from(params.slippage_bps) / f64::from(BPS_TOTAL);
        // Manual query-string assembly: reqwest is configured with
        // `default-features = false`, which disables `RequestBuilder::query`.
        // All values here are hex addresses or decimal numbers — already
        // URL-safe, no percent-encoding required.
        let url = format!(
            "{base}/swap/v1/quote\
             ?sellToken={sell:#x}\
             &buyToken={buy:#x}\
             &sellAmount={amount}\
             &takerAddress={taker:#x}\
             &slippagePercentage={slippage_pct:.6}",
            sell = params.sell_token,
            buy = params.buy_token,
            amount = params.sell_amount,
            taker = params.taker_address,
        );

        let mut request = self.client.get(&url);
        if let Some(key) = self.api_key.as_deref() {
            request = request.header("0x-api-key", key);
        }

        tracing::debug!(
            chain_id = params.chain_id,
            sell = %params.sell_token,
            buy = %params.buy_token,
            "0x quote request"
        );

        let response = request
            .send()
            .await
            // `reqwest::Error::Display` includes the request URL with our
            // `takerAddress` query param. Strip it so the wallet address is
            // not leaked into caller logs via `SwapError::Http`.
            .map_err(|e| SwapError::Http(e.without_url().to_string()))?;

        let status = response.status();
        if !status.is_success() {
            tracing::warn!(status = status.as_u16(), "0x quote failed");
            let retry_after_secs = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            let body = response.text().await.unwrap_or_default();
            let body_excerpt = sanitize_body(&body, BODY_EXCERPT_MAX);
            return Err(SwapError::ProviderStatus {
                status: status.as_u16(),
                body_excerpt,
                retry_after_secs,
            });
        }

        let dto: ZeroXQuoteResponse = response
            .json()
            .await
            // Strip URL for the same reason as `SwapError::Http` above.
            .map_err(|e| SwapError::Parse(e.without_url().to_string()))?;

        let quote = build_quote(dto, &params);
        self.cache_put(key, quote.clone(), now);
        Ok(quote)
    }

    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn supported_chains(&self) -> &[u64] {
        SUPPORTED_CHAIN_IDS
    }
}

/// Strip non-printable ASCII and truncate to at most `max` chars.
fn sanitize_body(body: &str, max: usize) -> String {
    body.chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .take(max)
        .collect()
}

/// Convert a parsed 0x DTO into a normalised [`SwapQuote`].
fn build_quote(dto: ZeroXQuoteResponse, params: &QuoteParams) -> SwapQuote {
    let slippage = U256::from(params.slippage_bps);
    let bps_total = U256::from(BPS_TOTAL);
    let kept = bps_total.saturating_sub(slippage);
    let minimum_buy_amount = dto.buy_amount.saturating_mul(kept) / bps_total;

    let price = dto.price.parse::<f64>().unwrap_or(0.0);
    let sources = dto
        .sources
        .into_iter()
        .map(|s| LiquiditySource {
            name: s.name,
            proportion: s.proportion.parse::<f64>().unwrap_or(0.0),
        })
        .collect();

    SwapQuote {
        provider: PROVIDER_NAME.to_string(),
        chain_id: params.chain_id,
        slippage_bps: params.slippage_bps,
        taker_address: params.taker_address,
        sell_token: params.sell_token,
        buy_token: params.buy_token,
        sell_amount: dto.sell_amount,
        buy_amount: dto.buy_amount,
        minimum_buy_amount,
        to: dto.to,
        data: dto.data,
        value: dto.value,
        gas_estimate: dto.gas,
        price,
        allowance_target: dto.allowance_target,
        sources,
    }
}

/// 0x `/swap/v1/quote` response. Unknown fields are ignored.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ZeroXQuoteResponse {
    to: Address,
    data: Bytes,
    #[serde(deserialize_with = "u256_decimal")]
    value: U256,
    #[serde(deserialize_with = "u64_decimal")]
    gas: u64,
    #[serde(deserialize_with = "u256_decimal")]
    sell_amount: U256,
    #[serde(deserialize_with = "u256_decimal")]
    buy_amount: U256,
    price: String,
    allowance_target: Option<Address>,
    sources: Vec<ZeroXSource>,
}

#[derive(Debug, Deserialize)]
struct ZeroXSource {
    name: String,
    proportion: String,
}

fn u256_decimal<'de, D: Deserializer<'de>>(d: D) -> Result<U256, D::Error> {
    let s = String::deserialize(d)?;
    U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)
}

fn u64_decimal<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let s = String::deserialize(d)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    fn sample_params() -> QuoteParams {
        QuoteParams {
            sell_token: address!("EeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
            buy_token: address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            sell_amount: U256::from(100_000_000_000_000_000u128),
            chain_id: 1,
            slippage_bps: 50,
            taker_address: address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"),
        }
    }

    fn sample_quote(params: &QuoteParams) -> SwapQuote {
        SwapQuote {
            provider: PROVIDER_NAME.to_string(),
            chain_id: params.chain_id,
            slippage_bps: params.slippage_bps,
            taker_address: params.taker_address,
            sell_token: params.sell_token,
            buy_token: params.buy_token,
            sell_amount: params.sell_amount,
            buy_amount: U256::from(312_000_000u128),
            minimum_buy_amount: U256::from(310_440_000u128),
            to: address!("Def1C0ded9bec7F1a1670819833240f027b25EfF"),
            data: Bytes::from(vec![0xd9, 0x62, 0x7a, 0xa4]),
            value: params.sell_amount,
            gas_estimate: 200_000,
            price: 3120.0,
            allowance_target: Some(address!("Def1C0ded9bec7F1a1670819833240f027b25EfF")),
            sources: vec![],
        }
    }

    #[test]
    fn base_url_per_chain() {
        assert_eq!(ZeroXProvider::base_url(1).unwrap(), "https://api.0x.org");
        assert_eq!(
            ZeroXProvider::base_url(42161).unwrap(),
            "https://arbitrum.api.0x.org"
        );
        assert_eq!(
            ZeroXProvider::base_url(8453).unwrap(),
            "https://base.api.0x.org"
        );
        assert_eq!(
            ZeroXProvider::base_url(10).unwrap(),
            "https://optimism.api.0x.org"
        );
        assert_eq!(
            ZeroXProvider::base_url(137).unwrap(),
            "https://polygon.api.0x.org"
        );
    }

    #[test]
    fn base_url_unsupported_chain() {
        let err = ZeroXProvider::base_url(99_999).unwrap_err();
        assert!(matches!(
            err,
            SwapError::UnsupportedChain { chain_id: 99_999 }
        ));
    }

    #[test]
    fn supported_chains_match_url_table() {
        for &id in SUPPORTED_CHAIN_IDS {
            ZeroXProvider::base_url(id).expect("supported chain must have URL");
        }
        assert_eq!(SUPPORTED_CHAIN_IDS.len(), ZEROX_CHAINS.len());
    }

    #[test]
    fn cache_hit_returns_same_quote() {
        let provider = ZeroXProvider::new(crate::http::build_http_client());
        let params = sample_params();
        let key = CacheKey::from_params(&params);
        let now = Instant::now();
        let quote = sample_quote(&params);
        provider.cache_put(key.clone(), quote.clone(), now);
        let hit = provider.cache_get(&key, now).expect("cached");
        assert_eq!(hit.buy_amount, quote.buy_amount);
        assert_eq!(hit.to, quote.to);
        assert_eq!(hit.taker_address, quote.taker_address);
    }

    #[test]
    fn cache_expiry_evicts_on_get() {
        let provider = ZeroXProvider::new(crate::http::build_http_client());
        let params = sample_params();
        let key = CacheKey::from_params(&params);
        let past = Instant::now()
            .checked_sub(Duration::from_secs(60))
            .expect("test clock");
        provider.cache_put_with_expiry(key.clone(), sample_quote(&params), past);
        let now = Instant::now();
        assert!(provider.cache_get(&key, now).is_none());
    }

    #[test]
    fn cache_lock_recovers_from_poison() {
        let provider = Arc::new(ZeroXProvider::new(crate::http::build_http_client()));
        let params = sample_params();
        let key = CacheKey::from_params(&params);

        let p2 = Arc::clone(&provider);
        let _ = std::thread::spawn(move || {
            let _guard = p2.cache.lock().expect("first lock");
            panic!("poison the mutex");
        })
        .join();

        // Mutex is now poisoned; cache_lock must still hand out a guard.
        provider.cache_put(key.clone(), sample_quote(&params), Instant::now());
        assert!(provider.cache_get(&key, Instant::now()).is_some());
    }

    #[test]
    fn dto_parse_golden() {
        let json = r#"{
            "to": "0xDef1C0ded9bec7F1a1670819833240f027b25EfF",
            "data": "0xd9627aa4",
            "value": "100000000000000000",
            "gas": "200000",
            "sellAmount": "100000000000000000",
            "buyAmount": "312000000",
            "price": "3120.0",
            "allowanceTarget": "0xDef1C0ded9bec7F1a1670819833240f027b25EfF",
            "sources": [{"name": "Uniswap_V3", "proportion": "1"}]
        }"#;
        let dto: ZeroXQuoteResponse = serde_json::from_str(json).expect("golden parse");
        assert_eq!(dto.gas, 200_000);
        assert_eq!(dto.buy_amount, U256::from(312_000_000u128));
        assert_eq!(dto.sell_amount, U256::from(100_000_000_000_000_000u128));
        assert_eq!(dto.value, U256::from(100_000_000_000_000_000u128));
        assert_eq!(dto.sources.len(), 1);
        assert_eq!(dto.sources[0].name, "Uniswap_V3");
        assert!(dto.allowance_target.is_some());
    }

    #[test]
    fn dto_parse_unknown_field_ignored() {
        let json = r#"{
            "to": "0x0000000000000000000000000000000000000001",
            "data": "0x",
            "value": "0",
            "gas": "21000",
            "sellAmount": "0",
            "buyAmount": "0",
            "price": "1.0",
            "sources": [],
            "futureField": "should be ignored",
            "anotherFuture": 123
        }"#;
        let dto: ZeroXQuoteResponse = serde_json::from_str(json).expect("unknown fields tolerated");
        assert!(dto.allowance_target.is_none());
        assert_eq!(dto.gas, 21_000);
    }

    #[test]
    fn dto_parse_invalid_decimal_rejected() {
        let json = r#"{
            "to": "0x0000000000000000000000000000000000000001",
            "data": "0x",
            "value": "not-a-number",
            "gas": "21000",
            "sellAmount": "0",
            "buyAmount": "0",
            "price": "0.0",
            "sources": []
        }"#;
        let result = serde_json::from_str::<ZeroXQuoteResponse>(json);
        assert!(result.is_err());
    }

    #[test]
    fn minimum_buy_amount_calculation() {
        let params = QuoteParams {
            slippage_bps: 50,
            ..sample_params()
        };
        let dto = ZeroXQuoteResponse {
            to: address!("0000000000000000000000000000000000000001"),
            data: Bytes::new(),
            value: U256::ZERO,
            gas: 21_000,
            sell_amount: U256::ZERO,
            buy_amount: U256::from(10_000u128),
            price: "1.0".to_string(),
            allowance_target: None,
            sources: vec![],
        };
        let quote = build_quote(dto, &params);
        // 10_000 × (10_000 − 50) / 10_000 = 9_950
        assert_eq!(quote.minimum_buy_amount, U256::from(9_950u128));
        assert_eq!(quote.chain_id, params.chain_id);
        assert_eq!(quote.taker_address, params.taker_address);
        assert_eq!(quote.slippage_bps, params.slippage_bps);
        assert_eq!(quote.provider, PROVIDER_NAME);
    }

    #[test]
    fn minimum_buy_amount_zero_slippage_keeps_full() {
        let params = QuoteParams {
            slippage_bps: 0,
            ..sample_params()
        };
        let dto = ZeroXQuoteResponse {
            to: address!("0000000000000000000000000000000000000001"),
            data: Bytes::new(),
            value: U256::ZERO,
            gas: 21_000,
            sell_amount: U256::ZERO,
            buy_amount: U256::from(10_000u128),
            price: "1.0".to_string(),
            allowance_target: None,
            sources: vec![],
        };
        let quote = build_quote(dto, &params);
        assert_eq!(quote.minimum_buy_amount, U256::from(10_000u128));
    }

    #[test]
    fn sanitize_body_strips_non_ascii_and_truncates() {
        let body = "hello \x00\x01 world\u{1F600}";
        let s = sanitize_body(body, 32);
        assert!(!s.contains('\x00'));
        assert!(!s.contains('\u{1F600}'));
        assert!(s.contains("hello"));

        let long = "a".repeat(1000);
        assert_eq!(sanitize_body(&long, 100).len(), 100);
    }

    #[test]
    fn provider_name_and_supported_chains() {
        let p = ZeroXProvider::new(crate::http::build_http_client());
        assert_eq!(p.name(), "0x");
        assert_eq!(p.supported_chains(), SUPPORTED_CHAIN_IDS);
    }
}
