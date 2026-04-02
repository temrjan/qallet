# Code Review — Qallet Full Codebase
> Date: 2026-04-02
> Standard: Codex rust.md v1.0 + architecture.md v1.1
> Status: rename done, wallet send in progress (stashed)

---

## Must fix (8)

### 1. `crates/txguard/src/types.rs:147` — Integer overflow в risk score
`findings.len() as u8` truncates silently. 256 findings → score 0.
**Fix:** `u8::try_from(findings.len()).unwrap_or(u8::MAX)` или `.min(10) as u8`

### 2. `crates/txguard/src/rules/send.rs:69-71` — Dead code
`if parsed.value.is_zero()` с пустым телом. Забытый early return.
**Fix:** добавить `return None;` или удалить блок

### 3. `crates/core/src/keyring/local.rs` — Нет zeroize на приватных ключах
`PrivateKeySigner` держит raw key в памяти. `decrypt_key` возвращает `Vec<u8>` без зануления.
**Fix:** добавить `zeroize` + `secrecy` deps, использовать `Zeroizing<Vec<u8>>`, impl Drop

### 4. `crates/cli/src/main.rs:60,82` — Пароль в CLI args виден в ps aux
**Fix:** `rpassword::prompt_password()` для интерактивного ввода, `#[arg(long, env = "QALLET_PASSWORD")]` как fallback

### 5. `crates/core/src/provider/chains.rs:102-104` — panic на пустых rpc_urls
`primary_rpc()` делает `&self.rpc_urls[0]` без bounds check.
**Fix:** вернуть `Option<&str>` или `.first()`

### 6. `crates/core/src/provider/multi.rs:120-148` — Cross-chain total misleading
Сумма ETH с разных L2 — не fungible. `total_formatted` вводит в заблуждение.
**Fix:** документировать caveat или убрать total

### 7. `crates/txguard/src/simulator/mod.rs:129` — i128 cap для eth_change
`value.try_into().unwrap_or(i128::MAX)` — теряет precision для больших значений.
**Fix:** документировать лимит или использовать I256

### 8. `crates/core/src/keyring/local.rs:47` — Raw key bytes на стеке
`signer.credential().to_bytes()` создаёт копию key на стеке, не зануляется.
**Fix:** обернуть в `Zeroizing`, занулить после encrypt_key

---

## Consider (14)

1. **goplus.rs:43** — reqwest::Client без timeout. Добавить 10s timeout + 5s connect_timeout
2. **multi.rs:152** — Новый provider на каждый RPC call. Кэшировать в HashMap
3. **multi.rs:230-334** — Дупликация fetch_gas_fees/fetch_estimate_gas/fetch_nonce. Вынести helper `with_provider()`
4. **Cargo.toml** — Нет `overflow-checks = true` в `[profile.release]`
5. **Cargo.toml** — Нет `unsafe_code = "deny"` в workspace lints
6. **Cargo.toml** — Нет clippy restriction lints (unwrap_used, indexing_slicing, panic)
7. **txguard/Cargo.toml** — Heavy deps (revm, reqwest) без feature gates. Parser-only consumer тянет EVM
8. **router/mod.rs:153** — `expect()` в library code. Заменить на proper error
9. **ci.yml** — Все steps в одном job. Разбить на parallel jobs (fmt, clippy, test, docs)
10. **Нет deny.toml** — Нет cargo-deny для license/vulnerability audit
11. **keyring/local.rs:182** — `Argon2::default()` — задокументировать params, pinpoint version
12. **parser/mod.rs:56** — `ParseError::EmptyCalldata` нигде не используется (dead variant)
13. **cli/main.rs:87** — tokio runtime для sync commands (decode, analyze). Избыточно
14. **keyring/local.rs:25** — Нет custom Drop для LocalKeyring (zeroize on drop)

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

---

## Next steps

1. Fix must-fix #3, #4, #8 (keyring security) — приоритет
2. Fix must-fix #1, #2, #5 (correctness)
3. Add overflow-checks, unsafe_code deny, deny.toml
4. Implement wallet send (stashed, plan ready)
5. Push + verify CI
