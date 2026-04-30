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

**M4 re-evaluation (2026-04-30):** Re-evaluated under M4 trust-boundary activation, remains Phase 2 (not M4 blocker). Rust-only patch без end-to-end `Vec<u8>` redesign даёт нулевой security gain — JS heap / Hermes interner / React fiber state копии всё равно non-zeroized. Industry baseline (MetaMask Mobile, Trust Wallet, Rainbow) zeroize across RN bridge не делают. Mitigation в M4 — FLAG_SECURE на Activity (применено в `MainActivity.kt:onCreate`) — bounds the screen-capture leakage window независимо от heap residual.

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

## C4 [MEDIUM]. Metro bundler — implicit reliance на npm workspaces hoisting

Source chain:
- `mobile/metro.config.js:15-21`:
  ```js
  resolver: {
    nodeModulesPaths: [
      path.resolve(projectRoot, 'node_modules'),
      path.resolve(workspaceRoot, 'node_modules'),
    ],
    disableHierarchicalLookup: true,
  }
  ```
- `packages/react-native-rustok-bridge/src/generated/rustok_mobile_bindings.ts:7`:
  ```ts
  import nativeModule, { type ... } from 'uniffi-bindgen-react-native';
  ```
  → `nativeModule` — **runtime** import (default), не type-only.
- `packages/react-native-rustok-bridge/package.json:30-35` объявляет `uniffi-bindgen-react-native` в **`devDependencies`**, не в `peerDependencies` или `dependencies`.

С `disableHierarchicalLookup: true` Metro не walks up по директориям — резолвит **только** через `nodeModulesPaths`. Сейчас `uniffi-bindgen-react-native` резолвится исключительно потому, что npm workspaces поднимают его в `<root>/node_modules`. Это hidden coupling: при любом изменении hoisting behavior (version conflict от другого workspace, manual `nohoist`, npm config change) пакет окажется в `packages/react-native-rustok-bridge/node_modules/`, которого в `nodeModulesPaths` нет — bundle упадёт с `Unable to resolve module uniffi-bindgen-react-native`.

Found in `/typescript-review` на M4 metro.config.js change (2026-04-30).

### Phase 2 fix (root cause в bridge package)

```jsonc
// packages/react-native-rustok-bridge/package.json
"peerDependencies": {
  "react": "*",
  "react-native": "*",
  "uniffi-bindgen-react-native": "0.31.0-2"
},
"devDependencies": {
  // оставить здесь же — peerDeps + devDeps для разработки самого пакета
  "uniffi-bindgen-react-native": "0.31.0-2",
  ...
}
```
И добавить `uniffi-bindgen-react-native` в `mobile/package.json` `dependencies`.

Альтернатива (workaround в metro.config.js, хуже — связывает config с layout):
```js
nodeModulesPaths: [
  path.resolve(projectRoot, 'node_modules'),
  path.resolve(workspaceRoot, 'node_modules'),
  path.resolve(workspaceRoot, 'packages/react-native-rustok-bridge/node_modules'),
],
```

Decision required at: Phase 2 start, при добавлении остальных command bindings (могут принести свои runtime-импорты с тем же hoisting-риском).

---

## Phase 2 entry condition

Все C1-C4 имеют документированные решения (не обязательно implementation).

---

# Phase 4-5 Production Polish

> **Контекст:** Items найденные в M3 review, которые НЕ блокируют POC / Phase 2 / Phase 3, но требуются перед public release (Phase 4-5 prep). Низкий приоритет, но трекать.

## P1. `peerDependencies: "*"` — tighten перед npm publish или мульти-app консамерами

Source: `packages/react-native-rustok-bridge/package.json:23-26`
```json
"peerDependencies": {
  "react": "*",
  "react-native": "*"
}
```

Проблема: `"*"` = «любая версия react/react-native подойдёт», но bridge компилируется против конкретного RN ABI (TurboModule layout, Hermes runtime, codegen output). Установка в проект с RN 0.74 (старая ABI) silently сломается в runtime — без warning от npm/yarn о mismatch.

Текущий monorepo single-consumer scenario (`mobile/` пинит `react-native: 0.85.2`) контролируем — никаких проблем. Но:
- Если когда-либо публикуем bridge в npm registry
- Если используем bridge в другом app в этом monorepo с другой RN версией

→ required tighten.

### Phase 4-5 fix

```json
"peerDependencies": {
  "react": ">=19.0.0 <20.0.0",
  "react-native": ">=0.85.0 <0.86.0"
}
```

Range отражает наш ABI-compatibility window. При major bump RN (0.86+) — обновлять explicitly с testing.

---

## P2. Add `armeabi-v7a` Android target

Source: `packages/react-native-rustok-bridge/ubrn.config.yaml`
```yaml
android:
  targets: [arm64-v8a, x86_64]
```

Проблема: текущий list покрывает modern ARM64 phones + x86_64 emulator, но игнорирует **`armeabi-v7a`** (32-bit ARM). По данным Android distribution dashboard, 32-bit устройства составляют ~3-5% активных Android — преимущественно в developing markets и budget tier.

Trade-off для M3-итерации: сейчас минимум targets ускоряет cross-compile cycle (~12 sec на target вместо ~18 sec для трёх). Production prep — добавить.

### Phase 4-5 fix

```yaml
android:
  targets: [arm64-v8a, armeabi-v7a, x86_64]
  apiLevel: 24
```

Опционально `i686-linux-android` (32-bit emulator) — но 32-bit emulators практически мёртвы, не нужно.

CI implication: cross-compile time увеличится на ~20-30%. APK size увеличится на ~12 MB (один extra .so per ABI). Trade-off оправдан для production user coverage.

---

**Конец документа.**
