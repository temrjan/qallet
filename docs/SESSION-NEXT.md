# Следующая сессия — Фикс unlock-архитектуры + решение по seed phrase

## Статус (2026-04-18)

- **103 теста зелёные, CI 5/5 green**
- **Android:** APK собирается (60 MB debug), Create Wallet работает, баланс грузится
- **Решённые ранее баги:** BUG-1 (Unlock button, CSS visibility), BUG-2 (balance race, 800ms retry) — коммит 527fa7d + bc2cb29

## Нерешённые проблемы

### Архитектурные (приоритет 1 — эта сессия)

| # | Проблема | Файл | Статус |
|---|----------|------|--------|
| A1 | TabBar виден на ВСЕХ роутах, включая Unlock и Create | `app/src/src/app.rs:23` | Ломает security/UX unlock flow |
| A2 | Home дублирует Unlock-экран (inline unlock-prompt) | `app/src/src/pages/home.rs:75-90` | Путаница "первый экран" |
| A3 | `navigate_to("/")` не работает на Android WebView | `app/src/src/bridge.rs:35` | 4 вызова в `unlock.rs`. См. `memory/rustok-android-navigate.md` |

### Функциональные (приоритет 2 — отдельная фаза)

| # | Проблема | Масштаб |
|---|----------|---------|
| F1 | **Нет seed phrase / recovery phrase** | Production crypto wallet без recovery — потеря пароля = потеря средств |
| F2 | Legacy keystore wallets не могут быть мигрированы в seed | Математически невозможно: ключ сгенерён через `PrivateKeySigner::random()` (`keyring/local.rs:56`), не derived |

---

## Часть 1 — Фикс архитектуры unlock (эта сессия)

**Референс UX:** MyTonWallet — tab bar ТОЛЬКО в authenticated state. Pre-auth flow (welcome / create / unlock) без таб бара.

### Перед кодом — проверки

1. **context7 `leptos_router` 0.7:**
   - Точный API `use_navigate()` — работает ли из `spawn_local`?
   - Есть ли `<Redirect />` компонент в 0.7?
   - Как навигировать после async операции?

2. **grep `app/src-tauri/src/commands.rs`:**
   - Есть ли `wallet_exists` / `is_wallet_initialized`?
   - Если нет — нужна новая Tauri команда для различения `Uninit` vs `Locked`

### Правки

**Правка 1. `app.rs` — условный TabBar**

В `App()` завести signal `wallet_state: { Uninit | Locked | Unlocked }`, инициализируемый на старте через существующий `is_wallet_unlocked` + (возможно новый) `wallet_exists`. Рендерить `<TabBar />` ТОЛЬКО при `Unlocked`.

**Правка 2. `home.rs` — убрать дубль**

Удалить ветку `Some(false) | None` (строки 75-90) с inline unlock-prompt. При not unlocked → `Redirect` на `/unlock` (или `/wallet/create` при `Uninit`). Home остаётся чисто authenticated экраном.

**Правка 3. `unlock.rs` + `bridge.rs` — починить навигацию**

Заменить `navigate_to("/")` (4 места: строки 78, 118, 154, 162) на `use_navigate()` из `leptos_router::hooks`.

**Fallback если `use_navigate` не работает в `spawn_local` на Android:**
менять root signal `wallet_state` из unlock, `<Redirect/>` в layout уведёт с `/unlock` сам.

Если сработает — `navigate_to` из `bridge.rs` удалить.

### Критерий успеха

- Первый экран при запуске без кошелька → `/wallet/create`, без таб бара
- Первый экран при запуске с кошельком → `/unlock`, без таб бара
- После успешного unlock → `/`, таб бар появляется
- На Android переход после unlock работает без `querySelector` хаков

---

## Часть 2 — Seed phrase support (ОТДЕЛЬНАЯ фаза, НЕ в этой сессии)

### Reality check

- Текущая схема: `PrivateKeySigner::random()` → Argon2id (19 MiB, 2 iter) → AES-256-GCM → blob `salt(16) || nonce(12) || ciphertext(32+16)` (см. `crates/core/src/keyring/local.rs:1-7`)
- Recovery = только пароль. Забыл → deposit потерян навсегда
- Legacy кошельки нельзя мигрировать в seed — ключ рандомный, обратной операции нет

### План v2 (черновик — уточнить после context7 + чтения keyring/mod.rs)

**Backend:**
- Зависимость: `bip39` crate + включить feature `mnemonic` в `alloy-signer-local = { workspace = true, features = ["mnemonic"] }`
- Шифрование seed = та же Argon2id + AES-GCM схема, что сейчас для private key. Новый тип `EncryptedSeed` (не keystore v3)
- Новые Tauri commands: `create_wallet_with_mnemonic(password)`, `import_wallet_from_mnemonic(words, password)`, `reveal_mnemonic(password)`
- Default derivation path: MetaMask-совместимый `m/44'/60'/0'/0/0` (проверить через context7)

**Legacy обработка:**
- Миграция в seed НЕ предлагается (невозможна)
- Settings → "Export Private Key" — для ручного импорта в любой seed-кошелёк (MetaMask/Rabby)
- Side-by-side: старый keystore-only кошелёк работает до удаления пользователем

**UI минимум:**
- Create: show 12 words → confirm 3 random words → passcode
- Import: textarea на 12 слов + BIP39 checksum validation
- Settings: "Show Recovery Phrase" с запросом пароля

### Что НЕ делаем (лишнее из референса MyTonWallet)

- TOS-чекбокс "Я соглашаюсь использовать ответственно"
- 3-checkbox backup intro с подтверждениями
- "Всё готово!" success screen
- Biometric отдельным экраном (уже встроен в UnlockPage)
- 24 слова (достаточно 12 — MetaMask стандарт, 128 бит энтропии)
- Optional BIP39 passphrase (25-е слово) — v2
- Мульти-derivation paths (Ledger/Trust Wallet) — v2
- Email recovery (требует custodial — не вяжется с zero-trust Rustok)
- Social recovery / MPC — вне scope Phase 3

---

## Контекст для старта

```bash
cd /Users/avangard/Workspace/projects/rustok
cargo test                    # ожидаем 103 зелёных
git log --oneline -10

# Android
source ~/.zshrc               # ANDROID_HOME, JAVA_HOME, NDK_HOME
adb devices                   # emulator-5554 (Pixel 8)
adb logcat --pid=$(adb shell pidof com.rustok.app)

# Сборка
cd app
cargo tauri android build --apk --debug
```

### Ключевые файлы

| Файл | Что там |
|------|---------|
| `app/src/src/app.rs:23` | TabBar в корне Router (фикс Правки 1) |
| `app/src/src/pages/home.rs:75-90` | Дубль unlock-экрана (фикс Правки 2) |
| `app/src/src/pages/unlock.rs:78,118,154,162` | 4 вызова `navigate_to("/")` (фикс Правки 3) |
| `app/src/src/bridge.rs:35` | `navigate_to()` — не работает на Android |
| `app/src/src/pages/wallet.rs` | Create flow — не трогаем в Части 1 |
| `crates/core/src/keyring/local.rs` | Argon2id+AES-GCM схема — переиспользовать для seed |
| `crates/core/src/keyring/mod.rs` | Прочитать ДО планирования seed-фазы (есть `export_keystore_json`) |
| `app/src-tauri/src/commands.rs` | 15 Tauri commands, сюда добавлять seed-related |

### Эмулятор / devices

- **Android AVD:** Pixel_8, Android 15 (API 35), arm64-v8a
- **iOS:** iPhone 17 Pro Simulator (`0x25B280...1CE91`, ~0.049 ETH Sepolia)
- **Android wallet:** `0x60EeF04...AECe7`

### Правила

- **Не смешивать Часть 1 и Часть 2 в одном PR.** Таб бар и seed — разные масштабы изменений
- **CI должен остаться зелёным после каждого коммита** (103 теста minimum)
- **Перед seed-фазой** — обновить этот документ с результатами context7 по alloy mnemonic API
