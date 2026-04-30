# Phase 2 — Session Handoff

> **Дата:** 2026-04-30
> **Сессия:** опытная — Group A + первый commit Group B закрыты
> **Branch:** `feat/phase2-core-api` (pushed up to commit 3)
> **Phase 2 progress:** **3/11 commits done (27%)**

---

## Что сделано

### Commits (3 закрыты, на remote)

| # | SHA | Title | Group | Net diff |
|---|---|---|---|---|
| 1 | `bd7174d` | `chore(bridge): C4 hoisting fix — peerDependencies for uniffi-bindgen-react-native` | A | +9/−5 |
| 2 | `e232c20` | `feat(bindings): adopt structured BindingsError taxonomy (C2/C3)` | A | +121/−9 |
| 3 | `e6cd6a0` | `refactor(core): wallet lifecycle service (C1-A: wallet_id + encrypt-at-rest + reveal-once + spawn_blocking)` | B (1/2) | +1054/−378 |

**Branch state:**
```
e6cd6a0 commit 3                                       ← unpushed before push;
                                                          NOW: pushed (e232c20..e6cd6a0)
e232c20 commit 2                                       ← pushed
bd7174d commit 1                                       ← pushed
9a59bd7 docs: add reviewer constitution               ← on main
b933e42 Merge PR #12 (swap-integration-plan)          ← on main
```

`origin/feat/phase2-core-api` синхронизирован с `feat/phase2-core-api`. Group A полностью closed (foundation: build hygiene + error taxonomy). Group B 1/2 — wallet lifecycle done, send/balance services next.

### Reviews пройдены

- Commit 1: `/typescript-review` — APPROVED WITH NITS (1 deferrable suggestion).
- Commit 2: `/security-review` GO + `/rust-review` APPROVED-WITH-NITS (2 nits applied in-commit: thiserror::Error на sub-kinds, ASCII em-dash → colon).
- Commit 3: `/security-review` GO (9 findings → 0 at confidence ≥8; 2 fixes applied: Vuln 6 has_wallet pure, Vuln 2 encrypt_with_password_blocking) + `/rust-review` APPROVED-WITH-NITS (4 LOW nits, 3 applied in-commit).

### Spike 0 (pre-Phase 2)

`uniffi 0.31.0-2` async fn export verification. **Outcome: GO (partial via codegen evidence)**:
- `cargo check` accepts `async fn` через `#[uniffi::export]` без feature flags.
- ubrn produces `Promise<T>` + `AbortSignal` + `rust_future_poll_*` machinery в TS bindings.
- **Runtime test deferred** (no device connected at the time). Will be validated на first real async-using commit (commit 6 generic tx или commit 7 swap module).

---

## Что дальше — commit 4 scope

**Title:** `refactor(core): extract send/preview/balance services + chain_id getter`

**Estimate:** ~3-4h pure work + reviews ≈ 5-6h realistic.

**Files affected (предположительно):**

NEW:
- `crates/core/src/send_service.rs` (or extend existing `crates/core/src/send.rs`) — wraps `preview_send`, `send_eth` from current commands.rs into service-layer functions taking `&WalletService` + `&MultiProvider`.

MODIFIED:
- `app/src-tauri/src/commands.rs` — `preview_send`, `send_eth`, `get_wallet_balance`, `get_balance` (4 commands) refactored to delegate via the new send service. Currently they extract address/signer from `WalletService` and call `rustok_core::send::preview_send`/`execute_send` directly — refactor will move that orchestration into core.
- `crates/core/src/provider/multi.rs` (or wherever current chain tracking lives) — add `current_chain_id() -> u64` getter. Audit needed: existing `MultiProvider` chains list — single primary chain или multi-chain selector?
- `app/src-tauri/src/commands.rs` — add new `get_chain_id` command (NEW — 23rd command для UI network badge per NATIVE-MIGRATION-PLAN.md Phase 2 list).

**Goal:**
- Decouple Tauri commands.rs from direct `rustok_core::send` invocations — desktop becomes thin shim parallel к будущему mobile bindings shim (commit 9).
- Establish chain selection API surface for future swap module (commit 7) которое нуждается в chain context.

**NOT в этом commit:**
- Signing primitives (commit 5 — sign_message EIP-191 / sign_typed_data EIP-712)
- Generic tx module (commit 6 — preview_transaction / sign_and_send_transaction async)
- Network selector full UI (Phase 7)

**Skills для commit 4:**
- `/codex` ✓ already loaded в session (architecture + pipeline)
- `/rust` ✓ already loaded (CORE.md)
- `/check` — после составления плана commit 4
- **`/rust-review`** — обязательно перед коммитом (Rust changes)
- **`/security-review`** — рекомендуется (send/balance касается keyring access чтобы получить signer); operator-discretion если нет new sensitive surface (signer access pattern unchanged from commit 3)

---

## Ключевые технические решения этой сессии

### C1-A wallet redesign (commit 3 — самое важное)

**Pattern**: encrypt-at-rest + wallet_id + reveal-once.

- `WalletService::create_wallet(password) → WalletId` — раз mnemonic не возвращается. Mnemonic encrypted-at-rest в `<data_dir>/.onboarding_mnemonic.encrypted` (Argon2id+AES-256-GCM, тот же scheme что keystore). Same password used для both files.
- `reveal_mnemonic_for_onboarding(wallet_id, password) → Zeroizing<String>` — atomic read+decrypt+remove-file. Removes only on successful decrypt (preserved across wrong-password retries). После remove → `MnemonicAlreadyRevealed` error.
- `WalletId = String` (EIP-55 mixed-case Address hex). Type alias чтобы не усложнять FFI marshalling — newtype в commit 9.
- Stale cleanup: ТОЛЬКО в `unlock` (post-Vuln-6 fix), не в `has_wallet` (которая теперь pure query). Eliminates damaging race против concurrent `create_wallet`/reveal.

### WalletService design

- `pub struct WalletService { data_dir: PathBuf, state: tokio::sync::Mutex<Option<UnlockedState>> }`.
- Constructed via `.setup()` callback в `lib.rs::run` (где AppHandle finally available для `path().app_data_dir()`). Registered как `Arc<WalletService>` через `app.manage(...)` параллельно к старому `AppState`.
- Tauri commands extract via `State<'_, Arc<WalletService>>` (multi-state extraction рядом с `State<'_, AppState>`). Idiomatic Tauri pattern.
- Single-file module (`crates/core/src/wallet.rs` ~870 lines). Не split на mod/service/storage/errors — single backend, не нужно abstraction yet (deferred к Phase 4+ при mobile-specific storage).

### spawn_blocking discipline (4/4 Argon2id paths)

`tokio::task::spawn_blocking` обязательно для всех Argon2id-touching путей:
1. `from_encrypted_blocking` — `unlock`
2. `from_mnemonic_blocking` — `create_wallet`, `import_from_mnemonic`
3. `decrypt_blocking` — `reveal_mnemonic_for_onboarding`
4. `encrypt_with_password_blocking` — `create_wallet` onboarding mnemonic encrypt (added в Vuln 2 fix)

Pattern: `move` ownership of `Zeroizing<String>` password and `Vec<u8>`/`Zeroizing<Vec<u8>>` payload into closure. After return → drop zeros password automatically.

### Error taxonomy (commit 2 — C2/C3)

```rust
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BindingsError {
    Wallet { kind: WalletErrorKind },
    Send { kind: SendErrorKind },
    Rpc { kind: RpcErrorKind },
    TxGuard { kind: TxGuardErrorKind },
    Encoding { kind: EncodingErrorKind },
    Swap { kind: SwapErrorKind },
    Internal,  // sensitive context never crosses FFI; tracing::error! Rust-side
}
```

Each `*Kind` sub-enum имеет `thiserror::Error` derive + per-variant `#[error("...")]` (Path A enforcement — prevents future field-bearing variants from leaking through Debug formatting). Currently only `WalletErrorKind::MnemonicGeneration` populated; others имеют `Reserved` placeholder, populated commit-by-commit в 3-9.

### encrypt_key/decrypt_key visibility change

`crates/core/src/keyring/local.rs` — `fn encrypt_key/decrypt_key` private → `pub(crate)` + re-export в `keyring/mod.rs`. Crate-internal only. Allows wallet service reuse Argon2id+AES-GCM scheme без duplicating crypto code. NOT public API.

### Cargo.toml changes

`crates/core/Cargo.toml`:
- `tokio = { workspace = true, features = ["sync"] }` — additive (sync feature не в workspace base).
- `qrcode = { version = "0.14", default-features = false, features = ["svg"] }` — moved from app/src-tauri (the actual user is now WalletService::current_qr_svg).
- `tempfile = "3"` — dev-dep only.

`packages/react-native-rustok-bridge/package.json` + `mobile/package.json` (commit 1):
- `uniffi-bindgen-react-native: 0.31.0-2` — moved from bridge devDeps к peerDeps + добавлено в mobile dependencies. Closes C4 hoisting coupling.

---

## Файлы затронутые и незатронутые

### Затронутые (commits 1-3)

**Cargo:**
- `Cargo.lock` (auto-regenerated)
- `crates/core/Cargo.toml` (deps)
- `mobile/package.json`, `packages/react-native-rustok-bridge/package.json`, `package-lock.json` (commit 1 only)

**Rust core:**
- `crates/core/src/wallet.rs` (NEW, ~870 lines)
- `crates/core/src/lib.rs` (`pub mod wallet;`)
- `crates/core/src/keyring/mod.rs` (re-exports)
- `crates/core/src/keyring/local.rs` (visibility + docstring)
- `crates/rustok-mobile-bindings/src/lib.rs` (commit 2 — error taxonomy)

**Tauri desktop:**
- `app/src-tauri/src/commands.rs` (refactored, −329 lines)
- `app/src-tauri/src/lib.rs` (`.setup()` callback)

### НЕ затронутые (планомерно — commit 4-11 будут править)

- `crates/core/src/send.rs` — будет refactored в commit 4
- `crates/core/src/provider/multi.rs` — chain_id getter в commit 4
- `crates/txguard/` — swap rules в commit 8
- `crates/rustok-mobile-bindings/src/lib.rs` — FFI exposure в commit 9 (29 commands + mirror types)
- `mobile/App.tsx` — adapt to new BindingsError shape в commit 9
- `mobile/src/screens/_DevHarness.tsx` — NEW в commit 10
- `app/src-tauri/src/biometric_storage.rs` — Tauri-specific, not in scope
- `crates/types/` — DTOs (`WalletInfo`, etc.) preserved, mirror types в bindings crate в commit 9

---

## Какие skills загружены, какие нужны для commit 4

### Загружены в этой сессии (можно reload в новой)

- `/codex` — architecture.md + pipeline.md (project-wide)
- `/rust` — CORE.md (general Rust standards)
- `/typescript` — CORE.md (для commit 1 package.json edits)

### Для commit 4 (Rust send/preview/balance services)

- `/codex` ✓ должен быть загружен (always)
- `/rust` ✓ нужен (Rust изменения)
- `/check` — после составления плана commit 4
- `/rust-review` — обязательно перед коммитом
- `/security-review` — operator-discretion (send/balance touches signer но pattern unchanged from commit 3)

---

## Quick start commands для новой сессии

```bash
# Путь ASCII-only (AGP не поддерживает кириллицу на Windows)
cd C:/Claude/projects/rustok

# Sanity check workspace state
git status
git log --oneline -5
git branch --show-current  # должно быть feat/phase2-core-api

# Pull latest (если работа в нескольких сессиях)
git pull --ff-only

# Verify workspace tests green
cargo test --workspace  # ожидается 130 passed

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
3. `docs/PHASE-2-CONSTRAINTS.md` — C1-C4 (C1-C4 closed; track resolution через memory)
4. `docs/SWAP-INTEGRATION-PLAN.md` — для commits 5-8 reference
5. `docs/NATIVE-MIGRATION-PLAN.md` — Phase 2 plan (29 commands list)
6. `app/src-tauri/src/commands.rs` (current state) — для commit 4 audit (preview_send, send_eth, get_wallet_balance, get_balance functions to refactor)
7. `crates/core/src/send.rs` — для understanding existing send/execute_send entry points
8. `crates/core/src/provider/multi.rs` — для understanding chain tracking (need `current_chain_id` getter)
9. Memory: `MEMORY.md` index → `project_rustok_status.md`, `feedback_push_policy.md`, `feedback_review_skills_trigger.md`

---

## Open risks / blockers

### Active risks

1. **uniffi async on real device** — Spike 0 partial GO via codegen evidence. Runtime validation deferred to first async-using commit (likely commit 6 generic tx). Risk: real-device test может выявить runtime issue, потребует blocking-on-runtime adapter pattern.

2. **alloy types FFI marshalling boilerplate** — `U256`, `Address`, `Bytes` не uniffi-derivable. Mirror types в bindings crate потребуются в commit 9. Estimate +1-2h на commit 9.

3. **C1-A breaks Phase 1 mobile/App.tsx demo** — current Phase 1 M4 demo uses `generateMnemonic()` returning raw String. After commit 9 BindingsError shape changes + future commits replace direct path. Per operator: deprecated path сохранён до Phase 4. Mobile demo продолжит работать через legacy `generate_mnemonic` export.

### Deferred items (track в PHASE-2-CONSTRAINTS.md или memory)

1. **Vuln 1 (atomic write)** — `std::fs::write` не атомичен; mid-write crash может leave corrupt keystore. Phase 4 hardening: tempfile + rename pattern. Track как deferred item (security-review confidence 2 после FP filter — design choice, не concrete vuln).
2. **Vuln 9 (signer clone lifetime в send_eth)** — pre-existing pattern, не regression. Phase 4 polish: closure-scoped signer to bound memory window.
3. **Concurrent regression test для has_wallet** — Vuln 6 fix is structural (race impossible by construction), но explicit test optional. Defer or skip.
4. **devDependencies duplicate в bridge package.json** (commit 1 nit) — Phase 4-5 hygiene.
5. **`generate_mnemonic_phrase()` deprecated path** — known C1 violation для legacy desktop UI. Removed Phase 4.

### Phase 2 entry conditions (still satisfied)

- ✅ M1+M2+M3+M4 closed (PR #10, #11 merged)
- ✅ C1-C4 decisions approved by operator (locked)
- ✅ SWAP-INTEGRATION-PLAN.md approved
- ✅ Branch `feat/phase2-core-api` from main `9a59bd7`
- ✅ `cargo test --workspace` green (130 tests)

### What blocks commit 4 — nothing

Audit step required (read `crates/core/src/send.rs` + `crates/core/src/provider/multi.rs` to understand current chain tracking) but no external blocker. ~30 min audit, then plan + /check + implementation.

---

## Memory state at handoff

- `project_rustok_status.md` — needs update to «Phase 2 in progress 3/11 commits done; Group A complete; Group B 1/2; commit 4 next». Update в начале новой сессии.
- `MEMORY.md` index — line about Rustok status уже says «Phase 2 in progress 2/11 commits done (Group A closed)». Needs bump до 3/11.
- `feedback_push_policy.md` — действует, активно используется (push triggered after commit 3 как backup для большой работы).
- `feedback_review_skills_trigger.md` — действует, все commits проходят соответствующие reviews.
- `feedback_verify_rule.md` — действует, все claims о коде verified через Read/Grep.

---

**Конец handoff. Новая сессия может начинать с `git status` + read этого документа + commit 4 audit.**
