# Code Review — Rustok Full Codebase
> Date: 2026-04-05 (updated)
> Previous review: 2026-04-02
> Standard: Codex rust.md v1.0 + architecture.md v1.1
> Status: Phase 2 DONE (81 tests, 0 must-fix).

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
| ~~M1~~ | `--password` в CLI args | Убран. `resolve_password()` через env/rpassword (коммит af31c52) |
| ~~M2~~ | i128 cap без документации | Задокументирован комментарием (simulator/mod.rs:129-130, коммит af31c52) |
| ~~M3~~ | `total_formatted` вводит в заблуждение | Переименован в `approximate_total_formatted` (multi.rs:73, коммит af31c52) |

---

## Must fix (0 remaining)

Все must-fix закрыты. Phase 1 чиста.

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

1. **Phase 2: Desktop app (Tauri 2.0 + Leptos)**
   - See `docs/PHASE2-LEPTOS-TAURI.md` for implementation guide
   - Шаги: types crate → Tauri scaffold → Leptos frontend → commands → pages
2. Добавить overflow-checks в release profile (Consider #2)
3. Добавить custom Drop для LocalKeyring (zeroize on drop) (Consider #7)
4. Push + verify CI
