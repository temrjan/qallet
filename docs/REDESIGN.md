# Редизайн — Rustok Wallet UI

> Документ для новых сессий. Читай ПОЛНОСТЬЮ перед началом работы.
> Обновляй раздел «Прогресс» после каждой сессии.

---

## 1. Контекст

**Цель:** Заменить текущий тёмно-amber UI на новый дизайн из репо
`rust-design` (navy + periwinkle палитра, 6-digit PIN, больше экранов).

**Репо нового дизайна:** https://github.com/temrjan/rust-design
— standalone Leptos 0.7 CSR prototype. Рендерится в браузере через Trunk.
Tauri НЕ подключён. Это дизайн-референс + готовый Rust/Leptos код.

**Основной репо:** `/Users/avangard/Workspace/projects/rustok/`
— production app. Сюда переносим код из rust-design с Tauri-wiring.

---

## 2. Воркфлоу (строго соблюдать)

```
INTAKE → PLAN → /check → DEVELOP → /rust-review → COMMIT → PUSH → CI
```

### Режимы

**LIGHT** — конфиг, 1 файл, косметика:
```
Изучи → Сделай → /check → diff → Коммит → Пуш → CI
```

**FULL** — фичи, рефакторинг, multi-file (текущая задача — FULL):
```
Изучи → /codex → План → /check → Исправь → /rust → Реализуй → /rust-review → diff → Коммит → Пуш → CI
```

### Скиллы и когда запускать

| Скилл | Когда | Обязательность |
|-------|-------|----------------|
| `/codex` | Перед написанием кода (загружает стандарты стека) | Обязателен для FULL |
| `/rust` | Перед написанием Rust-кода (загружает CORE + web/leptos) | Обязателен |
| `/rust-review` | После написания кода, перед коммитом | Обязателен |
| `/check` | После каждого плана — критикуй собственное решение | Обязателен |

**Домен для `/rust`:** `web/leptos` (Leptos 0.7, Tauri bridge, WASM).
Если затрагиваем keyring/crypto — добавлять `security/crypto`.

---

## 3. Архитектура интеграции

### Стратегия (выверена в сессии 2026-04-23)

**НЕ** заменять весь app.rs на state machine сразу.
`home.rs` и `settings.rs` используют `use_navigate()` — они требуют `<Router>`.
Убрать Router без миграции этих страниц = runtime panic.

**Текущий подход — инкрементальная замена:**
1. Заменить только страницы онбординга: `wallet.rs`, `unlock.rs`, `restore.rs`
2. Оставить `leptos_router` и `<Router>` нетронутыми
3. Полная миграция на state machine — следующий этап (после редизайна всех страниц)

### Структура файлов

```
rustok/app/src/src/
├── main.rs          — точка входа, объявляет mod components, mod pages
├── app.rs           — App компонент, Router, WalletState context, startup probe
├── bridge.rs        — tauri_invoke<A, R> helper (НЕ трогать)
├── components/
│   ├── mod.rs       — pub use passcode::{Keypad, PasscodeDots, PASSCODE_LENGTH}
│   └── passcode.rs  — ✅ ГОТОВО: Keypad + PasscodeDots + константы
└── pages/
    ├── mod.rs       — pub mod ... (добавить новые при необходимости)
    ├── unlock.rs    — ✅ ГОТОВО: PIN keypad, auto-unlock, biometric
    ├── restore.rs   — 🔄 TODO: phrase input + PIN setup wizard
    ├── wallet.rs    — 🔄 TODO: PIN wizard (SetPIN→Confirm→Phrase→Quiz→Backup)
    ├── home.rs      — ⏳ НЕ ТРОГАТЬ (пока не будем мигрировать на state machine)
    ├── settings.rs  — ⏳ НЕ ТРОГАТЬ
    ├── activity.rs  — ⏳ НЕ ТРОГАТЬ
    ├── send.rs      — ⏳ НЕ ТРОГАТЬ
    ├── receive.rs   — ⏳ НЕ ТРОГАТЬ
    └── analyze.rs   — ⏳ НЕ ТРОГАТЬ
```

### Дизайн-референс (rust-design)

```
src/screens/onboarding/
├── splash.rs          — Splash screen
├── welcome.rs         — Welcome: Create / Restore CTA
├── passcode.rs        — SetPasscode screen (uses components/passcode.rs)
├── confirm_passcode.rs — Confirm PIN (shake on mismatch)
├── create_reveal.rs   — Show blurred seed phrase → Tap to reveal
├── create_verify.rs   — 3-word quiz (positions [2, 6, 10])
├── restore.rs         — Phrase textarea + Private Key tab
└── mod.rs
src/screens/dark/
├── home.rs            — 3 variants: Base/Chart/Tokens
├── activity.rs        — Day-grouped tx list
├── send.rs            — Send form
├── receive.rs         — QR code screen
├── settings.rs        — Grouped sections + toggles
└── txguard.rs         — Transaction safety checker
src/app.rs             — State machine роутер (референс для будущей полной миграции)
```

---

## 4. Технические детали

### Новая палитра (дизайн-токены)

| Токен | Значение | Применение |
|-------|----------|------------|
| `BRAND` | `#0A1123` | bg тёмных экранов, текст на светлых |
| `SURFACE_ALT` | `#F6F7FB` | фон светлых экранов, кнопки keypad |
| `ACCENT` | `#8387C3` | periwinkle, активные элементы |
| `DANGER` | `#E06B6B` | ошибки |
| `DANGER_BG` | `rgba(224,107,107,0.12)` | фон ошибки |
| `SUCCESS` | `#4AB37B` | подтверждение |
| `TEXT_MUTED` | `#959BB5` | вторичный текст |
| `FONT` | Roboto, SF Pro, system-ui | основной шрифт |

В Rust-коде токены инлайним прямо в файлах (не выносим в отдельный tokens.rs —
архитектурное решение сессии, чтобы не тащить всю систему токенов).

### PIN vs Password

- `PASSCODE_LENGTH = 6` (компонент `components/passcode.rs`)
- Argon2id в бекенде работает с любой длиной → `validate_password` убрана из `import_wallet_from_mnemonic` (коммит этой сессии)
- Security note: 6-digit PIN = 10^6 комбинаций; при Argon2id default params (19MB, 2 iter) ~17 минут exhaustive brute-force если атакующий получил .json keystore. Known limitation, аналогично MetaMask Mobile.

### Tauri Bridge

Паттерн в rustok — `bridge::tauri_invoke<A, R>`:
```rust
use crate::bridge::tauri_invoke;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ImportArgs { phrase: String, password: String }

let result = tauri_invoke::<_, WalletInfo>("import_wallet_from_mnemonic",
    &ImportArgs { phrase, password: pin }).await;
```

Все Tauri команды уже зарегистрированы в `app/src-tauri/src/lib.rs`.

### Зарегистрированные команды (для онбординга)

| Команда | Что делает |
|---------|-----------|
| `has_wallet` | → `bool` — есть ли keystore файл |
| `is_wallet_unlocked` | → `bool` — кошелёк в памяти? |
| `generate_mnemonic_phrase` | → `String` — 12-word BIP39 фраза (без создания wallet) |
| `import_wallet_from_mnemonic` | `{ phrase, password }` → создаёт + сохраняет keystore |
| `unlock_wallet` | `{ password }` → расшифровывает keystore в память |
| `lock_wallet` | → очищает из памяти |
| `is_biometric_enabled` | → `bool` |
| `biometric_unlock_wallet` | → unlock через сохранённый PIN |
| `enable_biometric_unlock` | `{ password }` → сохраняет PIN в biometric.dat |

### CSS-анимации (добавлены в `styles/main.css`)

```css
@keyframes rw-shake { … }
.rw-shake            — применяется к PasscodeDots при ошибке
.rw-keypad-btn:active — тактильный feedback на нажатие
```

### set_timeout в Leptos 0.7

Доступен через `use leptos::prelude::*`:
```rust
set_timeout(move || { pin.set(String::new()); }, Duration::from_millis(500));
```

### Миграция внутренних тестеров

Текущие 2 тестера имеют keystore зашифрованный текстовым паролем ≥8 символов.
После обновления — PIN keypad не может его расшифровать.
**Решение:** перед тестом удалить `{address}.json` из app data dir вручную.
В будущих релизах добавить migration dialog.

---

## 5. Прогресс (обновлять после каждой сессии)

### Сессия 2026-04-23 — онбординг PIN (в процессе)

**Выполнено:**
- [x] Анализ rust-design репо (все экраны, архитектура)
- [x] Gap analysis: PIN vs password, restore flow баг, CreateVerify props
- [x] `commands.rs`: убрана `validate_password` из `import_wallet_from_mnemonic`
- [x] `styles/main.css`: `rw-shake`, `rw-keypad-btn:active` анимации
- [x] `components/passcode.rs`: `Keypad` + `PasscodeDots` + `PASSCODE_LENGTH`
- [x] `components/mod.rs`: публичный re-export
- [x] `main.rs`: добавлен `mod components`
- [x] `pages/unlock.rs`: PIN keypad, auto-unlock на 6-й цифре, shake, biometric

### Сессия 2026-04-24 — restore.rs

**Выполнено:**
- [x] `pages/restore.rs` — phrase input + 3-шаговый PIN wizard (Phrase → SetPin → ConfirmPin → import → /)

### Сессия 2026-04-25 — wallet.rs

**Выполнено:**
- [x] `pages/wallet.rs` — 5-шаговый PIN wizard: SetPin → ConfirmPin → ShowPhrase → Quiz → BackupConfirm → import → /
  - Blur overlay + tap-to-reveal для фразы
  - Quiz: chip grid 4 варианта, 3 вопроса, progress dots (answered=green, current=accent)
  - Wrong answer: red flash 500ms на чипах (DANGER_BG + DANGER border)
  - BackupConfirm: 3 checkbox (CheckboxItem компонент), error banner при ошибке бэкенда
  - Inline styles, navy+periwinkle палитра, 12 токенов
  - `cargo check` ✅ `clippy` ✅ (0 ошибок в wallet.rs) `110 тестов` ✅
  - Пройден `/rust-review`: 0 critical/error, 1 warning исправлен (silent error → error banner)

**Осталось в задаче онбординга:**
- [ ] `cargo tauri android build --apk` → тест на эмуляторе
- [ ] Коммит → пуш → CI

**После этой задачи (следующие задачи):**

- [ ] **Полный визуальный редизайн** — портировать `screens/dark/` из rust-design:
  - `home.rs` (3 варианта: Base/Chart/Tokens + PriceChart)
  - `activity.rs` (day-grouped, новые стили)
  - `send.rs` (новый UI)
  - `receive.rs` (новый QR экран)
  - `settings.rs` (grouped sections + toggles)
  - `txguard.rs` (transaction safety checker)
- [ ] **Миграция на state machine** — заменить leptos_router на `RwSignal<Screen>` (как в rust-design app.rs). Требует полного редизайна всех страниц.
- [ ] **Screen::Unlock** — добавить в state machine после миграции
- [ ] **Cloudflare Worker RPC proxy** — Settings toggle `rpc.rustokwallet.com` (scaffold в `deploy/rpc-proxy/`)
- [ ] **Phase 4** — Cross-chain via Across Protocol (`crates/bridge/`)
- [ ] **iOS TestFlight** — $99 Apple Developer Program
- [ ] **Show Recovery Phrase** — Settings → требует v2 keystore format, отдельный security PR

---

## 6. Как начать новую сессию

```bash
cd /Users/avangard/Workspace/projects/rustok

# 1. Проверить состояние
cargo test --workspace       # должно быть 112+ зелёных
git log --oneline -10        # что менялось?

# 2. Прочитать этот документ
# 3. Открыть rust-design для справки:
#    gh api repos/temrjan/rust-design/contents/src/... --jq '.content' | base64 -d

# 4. Запустить скиллы:
#    /codex     — стандарты стека
#    /rust      — Rust + web/leptos режим

# 5. Работать по FULL workflow
```

**Проверить перед первым коммитом:**
```bash
cargo test --workspace       # все тесты зелёные
cargo clippy --workspace -- -D warnings
git diff                     # только нужные изменения, нет попутных правок
```

**Android сборка и тест:**
```bash
source ~/.zshrc
$ANDROID_HOME/emulator/emulator -avd Pixel_8 -no-snapshot-load &
cd app && cargo tauri android build --apk --target aarch64 --split-per-abi
adb install -r gen/android/app/build/outputs/apk/arm64/release/app-arm64-release.apk
adb logcat -s rustok:V
```

---

## 7. Ссылки

| Ресурс | Где |
|--------|-----|
| Дизайн-прототип | https://github.com/temrjan/rust-design |
| Основной репо | https://github.com/temrjan/rustok |
| CI | https://github.com/temrjan/rustok/actions |
| Play Console | `com.rustok.app`, Internal Testing |
| API | https://api.rustokwallet.com |
| Landing | https://rustokwallet.com |
| Cloudflare Worker | https://rpc.rustokwallet.com/health |
| Keystore | `~/Keys/rustok-release.jks` (backup: Vaultwarden) |
| Vault debug | `ssh 7demo` → `/root/vault/debug/rustok-android-rustls-platform-verifier.md` |
