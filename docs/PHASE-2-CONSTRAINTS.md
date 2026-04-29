# Phase 2 Architectural Constraints

> **Контекст:** В M3 review (`/rust-review` post-merge audit) выявлены архитектурные элементы, которые НЕ блокируют M3 (POC scope), но требуют решения при старте Phase 2 (Core API extraction — все 22 команды rustok-core через uniffi).
>
> **Создан:** 2026-04-29 (M3 close)
> **Связь:** `docs/POC-FOUNDATION.md` §1.1, §10.3; rust-review session results

---

## C1 [HIGH]. Mnemonic / secrets через FFI теряют Zeroize property

Source chain:
- `crates/core/src/keyring/local.rs:101`:
  ```rust
  pub fn random_mnemonic_phrase() -> Result<Zeroizing<String>, KeyringError>
  ```
  → core правильно использует Zeroizing wrapper.
- `crates/rustok-mobile-bindings/src/lib.rs:31`:
  ```rust
  .map(|phrase| phrase.to_string())
  ```
  → `.to_string()` через Display impl создаёт **new non-zeroized heap allocation**. Original `Zeroizing<String>` dropped (его bytes zeroed корректно) — но новая `String`, которую возвращаем через FFI, **non-zeroized**.

Severity per `C:/Claude/codex/rust/review/checklist.md` §3.6 = **HIGH** (mnemonic = privkey-equivalent для wallet).

Why the wrapper does this: uniffi 0.31 не имеет registered conversion для `Zeroizing<String>` over FFI; `Result<Zeroizing<String>, _>` не сериализуется. Только `String` / `Vec<u8>` / primitive types зарегистрированы для FFI transit.

### Phase 2 options (выбор отложен)

**A.** Redesign API — не возвращать raw mnemonic. Encrypt-at-rest immediately на password от пользователя, return `wallet_id`. Семантически правильно для wallet UX (mnemonic shown один раз при onboarding, потом forgotten).

**B.** Custom uniffi type wrapper с `Zeroize` impl (требует upstream contribution в uniffi или fork uniffi-bindgen-react-native).

**C.** Принять risk — документировать что Rust-side ephemeral String зануляется не сразу. Worst option для wallet semantics: мобильные процессы могут жить в suspended state часами, OS memory protection не гарантирует очистку JNI heap; memory dumps в crash reports могут содержать sensitive bytes.

Decision required at: Phase 2 start, до того как добавлять остальные команды, которые также handle secrets (private keys, signed payloads).

---

## C2 [MEDIUM]. `BindingsError.message: String` — opaque error propagation

Source: `crates/rustok-mobile-bindings/src/lib.rs:33`:
```rust
.map_err(|e| BindingsError::MnemonicGeneration {
    message: e.to_string(),
})
```

Two problems:
1. **Opaque structure** — JS/TS layer получает unstructured message, не может pattern-match по типу ошибки.
2. **Risk утечки sensitive context** — если в Phase 2 underlying error содержит partial keystore paths / derivation paths / hex entropy fragments, всё попадает в `message` и отдаётся через FFI. Per `checklist.md` §3.1: «секреты в error типах — ошибки часто логируются и пробрасываются».

В M3 риск низкий (entropy-unavailable error не содержит секретов), но pattern закрепится для всех 22 команд → MEDIUM становится HIGH.

### Phase 2 redesign target

```rust
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BindingsError {
    #[error("entropy source unavailable")]
    EntropyUnavailable,
    #[error("invalid mnemonic length: expected 12, got {got}")]
    InvalidMnemonicLength { got: u32 },
    #[error("internal error")]
    Internal,  // для unexpected; log details на Rust side через tracing::error!, не отдавать через FFI
    // ... per command
}
```

Принципы:
- Structured variants (no opaque `message: String`)
- Log details через `tracing::error!` на Rust side
- FFI отдаёт только enum tag + minimal numeric fields, без sensitive context

---

## C3 [MEDIUM]. `BindingsError` enum scaling — single-variant → 30+

Текущий enum (`crates/rustok-mobile-bindings/src/lib.rs:11-18`) содержит один variant `MnemonicGeneration`. Когда экспортируем 22 команды rustok-core через uniffi, enum либо разбухнет до 30+ variants, либо потребует таксономию.

### Phase 2 proposed taxonomy (предварительно)

```rust
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BindingsError {
    #[error(transparent)]
    Wallet(#[from] WalletError),

    #[error(transparent)]
    Rpc(#[from] RpcError),

    #[error(transparent)]
    TxGuard(#[from] TxGuardError),

    #[error("FFI encoding error: {context}")]
    Encoding { context: String },
}
```

Где `WalletError`, `RpcError`, `TxGuardError` — re-exported из `rustok-core` / `txguard` через uniffi (если uniffi позволяет nested errors over FFI — нужен verify в Phase 2 prep).

Decision required at: Phase 2 start, синхронно с C2.

---

## Phase 2 entry condition

Все C1-C3 имеют документированные решения (не обязательно implementation).

---

**Конец документа.**
