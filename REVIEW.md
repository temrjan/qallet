# Code Review — Qallet Full Codebase
> Date: 2026-04-05 (updated)
> Previous review: 2026-04-02
> Standard: Codex rust.md v1.0 + architecture.md v1.1
> Status: Phase 1 — txguard + core + CLI functional, wallet send next

---

## Fixed since last review

| # | Issue | Fix |
|---|-------|-----|
| ~~#1~~ | Integer overflow в risk_score | `u8::try_from(...).unwrap_or(u8::MAX).min(10)` (types.rs:147) |
| ~~#2~~ | Dead code send.rs:69 | Переписано: placeholder для Phase 2 с `#[allow]` + комментарий |
| ~~#3~~ | Нет zeroize на приватных ключах | `Zeroizing::new(signer.credential().to_bytes())` (local.rs:48) |
| ~~#5~~ | Panic на пустых rpc_urls | `primary_rpc()` → `Option<&str>` через `.first()` (chains.rs:102) |
| ~~C1~~ | GoPlus без timeout | Добавлен 10s + 5s connect timeout (коммит 7018604) |
| ~~C2~~ | Новый provider на каждый call | Shared `reqwest::Client` (коммит 6379417) |
| ~~C5~~ | Нет `unsafe_code = "deny"` | Добавлен в workspace lints (Cargo.toml:18) |
| ~~C10~~ | Нет deny.toml | Добавлен cargo-deny (коммит 4fb3117) |

---

## Must fix (3 remaining)

### 1. `crates/cli/src/main.rs` — Пароль всё ещё можно передать через CLI args
`password: Option<String>` с `#[arg(long)]` — `--password "secret"` виден в `ps aux`.
Интерактивный ввод через `rpassword` уже добавлен (main.rs:500,519,521), но `--password` флаг не убран.
**Fix:** убрать `--password` из CLI args, оставить только `rpassword` prompt + `env = "QALLET_PASSWORD"`.

### 2. `crates/txguard/src/simulator/mod.rs:129` — i128 cap для eth_change
`value.try_into().unwrap_or(i128::MAX)` — теряет precision для значений > 170 141 183 ETH.
Практически недостижимо (total supply ETH ~120M), но нарушает принцип точности.
**Fix:** документировать лимит комментарием, или использовать `I256` из alloy-primitives.

### 3. `crates/core/src/provider/multi.rs:129` — Cross-chain total caveat
`total` суммирует ETH с разных L2, но это не fungible — нельзя потратить Arbitrum ETH на Base.
Документация добавлена (multi.rs:129 doc-comment), но `total_formatted` в структуре `UnifiedBalance` может ввести в заблуждение UI-потребителя.
**Fix:** переименовать в `approximate_total_formatted` или добавить поле `is_approximate: true`.

---

## Consider (7 remaining)

1. **multi.rs** — Дупликация fetch_gas_fees/fetch_estimate_gas/fetch_nonce. Вынести helper `with_provider()`
2. **Cargo.toml** — Нет `overflow-checks = true` в `[profile.release]`
3. **Cargo.toml** — Нет clippy restriction lints (unwrap_used, indexing_slicing, panic)
4. **txguard/Cargo.toml** — Heavy deps (revm, reqwest) без feature gates. Parser-only consumer тянет EVM
5. **router/mod.rs** — `expect()` в library code. Заменить на proper error
6. **ci.yml** — Все steps в одном job. Разбить на parallel jobs (fmt, clippy, test, docs)
7. **keyring/local.rs** — Нет custom Drop для LocalKeyring (zeroize on drop). `Zeroizing` покрывает `generate()`, но `decrypt_key` flow и `signer` field — нет.

---

## Good

- Архитектура: workspace layout по Codex (txguard lib, core domain, cli thin, api placeholder)
- Error handling: thiserror последовательно, proper variants, #[from], lowercase messages
- Type design: TransactionAction enum, Severity::weight(), Verdict, #[must_use]
- Тесты: 69 реальных тестов с настоящими данными (USDT addr, Uniswap Router)
- Saturating arithmetic в финансовых расчётах
- Custom Debug для LocalKeyring скрывает signer internals
- GoPlus client: чистое разделение raw/public types
- format_wei: хорошо протестирован (zero, whole, fractional, tiny, large)
- Zeroize на key bytes в generate() — правильный паттерн
- Shared HTTP client across providers — экономия ресурсов
- cargo-deny настроен для license/vulnerability audit

---

## Next steps

1. Fix must-fix #1 (убрать --password из CLI args)
2. Fix must-fix #3 (approximate_total)
3. Добавить overflow-checks в release profile
4. Implement wallet send
5. Добавить custom Drop для LocalKeyring (zeroize on drop)
6. Push + verify CI
