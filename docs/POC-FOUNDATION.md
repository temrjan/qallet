# POC-FOUNDATION — Phase 1 детальный план

> **Цель Фазы 1 (2-3 недели):** Доказать end-to-end что архитектура `React Native UI → uniffi → Rust core` работает на реальных устройствах (твой Android phone + твой iPhone). Bridge **одной** функции.
>
> **Критерий успеха:** Нажимаешь кнопку "Generate mnemonic" в Rustok app на iPhone и Android → получаешь валидную BIP-39 фразу из rustok-core.
>
> **Дата создания:** 2026-04-28
> **Статус:** READY TO START после согласования
> **Связанные документы:** `docs/NATIVE-MIGRATION-PLAN.md` (стратегия и onboarding A-O)

---

# 0. Перед стартом — обязательное чтение

> ⚠️ **Если ты AI-агент в новой сессии — сначала прочти `docs/NATIVE-MIGRATION-PLAN.md` секции A-O (Onboarding).** Этот документ предполагает что ты уже знаешь стек, workflow, правила и стратегические решения.

**Pre-requisite чтение для текущей фазы:**
1. `docs/NATIVE-MIGRATION-PLAN.md` — стратегия + onboarding A-O
2. `docs/RESEARCH-NATIVE-STACKS.md` — обоснование выбора uniffi-bindgen-react-native
3. **README репозитория `jhugman/uniffi-bindgen-react-native`** на GitHub — актуальный setup guide (НЕ полагаться на выдуманные команды в этом документе — всегда сверяться с upstream README)
4. **React Native New Architecture docs:** https://reactnative.dev/docs/the-new-architecture/landing-page
5. **uniffi book** (Mozilla): https://mozilla.github.io/uniffi-rs/

---

# 1. Цель и success criteria

## 1.1 Что считается "POC пройден" (binary checklist)

- [x] Ветка создана и merged: `feat/m3-uniffi-rn-bridge` → `main` через PR #10 (commit `f4580c1`, 2026-04-29)
- [x] `crates/rustok-mobile-bindings/` существует, компилируется через `cargo build --release` (M1)
- [x] uniffi экспортирует `generate_mnemonic() -> Result<String, BindingsError>` (обёртка над `rustok_core::keyring::LocalKeyring::random_mnemonic_phrase`) (M1)
- [x] `mobile/` директория содержит React Native **0.85.2** проект (New Architecture default) (M2)
- [x] Auto-generated TS bindings — расположение `packages/react-native-rustok-bridge/src/` (изменено в M3 из-за npm workspaces architecture; spirit fulfilled, не letter `mobile/src/native/`) (M3)
- [x] Auto-generated Kotlin TurboModule — расположение `packages/react-native-rustok-bridge/android/src/main/java/com/rustok/bridge/` (M3)
- [ ] Auto-generated Swift TurboModule в `packages/react-native-rustok-bridge/ios/...` (M5 — на Mac)
- [x] **Android APK сборка:** `gradlew app:assembleDebug` проходит, `librustok bridge .so` для arm64-v8a + x86_64 в APK (M3, Шаг 7)
- [x] **Android физ. устройство:** APK устанавливается → нажатие кнопки → BIP-39 фраза в UI (M4 — Xiaomi `JFLFG6MZSSL7WCF6`, 2026-04-30)
- [ ] **iPhone физ. устройство:** IPA устанавливается → нажатие кнопки → BIP-39 фраза в UI (M5)
- [x] Mnemonic валидируется через BIP-39 (12 слов, English wordlist, разные при повторных нажатиях, checksum валиден) (M4 — структура и uniqueness verified empirically; checksum verified **theoretical + empirical**: theoretical через конструкцию `coins_bip39::Mnemonic::<English>::new_with_count` + покрытие `crates/core/src/keyring/local.rs:357-401`; empirical через iancoleman.io 2026-04-29 — оператор проверил фразу из device-run, результат "Valid")
- [x] `docs/POC-FOUNDATION.md` §10 обновлён — versions §10.1, reproduction §10.2, known issues W7-W9 §10.3, performance baseline §10.4 (M3 close)

## 1.2 Что НЕ входит в POC (явные exclusions)

- ❌ Полноценный UI — только Hello World с одной кнопкой
- ❌ Все 22 команды rustok-core — только `generate_mnemonic`
- ❌ Async functions через uniffi (sync только; async — Phase 2)
- ⚠ Result/Error через uniffi — **РАЗРЕШЕНО** (M3 использует `Result<String, BindingsError>` + `#[derive(uniffi::Error)]` enum, см. `crates/rustok-mobile-bindings/src/lib.rs:30`); Record и Enum с data variants — Phase 2
- ❌ Биометрия / Keychain / Camera — Phase 4-5
- ❌ Navigation / multi-screen — Phase 3
- ❌ NativeWind / стилизация — Phase 3
- ❌ Tests (unit/integration) — добавляются в Phase 2 после core extraction

## 1.3 Что доказывает успешный POC

1. **uniffi-bindgen-react-native работает с нашим Rust core** — нет фундаментальных блокеров (✓ M3 — bindings генерируются, autolinking работает, `.so` в APK)
2. **Build pipeline на Windows + Mac работает** — нет environment-specific issues (✓ Windows M3 — known issues W7/W8/W9 в §10.3; ☐ Mac M5)
3. **Физические устройства принимают builds** — нет signing/policy issues (☐ Android M4, iOS M5)
4. **Performance acceptable** — латентность Rust → JS вызова <100ms на cold call (☐ M4 E2E)
5. **Code review показывает что архитектурные deferrals понятны и зафиксированы** — `/rust-review` + `/typescript-review` на закрытии каждого milestone; observations выше LOW → `docs/PHASE-2-CONSTRAINTS.md` (✓ M3 review pass: 1 HIGH, 2 MEDIUM, 2 LOW; LOW#1 audit отдельно, HIGH+MEDIUM в constraints)

После успешного POC мы committed на Native путь и стартуем Phase 2 (Core API extraction).
Phase 2 entry condition: items в `docs/PHASE-2-CONSTRAINTS.md` имеют документированные решения.

---

# 2. Pre-requisites — проверка окружения

## 2.1 На Windows (основной dev box)

| Tool | Версия | Команда проверки | Если нет |
|------|--------|------------------|----------|
| Node | 24.x | `node --version` | nvm-windows install |
| npm | 11.x | `npm --version` | вместе с Node |
| Rust | stable | `rustc --version` | rustup |
| cargo | stable | `cargo --version` | вместе с Rust |
| Java | 17 LTS | `java -version` | Eclipse Temurin / Microsoft OpenJDK |
| Android Studio | 2024.x+ | в Start menu | https://developer.android.com/studio |
| Android SDK | API 34+ | через Android Studio SDK Manager | SDK Manager |
| Android NDK | 26.x+ | через SDK Manager | SDK Manager |
| adb | latest | `adb --version` | в Android SDK platform-tools |
| Rust Android targets | — | `rustup target list --installed` ищем `*-linux-android*` | см. ниже |
| cargo-ndk | latest | `cargo ndk --version` | `cargo install cargo-ndk` |
| Watchman (опц.) | — | `watchman --version` | choco install watchman |

**Установка Rust Android targets (надо сделать в Day 1):**
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

**Environment variables (надо проверить):**
- `ANDROID_HOME` — путь к Android SDK (обычно `C:/Users/omadg/AppData/Local/Android/Sdk`)
- `ANDROID_NDK_HOME` или `NDK_HOME` — путь к NDK (внутри SDK, например `$ANDROID_HOME/ndk/26.x.x`)
- `JAVA_HOME` — путь к JDK 17

## 2.2 На Mac (для iOS milestone)

| Tool | Версия | Команда проверки | Если нет |
|------|--------|------------------|----------|
| macOS | 14+ Sonoma | About This Mac | system update |
| Xcode | 15.x+ | `xcodebuild -version` | App Store |
| Xcode Command Line Tools | latest | `xcode-select -p` | `xcode-select --install` |
| Apple Developer аккаунт | active | в Xcode → Preferences → Accounts | https://developer.apple.com |
| Cocoapods | 1.15+ | `pod --version` | `sudo gem install cocoapods` |
| Rust iOS targets | — | `rustup target list --installed` ищем `*-apple-ios*` | см. ниже |
| cargo-lipo или xcframework | — | `cargo lipo --version` | `cargo install cargo-lipo` (либо использовать xcframework вручную) |

**Установка Rust iOS targets (на Mac):**
```bash
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
```

## 2.3 Физические устройства

- **Android phone:** USB Debugging включён (Settings → About → tap Build number 7 раз → back → Developer options → USB Debugging)
- **iPhone:** Device включён в Xcode → Devices and Simulators, signing certificate настроен в Xcode → Preferences → Accounts

---

# 3. Структура работы — 6 milestones (2-3 недели)

> **Workflow на каждом milestone:** см. `NATIVE-MIGRATION-PLAN.md` §C (8-шаговый: Изучаю → План → /check → Исправляю → /codex+/rust(или /typescript) → Реализую → Ревью → Коммит).
>
> **Оценка времени** условная — может варьироваться на ±50% в зависимости от блокеров.

## Milestone 1: Bindings crate (2-3 дня)

**Цель:** Минимальный Rust crate, который через uniffi экспортирует одну функцию.

### Шаги
1. **Изучаю:**
   - Прочитать `app/src-tauri/src/commands.rs` — найти где реализован `generate_mnemonic_phrase()`. Скорее всего вызов из `rustok-core`.
   - Прочитать `crates/rustok-core/` структуру — найти модуль с mnemonic генерацией
   - Прочитать актуальный README `mozilla/uniffi-rs` — синтаксис проктомакросов (`#[uniffi::export]`)
2. **План sub-doc:** не нужен, тривиальный crate
3. **/check:** короткая самопроверка списка шагов
4. **/rust:** загрузить Rust стандарты
5. **Реализую:**
   - Создать `crates/rustok-mobile-bindings/Cargo.toml` с зависимостями: `uniffi`, `uniffi_macros`, `rustok-core`
   - В `[lib]` секцию: `crate-type = ["cdylib", "staticlib"]`
   - Создать `src/lib.rs`:
     - `uniffi::setup_scaffolding!()` (точный макрос — сверить с README)
     - Re-export функции `generate_mnemonic` (тонкая обёртка над `rustok_core::wallet::generate_mnemonic`)
   - Добавить crate в корневой `Cargo.toml` workspace `members = [...]`
6. **Локальный тест:** `cargo test -p rustok-mobile-bindings` (тест что функция возвращает 12 слов)
7. **Ревью:** `git diff`, искать попутные изменения
8. **Коммит** (по запросу): `feat(bindings): scaffold rustok-mobile-bindings crate with uniffi export`

### Gates
- `cargo build --release -p rustok-mobile-bindings` — зелёный на Windows
- `cargo test -p rustok-mobile-bindings` — зелёный, mnemonic валидный

### Возможные блокеры
- **rustok-core не экспортирует public API для mnemonic** → нужно сначала refactor: вынести функцию в `pub mod` (это уже **частичная Phase 2 работа** — фиксируем как acceptable)
- **uniffi версия конфликтует с alloy-rs/другими deps** → закрепить версию через `[patch.crates-io]` или искать совместимую

---

## Milestone 2: React Native scaffold (1-2 дня)

**Цель:** Bare RN 0.76+ проект, запускается Hello World на Android emulator/device.

### Шаги
1. **Изучаю:** README `react-native-community/cli` — актуальная команда инициализации (НЕ устаревший `npx react-native init`)
2. **План:** структура `mobile/` директории
3. **Реализую:**
   - Создать `mobile/` через `npx @react-native-community/cli@latest init Rustok` (точная команда — сверить с docs)
   - Убедиться что New Architecture включена (default в 0.76+, но проверить `gradle.properties` → `newArchEnabled=true` и `Podfile.properties.json` → `"newArchEnabled": "true"`)
   - Удалить boilerplate `App.tsx` → заменить на минимальный с одним экраном "Hello Rustok" + кнопка (пока без bridge)
   - Tsconfig strict mode
4. **Локальный тест:** `cd mobile && npx react-native run-android` (на эмуляторе или физ. устройстве)
5. **Ревью + коммит:** `chore(mobile): scaffold react native 0.76 with new architecture`

### Gates
- Metro bundler запускается без ошибок
- Hello World рендерится на Android физ. устройстве через USB

### Возможные блокеры
- **NDK не найден** → проверить `local.properties` в `mobile/android/`, добавить `ndk.dir=...`
- **Java версия не 17** → `JAVA_HOME` указывает не туда
- **Gradle daemon hangs** → `cd mobile/android && ./gradlew --stop && ./gradlew clean`

---

## Milestone 3: uniffi-bindgen-react-native setup (3-5 дней)

**Цель:** Bindings crate генерирует TurboModule (Kotlin + Swift) и TS wrapper, подключается в RN.

### Шаги
1. **Изучаю (приоритет — день 1):**
   - **Полностью прочитать README `jhugman/uniffi-bindgen-react-native`** — все шаги setup
   - Изучить examples репозитория (`examples/` директория) — найти minimal sample
   - Прочитать какие именно файлы генерятся и куда
2. **План:** написать sub-doc `docs/POC-MILESTONE-3-NOTES.md` с конкретными шагами setup из README (актуальная версия — не выдумывать)
3. **/check:** ревью плана setup
4. **Реализую:** **строго по README** — не отклоняться без причины. Типичный flow:
   - Установить tool: `npx uniffi-bindgen-react-native ...` (точная команда из README)
   - Сгенерировать UDL/proc-macro описание
   - Конфиг файл (если требуется)
   - Generate команда: записать в `mobile/scripts/gen-bindings.sh` (для воспроизводимости)
   - Подключить generated Kotlin/Swift в Android/iOS проекты RN (modify `build.gradle`, `Podfile`)
   - Импортировать generated TS в `mobile/src/native/rustok.ts`
5. **Тест:** Cross-compile Rust для Android target:
   - `cd crates/rustok-mobile-bindings && cargo ndk -t arm64-v8a build --release`
   - Скопировать `.so` в `mobile/android/app/src/main/jniLibs/arm64-v8a/`
6. **Ревью + коммит:** `feat(mobile): setup uniffi-bindgen-react-native + auto-generated bindings`

### Gates
- `npx uniffi-bindgen-react-native generate ...` отрабатывает без ошибок
- Generated файлы появляются в ожидаемых директориях
- TypeScript binding импортируется без `Cannot find module` ошибок

### Возможные блокеры (вероятные!)
- **uniffi-bindgen-react-native не 1.0** — возможны breaking changes / неполная docs. Решение: pin exact версию, читать changelog, не апгрейдить без необходимости.
- **NDK build fails** для Rust → сверять Android NDK version с cargo-ndk requirements
- **TurboModule registration fails** → проверить что New Architecture включена (см. Milestone 2)
- **Generated Swift/Kotlin code не компилируется** → возможно баг tool-а, искать issues на GitHub

### Решение если блокер серьёзный
Если на 5-й день Milestone 3 не сдвинулся — **СТОП**, перечитать `NATIVE-MIGRATION-PLAN.md §10 (Revert path)`. Это первый concrete checkpoint где revert может быть оправдан.

---

## Milestone 4: First call end-to-end на Android (2-3 дня)

**Цель:** Кнопка в RN UI → вызов Rust через JSI → BIP-39 mnemonic в UI на физ. устройстве.

### Шаги
1. **Изучаю:** какой тип возвращает generated TS функция (`Promise<string>` или `string`?)
2. **Реализую:**
   - В `mobile/App.tsx`:
     ```tsx
     import { generateMnemonic } from './src/native/rustok';
     
     const [mnemonic, setMnemonic] = useState<string | null>(null);
     const onPress = async () => {
       const phrase = await generateMnemonic();  // или sync — зависит от uniffi config
       setMnemonic(phrase);
     };
     // <Button onPress={onPress}>Generate</Button>
     // <Text>{mnemonic ?? '—'}</Text>
     ```
   - Реализовать минимальный UI (TouchableOpacity + Text, без библиотек)
3. **Тест на физ. Android:**
   - USB кабель → `adb devices` → подтвердить что устройство видно
   - `npx react-native run-android` → APK установится на устройство
   - Открыть app → нажать кнопку → mnemonic появляется
4. **Validate mnemonic:** установить в test `bip39` npm package → проверить что фраза валидна (12 слов, checksum OK)
5. **Ревью + коммит:** `feat(mobile): hello rustok end-to-end rust → rn on android`

### Gates
- На физ. Android phone: кнопка → mnemonic в UI
- Mnemonic валидный BIP-39
- Латентность приемлемая (видимо мгновенно, <500ms)

### Возможные блокеры
- **`Cannot find native module 'Rustok'`** → TurboModule не зарегистрирован. Ревизировать MainApplication.kt (Android) — должен быть register call.
- **Crash при вызове** → JNI ABI mismatch, скорее всего `.so` не той архитектуры. Проверить `arm64-v8a` для современных устройств.
- **Empty/null возврат** → вероятно ошибка в Rust (panic захватывается?). Проверить `adb logcat`.

---

## Milestone 5: iOS parity (2-3 дня — на Mac!)

**Цель:** То же самое на iOS Simulator + физ. iPhone.

### Pre-requisites
- Перенести codebase на Mac (git push + clone, или sync через iCloud/Dropbox)
- Все Pre-requisites Mac из §2.2 выполнены

### Шаги
1. **Изучаю:** README uniffi-bindgen-react-native iOS-specific раздел
2. **Реализую:**
   - Cross-compile Rust для iOS на Mac:
     - `cargo build --target aarch64-apple-ios --release` (для физ. устройства)
     - `cargo build --target aarch64-apple-ios-sim --release` (для Simulator на Apple Silicon)
   - Создать xcframework: `xcodebuild -create-xcframework ...` (точная команда — README)
   - Поместить xcframework в `mobile/ios/Frameworks/`
   - `cd mobile/ios && pod install`
   - Открыть `mobile/ios/Rustok.xcworkspace` в Xcode
   - Signing: настроить team в `Signing & Capabilities`
3. **Тест на iOS Simulator:**
   - `npx react-native run-ios`
   - Кнопка → mnemonic → валидно
4. **Тест на физ. iPhone:**
   - Подключить iPhone через USB → разрешить trust
   - В Xcode выбрать device → Run
   - Кнопка → mnemonic → валидно
5. **Ревью + коммит:** `feat(mobile): ios parity for hello rustok`

### Gates
- iOS Simulator: app запускается, кнопка работает
- Физ. iPhone: app запускается, кнопка работает
- Латентность приемлемая

### Возможные блокеры
- **Signing failed** → проверить Apple Developer аккаунт активен, certificate в Keychain
- **App crashes on launch** → проверить что xcframework содержит правильную архитектуру (sim vs device)
- **Pod install fails** → `cd mobile/ios && pod repo update && pod install`

---

## Milestone 6: README + reproduce documentation (1-2 дня)

**Цель:** Любой человек (или AI-агент в новой сессии) может воспроизвести POC по этому документу.

### Шаги
1. Обновить эту секцию §10 ниже с **точными командами** которые использовались (без выдумок — то что реально сработало)
2. Обновить `mobile/README.md` с quick-start
3. Зафиксировать версии всех инструментов в `mobile/package.json`, `crates/rustok-mobile-bindings/Cargo.toml` через exact pins
4. Скриншот работающего app на iOS + Android (для PR description)
5. **Ревью + финальный коммит:** `docs: poc reproduce guide + final pins`

---

# 4. Acceptance criteria для перехода к Phase 2

- [ ] Все 11 пунктов из §1.1 ✅
- [ ] §10 этого документа заполнен реальными командами
- [ ] Pull request `feat/native-rn-poc → main` создан, проходит CI
- [ ] Личный smoke-тест: пользователь нажал кнопку на iPhone → увидел mnemonic
- [ ] Список зависимостей и версий зафиксирован
- [ ] **Решение пользователя:** "POC прошёл, идём в Phase 2"

---

# 5. Что делать если POC провалился

> Это **первая** реальная checkpoint где revert на WebView план оправдан.

См. `NATIVE-MIGRATION-PLAN.md §10` для concrete blockers. Кратко:
- Если `uniffi-bindgen-react-native` оказался слишком сырым/непригодным → revert
- Если iOS не работает по фундаментальной причине (App Store policy на FFI?) → revert
- Если performance overhead > 500ms на простой call → revert

**Если revert:**
1. Заархивировать `feat/native-rn-poc` ветку как `archive/native-poc-failed-2026-XX`
2. Восстановить `docs/_archive/FRONTEND-IMPLEMENTATION-WEBVIEW.md` → `docs/FRONTEND-IMPLEMENTATION.md`
3. Создать `docs/POC-RETROSPECTIVE.md` с детальным анализом почему провалился (для будущих попыток когда инструменты созреют)
4. Стартовать Phase 0 WebView плана

**Что НЕ повод для revert:**
- "Сложно" / "медленно учиться" — нормальная цена за правильную архитектуру
- "Уже потратили 2 недели" — sunk cost
- "Хочется быстрее показать что-то" — желание, не блокер

---

# 6. Workflow напоминание

Каждый milestone проходит через 8 шагов из `NATIVE-MIGRATION-PLAN.md §C`:

1. Изучаю → 2. План → 3. /check → 4. Исправляю → 5. /codex (+ /rust или /typescript) → 6. Реализую → 7. Ревьюю (+ /rust-review или /typescript-review) → 8. Коммит → пуш → CI

**Между КАЖДЫМ шагом — пауза, ждать "да" от пользователя.**

**Коммит и Push — только по явному запросу пользователя.**

---

# 7. Что НЕ делать в этой фазе

- ❌ Не оптимизировать UI (Hello World — это всё)
- ❌ Не добавлять навигацию, темы, Tailwind
- ❌ Не добавлять остальные 21 команду — только `generate_mnemonic`
- ❌ Не пытаться сделать тесты (E2E, unit) — Phase 2+
- ❌ Не пытаться настроить CI workflows для mobile — Phase 8
- ❌ Не удалять `app/src/` или `app/src-tauri/` — это в Phase 8

---

# 8. Команды-шпаргалка (будут уточнены в §10 после POC)

```bash
# Workspace проверка
# ВАЖНО: путь должен быть ASCII-only (AGP не поддерживает кириллицу на Windows)
cd C:/Claude/projects/rustok
git status
git log --oneline -5
cargo test --workspace

# Создание ветки
git checkout -b feat/native-rn-poc

# Rust bindings
cd crates/rustok-mobile-bindings
cargo build --release
cargo test

# Cross-compile для Android (на Windows)
cargo ndk -t arm64-v8a build --release

# RN dev (Android) — через PowerShell, не Git Bash!
cd mobile
npm install

# Создать mobile/android/local.properties (gitignored, создавать вручную):
# sdk.dir=C\:\\Users\\omadg\\AppData\\Local\\Android\\Sdk

# Metro — в отдельном терминале:
npx react-native start --port 8081

# Сборка и установка APK (PowerShell из mobile/android/):
.\gradlew.bat app:installDebug -PreactNativeDevServerPort=8081

# Reverse port для физ. Android устройства (обязательно!):
adb reverse tcp:8081 tcp:8081

# RN dev (iOS — на Mac)
cd mobile/ios
pod install
cd ..
npx react-native run-ios

# Generate uniffi bindings (точная команда — из README!)
npx uniffi-bindgen-react-native generate \
  --crate ../crates/rustok-mobile-bindings \
  --out-dir src/native
# ↑ это PLACEHOLDER, сверять с актуальным README
```

---

# 9. Риски Phase 1 (специфичные)

| Риск | Вероятность | Митигация |
|------|-------------|-----------|
| uniffi-bindgen-react-native не работает out-of-box | Medium | Внимательно читать README + examples + issues. Если не получается за 5 дней — revert. |
| Rust core не имеет публичного API для mnemonic | High | Refactor частично в Milestone 1 (legitimate Phase 2 prep) |
| Android NDK / build chain on Windows не работает | Medium | WSL2 как fallback. Или сразу делать Milestone 1-3 на Mac. |
| iOS Simulator не запускается на Mac (старый Xcode) | Low | Update Xcode до latest |
| Apple Developer аккаунт не настроен / истёк | Medium | Renew $99/year, sign certificates перед Milestone 5 |
| Performance: cold call > 500ms | Low | Если случилось — профайлинг через Xcode Instruments / Android Profiler |
| Версии RN 0.76+ ломают uniffi-bindgen-react-native | Medium | Pin exact RN version, не upgrade без необходимости |

---

# 10. Reproduce steps (заполняется ПОСЛЕ прохождения POC)

> Этот раздел сейчас пустой. Заполняется в Milestone 6 точными командами и версиями которые реально сработали.

## 10.1 Final versions (частично — обновляется по мере прохождения milestones)
- React Native: 0.85.2
- uniffi: `=0.31.0` (exact pin, aligned with ubrn workspace)
- uniffi-bindgen-react-native: `0.31.0-2` (npm latest)
- cargo-ndk: 4.1.2 (`cargo install cargo-ndk`)
- Rust Android targets: `aarch64-linux-android`, `x86_64-linux-android` (`rustup target add ...`)
- Android NDK: 27.1.12297006 (auto-установлен Gradle; для cargo-ndk выставлять inline `ANDROID_NDK_HOME` чтобы синхронизировать toolchain)
- AGP: 8.12.0 (resolved by RN gradle plugin)
- Gradle: 8.13 (не 9.x!)
- Android Build-Tools: 36.0.0
- Android Platform: 36 (compileSdk + targetSdk; minSdk = 24)
- Xcode: TBD (M5)
- Node: 24.11.1
- Java: OpenJDK 21.0.10 (Android Studio bundled)

## 10.2 Step-by-step reproduction

Воспроизведение состояния M3 close (commit `f4580c1` на `main`).

### Pre-requisites (см. также §2.1)

| Tool | Version | Установка |
|------|---------|-----------|
| Node | 24+ | nvm-windows |
| npm | 11+ | вместе с Node |
| Rust stable | 1.85+ | rustup |
| cargo-ndk | 4.1.2 | `cargo install cargo-ndk` |
| Rust Android targets | aarch64-linux-android, x86_64-linux-android | `rustup target add aarch64-linux-android x86_64-linux-android` |
| Android Studio | 2024+ | официальный сайт |
| Android SDK Platform | 36 (compileSdk + targetSdk; minSdk = 24) | через SDK Manager |
| Android NDK | 27.1.12297006 | auto-installed Gradle |
| Java | 17 или 21 (OpenJDK) | Android Studio bundled |

### Шаги

```bash
# 1. Clone + checkout main (M3 уже merged через PR #10)
git clone git@github.com:temrjan/rustok.git
cd rustok                                        # CRITICAL: путь ASCII-only (W1)
git checkout main
git log --oneline -3                             # ожидаем: f4580c1 Merge PR #10

# 2. Verify Rust workspace builds
cargo test --workspace                           # 38 unit + 2 doctests pass

# 3. Manual: создать mobile/android/local.properties (gitignored, W4)
#    Содержит путь к Android SDK
echo "sdk.dir=C\:\\Users\\<user>\\AppData\\Local\\Android\\Sdk" \
     > mobile/android/local.properties

# 4. NPM install (root — workspaces hoist deps в <root>/node_modules)
npm install                                      # 864+ packages, 30-60s

# 5. W7 workaround: удалить prettier bash-shim после каждого npm install
#    Иначе ubrn:android упадёт с Os error 193 на format_directory step
rm node_modules/.bin/prettier                    # оставить .cmd / .ps1

# 6. Generate uniffi bridge (Android only; iOS — на Mac в M5)
ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.1.12297006" \
  npm run ubrn:android --workspace=react-native-rustok-bridge -- --release
# Cold cache: ~5-10 min (rustok-core + alloy + revm — heavy compile)
# Warm cache: ~30s

# 7. Verify generated artifacts
ls packages/react-native-rustok-bridge/android/src/main/jniLibs/
# expect: arm64-v8a/librustok_mobile_bindings.a
#         x86_64/librustok_mobile_bindings.a

# 8. Gradle assembleDebug (PowerShell на Windows из W3)
cd mobile/android
.\gradlew.bat app:assembleDebug                  # ~3-4 min cold, ~5-30s warm

# 9. Verify APK
ls app/build/outputs/apk/debug/app-debug.apk     # ~144 MB
unzip -l app/build/outputs/apk/debug/app-debug.apk | grep librustok
# expect: lib/arm64-v8a/libreact-native-rustok-bridge.so
#         lib/x86_64/libreact-native-rustok-bridge.so

# 10. (Optional) TypeScript verify
cd ../../mobile && npx tsc --noEmit              # clean, exit 0

# ---- M4: device run (продолжение, после M3 APK build) ----

# 11. Подключить Android phone по USB, USB Debugging on
adb devices                                      # ожидаем "<id> device" (не unauthorized)

# 12. Установить APK на устройство
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk
# expect: Success
# Xiaomi: разблокировать phone, принять диалог установки (W6).
#   В нашей сессии 2026-04-30 диалог НЕ появился — вероятно из-за того что предыдущая
#   debug-сборка под `com.rustok` уже была installed. Если W6 срабатывает —
#   `adb uninstall com.rustok.app` (если осталось от Tauri) и `adb uninstall com.rustok`
#   перед повторным install.

# 13. Metro server (отдельный терминал) — workspace-aware конфиг см. mobile/metro.config.js
cd mobile && npx react-native start --port 8081 --reset-cache
# expect: "Welcome to Metro" + "BUNDLE 100%" после первой загрузки app
# --reset-cache обязателен ПЕРВЫЙ раз после install / после изменения metro.config.js

# 14. Reverse port для физ. устройства (W5)
adb reverse tcp:8081 tcp:8081

# 15. Запустить app на phone, тапнуть "Generate"
# Verify (per docs/M4-TASK-DESCRIPTION.md §3):
#   3.4 — ровно 12 слов на экране
#   3.5 — все слова английские lowercase, BIP-39 wordlist
#   3.6 — checksum валиден (теория ниже в шаге 16)
#   3.7 — несколько нажатий → разные mnemonic'и
#   3.8 — cold call <100ms

# 16. (Optional) Эмпирическая checksum-проверка
# Открыть https://iancoleman.io/bip39/ → вставить полученную фразу → "Valid".
# ⚠️ БЕЗОПАСНОСТЬ: используй валидатор ТОЛЬКО для тестовой проверки. Этот mnemonic
# никогда не использовать для real wallet — он скомпрометирован экспозицией в
# браузере / clipboard / DOM history.
```

### Time on cold cache vs warm

| Step | Cold | Warm |
|------|------|------|
| `npm install` | ~60s | ~10s |
| `ubrn:android --release` | ~5-10 min | ~30s |
| `gradle assembleDebug` | ~3-4 min | ~5-30s |
| **Total full reproduction** | ~10-15 min | ~1-2 min |

Heavy cold-time consumers: rustok-core dependencies (alloy-rs, revm, tokio) — ~5-7 min compile в release mode под Android targets.

### Known workarounds (см. §10.3)

- W1-W6 (M2 era): non-ASCII path, Gradle 9, gradlew.bat, local.properties, adb reverse, Xiaomi install
- W7 (M3): prettier bash-shim removal after npm install (см. шаг 5 выше)
- W8 (M3): ubrn android scaffold AGP 8 fix — applied в commit `1f2fc2d` для bridge package; persists через `ubrn:clean`
- W9 (M3): RN gradle plugin path overrides — applied в commit `e531e9a` (`mobile/android/settings.gradle`) и `mobile/android/app/build.gradle` (`reactNativeDir`/`codegenDir`/`cliFile`)

## 10.3 Known issues / workarounds (обновляется по ходу)

**W1 — Non-ASCII project path (Windows):**
AGP падает с "project path contains non-ASCII characters" если путь содержит кириллицу.
Решение: проект должен лежать в ASCII-пути. Верный путь: `C:\Claude\projects\rustok\`

**W2 — Gradle 9.x несовместим с AGP 8.x:**
Ошибка: `JvmVendorSpec.IBM_SEMERU` не найден. Gradle 9.x убрал это поле.
Решение: `gradle-wrapper.properties` → `gradle-8.13-bin.zip`

**W3 — gradlew.bat не запускается из Git Bash:**
`react-native run-android` вызывает `gradlew.bat` без `.\`, что не работает в bash.
Решение: запускать `.\gradlew.bat app:installDebug` из PowerShell напрямую.

**W4 — local.properties не создаётся автоматически:**
Gradle не находит Android SDK без `mobile/android/local.properties`.
Создать вручную: `sdk.dir=C\:\\Users\\omadg\\AppData\\Local\\Android\\Sdk`

**W5 — Metro недоступен на физ. устройстве:**
"Unable to load script" при запуске на телефоне через USB.
Решение: `adb reverse tcp:8081 tcp:8081` перед запуском приложения.

**W6 — Установка APK отклоняется (Xiaomi):**
`INSTALL_FAILED_USER_RESTRICTED` — на экране появляется диалог подтверждения.
Решение: разблокировать телефон перед `adb install`, принять диалог вручную.

**W7 — ubrn TS-форматирование падает на Windows (`Os error 193`):**
ubrn (0.31.0-2) вызывает `Command::new("<root>/node_modules/.bin/prettier")` без расширения — на Windows это bash-shim, не PE-binary. `CreateProcess` падает с error code 193 («%1 is not a valid Win32 application»). Stack: `ubrn_bindgen::bindings::gen_typescript::util::format_directory` → `ubrn_common::commands::run_cmd_quietly`. Upstream: `jhugman/uniffi-bindgen-react-native#302` (open).
Причина: `fmt::prettier()` использует `resolve(out_dir, "node_modules/.bin/prettier")` без extension lookup, в отличие от `clang_format()` в том же файле который использует `which::which("clang-format")`.
Решение: после каждого `npm install` удалять bash-shim:
```bash
rm node_modules/.bin/prettier
```
(`prettier.cmd` и `prettier.ps1` оставлять — другие тулы их используют через PATH). ubrn попадает в else-ветку с `eprintln!("No prettier found...")` — graceful fallback, bindings генерируются без TS-форматирования. Prettier можно прогнать вручную после генерации: `npx prettier --write packages/react-native-rustok-bridge/src/`.
TODO upstream: PR в ubrn заменив `resolve(...)` на `which::which("prettier")` для Windows-aware extension lookup.

**W8 — ubrn android scaffold с двумя AGP 8.x блокерами:**
`packages/react-native-rustok-bridge/android/build.gradle` (генерируется ubrn) содержит две несовместимости с AGP 8.x:
1. Внутри `supportsNamespace()` ветки (AGP 7.3+) указано `manifest.srcFile "src/main/AndroidManifestNew.xml"` — этот файл ubrn НЕ генерирует, есть только `AndroidManifest.xml`.
2. `AndroidManifest.xml` содержит `package="com.rustok.bridge"` — error в AGP 8.x когда `namespace` объявлен в build.gradle (deprecated с AGP 7.4, removed/error в 8.0+).
У нас AGP 8.12.0 → оба блокера активны → Gradle assemble упадёт без правки.
Решение (применять при первом smoke build, persists через `ubrn:clean` так как scaffold-файлы не нюкаются):
- `packages/react-native-rustok-bridge/android/build.gradle`: убрать блок `sourceSets { main { manifest.srcFile "src/main/AndroidManifestNew.xml" } }` внутри namespace-ветки — AGP default = `src/main/AndroidManifest.xml`.
- `packages/react-native-rustok-bridge/android/src/main/AndroidManifest.xml`: убрать атрибут `package="com.rustok.bridge"` (namespace остаётся в build.gradle).
TODO upstream: report bug в ubrn о несоответствии scaffold и AGP 8 conventions.

**W9 — RN gradle plugin paths assume `<app>/node_modules` (npm workspaces hoist в `<repo-root>/node_modules`):**
RN gradle plugin defaults жёстко прибиты к `<app>/node_modules/...` (для классической single-package структуры). В npm workspaces deps хойстятся в `<repo-root>/node_modules/`, поэтому defaults не находят файлы. Два failure modes:
1. `mobile/android/settings.gradle:1` — `pluginManagement { includeBuild("../node_modules/@react-native/gradle-plugin") }` resolve'ится к `mobile/node_modules/@react-native/gradle-plugin` (не существует). Error: «Included build does not exist».
2. После фикса (1), `apply plugin: "com.facebook.react.rootproject"` падает с «`mobile/node_modules/react-native/ReactAndroid/gradle.properties` does not exist» — RN root project plugin читает `reactNativeDir` extension с convention `root.dir("node_modules/react-native")` (см. `@react-native/gradle-plugin/.../ReactExtension.kt:36-39`).
Решение (две правки):
- `mobile/android/settings.gradle`: оба `includeBuild` пути `../node_modules/...` → `../../node_modules/...` (settings.gradle лежит в `mobile/android/`, `../..` = repo root).
- `mobile/android/app/build.gradle`: внутри `react { ... }` блока добавить overrides:
  ```groovy
  reactNativeDir = file("../../../node_modules/react-native")
  codegenDir = file("../../../node_modules/@react-native/codegen")
  cliFile = file("../../../node_modules/react-native/cli.js")
  ```
  (`app/build.gradle` лежит в `mobile/android/app/`, `../../..` = repo root.) `react { reactNativeDir }` propagat'ится в root project plugin через `ReactPlugin.kt:67`.
Verify: `gradlew app:assembleDebug` → `BUILD SUCCESSFUL`, APK содержит `lib/{arm64-v8a,x86_64}/libreact-native-rustok-bridge.so`.

## 10.4 Performance baseline
- **Cold call latency: <100ms** (M4, 2026-04-30 — Xiaomi `JFLFG6MZSSL7WCF6`). Mnemonic появляется на экране без перцептивной задержки сразу после tap. Точное измерение через Android Profiler / `tracing` instrumentation — Phase 4-5.
- **Hot call latency: <50ms** (M4). Повторные нажатия — мгновенно, без visible delay. Без instrument detection границы.
- APK size (debug, all 4 ABIs): **144 MB** (M3, includes libreactnative ~22 MB × 4 ABIs + librustok bridge ~12 MB × 2 ABIs + Hermes runtime). Release builds с ABI splits будут существенно меньше.
- Cold build time (Gradle assembleDebug, hot Rust + npm cache): **3m 35s** (M3)
- IPA size: TBD MB (M5 — на Mac)

---

**Конец документа.**
