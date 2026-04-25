# Rustok Redesign — Theme Parity Plan

> **Что это:** Single source of truth для следующего этапа редизайна
> Rustok — добавление theme switching (light/dark) с гибридной
> стратегией. Любая сессия читает этот документ первым и может
> продолжить с любого пункта.
>
> **Создан:** 2026-04-25 после аудита `rust-design` репо и текущего кода.
> **Текущий статус:** План одобрен, реализация не начата.

---

## 0. Стиль работы — читать перед каждой сессией

### Контекст проекта

`Rustok` — open-source Rust Ethereum-кошелёк (Tauri 2.0 + Leptos 0.7 +
alloy-rs 1.8). Solo-разработчик. Каждый коммит должен попадать в `main`
через CI зелёного цвета. **Никакого редактирования через SSH**, всё локально.

### Главные документы (по приоритету чтения)

1. **`CLAUDE.md`** — короткий entrypoint, обновлять не надо.
2. **Этот файл** — план theme parity, single source of truth.
3. `docs/REDESIGN.md` — общий контекст и история редизайна.
4. `docs/SESSION-NEXT.md` — следующие задачи после theme parity.
5. `~/codex/standards/architecture.md` + `pipeline.md` — codex базовые
   стандарты.

### Стек подкачивается через скиллы

| Скилл | Когда запускать | Зачем |
|---|---|---|
| `/codex` | До написания любого нетривиального кода | Архитектура + pipeline стандарты |
| `/rust` (с `web/leptos`) | До правок в `app/src/src/**` или `crates/core` | CORE Rust + Leptos 0.7 + Tauri-bridge паттерны |
| `/rust-review` | После завершения блока изменений | Senior-style review, finds что clippy не видит |
| `/check` | После каждого «вот мой план» / «вот моё решение» | Сам себе критик: проверь факты, edge cases, простоту |
| `/ultrareview` | По желанию пользователя на готовый PR/branch | Multi-agent cloud review (billed) |

### Workflow

| Тип задачи | Цикл |
|---|---|
| **LIGHT** (1 файл, конфиг, doc) | Изучи → Сделай → `/check` → diff → коммит → push → CI |
| **FULL** (фича, multi-file, security, infrastructure) | Изучи → `/codex` → План → `/check` → `/rust` → Реализуй → `/rust-review` → diff → коммит → push → CI |

**Неизменное ядро:** `/check` после каждого плана + `git diff` перед
коммитом + ждать CI зелёного. Никогда не «фикс попутно».

### Команды старта новой сессии

```bash
cd /Users/avangard/Workspace/projects/rustok

# 1. Стейт
git status                   # должно быть clean
git log --oneline -10
cargo test --workspace       # 112+ зелёных
gh run list --limit 3        # CI status

# 2. Прочитать этот документ полностью.

# 3. Посмотреть прогресс:
grep -c "^- \[x\]" docs/REDESIGN-AUDIT.md   # сколько чек-боксов закрыто
grep -c "^- \[ \]" docs/REDESIGN-AUDIT.md   # сколько осталось

# 4. Запустить нужные скиллы (см. таблицу выше).

# 5. Перед коммитом:
cargo check --target wasm32-unknown-unknown   # frontend
cargo check -p rustok-desktop                  # tauri backend
cargo clippy --workspace                       # workspace lint
cargo test --workspace
git diff                                       # без попутных правок
```

### Android тест на эмуляторе

```bash
source ~/.zshrc                               # ANDROID_HOME, JAVA_HOME, NDK_HOME
$ANDROID_HOME/emulator/emulator -avd Pixel_8 -no-snapshot-load &
cd app && cargo tauri android build --apk --target aarch64 --split-per-abi
adb install -r src-tauri/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release.apk
adb shell am force-stop com.rustok.app
adb shell am start -n com.rustok.app/.MainActivity
adb logcat -s rustok:V
```

**Тестовый PIN на эмуляторе:** `111111`. Адрес тестового кошелька
`0x542E…B1B0` (актуальный на 2026-04-24).

### Правила качества (ядро)

- **Read before Write.** Перед изменением файла — прочитай его + 2-3
  похожих + найди вызовы через `grep`.
- **Verify, don't guess.** Любой API библиотеки — `context7` или WebSearch.
  Не пиши по памяти.
- **One thing at a time.** Закончи задачу полностью → `cargo check` →
  `cargo clippy` → `/check` → следующая.
- **Self-review.** Перечитай свой `git diff`, особенно `unsafe` и
  `.unwrap()`.
- **Conventional commits**: `feat(ui)`, `fix(ui)`, `style(ui)`,
  `chore(deps)`, `docs(...)`. CI падает на пустых сообщениях.
- **Коммит-сообщение** должно объяснять «почему», не «что».

### Не делать

- Не возвращать `rustls-platform-verifier` (см. `docs/SESSION-NEXT.md`
  историю TLS бага).
- Не редактировать через SSH/cargo прямо на сервере.
- Не пушить с красным CI — фиксить локально сначала.
- Не делать «попутные улучшения» в чужих файлах посреди фичи.

---

## 1. Контекст этого документа

### Что уже сделано (фундамент закрыт)

Dark-редизайн всех экранов завершён. Каждый экран портирован из
`rust-design` репо и работает на Android emulator.

| Слой | Статус |
|---|---|
| Foundation (`tokens.rs`, `components/{icons,button,logo,dark_shell}.rs`) | ✅ |
| Onboarding (welcome, wallet wizard, restore, unlock) | ✅ light статика |
| Dark screens (home, receive, activity, settings, send, analyze) | ✅ navy hardcoded |
| Shell (body bg + bottom tab bar) | ✅ navy + periwinkle |
| Tauri-plugin-clipboard-manager | ✅ |
| Fixes: QR center, overscroll, safe-area math, Android keyboard | ✅ |

CI зелёный на `7c2381e`. APK на эмуляторе с PIN `111111`.

### Найденная проблема (этот документ закрывает)

1. **Перепад «light Unlock → dark Home»** — пользователь видит каждый
   день. Технически unlock — recurring auth screen, не one-time
   onboarding.
2. **Нет theme switcha** — у нас 64 точки хардкода `BG_DARK`,
   `SURFACE_DARK`, `TEXT_LIGHT`, `BORDER_DARK`, `CARD_DARK` на 8 файлов.
   Инвертировать тему через одно действие невозможно.
3. **Нет Settings → Light mode toggle** — в `rust-design/screens/dark/
   settings.rs` он есть, у нас отсутствует.
4. **Нет Splash экрана** — в rust-design есть 1.4s брендовый шлюз.
5. **Нет CreateSuccess экрана** — у нас сразу `navigate("/")` после
   import. В rust-design это отдельный `CreateSuccessScreen` (живёт
   в `create_verify.rs`, не в отдельном файле).

### Гибридная стратегия (выбрана 2026-04-25)

После анализа `rust-design` (`screens/onboarding/mod.rs`: «Onboarding
screens — always light-themed») выяснилось: **rust-design не покрывает
наш use case Unlock-экрана** — у них упрощённая state-machine без отдельного
auth-экрана. Их правило «onboarding always light» рассчитано на one-time
first-impression и не отвечает на recurring authentication.

**Правило:** граница не «onboarding vs main app», а **one-time vs recurring**.

| Категория | Экраны | Подход | Почему |
|---|---|---|---|
| **One-time onboarding** | Splash, Welcome, SetPasscode, ConfirmPasscode, CreateReveal, CreateVerify, BackupConfirm, CreateSuccess, Restore | static **light** | First-time бренд-experience; theme preferences ещё не существуют; пользователь видит этот flow один раз; на light фоне seed-фраза легче читается |
| **Recurring** | **Unlock** + Home + Receive + Activity + Settings + Send + TxGuard | **switchable** через `ThemeKind` context | Recurring screens; перепад болит каждый день; единая тема снимает cognitive jolt |

**Settings → Light mode toggle** переключает только recurring screens.
Onboarding всегда светлый.

---

## 2. Эталонная цепочка экранов (как должно быть)

```
Cold start
   │
   ▼
Splash (1.4s, dark static, anti-FOUC аура)
   │
   ├── state == Uninit  ──────────────► Welcome
   ├── state == Locked  ──────────────► Unlock (theme-aware)
   └── state == Unlocked ─────────────► Home (theme-aware)


Welcome (light static)
   │
   ├── "Create new wallet" ──► SetPasscode (light)
   │                              └─► ConfirmPasscode (light)
   │                                    └─► CreateReveal (light, blurred phrase)
   │                                          └─► CreateVerify (light, 3 quiz)
   │                                                └─► BackupConfirm (light, 3 checks)
   │                                                      └─► CreateSuccess (light) ──► Home
   │
   └── "I already have a wallet" ──► Restore (light, phrase + PIN) ──► Home


Home (theme-aware)
   ├── Send → Preview → Result → Home
   ├── Receive
   ├── Scan (TxGuard)
   └── Bottom tab → Activity / Settings (Settings → Light mode toggle)
```

---

## 3. Текущее состояние vs эталон

| # | Экран | rust-design | наш сейчас | gap |
|---|---|---|---|---|
| 1 | **Splash** | dark static, 1.4s auto-advance | ❌ нет экрана | **создать** |
| 2 | Welcome | dark static | dark static | ✅ |
| 3 | SetPasscode | light static | light static | ✅ |
| 4 | ConfirmPasscode | light static | light static (Step::ConfirmPin в wallet.rs) | ✅ |
| 5 | CreateReveal (blurred phrase) | light static | light static (Step::ShowPhrase) | ✅ |
| 6 | CreateVerify (3 quiz) | light static | light static (Step::Quiz) | ✅ |
| 7 | BackupConfirm (3 checks) | light static | light static (Step::BackupConfirm) | ✅ |
| 8 | **CreateSuccess** | light static (в `create_verify.rs`) | ❌ нет — instant `navigate("/")` | **портировать** |
| 9 | Restore | light static | light static | ✅ |
| 10 | **Unlock** | (нет в эталоне) | light **hardcoded** | **сделать switchable** |
| 11 | Home | switchable (CSS vars) | dark **hardcoded** | **на CSS vars** |
| 12 | Send | switchable (CSS vars) | dark hardcoded | **на CSS vars** |
| 13 | Receive | switchable | dark hardcoded | **на CSS vars** |
| 14 | Activity | switchable | dark hardcoded | **на CSS vars** |
| 15 | Settings | switchable + Light mode toggle | dark hardcoded, **toggle нет** | **на CSS vars + toggle** |
| 16 | TxGuard (`/scan` → analyze) | switchable | dark hardcoded | **на CSS vars** |
| 17 | Bottom tab bar | `var(--rw-tab-bg)` | navy hardcoded | **на CSS vars** |

---

## 4. Theme infrastructure — детальный план

### 4.1. CSS-переменные в `app/src/index.html`

Вставить в `<head>`:

```html
<!-- Anti-FOUC: set data-theme before CSS is applied -->
<script>
  (function(){
    try {
      var t = localStorage.getItem('rustok.theme');
      if (t === 'light') document.documentElement.setAttribute('data-theme','light');
    } catch(e) {}
  })();
</script>
<meta name="theme-color" content="#0A1123"/>
<style>
  :root {
    --rw-bg:         #0A1123;
    --rw-surface-1:  #141A33;
    --rw-surface-2:  #1C2244;
    --rw-border:     #242B4C;
    --rw-text:       #FFFFFF;
    --rw-card:       linear-gradient(160deg, #141A33 0%, #0D1328 100%);
    --rw-switch-off: rgba(255,255,255,0.14);
    --rw-tab-bg:     rgba(10,17,35,0.92);
    --rw-neutral-mid: #959BB5;
  }
  :root[data-theme="light"] {
    --rw-bg:         #F6F7FB;
    --rw-surface-1:  #FFFFFF;
    --rw-surface-2:  #F0F1F8;
    --rw-border:     #E5E8F2;
    --rw-text:       #0A1123;
    --rw-card:       linear-gradient(160deg, #FFFFFF 0%, #F6F7FB 100%);
    --rw-switch-off: rgba(0,0,0,0.12);
    --rw-tab-bg:     rgba(246,247,251,0.92);
    --rw-neutral-mid: #6B7088;
  }
</style>
```

### 4.2. `app/src/src/tokens.rs` — добавить `pub mod css`

После существующих констант:

```rust
/// CSS variable references for the switchable theme.
///
/// Use these on the recurring app surfaces (Unlock + main app screens)
/// where the user expects light/dark to follow Settings toggle.
/// One-time onboarding screens stay on the static `t::*` constants.
pub mod css {
    pub const BG: &str = "var(--rw-bg)";
    pub const SURFACE: &str = "var(--rw-surface-1)";
    pub const SURFACE_2: &str = "var(--rw-surface-2)";
    pub const BORDER: &str = "var(--rw-border)";
    pub const TEXT: &str = "var(--rw-text)";
    pub const CARD: &str = "var(--rw-card)";
    pub const SWITCH_OFF: &str = "var(--rw-switch-off)";
    pub const TAB_BG: &str = "var(--rw-tab-bg)";
    pub const NEUTRAL_MID: &str = "var(--rw-neutral-mid)";
}
```

### 4.3. `app/src/src/app.rs` — `ThemeKind` enum + context + persist

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind { Dark, Light }

const STORAGE_KEY_THEME: &str = "rustok.theme";

fn load_theme() -> ThemeKind {
    use web_sys::window;
    let win = match window() { Some(w) => w, None => return ThemeKind::Dark };
    let storage = match win.local_storage().ok().flatten() {
        Some(s) => s, None => return ThemeKind::Dark,
    };
    match storage.get_item(STORAGE_KEY_THEME).ok().flatten().as_deref() {
        Some("light") => ThemeKind::Light,
        _ => ThemeKind::Dark,
    }
}

// В App component:
let theme = RwSignal::new(load_theme());
provide_context(theme);

Effect::new(move |_| {
    let (attr, color) = match theme.get() {
        ThemeKind::Dark  => ("dark",  "#0A1123"),
        ThemeKind::Light => ("light", "#F6F7FB"),
    };
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(STORAGE_KEY_THEME, attr);
        }
        if let Some(doc) = win.document() {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-theme", attr);
            }
            if let Ok(Some(meta)) = doc.query_selector("meta[name=\"theme-color\"]") {
                let _ = meta.set_attribute("content", color);
            }
        }
    }
});
```

### 4.4. `app/src/src/pages/settings.rs` — Light mode toggle

В секцию **Appearance** (новая, перед Actions):

```rust
let theme = use_context::<RwSignal<ThemeKind>>()
    .expect("ThemeKind context missing");
let light_mode = RwSignal::new(theme.get_untracked() == ThemeKind::Light);

Effect::new(move |_| {
    theme.set(if light_mode.get() { ThemeKind::Light } else { ThemeKind::Dark });
});

view! {
    <SectionTitle label="Appearance"/>
    <Section>
        <ToggleRow
            label="Light mode"
            caption=move || if light_mode.get() {
                "Light surfaces"
            } else {
                "Dark surfaces (default)"
            }
            icon=IconKind::Eye  // добавить в icons.rs если нужно
            on=light_mode
            on_click=Callback::new(move |()| light_mode.set(!light_mode.get_untracked()))
        />
    </Section>
}
```

---

## 5. Чек-лист (атомарные коммиты)

### A — Theme infrastructure (без визуальных изменений)

- [ ] `app/src/index.html`: добавить `<meta theme-color>`, Anti-FOUC
      script, `<style>` с CSS-переменными `--rw-*` (dark default + light
      override).
- [ ] `app/src/src/tokens.rs`: добавить `pub mod css { … }` (8-9 переменных).
- [ ] `app/src/src/app.rs`: `ThemeKind` enum + `load_theme()` +
      `provide_context(theme)` + Effect для persist + sync `data-theme`
      атрибута и `<meta theme-color>`.
- [ ] `cargo check` + `cargo clippy` зелёные.
- [ ] **Коммит:** `feat(ui): add theme infrastructure (CSS vars + ThemeKind)`.

### B — Migrate recurring screens на CSS vars

Для каждого файла: заменить `t::BG_DARK` → `t::css::BG`,
`t::SURFACE_DARK` → `t::css::SURFACE`, `t::SURFACE_DARK_2` →
`t::css::SURFACE_2`, `t::BORDER_DARK` → `t::css::BORDER`,
`t::TEXT_LIGHT` → `t::css::TEXT`, `t::CARD_DARK` → `t::css::CARD`.
Семантика (`t::ACCENT`, `t::SUCCESS`, `t::DANGER`, `t::WARN`,
`t::SUCCESS_BG` etc.) — НЕ трогать.

- [ ] `app/src/src/components/dark_shell.rs` (3 точки) — попутно
      переименовать в `app_shell.rs`, обновить mod.rs / re-exports.
- [ ] `app/src/src/pages/home.rs` (12 точек).
- [ ] `app/src/src/pages/receive.rs` (6 точек).
- [ ] `app/src/src/pages/activity.rs` (7 точек).
- [ ] `app/src/src/pages/settings.rs` (8 точек).
- [ ] `app/src/src/pages/send.rs` (17 точек).
- [ ] `app/src/src/pages/analyze.rs` (10 точек).
- [ ] `app/src/src/pages/unlock.rs` — переход с `BG = #F6F7FB` хардкод
      на `t::css::BG`. Убрать локальные константы `BG`, `BRAND`, `MUTED` —
      заменить на `t::css::*`.
- [ ] `cargo check` зелёный.
- [ ] **Финальный grep:** `grep -rn "BG_DARK\|SURFACE_DARK\|TEXT_LIGHT\|
      CARD_DARK\|BORDER_DARK\|SURFACE_DARK_2" app/src/src/pages app/src/
      src/components` должен быть **пуст** (или показать только comment).
- [ ] **Коммит:** `feat(ui): switch recurring screens to CSS variables`.

### C — Settings → Light mode toggle

- [ ] `pages/settings.rs`: добавить `Appearance` секцию с
      `ToggleRow "Light mode"`. Использовать `use_context::<RwSignal<
      ThemeKind>>()`. Effect синхронизирует.
- [ ] Добавить `IconEye` или похожую в `icons.rs` если ещё нет.
- [ ] Manual test: тоггл переключает все recurring screens live (без
      перезапуска).
- [ ] **Коммит:** `feat(ui): light mode toggle in settings`.

### D — Bottom tab bar + body на CSS vars

- [ ] `app/src/styles/main.css`:
  ```css
  body { background: var(--rw-bg); color: var(--rw-text); }
  .tab-bar {
      background: var(--rw-tab-bg);
      border-top: 1px solid var(--rw-border);
      backdrop-filter: blur(20px);
      -webkit-backdrop-filter: blur(20px);
  }
  .tab-bar a { color: var(--rw-neutral-mid); }
  /* active a[aria-current="page"] остаётся ACCENT periwinkle */
  ```
- [ ] **Коммит:** `style(ui): bottom tab bar follows theme`.

### E — Splash экран

- [ ] Изучить `rust-design/src/screens/onboarding/splash.rs` (1.8 KB).
- [ ] Создать `app/src/src/pages/splash.rs`:
  * Static dark (`t::BG_DARK`).
  * Логотип + wordmark + 3 пульсирующих dots (`@keyframes rw-pulse`).
  * Auto-advance: `set_timeout(1400)` с one-shot guard. После timeout
    смотрим `WalletState`:
    * `Uninit` → `/welcome`
    * `Locked` → `/unlock`
    * `Unlocked` → `/`
- [ ] Добавить `@keyframes rw-pulse` в `main.css`.
- [ ] `app/src/src/app.rs`: route `path!("/splash")` + изменить дефолт.
      Все nav_guards оставить как есть (Splash просто прокладка перед
      первым роутом).
- [ ] Manual test: cold start → видим Splash 1.4s → правильный target.
- [ ] **Коммит:** `feat(ui): splash screen`.

### F — CreateSuccess экран

- [ ] Изучить `CreateSuccessScreen` в
      `rust-design/src/screens/onboarding/create_verify.rs`.
- [ ] В `pages/wallet.rs`: добавить `Step::Success` в `enum Step`.
      После удачного `import_wallet_from_mnemonic` ставить
      `step.set(Step::Success)` вместо `navigate("/")`.
- [ ] UI Success: green check disc + "Wallet ready" + "Continue" CTA.
      Continue → `auth_state.set(Unlocked) + navigate("/")`.
- [ ] То же самое для `pages/restore.rs` (после успешного import).
- [ ] **Коммит:** `feat(ui): create success screen after wallet creation`.

### G — Manual QA проход (не коммит, blocker для merge)

Для **Dark theme** (default):
- [ ] Cold start → Splash (1.4s) → Welcome (dark hero).
- [ ] Welcome → Create → SetPasscode → Confirm → Reveal → Verify →
      BackupConfirm → CreateSuccess → Home (dark).
- [ ] Welcome → Restore → PIN → Home (dark).
- [ ] Home → Send → Preview → Result → Home.
- [ ] Home → Receive (QR centered).
- [ ] Home → Scan (TxGuard).
- [ ] Bottom tab → Activity / Settings.
- [ ] Settings → toggle Light mode → весь recurring app становится
      light без перезапуска. Onboarding пройти заново — он остаётся light.
- [ ] Force-stop + relaunch → выбранная тема сохранена (localStorage).
- [ ] Anti-FOUC: при light theme cold start нет вспышки dark до WASM
      bootstrap.

Для **Light theme**:
- [ ] Toggle в Settings → Light.
- [ ] Lock wallet → Unlock screen в light.
- [ ] Все recurring screens читаются (контраст текст/фон ≥ AA).
- [ ] Periwinkle accent (`#8387C3`) на light surface — проверить что
      кнопки видны (на rust-design тестировано — должно работать).

### H — Документация (один коммит после G)

- [ ] `docs/REDESIGN.md` § 5: добавить «Сессия 2026-04-25 — theme
      parity».
- [ ] `docs/SESSION-NEXT.md`: убрать пункт «navy body + tab bar», обновить
      § 1 на «theme switching реализован, дальше Phase 4 / iOS / price
      feed».
- [ ] `docs/COMPONENTS.md`: pages bumped to 12 (+Splash); добавить
      ThemeKind в список contexts; упомянуть `tokens::css` модуль.
- [ ] `README.md`: упомянуть light/dark theme + Settings toggle.
- [ ] Memory `~/.claude/.../memory/rustok-redesign.md`: обновить статус.
- [ ] **Коммит:** `docs: theme parity wrap-up`.

---

## 6. Acceptance criteria

Перед закрытием задачи **все** должны быть `true`:

1. ✅ `cargo test --workspace` зелёный.
2. ✅ `cargo clippy --workspace` без warnings.
3. ✅ `grep -rn "BG_DARK\|SURFACE_DARK\|TEXT_LIGHT\|CARD_DARK\|
   BORDER_DARK\|SURFACE_DARK_2" app/src/src/pages app/src/src/components`
   возвращает только onboarding pages (welcome, wallet, restore — там
   light static оставлен) или пусто.
4. ✅ Settings → Light mode toggle → весь recurring app переключается
   без перезапуска.
5. ✅ Перезапуск приложения → тема сохранена (localStorage).
6. ✅ Cold start с light theme → нет dark FOUC.
7. ✅ Splash → правильный target (Welcome/Unlock/Home) в зависимости от
   `WalletState`.
8. ✅ После CreateSuccess → Home, без skipped экранов.
9. ✅ Android emulator + физический Pixel: оба flow проходят.
10. ✅ CI зелёный, документация обновлена.

---

## 7. Риски и нерешённые вопросы

- **CSS vars + Android WebView 124+** — поддерживается с 2018, но
  `var()` в `linear-gradient()` (наш `--rw-card`) — проверить отдельно.
  Backup plan: вынести gradient как два класса `.card-dark` / `.card-light`.
- **Splash на каждый запуск vs только cold start** — rust-design делает
  на каждый запуск. Mobile UX argument против: Unlocked user не должен
  ждать splash. Решение: показывать только когда `state == Loading`,
  редиректить сразу при `Unlocked` (Effect в Splash, не auto 1.4s).
  Финальное решение оставлено на момент реализации (часть E).
- **Backup phrase contrast** — на light surface seed-фраза monospace
  читается. На dark surface (если переключим тоже) нужен `font-weight:
  600` минимум для AA-контраста. Поэтому wallet wizard остаётся light.
- **iOS симулятор** — пока не тестировано (нет $99 Apple Dev). Anti-FOUC
  и `localStorage` работают одинаково в WKWebView и Android WebView.
- **Tauri-mobile cold start** — WASM bootstrap занимает 1-2s на Android.
  Anti-FOUC script срабатывает за <50ms — пользователь видит сразу
  правильный фон.

---

## 8. Команды быстрого старта (после прочтения)

```bash
# Часть A — фундамент
$EDITOR app/src/index.html               # добавить style + anti-FOUC
$EDITOR app/src/src/tokens.rs            # + pub mod css
$EDITOR app/src/src/app.rs               # + ThemeKind + provide_context
cargo check --target wasm32-unknown-unknown
git add -p && git commit -m "feat(ui): add theme infrastructure …"

# Часть B — миграция recurring screens
for f in dark_shell home receive activity settings send analyze unlock; do
    # mass-replace через sed или вручную
    echo "migrating $f..."
done
cargo check --target wasm32-unknown-unknown
grep -rn "BG_DARK\|SURFACE_DARK\|TEXT_LIGHT\|CARD_DARK\|BORDER_DARK" \
    app/src/src/{pages,components} | grep -v -E "wallet\.rs|restore\.rs|welcome\.rs"
# должно быть пусто
git commit -m "feat(ui): switch recurring screens to CSS variables"

# Часть C — toggle
# ... etc
```

---

## 9. После завершения этого плана

Следующие задачи (в `docs/SESSION-NEXT.md`):

- Cloudflare Worker RPC proxy toggle.
- Phase 4: Cross-chain via Across Protocol.
- iOS TestFlight ($99 Apple Dev).
- v2 keystore + Show Recovery Phrase.
- Price feed (CoinGecko) → HomeVariant::Chart.
- v0.1.3 Play Console Internal Testing release.
