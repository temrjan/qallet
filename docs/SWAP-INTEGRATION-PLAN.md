# Swap Integration Plan — 0x API + Signing Pipeline

> **Создан:** 2026-04-30
> **Статус:** APPROVED — реализация начинается с Phase 2 (Rust core additions)
> **Связь:** `docs/NATIVE-MIGRATION-PLAN.md` (Phases 2, 5), `docs/PHASE-2-CONSTRAINTS.md`
> **Primary API:** 0x Swap API
> **Backup API:** 1inch Classic Swap API
> **Тестовая сеть:** Arbitrum One (mainnet, gas ~$0.01-0.03 per swap)

---

## 0. Executive Summary

Rustok интегрирует DEX swap через агрегатор API (0x primary, 1inch backup). Кошелёк выступает как signing tool — не биржа, не custodian. Пользователь выбирает пару токенов, получает котировку от API, txguard анализирует calldata перед подписью, кошелёк подписывает и отправляет on-chain.

Ключевое архитектурное решение: **все транзакции (включая swap) проходят через txguard до подписи.** Это главное отличие Rustok от MetaMask/Trust Wallet — пользователь видит human-readable разбор того, что подписывает.

---

## 1. Что есть сейчас (baseline)

| Компонент | Статус | Файл |
|-----------|--------|------|
| `sign_hash(B256)` | Есть | `crates/core/src/keyring/local.rs:186` |
| `sign_message()` (EIP-191) | **НЕТ** | — |
| `sign_typed_data()` (EIP-712) | **НЕТ** | — |
| Generic tx signing | **НЕТ** (только ETH transfer через provider) | `crates/core/src/send.rs` |
| txguard arbitrary calldata | **Есть** (ERC-20, permit, unknown selector) | `crates/txguard/src/parser/` |
| txguard Swap rules category | **Есть** (пустая реализация) | `crates/txguard/src/types.rs:82` |
| HTTP client (reqwest) | **Есть** | `crates/core/src/http.rs` |
| Multi-chain RPC (Arbitrum, Base, etc.) | **Есть** | `crates/core/src/provider/multi.rs` |
| 0x / 1inch API client | **НЕТ** | — |

---

## 2. Архитектура swap flow (целевая)

```
User: выбирает ETH → USDC, вводит amount
        |
        v
[RN: SwapScreen.tsx]
        |
        v
await rustok.getSwapQuote({ sellToken, buyToken, amount, chainId })
        |
        v
[Rust: swap::get_quote()] ──HTTP──> 0x API /swap/v1/quote
        |                                      |
        v                                      v
[Rust: получает calldata + quote]         0x returns:
        |                                 - to (router address)
        |                                 - data (calldata)
        |                                 - value (ETH amount)
        |                                 - gas estimate
        |                                 - price / guaranteedPrice
        v
[Rust: txguard::analyze(to, data, value)]
        |
        v
[Rust: returns SwapPreview { quote, verdict, warnings }]
        |
        v
[RN: ConfirmSwapScreen — показывает анализ txguard]
        |
        v
User: нажимает Confirm
        |
        v
await rustok.executeSwap({ quote_id })
        |
        v
[Rust: sign_and_send_transaction(to, data, value, gas)]
        |
        v
[On-chain: swap executed]
        |
        v
[RN: success toast + navigate to Activity]
```

---

## 3. Phase 2 — Rust Core Additions (signing pipeline)

Эти изменения идут В СОСТАВЕ Phase 2 (Core API extraction), не отдельно. Phase 2 расширяется с 22 до 29 команд (22 legacy + get_chain_id + 4 signing + 2 swap).

### 3.1 Signing primitives (keyring layer)

Файл: `crates/core/src/keyring/local.rs`

```rust
// EIP-191: personal_sign
// Prefixes message with "\x19Ethereum Signed Message:\n{len}"
pub fn sign_message(&self, message: &[u8]) -> Result<Signature, KeyringError> {
    let hash = eip191_hash_message(message);
    self.sign_hash(&hash)
}

// EIP-712: sign_typed_data
// Takes pre-computed struct hash + domain separator
pub fn sign_typed_data(
    &self,
    domain_separator: &B256,
    struct_hash: &B256,
) -> Result<Signature, KeyringError> {
    let hash = eip712_signing_hash(domain_separator, struct_hash);
    self.sign_hash(&hash)
}
```

Зависимости: `alloy-sol-types` (уже в workspace для EIP-712 hashing).

**C1 constraint:** `sign_message` и `sign_typed_data` возвращают `Signature` (hex string) — не secret material, zeroize не требуется. Но входные данные (message bytes, struct hash) проходят через FFI как `Vec<u8>` / `String` — те же ограничения что в `docs/PHASE-2-CONSTRAINTS.md` §C1. Решение по C1 применяется ко всем FFI-функциям единообразно.

### 3.2 Generic transaction signing (send layer)

Файл: новый `crates/core/src/sign.rs`

```rust
/// Sign and broadcast an arbitrary transaction (not just ETH transfer).
/// Used for swap execution, contract interactions, approvals.
pub async fn sign_and_send_transaction(
    keyring: &LocalKeyring,
    provider: &MultiProvider,
    tx: TransactionRequest,  // alloy TransactionRequest
    chain_id: u64,
) -> Result<B256, SendError> {
    // 1. Fill gas, nonce via provider
    // 2. Sign with keyring's PrivateKeySigner
    // 3. Broadcast via provider
    // 4. Return tx hash
}

/// Preview arbitrary transaction through txguard WITHOUT signing.
pub async fn preview_transaction(
    provider: &MultiProvider,
    tx: &TransactionRequest,
    chain_id: u64,
) -> Result<TransactionPreview, SendError> {
    // 1. Estimate gas
    // 2. Run txguard::analyze(to, data, value)
    // 3. Return preview with verdict + gas estimate + human-readable breakdown
}
```

### 3.3 Swap module (0x API client)

Файл: новый `crates/core/src/swap/mod.rs`

```rust
pub mod zero_x;    // 0x API client
pub mod types;     // SwapQuote, SwapPreview, SwapError

// Trait для swap providers — позволяет подключить 1inch без рефакторинга
pub trait SwapProvider: Send + Sync {
    async fn get_quote(&self, params: QuoteParams) -> Result<SwapQuote, SwapError>;
    fn name(&self) -> &str;
    fn supported_chains(&self) -> &[u64];
}
```

Файл: `crates/core/src/swap/zero_x.rs`

```rust
pub struct ZeroXProvider {
    client: reqwest::Client,
    api_key: Option<String>,  // optional, free tier works without
}

impl SwapProvider for ZeroXProvider {
    async fn get_quote(&self, params: QuoteParams) -> Result<SwapQuote, SwapError> {
        // GET https://api.0x.org/swap/v1/quote
        // ?sellToken={}&buyToken={}&sellAmount={}&chainId={}
        // Parse response → SwapQuote { to, data, value, gas, price, ... }
    }
}
```

Файл: `crates/core/src/swap/types.rs`

```rust
pub struct QuoteParams {
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: U256,         // in wei
    pub chain_id: u64,
    pub slippage_bps: u16,        // basis points, default 50 (0.5%)
    pub taker_address: Address,   // user's wallet address
}

pub struct SwapQuote {
    pub provider: String,          // "0x" | "1inch"
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: U256,
    pub buy_amount: U256,          // expected output
    pub minimum_buy_amount: U256,  // after slippage
    pub to: Address,               // router contract
    pub data: Bytes,               // calldata for on-chain execution
    pub value: U256,               // ETH to send (for ETH→token swaps)
    pub gas_estimate: u64,
    pub price: f64,                // sell/buy price ratio
    pub sources: Vec<LiquiditySource>,  // which DEXs 0x routes through
}

pub struct SwapPreview {
    pub quote: SwapQuote,
    pub verdict: txguard::Verdict,  // txguard analysis result
    pub warnings: Vec<String>,
    pub gas_cost_eth: U256,
    pub total_cost_eth: U256,       // value + gas
}
```

### 3.4 uniffi exports (bindings layer)

Файл: `crates/rustok-mobile-bindings/src/lib.rs` — добавляются 6 новых команд:

```rust
// Signing (для WalletConnect в будущем)
#[uniffi::export]
pub fn sign_message(message: Vec<u8>) -> Result<String, BindingsError>;

#[uniffi::export]
pub fn sign_typed_data(domain_separator: String, struct_hash: String) -> Result<String, BindingsError>;

// Swap
#[uniffi::export]
pub async fn get_swap_quote(params: SwapQuoteParams) -> Result<SwapQuote, BindingsError>;

#[uniffi::export]
pub async fn execute_swap(quote: SwapQuote) -> Result<String, BindingsError>;  // returns tx hash

// Generic transaction (для approve, arbitrary contract calls)
#[uniffi::export]
pub async fn preview_transaction(to: String, data: String, value: String, chain_id: u64) -> Result<TransactionPreview, BindingsError>;

#[uniffi::export]
pub async fn send_transaction(to: String, data: String, value: String, chain_id: u64) -> Result<String, BindingsError>;
```

Итого Phase 2: 22 (existing) + get_chain_id + 4 signing + 2 swap = **29 команд** через uniffi.

### 3.5 txguard swap rules (расширение)

Файл: `crates/txguard/src/parser/calldata.rs` — добавить known selectors:

```rust
// Uniswap V2 Router
const SWAP_EXACT_TOKENS: [u8; 4] = [0x38, 0xed, 0x17, 0x39];  // swapExactTokensForTokens
const SWAP_EXACT_ETH: [u8; 4] = [0x7f, 0xf3, 0x6a, 0xb5];    // swapExactETHForTokens

// Uniswap V3 Router
const EXACT_INPUT_SINGLE: [u8; 4] = [0x41, 0x4b, 0xf3, 0x89];  // exactInputSingle
const MULTICALL: [u8; 4] = [0xac, 0x96, 0x50, 0xd8];           // multicall

// 0x Settlement
const FILL_QUOTE: [u8; 4] = [...];  // fillQuoteTransformerData
```

Файл: `crates/txguard/src/rules/swap.rs` — правила анализа:

```rust
// R1: Slippage check — buy_amount vs minimum_buy_amount > threshold
// R2: Router verification — is `to` address a known DEX router?
// R3: Token approval check — does swap require prior approve()? Warn if unlimited
// R4: Price impact — если > 5% от market price, WARNING
// R5: Sandwich risk — если gas price anomalous, INFO
```

---

## 4. Phase 3 — Design (swap в дизайн-системе)

В Phase 3 (Design System) закладываем компоненты для swap UI:

```
<TokenSelector>    — выбор токена из списка (поиск, balances)
<SwapInput>        — ввод amount с token badge и USD эквивалентом
<SwapRoute>        — визуализация маршрута (ETH → USDC via Uniswap V3)
<PriceImpact>      — badge с цветом (green/yellow/red)
<TxGuardBadge>     — компактный verdict indicator
```

Token list: используем 0x token list API (`/swap/v1/tokens`) или Uniswap default token list (npm package `@uniswap/default-token-list`).

---

## 5. Phase 5 — Swap Screen (реальная реализация)

Swap screen перестаёт быть placeholder. Конкретные экраны:

### 5.1 SwapScreen

```
┌─────────────────────────┐
│  Swap                   │
│                         │
│  ┌───────────────────┐  │
│  │ ETH         ▼     │  │
│  │ 0.1               │  │
│  │ ≈ $312.50         │  │
│  └───────────────────┘  │
│         ↕ (swap button) │
│  ┌───────────────────┐  │
│  │ USDC        ▼     │  │
│  │ ~312.00           │  │
│  └───────────────────┘  │
│                         │
│  Route: Uniswap V3     │
│  Price impact: 0.05%   │
│  Slippage: 0.5%        │
│                         │
│  [    Review Swap    ]  │
└─────────────────────────┘
```

### 5.2 ConfirmSwapScreen

```
┌─────────────────────────┐
│  Confirm Swap           │
│                         │
│  0.1 ETH → 312.00 USDC │
│                         │
│  ┌─ TxGuard Analysis ─┐ │
│  │ ✓ Known router      │ │
│  │ ✓ Slippage OK       │ │
│  │ ✓ No unusual perms  │ │
│  │ Risk: LOW           │ │
│  └─────────────────────┘ │
│                         │
│  Network: Arbitrum      │
│  Gas: ~$0.02            │
│  Min. received: 310.44  │
│                         │
│  [   Confirm Swap    ]  │
└─────────────────────────┘
```

### 5.3 Token Approval Flow

Если swap требует ERC-20 approve (token → token swap):

1. Detect: 0x API response содержит `allowanceTarget`
2. Check: `allowance(user, spender)` через RPC
3. If insufficient: показать approve screen ПЕРЕД swap
4. txguard анализирует approve calldata (unlimited vs exact amount)
5. Sign approve → wait confirmation → proceed to swap

---

## 6. API Integration Details

### 6.1 0x Swap API (primary)

**Base URL:** `https://api.0x.org` (Ethereum), `https://arbitrum.api.0x.org` (Arbitrum), etc.

**Endpoints:**

| Endpoint | Purpose |
|----------|---------|
| `GET /swap/v1/price` | Quick price check (no calldata, faster) |
| `GET /swap/v1/quote` | Full quote with executable calldata |
| `GET /swap/v1/tokens` | Supported token list |

**Quote request:**
```
GET /swap/v1/quote
  ?sellToken=0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE  (ETH)
  &buyToken=0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48   (USDC)
  &sellAmount=100000000000000000  (0.1 ETH in wei)
  &takerAddress=0x...
  &slippagePercentage=0.005
```

**Quote response (ключевые поля):**
```json
{
  "to": "0xDef1C0ded9bec7F1a1670819833240f027b25EfF",  // 0x Exchange Proxy
  "data": "0xd9627aa4...",                               // calldata
  "value": "100000000000000000",                         // ETH to send
  "gas": "200000",
  "gasPrice": "...",
  "buyAmount": "312000000",                              // USDC (6 decimals)
  "guaranteedPrice": "3100.0",
  "sources": [{"name": "Uniswap_V3", "proportion": "1"}],
  "allowanceTarget": "0xDef1C0ded9bec7F1a1670819833240f027b25EfF"
}
```

**Pricing:** Free tier available. 0x charges 0.15% affiliate fee on select pairs. Optional: зарегистрировать API key для higher rate limits.

### 6.2 1inch Classic Swap API (backup)

**Base URL:** `https://api.1inch.dev/swap/v6.0/{chainId}`

**Endpoints:**

| Endpoint | Purpose |
|----------|---------|
| `GET /quote` | Price quote (no calldata) |
| `GET /swap` | Full swap with calldata |
| `GET /tokens` | Token list |
| `GET /approve/allowance` | Check approval |
| `GET /approve/transaction` | Get approve calldata |

**Отличия от 0x:**
- Требует API key (бесплатный через Developer Portal)
- Approve helpers встроены в API (удобнее)
- Response shape отличается (маппинг нужен)

**Когда переключаемся на 1inch:**
- 0x rate limit exceeded
- 0x не поддерживает конкретную chain
- 0x quote хуже 1inch на >0.5% (можно сравнивать обе котировки)

### 6.3 Chain support

| Chain | 0x | 1inch | Arbitrum gas |
|-------|-----|-------|-------------|
| Ethereum | Да | Да | — |
| Arbitrum | Да | Да | ~$0.01-0.03 |
| Base | Да | Да | ~$0.01 |
| Optimism | Да | Да | ~$0.01 |
| Polygon | Да | Да | ~$0.001 |

**Тестирование:** Arbitrum One (mainnet). Gas дешёвый, finality быстрая. Закидываем 0.01-0.05 ETH, хватит на десятки тестовых swaps.

---

## 7. Security — txguard integration for swaps

Каждый swap проходит через txguard ДО подписи. Это non-negotiable.

### 7.1 Analysis flow

```rust
// В swap::get_quote() после получения calldata от 0x:
let parsed = txguard::parser::parse(
    quote.to,       // router address
    &quote.data,    // calldata bytes
    quote.value,    // ETH value
);

let verdict = txguard::rules::evaluate(&parsed, &context);

// Verdict содержит:
// - risk_level: Low | Medium | High | Critical
// - warnings: Vec<Warning>
// - actions: Vec<Action> (что делает транзакция)
// - recommendation: Proceed | ReviewCarefully | DoNotSign
```

### 7.2 Swap-specific rules

| Rule | Severity | Trigger |
|------|----------|---------|
| Unknown router | HIGH | `to` address не в whitelist known DEX routers |
| Excessive slippage | MEDIUM | slippage > 3% |
| Price impact | MEDIUM | > 5% deviation from oracle price |
| Unlimited approval | MEDIUM | approve(spender, type(uint256).max) |
| Multicall complexity | INFO | multicall с > 3 sub-calls |
| Fresh contract | HIGH | router deployed < 7 days ago |
| Unverified contract | MEDIUM | router not verified on block explorer |

### 7.3 Router whitelist

Хардкодим known router addresses per chain:

```rust
const KNOWN_ROUTERS: &[(u64, &[Address])] = &[
    (1, &[       // Ethereum
        address!("Def1C0ded9bec7F1a1670819833240f027b25EfF"),  // 0x Exchange Proxy
        address!("E592427A0AEce92De3Edee1F18E0157C05861564"),  // Uniswap V3 Router
        address!("7a250d5630B4cF539739dF2C5dAcb4c659F2488D"),  // Uniswap V2 Router
        address!("1111111254EEB25477B68fb85Ed929f73A960582"),  // 1inch V5 Router
    ]),
    (42161, &[   // Arbitrum
        // ... same contracts, different addresses
    ]),
];
```

---

## 8. WalletConnect v2 — следующий этап после swap

WalletConnect НЕ входит в текущий swap plan, но signing pipeline (Phase 2) подготавливает инфраструктуру:

- `sign_message()` → WalletConnect `personal_sign` requests
- `sign_typed_data()` → WalletConnect `eth_signTypedData_v4` requests
- `sign_and_send_transaction()` → WalletConnect `eth_sendTransaction` requests
- txguard analysis → WalletConnect transaction review (то же что для swap)

WalletConnect v2 integration — отдельный план после Phase 5. Требует:
- `@walletconnect/react-native-compat` package
- Deep link handling (wc:// URI scheme)
- Session management (approve/reject connections)
- Request routing (sign → txguard → confirm → sign → respond)

Все signing primitives к этому моменту уже будут в ядре.

---

## 9. Testing Strategy

### 9.1 Unit tests (Rust)

```rust
// crates/core/src/keyring/ — sign_message, sign_typed_data
#[test] fn test_personal_sign_matches_ethers_js();  // compatibility
#[test] fn test_eip712_domain_separator_hashing();
#[test] fn test_sign_typed_data_matches_metamask();

// crates/core/src/swap/ — API client
#[test] fn test_parse_0x_quote_response();
#[test] fn test_parse_1inch_quote_response();
#[test] fn test_swap_provider_trait_dispatch();

// crates/txguard/ — swap rules
#[test] fn test_known_router_passes();
#[test] fn test_unknown_router_high_risk();
#[test] fn test_excessive_slippage_warning();
#[test] fn test_unlimited_approve_warning();
```

### 9.2 Integration tests (Arbitrum mainnet)

Ручные тесты на физическом устройстве с реальными транзакциями:

| Test | Expected | Cost |
|------|----------|------|
| ETH → USDC swap (0.001 ETH) | Success, USDC received | ~$0.04 |
| USDC → ETH swap | Success, ETH received | ~$0.03 |
| Token approve flow | Approve tx + swap tx (2 steps) | ~$0.05 |
| High slippage warning | txguard shows MEDIUM warning | ~$0.03 |
| Cancel after preview | No tx sent, no cost | $0.00 |
| 1inch fallback | Switch provider, same result | ~$0.04 |

**Budget:** 0.05 ETH на Arbitrum (~$150 at current prices) хватит на полный test suite.

### 9.3 Automated tests (CI)

- Mock 0x API responses для unit tests (no network calls in CI)
- txguard rule tests с hardcoded calldata samples
- Signing compatibility tests (compare output with ethers.js reference vectors)

---

## 10. Implementation Timeline

| Phase | What | Swap-related additions |
|-------|------|----------------------|
| **Phase 2** | Core API extraction | `sign_message`, `sign_typed_data`, `sign_and_send_transaction`, `preview_transaction`, swap module, 0x API client, txguard swap rules, 6 new uniffi exports |
| **Phase 3** | Design system | `<TokenSelector>`, `<SwapInput>`, `<SwapRoute>`, `<PriceImpact>`, `<TxGuardBadge>` components |
| **Phase 5** | Wallet screens | SwapScreen, ConfirmSwapScreen, TokenApproveScreen — fully functional, tested on Arbitrum |
| **Post-Phase 5** | WalletConnect v2 | Session management, deep links, request routing (signing infra already in place) |

---

## 11. Risks

| Risk | Severity | Mitigation |
|------|----------|-----------|
| 0x API changes/deprecation | Medium | SwapProvider trait — switch to 1inch in hours |
| Rate limiting on free tier | Low | Cache quotes (30s TTL), lazy fetch on input change |
| Calldata parsing incomplete | Medium | txguard Unknown fallback + known router whitelist |
| Token approval UX confusion | Medium | Clear 2-step flow, explain in UI why approve needed |
| Price manipulation (sandwich) | Low | Slippage protection built into 0x quote, txguard warning |
| API key leak in APK | Medium | API key stored in Rust (not JS), obfuscated in release build. Or: proxy through our backend |

---

## 12. Decision Log

| Decision | Rationale | Date |
|----------|-----------|------|
| 0x primary, 1inch backup | 0x: mature SDK, good docs, free tier. 1inch: Classic API equally capable, good backup. SwapProvider trait makes switching trivial. | 2026-04-30 |
| Arbitrum for testing (not testnet) | Li.Fi no testnet, 1inch no testnet. 0x has Sepolia but limited pairs. Arbitrum mainnet: real liquidity, $0.01-0.03 gas, fast finality. | 2026-04-30 |
| All swaps through txguard | Core product differentiator. Non-negotiable. | 2026-04-30 |
| No WebView DApp Browser | Apple blocks WebView DApp browsers on iOS. WalletConnect v2 instead. | 2026-04-30 |
| Signing pipeline in Phase 2 | sign_message + sign_typed_data needed for both swap (future permit2) and WalletConnect. Building once, using twice. | 2026-04-30 |

---

**Конец документа.**
