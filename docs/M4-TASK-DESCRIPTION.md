# M4 ТЗ — End-to-End на физическом Android

> **Цель:** Установить APK на физический Android (Xiaomi), нажать кнопку, увидеть валидную BIP-39 фразу из Rust core.
>
> **Критерий успеха:** Кнопка нажата → 12 слов из BIP-39 wordlist → валидируется bip39 library.
>
> **Версия:** 1.0
> **Дата создания:** 2026-04-29
> **Создан после:** M3 close (PR #10 merged)
> **Целевая ветка:** `feat/m4-android-e2e` (новая ветка от main)

---

# 0. Что уже работает (вход в M4)

После M3 close:
- ✅ APK собирается через `gradlew assembleDebug` (~3m 35s, 144 MB)
- ✅ APK содержит `librustok_mobile_bindings.so` для arm64-v8a + x86_64
- ✅ React Native autolinking подцепляет bridge package
- ✅ `mobile/App.tsx` импортирует `generateMnemonic` из `react-native-rustok-bridge`
- ✅ TurboModule glue (Kotlin + JNI + cpp-adapter) собран в APK

**Что НЕ доказано** (это и есть M4):
- ❌ APK реально устанавливается на физический Xiaomi
- ❌ Нажатие кнопки реально вызывает Rust код
- ❌ Возвращаемый mnemonic — валидная BIP-39 фраза
- ❌ Латентность Rust → JS вызова приемлема (< 100ms cold call)

---

# 1. Pre-requisites

| Требование | Проверка |
|---|---|
| Физический Android phone (Xiaomi из плана) | подключён по USB |
| USB Debugging включён на phone | Settings → About → Build number ×7 → Developer options → USB Debugging |
| ADB видит устройство | `adb devices` показывает device ID |
| `adb reverse tcp:8081 tcp:8081` доступен | Команда не падает (Metro port forwarding для Hot Reload) |
| Известное W6 (Xiaomi INSTALL_FAILED_USER_RESTRICTED) | Phone разблокирован при установке, готов принять диалог подтверждения |

---

# 2. Структура работы — 4 шага

## Шаг 1 — Установка APK на устройство (низкий риск)

### Что делаем
Берём собранный APK из M3 (`mobile/android/app/build/outputs/apk/debug/app-debug.apk`) и устанавливаем на phone через ADB.

### Команды
```bash
adb devices                                    # подтвердить что phone виден
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk
```

### Possible blockers
| Симптом | Митигация |
|---|---|
| `INSTALL_FAILED_USER_RESTRICTED` | Разблокировать phone, принять диалог установки (W6) |
| `INSTALL_FAILED_OLDER_SDK` | Phone Android < min SDK (24); проверить версию Android |
| `INSTALL_FAILED_UPDATE_INCOMPATIBLE` | Удалить старую версию: `adb uninstall com.rustok` |
| `device unauthorized` | На phone принять "Allow USB Debugging" prompt |

### Verify
- App icon "Rustok" появился на phone
- App запускается без crash
- Видна Hello World страница с кнопкой "Generate"

### Atomic commit (если требуется доработка)
Только если нашёл реальную проблему. Сама установка — read-only operation.

---

## Шаг 2 — Запуск с Metro (Hot Reload для отладки)

### Что делаем
Подключить Metro server для hot reload и логирования. Это **не обязательно** для финального теста, но критично для отладки runtime errors.

### Команды (в ДВУХ терминалах)
```bash
# Terminal 1 — Metro server
cd mobile
npx react-native start --port 8081

# Terminal 2 — port forwarding
adb reverse tcp:8081 tcp:8081

# Terminal 3 — логи устройства
adb logcat | grep -i "rustok\|reactnative\|fatal"
```

### Possible blockers
| Симптом | Митигация |
|---|---|
| Metro не стартует | Port 8081 занят: `lsof -i :8081` (или Windows: `netstat -ano \| findstr :8081`) |
| App: "Unable to load script" | Metro не доступен — `adb reverse` не сработал, перезапустить |
| App crash на старте | Скорее всего bridge не подцепился — см. Шаг 3 |

### Verify
- Metro показывает "BUNDLE 100%" в своём терминале
- App открывается на phone без ошибок
- В adb logcat видны логи React Native (даже если пустые)

---

## Шаг 3 — Нажатие кнопки и анализ output (КРИТИЧНЫЙ ШАГ)

### Что делаем
Нажимаем "Generate" на экране, наблюдаем что произойдёт.

### Возможные сценарии

**Сценарий A — успех (целевой):**
- На экране отображаются 12 слов
- Слова выглядят как английские слова из BIP-39 wordlist
- Многократные нажатия дают разные mnemonic'и

**Сценарий B — runtime crash при нажатии:**
- Возможные причины:
  - JNI bridge не нашёл нативный метод (mismatch имён в Kotlin)
  - `librustok_mobile_bindings.so` не загрузился (mismatch архитектур)
  - Rust panic при инициализации
- Диагностика: `adb logcat | grep -E "AndroidRuntime|FATAL|rustok"`

**Сценарий C — кнопка работает, но mnemonic странный:**
- Пустая строка
- Не 12 слов
- Слова не из BIP-39 wordlist
- Сообщение об ошибке вместо mnemonic
- Разные нажатия дают **одинаковые** mnemonic'и (entropy issue!)

### Verify steps (последовательно)

**Verify 1: Структура output'а**
- Считай слова: должно быть **ровно 12**
- Каждое слово должно быть из BIP-39 English wordlist (2048 слов)
- Тест: открой https://github.com/bitcoin/bips/blob/master/bip-0039/english.txt и проверь несколько случайных слов

**Verify 2: Криптографическая валидность**
- BIP-39 mnemonic имеет встроенный checksum
- Невалидный mnemonic = последнее слово не соответствует checksum
- Тест: вставить mnemonic в **любой** BIP-39 validator (например https://iancoleman.io/bip39/) — должен сказать "valid"
- ⚠️ **БЕЗОПАСНОСТЬ:** Используй валидатор **только** для тестовой проверки. Не используй этот mnemonic для реальных кошельков.

**Verify 3: Воспроизводимость (entropy quality)**
- Нажми кнопку 5-10 раз подряд
- Запиши все полученные mnemonic'и
- Все должны быть **разными**
- Если хотя бы два одинаковых — катастрофа (фиксированная entropy)

**Verify 4: Латентность**
- Нажми кнопку — измерь время до отображения mnemonic
- Для cold call (первое нажатие после старта): должно быть < 100ms
- Для hot calls (повторные): должно быть < 50ms
- Если > 500ms cold — performance issue, требует диагностики

### Atomic commits (по результатам)
- Если всё работает → нет коммита, переход к Шагу 4
- Если найден runtime issue → fix → atomic commit
- Если нужны UI улучшения для теста (debug info) → атомарно

---

## Шаг 4 — Security review и финализация

### Что делаем
Перед закрытием M4 — обязательный `/security-review` на изменения от M4.

**Это первый случай в проекте когда mnemonic реально проходит через trust boundary Rust↔UI.** Согласно REVIEWER-CONSTITUTION §9.6 — `/security-review` обязателен.

### Команды
```bash
# Запустить /security-review skill на diff M4
# (executor скилл, инструкция через Reviewer'а)
```

### Что искать в /security-review
1. **Mnemonic в логах** — нет ли случайных `console.log(mnemonic)` или `println!("{:?}", mnemonic)` в Rust
2. **Mnemonic в crash reports** — error handling должен **редактировать** sensitive data
3. **Mnemonic в JS state дольше необходимого** — useState с mnemonic должен очищаться после показа
4. **Mnemonic в clipboard** — есть ли копирование? Если да, должно быть с auto-clear таймером
5. **Mnemonic в screenshots** — Android FLAG_SECURE на screen с mnemonic
6. **adb logcat output** — phone в debug mode может leak'ать через logcat. Reviewer должен проверить что mnemonic **не попадает** в production-equivalent логи

### Verify (security)
- `/security-review` не находит CRITICAL/HIGH issues
- Любые HIGH issues имеют документированный fix или принятый risk в `PHASE-2-CONSTRAINTS.md`

### Финализация
- Атомарные коммиты (по результатам Шагов 1-4)
- Update `docs/POC-FOUNDATION.md`:
  - §1.1 binary checklist: отметить M4 ✓
  - §10.2 reproduce steps: добавить шаги установки и запуска
  - §10.4 performance baseline: добавить cold/hot call latency
- Update `docs/CLAUDE.md`: статус Phase 1 → M4 ✓
- PR `feat/m4-android-e2e → main` через GitHub
- Merge после CI green

---

# 3. Гейты для M4

| Gate | Условие |
|---|---|
| 3.1 | APK устанавливается без блокировки на Xiaomi |
| 3.2 | App запускается без crash |
| 3.3 | Кнопка вызывает Rust (видно в logcat либо по результату) |
| 3.4 | 12 слов отображаются на экране |
| 3.5 | Все 12 слов из BIP-39 wordlist |
| 3.6 | Mnemonic валиден по BIP-39 checksum |
| 3.7 | Многократные нажатия → разные mnemonic'и |
| 3.8 | Cold call latency < 100ms (или объяснимая причина если больше) |
| 3.9 | `/security-review` не находит CRITICAL/HIGH issues |
| 3.10 | M4 PR смержен в main |

**M4 closed = все 10 гейтов пройдены.**

---

# 4. Risks & contingencies

## Риск 1 — APK устанавливается, но crash при старте
**Симптом:** App icon видим, но приложение падает сразу или белый экран.
**Митигация:**
- adb logcat → найти stack trace
- Скорее всего mismatch RN ABI или отсутствует .so для текущей архитектуры
- Проверить: `aapt dump badging app-debug.apk | grep native-code` — какие ABI поддерживает APK
- Проверить: `adb shell getprop ro.product.cpu.abi` — какая ABI у phone

## Риск 2 — Rust panic при вызове generate_mnemonic
**Симптом:** App работает, но при нажатии кнопки crash с FATAL exception.
**Митигация:**
- adb logcat → найти Rust panic message
- Скорее всего entropy source unavailable либо JNI registration issue
- Можем потребоваться fix в `crates/rustok-mobile-bindings/src/lib.rs`

## Риск 3 — Mnemonic генерируется, но не валиден
**Симптом:** 12 слов отображаются, но bip39 validator говорит "invalid".
**Митигация:**
- Проверить какую entropy source использует core
- Проверить BIP-39 wordlist version (English standard, не китайский)
- Это КРИТИЧНО — невалидный mnemonic = неработающий wallet

## Риск 4 — Воспроизводимые mnemonic'и (entropy issue)
**Симптом:** Два нажатия → одинаковый mnemonic.
**Митигация:**
- BLOCKER. Не двигаться дальше до фикса.
- Проверить что `random_mnemonic_phrase()` использует system RNG, не deterministic seed
- Это catastrophic security issue если попадёт в production

## Риск 5 — Латентность > 1 секунды
**Симптом:** Кнопка нажата, mnemonic появляется через 1-3 секунды.
**Митигация:**
- Не блокер для M4 (POC scope), но беспокоит для UX
- Профилирование через Android Profiler / Xcode Instruments (Phase 4-5)
- Документировать в `PHASE-2-CONSTRAINTS.md` Phase 4-5 polish

## Риск 6 — Не доходим до Шага 3 из-за инфраструктуры
**Симптом:** Невозможно установить APK / Metro не работает / phone не видим.
**Митигация:**
- Это **не code issue**, это environment issue
- Возможно нужен другой phone, другой USB cable, другой ADB version
- Не блокирует архитектурную ставку — попробовать на эмуляторе как fallback
- Документировать как M4 partial и продолжить M5 (iOS)

---

# 5. Что НЕ делаем в M4

- ❌ UI/UX полировка (Phase 3)
- ❌ Биометрия / Keychain / safe storage (Phase 4-5)
- ❌ Многоэкранная навигация (Phase 3)
- ❌ NativeWind / стилизация (Phase 3)
- ❌ Async functions через uniffi (M3 exclusion остаётся)
- ❌ E2E на iOS (это M5)
- ❌ Production signing (Phase 8)
- ❌ Release builds (Phase 8)

---

# 6. Workflow для M4 (с REVIEWER-CONSTITUTION v1.2)

Каждый шаг M4 проходит через тот же workflow что был в M3:

1. Изучаю → 2. План → 3. /check → 4. Исправляю → 5. Реализую → 6. /rust-review или /typescript-review → 7. /security-review (M4 mandatory) → 8. Коммит → 9. Push (per policy C — после логических групп)

**Между КАЖДЫМ шагом — пауза, ждать "да" от оператора.**

**Skills reminders от Reviewer'а** — обязательны для security-relevant changes (см. §9.6).

---

# 7. Что делать если M4 провалится

В отличие от M3, M4 **не блокер** для архитектурной ставки. Если APK не работает на физическом Android — можем:

1. Попробовать на эмуляторе (validate что архитектура работает в принципе)
2. Перейти к M5 (iOS) — может проблема Xiaomi-specific
3. Зафиксировать M4 как partial, добавить Android-specific fixes в Phase 4-5

**M3 (architecture build works) — это закрыто и не отзывается.** M4 — это validation на реальном железе.

---

# 8. Estimated time

| Шаг | Оптимистично | Реалистично | Если проблемы |
|---|---|---|---|
| 1. Install APK | 5 мин | 15 мин | 1-2 часа |
| 2. Metro setup | 10 мин | 30 мин | 1-2 часа |
| 3. Button + verify | 30 мин | 1-2 часа | 4-8 часов |
| 4. Security review + finalize | 1 час | 2 часа | 4 часа |
| **Итого** | **~2 часа** | **~4-5 часов** | **~1-2 дня** |

---

# 9. Что загрузить в новую сессию для M4

```
1. REVIEWER-CONSTITUTION-v1.2.md (operating system Reviewer'а)
2. M3-SESSION-HANDOFF.md (история M3)
3. Этот документ M4-TASK-DESCRIPTION.md (план M4)
4. PHASE-2-CONSTRAINTS.md (архитектурный долг)
5. POC-FOUNDATION.md (общий план Phase 1)
6. CLAUDE.md (project instructions)

Если Cowork даёт доступ к папке проекта — загрузи всю
C:\Claude\projects\rustok\.
```

---

# 10. Quick start prompt для новой сессии

```
[ВЛОЖЕНИЯ: REVIEWER-CONSTITUTION-v1.2.md, M3-SESSION-HANDOFF.md, 
M4-TASK-DESCRIPTION.md, PHASE-2-CONSTRAINTS.md]

Привет. Я Темирлан, продолжаю работу над Rustok.

M3 закрыт (PR #10 merged). Стартую M4 — E2E на физическом Android.
Детальный план в M4-TASK-DESCRIPTION.md.

Constitution v1.2 — твой operating system. Особое внимание на §9.6
(skills reminders) и §6.7 (workflow shortcuts) — это слабые места
executor'а из M3 опыта.

M4 = первый случай когда mnemonic реально проходит через bridge
к UI. /security-review на любых security-relevant изменениях
(§9.6 constitution) — обязателен.

Подтверди загрузку всех документов и переходи в режим Reviewer.
Готов начать с Шага 1 (Install APK на physical Xiaomi).
```

---

**Конец ТЗ M4.**
