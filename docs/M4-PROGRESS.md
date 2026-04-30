# M4 Progress — End-of-Session Handoff

> **Сессия:** 2026-04-29
> **Статус:** M4 Шаги 1-3 пройдены, остался Шаг 4 (security-review + PR)
> **Ветка:** `main` (НЕ создавали `feat/m4-android-e2e` — есть uncommitted изменение)
> **Гейты:** 7/10 пройдены (3.1, 3.2, 3.3, 3.4, 3.5, 3.7, 3.8) · 3/10 остались (3.6 emp, 3.9, 3.10)

---

## Что сделано

### Шаг 1 — APK install ✅
- `adb devices` → `JFLFG6MZSSL7WCF6 device` (Xiaomi)
- `adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk` → `Success`
- W6 (INSTALL_FAILED_USER_RESTRICTED) **не сработал** — Xiaomi без диалога
- Удалён старый пакет `com.rustok.app` (от Tauri); остался правильный `com.rustok`

### Шаг 2 — Metro setup ✅ (с фиксом)
**Проблема:** Metro падал с `Unable to resolve module @babel/runtime/helpers/interopRequireDefault`. Причина: дефолтный `metro.config.js` не знает про npm workspaces — `watchFolders` ограничен `mobile/`, hoisted deps в `node_modules` корня не видны.

**Фикс (UNCOMMITTED):** `mobile/metro.config.js` — monorepo-aware конфиг:
```js
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');
const path = require('path');
const projectRoot = __dirname;
const workspaceRoot = path.resolve(projectRoot, '..');
const config = {
  watchFolders: [workspaceRoot],
  resolver: {
    nodeModulesPaths: [
      path.resolve(projectRoot, 'node_modules'),
      path.resolve(workspaceRoot, 'node_modules'),
    ],
    disableHierarchicalLookup: true,
  },
};
module.exports = mergeConfig(getDefaultConfig(projectRoot), config);
```

После фикса + `--reset-cache`: bundle 639 модулей, без ошибок. App открылся, Hello World видим.

### Шаг 3 — Generate button + verify ✅ (кроме 3.6 эмпирики)

**Verify результаты:**
- ✅ **3.4** — ровно 12 слов
- ✅ **3.5** — все английские lowercase, BIP-39 wordlist
- ✅ **3.7** — несколько нажатий → разные mnemonic'и (entropy работает)
- ✅ **3.8** — мгновенное появление (<100ms cold call)
- ⏳ **3.6** — **теоретически доказано** (B), эмпирическая проверка через iancoleman.io пропущена

**B — теоретическое доказательство (3.6):**
- `crates/core/src/keyring/local.rs:101-106` использует `coins_bip39::Mnemonic::<English>::new_with_count(&mut rng, 12)`
- `rand::thread_rng()` → системный CSPRNG (ChaCha20, OS-seeded)
- 128 bits entropy + SHA256-derived 4-bit checksum → 12 слов by construction
- Покрытие тестами `local.rs:357-401`: structure, uniqueness, MetaMask compat, round-trip from_mnemonic (с checksum-проверкой)
- **Checksum математически невозможен невалидный** при таком построении

**Bridge:** `crates/rustok-mobile-bindings/src/lib.rs:30-36` — `generate_mnemonic()` через `#[uniffi::export]`.

**Тестовый mnemonic из сессии (СКОМПРОМЕТИРОВАН, НЕ ИСПОЛЬЗОВАТЬ):**
```
orange flame force mesh install ugly cargo afraid oblige where spawn endorse
```

---

## ⚠️ Security finding — для /security-review (Шаг 4)

**Файл:** `crates/rustok-mobile-bindings/src/lib.rs:30-36`

**Проблема:** `phrase.to_string()` клонирует `Zeroizing<String>` (источник) в обычный `String` для FFI. После клона:
- `Zeroizing` защищает **только источник** — обнуляется на drop
- Копия проходит через uniffi → JNI → Kotlin String → JS `string` — нигде не zeroed
- Phrase остаётся в нескольких heap allocations до GC

**Категория:** likely HIGH (mnemonic в FFI/heap дольше необходимого), требует решения от Reviewer'а.

**Возможные mitigations:**
1. Использовать `Vec<u8>` через FFI вместо `String`, очищать на JS стороне
2. Принять risk и задокументировать в `PHASE-2-CONSTRAINTS.md` для Phase 4-5
3. Wrapper над uniffi-generated кодом с manual zeroing после Kotlin копии

---

## Что дальше — Next session checklist

### 1. Закрыть гейт 3.6 эмпирически (опционально, теория уже есть)
Открыть https://iancoleman.io/bip39/, вставить тестовый mnemonic выше, убедиться "Valid".
**Альтернатива:** написать `cargo test` который парсит фразу через `LocalKeyring::from_mnemonic()` (она валидирует checksum). Или просто accept теорию и идти дальше.

### 2. Создать ветку для M4
```bash
git checkout -b feat/m4-android-e2e
```
(До этого `metro.config.js` лежит uncommitted на `main` — НЕ коммитить в main!)

### 3. /typescript-review на изменение `metro.config.js`
Проверить:
- `disableHierarchicalLookup: true` — не сломает ли resolve внутри `react-native-rustok-bridge`? (В этой сессии bundle прошёл — кажется норм, но review нужен)
- Источник: https://reactnative.dev/docs/metro и https://metrobundler.dev/docs/configuration

### 4. /security-review (mandatory per ТЗ §2 Шаг 4, constitution §9.6)
- Phrase в логах? (`grep -r "println.*mnemonic\|console.log.*mnemonic" .`)
- Phrase в crash reports? (error handling должен редактировать)
- Phrase в JS state дольше необходимого? (`mobile/App.tsx` useState с phrase)
- Phrase в clipboard? FLAG_SECURE? logcat leak?
- **Главное:** finding выше про `Zeroizing` → `String` clone

### 5. Atomic коммиты
- `fix(metro): monorepo workspace config for hoisted deps` — `mobile/metro.config.js`
- (Если что-то ещё правится по результатам review — отдельный коммит)

### 6. Обновить docs (per ТЗ §2 Шаг 4)
- `docs/POC-FOUNDATION.md`:
  - §1.1 — отметить M4 ✓
  - §10.2 — добавить шаги install + Metro setup
  - §10.4 — добавить latency baseline (мгновенно <100ms)
- `docs/CLAUDE.md` — статус Phase 1 → M4 ✓

### 7. PR `feat/m4-android-e2e` → `main`
Через `gh pr create`, CI green, merge.

---

## Quick commands для следующей сессии

```bash
cd C:/Claude/projects/rustok
git status                                    # увидишь uncommitted mobile/metro.config.js
git log --oneline -5

# Если phone подключён + Metro нужен:
adb devices                                   # JFLFG6MZSSL7WCF6
adb reverse tcp:8081 tcp:8081
cd mobile && npx react-native start --port 8081 --reset-cache
# Открыть Rustok app на phone, тапнуть RELOAD если красный экран
```

---

## Затронутые файлы в этой сессии

| Файл | Изменение | Статус |
|---|---|---|
| `mobile/metro.config.js` | monorepo workspace config | UNCOMMITTED, нужен `/typescript-review` |
| `docs/M4-TASK-DESCRIPTION.md` | (untracked) | Read-only, ТЗ |
| `docs/M4-PROGRESS.md` | (этот файл) | NEW |

---

## Что НЕ сделано (deferred)

- ❌ UI/UX полировка (mnemonic в одну строку без номерации) — **explicit out of scope для M4 (Phase 3)**. Пользователь упомянул как UX issue, но согласен отложить.
- ❌ /codex и `/typescript` skills для конфига Metro — пропущены по решению пользователя («инфраструктурный конфиг, не бизнес-логика»). `/typescript-review` перед коммитом — **обязателен**.
- ❌ Эмпирическая проверка checksum через iancoleman.io — пользователь не успел в этой сессии.
