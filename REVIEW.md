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

Все must-fix закрыты. Phase 1 + Phase 2 чисты.

---

## Consider (6 remaining)

1. **multi.rs** — Дупликация fetch_gas_fees/fetch_estimate_gas/fetch_nonce. Вынести helper `with_provider()`
2. **Cargo.toml** — Нет `overflow-checks = true` в `[profile.release]`
3. **Cargo.toml** — Нет clippy restriction lints (unwrap_used, indexing_slicing, panic)
4. **txguard/Cargo.toml** — Heavy deps (revm, reqwest) без feature gates. Parser-only consumer тянет EVM
5. **router/mod.rs** — `expect()` в library code. Заменить на proper error
6. **keyring/local.rs** — Нет custom Drop для LocalKeyring (zeroize on drop). `Zeroizing` покрывает `generate()`, но `decrypt_key` flow и `signer` field — нет.

---

## Good

- Архитектура: workspace layout по Codex (txguard lib, core domain, cli thin, api placeholder)
- Error handling: thiserror последовательно, proper variants, #[from], lowercase messages
- Type design: TransactionAction enum, Severity::weight(), Verdict, #[must_use]
- Тесты: 81 тест (txguard 38, core 33, desktop 8, doctests 2)
- Saturating arithmetic в финансовых расчётах
- Custom Debug для LocalKeyring скрывает signer internals
- GoPlus client: чистое разделение raw/public types
- format_wei: хорошо протестирован (zero, whole, fractional, tiny, large)
- Zeroize на key bytes в generate() — правильный паттерн
- Shared HTTP client across providers — экономия ресурсов
- cargo-deny настроен для license/vulnerability audit
- Phase 2: shared types crate (core ↔ frontend без U256 в WASM)
- Phase 2: pure helper extraction в commands.rs для testability
- Phase 2: server-side QR SVG generation (не тянет deps в WASM)
- Phase 2: CSP enabled, keystore 0600 permissions, Mutex safety documented
- Phase 2: CI с Tauri system deps, все 5 jobs зелёные

---

## Next steps

1. **Phase 3: Mobile (iOS + Android)**
   - Кросс-компиляция core на ARM targets
   - Tauri mobile builds (iOS + Android)
   - Passkey auth (WebAuthn), biometric unlock
2. Добавить overflow-checks в release profile (Consider #2)
3. Добавить custom Drop для LocalKeyring (zeroize on drop) (Consider #6)
4. Analyze page: добавить поле value для ETH transfer analysis
