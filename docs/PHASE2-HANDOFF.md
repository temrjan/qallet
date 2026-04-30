# Phase 2 — Session Handoff

> **Дата:** 2026-04-30
> **Branch:** `feat/phase2-core-api` (pushed up to commit 6)
> **Phase 2 progress:** **6/11 commits done (54%)** — Groups A + B + C closed; D (swap) next.

---

## Что сделано

### Commits (6 закрыты, на remote)

| # | SHA | Title | Group | Net diff |
|---|---|---|---|---|
| 1 | `bd7174d` | `chore(bridge): C4 hoisting fix — peerDependencies for uniffi-bindgen-react-native` | A | +9/−5 |
| 2 | `e232c20` | `feat(bindings): adopt structured BindingsError taxonomy (C2/C3)` | A | +121/−9 |
| 3 | `e6cd6a0` | `refactor(core): wallet lifecycle service (C1-A: wallet_id + encrypt-at-rest + reveal-once + spawn_blocking)` | B | +1054/−378 |
| 4 | `9dcc734` | `refactor(core): extract send/preview/balance services + chain_id getter` | B | +174/−31 |
| 5 | `e918c6b` | `feat(core): EIP-191 sign_message + EIP-712 sign_typed_data primitives` | C | +207/−2 |
| 6 | `ebfdfd7` | `feat(core): generic tx module — preview_transaction + sign_and_send_transaction` | C | +266/−0 |

`origin/feat/phase2-core-api` синхронизирован. Groups A+B+C полностью closed: foundation hygiene, error taxonomy, wallet lifecycle, send service refactor, signing primitives, generic tx module.

### Reviews пройдены

- **Commit 1:** `/typescript-review` APPROVED (1 deferrable nit).
- **Commit 2:** `/security-review` GO + `/rust-review` APPROVED-WITH-NITS (2 nits applied: `thiserror::Error` на sub-kinds, ASCII em-dash → colon).
- **Commit 3:** `/security-review` GO (9 findings, 0 above conf 8; 2 fixes applied: Vuln 6 `has_wallet` pure, Vuln 2 `encrypt_with_password_blocking`) + `/rust-review` APPROVED-WITH-NITS (4 LOW, 3 applied).
- **Commit 4:** `/check` (7 findings applied) + `/rust-review` APPROVED-WITH-NITS (1 MEDIUM out-of-scope mods → resolved by selective `git add`; 3 LOW kept) + `/security-review` GO.
- **Commit 5:** `/check` (7 findings applied) + `/rust-review` APPROVED-WITH-NITS (2 LOW: test-name overstatement + docstring completeness — accepted) + `/security-review` GO (EIP-712 byte-order verified, EIP-191 delegated to alloy, no new private-key materialization).
- **Commit 6:** `/check` (7 findings applied) + `/rust-review` APPROVED-WITH-NITS (2 LOW perf: per-call ProviderBuilder + sequential RPCs — both deferred to Phase 4-5) + `/security-review` GO (trust boundary explicit, sanity checks correct, signer clone-and-drop safe, CREATE rejected).

### Spike 0 (pre-Phase 2) — uniffi async

`uniffi 0.31.0-2` async fn export verification. **Outcome: GO (partial via codegen evidence)**. Runtime test deferred — first real device exercise lands в commit 9 (FFI exposure) or commit 10 (RN smoke harness). Generic tx module (commit 6) introduces real async surface; Spike 0 invariant unchanged but pending validation.

---

## Что дальше — commit 7 scope

**Title:** `feat(core): swap module — SwapProvider trait, ZeroXProvider, types, 0x API client (1inch as todo!() stub)`

**Group:** D (swap + txguard).

**Estimate:** ~5-7h pure work + reviews ≈ 8-10h realistic. Largest commit since wallet lifecycle (commit 3).

**Files affected (предположительно):**

NEW:
- `crates/core/src/swap/mod.rs` — `pub mod zero_x; pub mod types;` + `pub trait SwapProvider`.
- `crates/core/src/swap/types.rs` — `QuoteParams`, `SwapQuote`, `SwapPreview`, `LiquiditySource`, `SwapError`.
- `crates/core/src/swap/zero_x.rs` — `pub struct ZeroXProvider { client: reqwest::Client, api_key: Option<String> }` + `impl SwapProvider`.
- `crates/core/src/swap/one_inch.rs` (или inline в mod.rs) — `pub struct OneInchProvider;` с `todo!()` заглушкой методов trait per operator decision (1inch — stub, не функциональная).

MODIFIED:
- `crates/core/src/lib.rs` — `pub mod swap;`.
- `crates/core/Cargo.toml` — возможно нужен `serde` features для quote response deserialization (verify уже есть).

NOT MODIFIED в commit 7 (ставится в commit 8):
- `crates/txguard/src/rules/swap.rs` — swap-specific rules (router whitelist, slippage, approval analysis). Отдельный commit для txguard side.

**Goal:**
- Implement 0x Swap API v1 client (primary): `/swap/v1/quote` endpoint → `SwapQuote { to, data, value, gas, buyAmount, allowanceTarget, sources, ... }`.
- Establish `SwapProvider` trait abstraction для будущей замены/добавления провайдеров.
- 1inch stub demonstrates trait extensibility без полной реализации.
- API key через `option_env!("ZERO_X_API_KEY")` Rust-side (per operator-locked decision; proxy в Phase 8).
- Quote cache 30s TTL (per SWAP-INTEGRATION-PLAN.md §11 R2 mitigation).
- НЕ broadcast — quote только returns calldata; broadcast делается через `crate::sign::sign_and_send_transaction(keyring, provider, tx_from_quote, chain_id)` (commit 6).

**NOT в этом commit:**
- txguard swap rules (commit 8)
- WalletService::get_swap_quote / execute_swap orchestration wrappers (commit 9 FFI или earlier если permit signing для commit 7 нужен)
- Tauri commands swap UI (commit 9-10)
- 1inch full implementation (post-Phase 2 если нужен)
- Token list API (`/swap/v1/tokens`) — commit 9-10 при UI
- Approval flow (2-step approve+swap) — commit 9-10 UI orchestration
- WalletConnect adapter (post-Phase 5)

**Skills для commit 7:**
- `/codex` ✓ already loaded (architecture + pipeline)
- `/rust` ✓ already loaded (CORE.md)
- `/check` — обязательно после составления плана (это будет largest /check сессия)
- `/rust-review` — обязательно перед коммитом
- `/security-review` — **обязателен** (HTTP API client + JSON deserialization внешнего источника = новый attack surface)

---

## Ключевые технические решения (cumulative, commits 1-6)

### C1-A wallet redesign (commit 3)

- `WalletService::create_wallet(password) → WalletId` — mnemonic не возвращается. Encrypted-at-rest в `<data_dir>/.onboarding_mnemonic.encrypted` (Argon2id+AES-256-GCM, тот же scheme что keystore).
- `reveal_mnemonic_for_onboarding(wallet_id, password) → Zeroizing<String>` — atomic read+decrypt+remove. Removes only on success. После remove → `MnemonicAlreadyRevealed`.
- `WalletId = String` (EIP-55 mixed-case Address hex). Newtype в commit 9.
- Stale cleanup: ТОЛЬКО в `unlock` (post-Vuln-6 fix), не в `has_wallet` (pure query).

### Service-layer methods on WalletService (commit 4)

- `WalletService::balance(provider) → Result<UnifiedBalance, WalletServiceError>`
- `WalletService::preview_send(provider, to, amount_wei) → Result<SendPreview, _>`
- `WalletService::execute_send(provider, to, amount_wei) → Result<SendResult, _>`
- New error variant `WalletServiceError::Send(#[from] SendError)` с `#[error(transparent)]`.
- `MultiProvider::primary_chain_id() → Option<u64>` getter (Phase 7 заменит на selector со state).
- `get_chain_id` Tauri command (sync) for UI network badge.

### Signing primitives on LocalKeyring (commit 5)

- `sign_message(message: &[u8]) → Signature` — EIP-191 personal_sign через `alloy_primitives::eip191_hash_message`.
- `sign_typed_data(domain_separator: &B256, struct_hash: &B256) → Signature` — EIP-712 inline framing (`0x19 || 0x01 || domain || struct`), 66-byte stack buffer, no `alloy-sol-types` dep.
- `sign_hash` остаётся public с SAFETY note про prefix-aware sibling methods (defuses phishing pitfall).

### Generic tx module (commit 6 — sign.rs)

- `preview_transaction(provider, &tx, from, chain_id) → Result<TransactionPreview, SendError>` — txguard analysis + estimate_gas + gas_fees. ВСЕГДА returns Ok (даже при verdict.action == Block) — caller рендерит analysis в UI.
- `sign_and_send_transaction(keyring, provider, tx, chain_id) → Result<B256, SendError>` — sanity-checks (chain_id/from mismatch reject), preview, defense-in-depth Block check, fill missing nonce/gas/fees, broadcast via alloy ProviderBuilder с EthereumWallet.
- **Naming divergence от send.rs::preview_send** (которая returns Err(Blocked) рано) — задокументировано в module-level docstring.
- `tx.to == None` (CREATE) явно rejected — wallet'ы не deploy contracts от user'а.
- `compute_costs` private helper + 2 unit tests (saturating arithmetic).

### Argon2id non-blocking discipline (commit 3 — actual в keyring/wallet)

`tokio::task::spawn_blocking` обязательно для всех Argon2id путей:
1. `from_encrypted_blocking` — `unlock`
2. `from_mnemonic_blocking` — `create_wallet`, `import_from_mnemonic`
3. `decrypt_blocking` — `reveal_mnemonic_for_onboarding`
4. `encrypt_with_password_blocking` — `create_wallet` onboarding mnemonic encrypt

Pattern: `move` ownership of `Zeroizing<String>` password + payload в closure. После return → automatic zeroize.

### Error taxonomy (commit 2 — bindings)

```rust
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BindingsError {
    Wallet { kind: WalletErrorKind },
    Send { kind: SendErrorKind },
    Rpc { kind: RpcErrorKind },
    TxGuard { kind: TxGuardErrorKind },
    Encoding { kind: EncodingErrorKind },
    Swap { kind: SwapErrorKind },     // populated в commit 9 для commit 7 swap surface
    Internal,                          // sensitive context never crosses FFI
}
```

Path A enforcement: `thiserror::Error` derive на всех `*Kind` sub-enums + per-variant `#[error("...")]`. Parent variants используют `{kind}` (Display), не `{kind:?}` (Debug). Currently populated: `WalletErrorKind::MnemonicGeneration`. Other variants заполняются commit-by-commit (3-9). SwapErrorKind будет нужен в commit 9 при FFI exposure.

### encrypt_key/decrypt_key visibility (commit 3)

`crates/core/src/keyring/local.rs` — `fn encrypt_key/decrypt_key` private → `pub(crate)` + re-export в `keyring/mod.rs`. Crate-internal only. NOT public API.

### Cargo.toml deps changes

`crates/core/Cargo.toml` (commit 3):
- `tokio = { workspace = true, features = ["sync"] }` — additive
- `qrcode = { version = "0.14", default-features = false, features = ["svg"] }` — moved from app/src-tauri
- `tempfile = "3"` — dev-dep only

`packages/react-native-rustok-bridge/package.json` + `mobile/package.json` (commit 1):
- `uniffi-bindgen-react-native: 0.31.0-2` — moved from bridge devDeps к peerDeps + добавлено в mobile dependencies. Closes C4.

Commits 4-6: zero new deps (existing alloy-* / txguard / reqwest workspace deps).

---

## Файлы затронутые и незатронутые

### Затронутые (commits 1-6)

**Cargo:**
- `Cargo.lock` (auto-regenerated commits 1, 3)
- `crates/core/Cargo.toml` (commit 3 deps)
- `mobile/package.json`, `packages/react-native-rustok-bridge/package.json`, `package-lock.json` (commit 1)

**Rust core:**
- `crates/core/src/wallet.rs` (NEW commit 3 ~870 lines, +109 commit 4)
- `crates/core/src/sign.rs` (NEW commit 6 ~245 lines)
- `crates/core/src/lib.rs` (`pub mod wallet;` commit 3, `pub mod sign;` commit 6)
- `crates/core/src/keyring/mod.rs` (re-exports commit 3)
- `crates/core/src/keyring/local.rs` (visibility commit 3, +207 commit 5 — sign_message + sign_typed_data + tests)
- `crates/core/src/provider/multi.rs` (`primary_chain_id` commit 4 +33)
- `crates/core/src/provider/chains.rs` (invariant test commit 4)
- `crates/rustok-mobile-bindings/src/lib.rs` (commit 2 error taxonomy)

**Tauri desktop:**
- `app/src-tauri/src/commands.rs` (commit 3 refactored −329, commit 4 −30 boilerplate + `get_chain_id`)
- `app/src-tauri/src/lib.rs` (`.setup()` commit 3, `get_chain_id` registration commit 4)

**Docs:**
- `CLAUDE.md` + `docs/NATIVE-MIGRATION-PLAN.md` + `docs/REVIEWER-CONSTITUTION.md` (commit 97f35b6 docs-only — после commit 4)
- `docs/PHASE2-HANDOFF.md` (this file — created during commit 4 session, updated this session)

### НЕ затронутые (планомерно — commit 7-11 будут править)

- `crates/core/src/swap/` — НЕ существует, NEW в commit 7
- `crates/txguard/src/rules/swap.rs` — NEW в commit 8 (router whitelist, slippage, approval, price impact)
- `crates/txguard/src/parser/calldata.rs` — добавление swap selectors в commit 8 (Uniswap V2/V3, 0x Settlement)
- `crates/rustok-mobile-bindings/src/lib.rs` — FFI exposure commit 9 (29 commands + alloy mirror types)
- `mobile/App.tsx` / `mobile/src/screens/_DevHarness.tsx` — commit 9-10 demo screens
- `app/src-tauri/src/commands.rs` swap commands — commit 9 если desktop exposes
- `crates/core/src/wallet.rs` `WalletService::get_swap_quote/execute_swap` — commit 9 или earlier если EIP-2612 permit signing нужен в commit 7

---

## Какие skills нужны для commit 7

### Загружены в текущей session

- `/codex` — architecture.md + pipeline.md
- `/rust` — CORE.md + INDEX.md
- `/rust-review` — checklist.md
- `/security-review` — встроенный flow

### Нужны для commit 7

- `/codex` ✓ должен быть загружен (always)
- `/rust` ✓ нужен (Rust изменения)
- `/check` — обязательно после составления плана
- `/rust-review` — обязательно
- **`/security-review` — ОБЯЗАТЕЛЕН** (новый HTTP API client + JSON deserialization из untrusted external source = реальный новый attack surface; commit 7 первый где wallet talks к non-RPC HTTP endpoint)

Возможные дополнительные KB файлы:
- `blockchain/alloy.md` — если swap module использует alloy types (Address, U256, Bytes для calldata) — уже знакомо, но reload OK
- `security/crypto.md` — если EIP-2612 permit signing будет в commit 7 (использует `sign_typed_data` из commit 5)

---

## Quick start commands для новой сессии

```bash
# Путь ASCII-only (AGP не поддерживает кириллицу на Windows)
cd C:/Claude/projects/rustok

# Sanity check workspace state
git status
git log --oneline -8
git branch --show-current  # должно быть feat/phase2-core-api

# Pull latest (если работа в нескольких сессиях)
git pull --ff-only

# Verify workspace tests green
cargo test --workspace  # ожидается 146 passed (commit 6 baseline)

# Verify clippy clean
cargo clippy --workspace --all-targets -- -D warnings

# Verify fmt
cargo fmt --all --check

# Если нужно re-run Tauri desktop check
cargo check -p rustok-desktop
```

**Прочитать в новой сессии (порядок):**

1. `CLAUDE.md` (project root)
2. **Этот документ** — `docs/PHASE2-HANDOFF.md` (handoff context)
3. `docs/PHASE-2-CONSTRAINTS.md` — C1-C4 (все closed: C1 commit 3, C2/C3 commit 2, C4 commit 1)
4. `docs/SWAP-INTEGRATION-PLAN.md` §3.3-3.4, §6, §7 — commit 7 scope (0x API + SwapProvider trait + 1inch stub + Router whitelist preview)
5. `docs/NATIVE-MIGRATION-PLAN.md` Phase 2 — commit numbering reference
6. `crates/core/src/sign.rs` (current state) — commit 6 baseline; commit 7 swap calls into это для broadcast
7. `crates/core/src/keyring/local.rs` (sign_message / sign_typed_data) — для EIP-2612 permit если нужен
8. `crates/core/src/http.rs` (existing reqwest client builder) — для swap module HTTP client setup
9. Memory: `MEMORY.md` index → `project_rustok_status.md`, `feedback_push_policy.md`, `feedback_review_skills_trigger.md`

---

## Open risks / blockers

### Active risks (commit 7-related)

1. **uniffi async on real device** — Spike 0 partial GO via codegen. Runtime validation deferred to commit 9-10 (mobile FFI exposure / DevHarness). Risk: device test может выявить runtime issue в `sign_and_send_transaction` или future `get_swap_quote` async chain, потребует blocking-on-runtime adapter pattern.

2. **alloy mirror types для FFI** — `U256`, `Address`, `Bytes`, `B256`, `Signature` не uniffi-derivable. Mirror types в bindings crate обязательны в commit 9. Estimate +1-2h на commit 9.

3. **0x API rate limiting** — free tier rate limit. Mitigation: quote cache 30s TTL в commit 7 (per SWAP-INTEGRATION-PLAN.md §11). API key через `ZERO_X_API_KEY` env (operator-locked).

4. **0x API JSON shape evolution** — 0x v1 API stable но could change. Mitigation: `#[serde(deny_unknown_fields)]` для defensive deserialization (per crypto.md / Codex rules).

5. **Per-call ProviderBuilder anti-pattern (LOW from commit 6 review)** — sign_and_send creates fresh HTTP pool per call. Pre-existing pattern in send.rs. Defer to Phase 4-5 cross-file refactor.

### Deferred items

1. **Vuln 1 (atomic write)** — `std::fs::write` для keystore не атомичен. Phase 4 hardening.
2. **Vuln 9 (signer clone lifetime в send_eth)** — pre-existing pattern. Phase 4 polish.
3. **devDependencies duplicate в bridge package.json** (commit 1 nit) — Phase 4-5 hygiene.
4. **`generate_mnemonic_phrase()` deprecated path** — known C1 violation. Removed Phase 4 cleanup.
5. **`sign_message_known_vector` test name overstatement** (commit 5 LOW) — accept или rename в Phase 4-5.
6. **`sign_typed_data` docstring manual-hash path** (commit 5 LOW) — accept или expand при first consumer (commit 7 EIP-2612).
7. **Per-call ProviderBuilder + sequential RPCs** (commit 6 LOW × 2) — Phase 4-5 perf pass.

### Phase 2 entry conditions (still satisfied)

- ✅ M1+M2+M3+M4 closed (PR #10, #11 merged)
- ✅ C1-C4 decisions approved (locked, all four closed)
- ✅ SWAP-INTEGRATION-PLAN.md approved
- ✅ Branch `feat/phase2-core-api` from main `9a59bd7`
- ✅ `cargo test --workspace` green (146 tests as of commit 6)

### What blocks commit 7 — nothing

Audit step required: read SWAP-INTEGRATION-PLAN.md §3.3-3.4 (swap module structure), §6 (0x API endpoint shapes), §7 (txguard rules preview but defer impl to commit 8). ~30-45 min audit, then plan + /check + implementation.

---

## Memory state at handoff

- `project_rustok_status.md` — needs update to «Phase 2 in progress 6/11 commits done; Groups A+B+C closed; commit 7 (swap module) next». Update в конце commit-6 session или start commit-7 session.
- `MEMORY.md` index — line про Rustok status показывает «Phase 2 in progress 5/11 commits done (Groups A+B closed; Group C 1/2)» (post commit 5 update). Needs bump до 6/11 (Groups A+B+C closed).
- `feedback_push_policy.md` — действует, push после каждого commit (cadence 1 атомарный коммит на ветке backed up by remote).
- `feedback_review_skills_trigger.md` — действует, все commits пройдены.
- `feedback_verify_rule.md` — действует, все claims о коде verified через Read/Grep + alloy source registry checks.

---

**Конец handoff.** Новая сессия может начинать с `git status` + read этого документа + commit 7 audit (SWAP-INTEGRATION-PLAN.md §3.3-3.4, §6, §7).
