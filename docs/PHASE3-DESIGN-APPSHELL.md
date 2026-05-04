# PHASE 3 — Design system + AppShell

**Status:** Planning · not started · awaiting kickoff approval
**Created:** 2026-05-04
**Owner:** temrjan
**Source plan:** `docs/NATIVE-MIGRATION-PLAN.md` § Phase 3
**Predecessor:** Phase 2 closed 2026-05-01 (PR #13, 11 atomic commits, 227 tests, C1-C4 resolved)
**Successor:** Phase 4 — Onboarding flow (blocked by this phase)

---

## 1. Scope

### Включено
- **Design tokens** — colors, typography, spacing, radii (single source of truth)
- **Theming** — light / dark / system + manual override, persisted (synchronous MMKV read до первого render → no FOIT)
- **Component library** — Button, Input, Modal (bottom sheet), Toast, Spinner, Switch, PageHeader
- **AppShell** — safe-area aware layout wrapper, навигационная оболочка
- **Splash / init screen** — закрывает UI flash во время bridge async hydration
- **Navigation** — React Navigation v7: BottomTabs (Wallet / Activity / TxGuard / Settings) + native-stack для модальных flow
- **State stores** — `themeStore`, `uiStore`, `networkStore`, `walletStore` (Zustand 5 + MMKV persist with `version: 1`)
- **Routing logic** — три состояния app: `no_wallet` / `locked` / `unlocked` → правильный entry screen
- **NetworkBadge** — readonly badge с реальным `chainId` (компонент + связанный store, M4)
- **Stub screens** — placeholders для каждого таба + Welcome + UnlockPin (наполнятся в Phase 4-5)
- **Components dev screen** — `__DEV__`-only inventory page для smoke-проверки всего kit'a
- **a11y baseline** — labels, roles, контраст ≥ WCAG AA, respect `prefers-reduced-motion` + system font scaling, RTL-aware logical paddings
- **CI updates** — `.github/workflows/ci.yml` обновлён под новые npm deps + jest tests на компонентах
- **Phase 3 handoff doc** — `docs/PHASE3-HANDOFF.md` на close (стиль Phase 2)
- **README update** — overview `mobile/` структуры

### Не включено (defer)
- Onboarding screens (Welcome → KeepItSafe → ShowPhrase → Quiz → CreatePin → ConfirmPin) — **Phase 4**
- Реальный контент Wallet / Activity / TxGuard tabs — **Phase 5+**
- Сложные анимации (PinDots reveal, layout transitions) — **Phase 4**
- Полноценный network selector — **Phase 7** (Phase 3 даёт readonly `<NetworkBadge>`)
- WalletConnect, Hardware wallet, AI router — **Phase 5+**
- iOS smoke (доступно только из Mac-сессии; см. R3) — отдельный milestone **M5-iOS-Phase3**

---

## 2. Milestones

> Pattern совпадает с Phase 1/2: каждый milestone = 2-4 атомарных коммита, gate перед merge'ем.
>
> **Total scope:** ~10 commits across M1-M4 + close-out doc commit ≈ **11 atomic commits** — аналог Phase 2 (11 commits, 113 → 227 tests).

### M1 — Design tokens + theming foundation (2 commits)

**Goal:** NativeWind v4 настроен, переключение light/dark/system работает end-to-end без FOIT.

**Deliverables:**
- `mobile/tailwind.config.js` — токены (periwinkle `#8387C3`, accent `#3A3E6C`, muted `#8A8CAC`, semantic colors success/warn/danger, typography scale, spacing 4-base, radii)
- `mobile/src/theme/tokens.ts` — типизированный экспорт токенов для not-NativeWind кода
- `mobile/src/stores/themeStore.ts` — Zustand store, persist через MMKV с `version: 1`
- **Synchronous MMKV read до первого render** в `App.tsx` (закрывает R4 — theme flash)
- `App.tsx` — ThemeProvider + system-mode listener
- Тестовый theme switch внутри `__dev__/ComponentsScreen.tsx` (НЕ отдельный Settings stub — упрощение vs первого draft'а)
- **M1 spike pre-check:** verify актуальное `mobile/package.json` (какие из новых deps уже стоят), verify REVIEWER-CONSTITUTION v1.3 точные требования к sign-off (§6.8/§6.9/§9.8)

**Commits:**
- `feat(mobile): NativeWind v4 + design tokens`
- `feat(mobile): themeStore (light/dark/system) + sync MMKV persist`

**Gate:** переключение темы применяется без перезагрузки app, сохраняется между запусками, никакого flash на cold-start.

### M2 — Component library + dev screen (3 commits)

**Goal:** UI-kit готов для Phase 4 onboarding и для всех будущих экранов.

**Deliverables:**
- Primitives: `<Button variant="primary|secondary|ghost|danger" size="sm|md|lg">`, `<Input>` (text / password / error state), `<Spinner>`, `<Switch>` (native)
- Overlays: `<Modal>` поверх `@gorhom/bottom-sheet` (full-screen — отдельный variant), `<Toast>` через `react-native-toast-message`
- Layout: `<PageHeader>` (title + back-кнопка + optional right action)
- `mobile/src/__dev__/ComponentsScreen.tsx` — каталог всех компонентов в обоих темах для smoke-проверки

> **NetworkBadge перенесён в M4** — компонент бессмысленно делать без `networkStore`, иначе двойная работа.

**Commits:**
- `feat(mobile): primitive components (Button, Input, Spinner, Switch)`
- `feat(mobile): overlays (bottom sheet modal + toast) + page header`
- `feat(mobile): components dev screen`

**Gate:** все компоненты рендерятся корректно в light + dark, accessibility labels на всех интерактивных, dev screen открывается через hidden gesture внутри Settings tab (после M3) или через прямой route в DEV-сборке.

### M3 — AppShell + navigation skeleton (2 commits)

**Goal:** структура приложения с реальным state-based роутингом.

**Deliverables:**
- `<AppShell>` — `react-native-safe-area-context` + общая оболочка
- React Navigation v7 setup: `@react-navigation/native`, `@react-navigation/bottom-tabs`, `@react-navigation/native-stack`
- Bottom tabs: Wallet / Activity / TxGuard / Settings (placeholder screens с явным маркером "Phase 5 placeholder")
- Stack screens: Welcome (placeholder), UnlockPin (placeholder)
- `RootNavigator` с routing logic от walletStore:
  - `has_wallet === false` → Welcome stack
  - `has_wallet && !is_unlocked` → UnlockPin
  - `has_wallet && is_unlocked` → Tabs (Wallet)
- Deep-link config (стуб для Phase 6)
- **Settings tab** — содержит theme switcher (мигрирует из M1 dev surface), placeholder для остального

**Commits:**
- `feat(mobile): AppShell + react-navigation v7 setup`
- `feat(mobile): root navigator + state-based routing (3 states)`

**Gate:** все 4 таба переключаются native gestures на Android (Pixel 8 emulator + JFLFG6MZSSL7WCF6 Xiaomi), три ветки routing'a покрыты smoke-тестами вручную, system-back на Android корректно работает. **iOS swipe-back gate откладывается до M5-iOS-Phase3 (Mac session).**

### M4 — Stores + bridge wiring + init flow + CI (3 commits)

**Goal:** stores подключены к WalletHandle, cold-start app корректно определяет state, CI обновлён.

**Deliverables:**
- `walletStore` — address, balance, locked state, refresh actions, **error state** (на bridge throw → Toast notification)
- `networkStore` — chainId через `getChainId()`, refresh
- `uiStore` — `balanceHidden`, активные модалки
- `<NetworkBadge>` — теперь с реальным store (перенесено из M2)
- Hooks: `useWallet()`, `useNetwork()`, `useTheme()`, `useUI()` — типизированные обёртки над Zustand-селекторами
- App init flow: при cold-start `Splash` → bridge ready → `has_wallet()` + `is_wallet_unlocked()` → store hydration → правильный route без flash
- **CI updates:** `.github/workflows/ci.yml` — npm cache keys для новых deps, jest test step для `mobile/src/components/__tests__/` и `mobile/src/stores/__tests__/`

**Commits:**
- `feat(mobile): wallet/network/ui stores + NetworkBadge`
- `feat(mobile): app init flow + splash screen + bridge integration`
- `chore(ci): update workflows for Phase 3 mobile deps + jest`

**Gate:** cold-start корректно ведёт в одну из 3 веток без flash, NetworkBadge показывает текущий chainId, balance скрывается тумблером в Settings, bridge errors попадают в Toast (не crash).

---

## 3. Technical decisions

### 3.1 Navigation — React Navigation v7

**Choice:** `@react-navigation/native@7` + `@react-navigation/bottom-tabs@7` + `@react-navigation/native-stack@7`.

**Why:** bare RN 0.85.2 без Expo SDK; v7 = стандарт сообщества, full Fabric support, native gestures, deep-linking.

**Rejected:** Expo Router (требует Expo runtime, invasive); RN Navigation by Wix (steep learning curve, менее активный maintenance).

### 3.2 Theming — NativeWind v4 + Zustand

**Choice:** NativeWind v4 для `className`-стилей, Zustand store для themeMode (synchronous MMKV read).

**Why:** `dark:` variant + CSS-vars из коробки, tailwind-like API быстро итерируется.

**Rejected:** Restyle (boilerplate-heavy); plain StyleSheet (нет shared design language).

### 3.3 Component pattern — variants + composition

**Choice:** функциональные компоненты + `variant` prop через `class-variance-authority` (cva) с `clsx` для className concatenation.

**Why:** явные варианты, self-documenting API, совместимо с NativeWind.

### 3.4 State management — Zustand 5 + MMKV persist

**Choice:** Zustand 5 со связкой `react-native-mmkv`, persist с `version: 1` (под будущие миграции).

**Why:** уже использовался в Phase 1/2, MMKV значительно быстрее AsyncStorage, minimal API.

### 3.5 Modal pattern — bottom sheet first; RN Modal как acceptable fallback

**Choice:** `@gorhom/bottom-sheet@5` как основной overlay (mobile-native UX, единая инфраструктура для Phase 4 reveal-паттернов, корректный ОС back-button).

**Fallback (R2):** если @gorhom/bottom-sheet не работает на New Arch — допустим RN core `<Modal>` для full-screen variant, с осознанным trade-off (no native gestures). Не "rejected", а "secondary choice".

---

## 4. Dependencies

### Из Phase 2 (всё DONE 2026-05-01)
Bridge `packages/react-native-rustok-bridge` экспортирует **24 commands** через `WalletHandle`.

**Используются Phase 3:**
- `has_wallet()`, `is_wallet_unlocked()` — routing logic (M3, M4)
- `get_chain_id()` — `<NetworkBadge>` (M4)
- `lock_wallet()` — UnlockPin stub переход обратно (M3)
- `get_address()`, `get_balance()` — walletStore hydration (M4)

> **M1 spike:** verify точные имена этих 6 commands в `packages/react-native-rustok-bridge/src/` (TS-обёртки сгенерированы uniffi). Если имена отличаются — обновить план.

**Не используются Phase 3 — берёт Phase 4+:**
- `unlock_wallet`, `create_wallet_with_mnemonic`, `restore_from_phrase`, `send_eth`, `preview_send`, `analyze_transaction`, `sign_message`, `sign_typed_data`, `preview_transaction`, `send_transaction`, `get_swap_quote`, `execute_swap`, biometric_*, proxy_*, transaction_history.

### Что блокирует Phase 4 (Onboarding)
Phase 4 не стартует пока Phase 3 не закрыт:
- AppShell готов и принимает screens
- Stack может рендерить Welcome / KeepItSafe / ShowPhrase / Quiz / CreatePin / ConfirmPin
- Примитивы готовы (`<Button>`, `<Input>`, `<Modal>` для quiz reveal)
- Theme + tokens — все экраны Phase 4 их consume'ят
- `walletStore` готов для финального `createWalletWithMnemonic` callback'a

### Внешние npm-зависимости (новые)

| Пакет | Версия | Используется |
|-------|--------|--------------|
| `@react-navigation/native` | 7.x | M3 |
| `@react-navigation/bottom-tabs` | 7.x | M3 |
| `@react-navigation/native-stack` | 7.x | M3 |
| `react-native-screens` | latest | M3 (peer) |
| `react-native-safe-area-context` | latest | M3 (peer + AppShell) |
| `react-native-gesture-handler` | latest | M3 (peer) |
| `nativewind` | 4.x | M1 |
| `tailwindcss` | 3.4 | M1 (peer) |
| `zustand` | 5.x | M1, M4 |
| `react-native-mmkv` | latest | M1 (verify в M1 spike — может уже стоять) |
| `@gorhom/bottom-sheet` | 5.x | M2 |
| `react-native-reanimated` | по OQ5 | M2 (peer для bottom-sheet) |
| `react-native-toast-message` | latest | M2 |
| `lucide-react-native` | latest | M2 |
| `class-variance-authority` | latest | M2 |
| `clsx` | latest | M2 (cn-helper для cva) |

> **Note:** актуальный `mobile/package.json` нужно проверить в M1 spike — некоторые из этих пакетов могут уже стоять с Phase 1.

---

## 5. Constraints (UI-аналог PHASE-2-CONSTRAINTS.md)

> **C5 "no regressions" удалён** — дублирует CI gate (см. Exit criteria item 1).

### C1 — Accessibility (WCAG 2.1 AA baseline)

**Constraint:**
- Все интерактивные компоненты обязаны иметь `accessibilityLabel`, `accessibilityRole`
- Контраст текста ≥4.5:1 / крупного текста ≥3:1
- Respect `AccessibilityInfo.isReduceMotionEnabled()` (отключать bottom-sheet animations)
- Respect system font scaling (`allowFontScaling=true` default + sane max-cap)

**Verify:**
- Manual review checklist на каждый компонент в M2 dev screen (custom ESLint rule — overhead 1-2 дня, заменено на checklist)
- Контраст проверен на токенах в обоих темах (manual + Stark plugin)
- Smoke screen reader: TalkBack (Android), VoiceOver (iOS deferred)

**Resolution section:** заполняется на close M2.

### C2 — Theme parity

**Constraint:** каждый компонент работает идентично в light и dark; никаких hardcoded цветов в JSX (manual review checklist, не custom ESLint rule).

**Verify:**
- Manual code review: hex/rgb literal в JSX = блок при review
- Components dev screen рендерится в обоих режимах без визуальных регрессий — screenshot grid в PR
- Theme switch без unmount (через NativeWind `dark:` variant + CSS vars)

**Resolution section:** заполняется на close M2.

### C3 — Safe area + responsive + RTL-aware

**Constraint:**
- Layout корректен на iPhone с notch / Dynamic Island, Android без notch, маленьких устройствах (iPhone SE)
- Logical paddings (`paddingStart`/`paddingEnd`), не physical (`paddingLeft`/`paddingRight`) — для будущего RTL (Phase 7+)

**Verify:**
- `useSafeAreaInsets()` в AppShell вместо hardcoded paddings
- Manual smoke на минимум 2 размерах: Pixel 8 (real device, M3) + small emulator (Pixel 4a)
- iOS оставляем deferred (R3)

**Resolution section:** заполняется на close M3.

### C4 — Performance budget

**Constraint:**
- Cold-start (no_wallet → Welcome) ≤ **2.0 s** на real device (Pixel 6 baseline или JFLFG6MZSSL7WCF6 Xiaomi)
- Theme switch ≤ **100 ms** (визуально мгновенно)
- Tab switch ≤ **50 ms**
- Bundle size после M4 ≤ **+1.5 MB** к baseline после Phase 2

**Verify:**
- Hermes profiler на cold-start (release build) **на real device**, не emulator
- `npx react-native bundle` сравнение размеров до/после
- Frame drops через `react-native-performance` или Flipper

**Resolution section:** заполняется на close M4.

---

## 6. Exit criteria

Phase 3 закрыт когда **все** ниже = true:

1. ✅ M1-M4 merged в `main`. Все коммиты compliant с REVIEWER-CONSTITUTION v1.3 (atomic, conventional, sign-off, applicable review-skill passed). CI gate зелёный (Rust 227 tests без регрессий, RN typecheck + ESLint + jest зелёные).
2. ✅ Tests coverage:
   - **Stores + hooks:** ≥ 80% line coverage (`mobile/src/stores/__tests__/`, hooks tested через `@testing-library/react-native`)
   - **Components:** snapshot existence (smoke-проверка что render не падает в обоих темах), без жёсткого coverage threshold
3. ✅ Manual smoke на Android: Pixel 8 emulator + JFLFG6MZSSL7WCF6 Xiaomi (real device). iOS smoke deferred до M5-iOS-Phase3 (Mac session).
4. ✅ Constraints C1-C4 закрыты — Resolution sections заполнены в этом доке.
5. ✅ PR содержит screenshots: 3 routing states (no_wallet → Welcome, locked → UnlockPin, unlocked → Tabs с явным маркером "Phase 5 placeholder") + theme parity grid (light + dark всех компонентов из M2 dev screen).
6. ✅ `docs/PHASE3-HANDOFF.md` написан (стиль Phase 2 handoff): final state, что сделано / отложено / known issues.
7. ✅ `README.md` обновлён — overview `mobile/` структуры (theme, components, stores, navigation).
8. ✅ Workflow на каждый milestone: `/workflow` → `/check` (≥5 problems в 5 категориях) → `/typescript` → код → `/typescript-review` → коммит. `/security-review` обязателен только если milestone экспортирует secrets через MMKV (по факту в Phase 3 — нет, но gate должен быть проверен).

---

## 7. Open questions (per-milestone deadline)

| # | Вопрос | Должен быть решён до | Влияние |
|---|--------|---------------------|---------|
| OQ1 | iOS parity strategy — confirm defer to Mac session (default per Phase 1 M5) или push for cloud Mac runner (GitHub Actions macOS) | **до старта M3** | M3 gate iOS swipe-back, C3 verify на iOS, exit item 3 |
| OQ2 | Bottom sheet vs RN Modal — confirm gorhom как primary + RN Modal как fallback (R2), или нужен явный отдельный `<Modal>` поверх RN core с самого начала | **до старта M2** | M2 deliverables, §3.5 |
| OQ3 | Components dev screen в production — `__DEV__` flag (стандарт RN) или удалить перед Phase 5 | **до close M2** | M2 dev screen lifecycle |
| OQ4 | NetworkBadge readonly — confirm readonly (только chain icon + label, без tap), или сразу minimal toggle mainnet/testnet | **до старта M2** (компонент сам), **до старта M4** (store actions) | M2 component, M4 networkStore |
| OQ5 | Reanimated — 3.x stable или 4.x (бета). Phase 4 onboarding всё равно использует. | **до старта M2** (peer dep для @gorhom/bottom-sheet) | M2 install, deps table |

> **M1 не блокирован ни одним open question** — стартует сразу после approve плана.

---

## 8. Risks

| # | Риск | Вероятность | Митигация |
|---|------|:---:|---|
| R1 | NativeWind v4 несовместим с RN 0.85 New Arch | Low | M1 spike (1 день) — простой Hello + dark switch до начала M2 |
| R2 | `@gorhom/bottom-sheet` падает на New Arch | Med | Fallback: RN core `<Modal>` для full-screen (см. §3.5 — acceptable fallback, не идеал) |
| R3 | iOS parity невозможна без Mac → блок Phase 4 | Med | Phase 4 onboarding можно стартовать на Android-only; iOS parity отложить как M5-iOS-Phase3 (Mac session) |
| R4 | Theme switch flash на cold-start (FOIT) | Low | M1 deliverable: synchronous MMKV read в `App.tsx` до первого `<NavigationContainer>` render |
| R5 | Bundle size превысит C4 budget | Low | Tree-shake `lucide-react-native` (named imports), audit deps на M4 close |

> **R6 (NativeWind className race с Reanimated worklets) удалён** — Phase 3 не использует Reanimated напрямую (скрыт внутри @gorhom/bottom-sheet). Перенесён в Phase 4 plan, где будут анимированные компоненты (PinDots reveal).

---

## 9. References

- **Source plan section:** `docs/NATIVE-MIGRATION-PLAN.md` § Phase 3 (Design system + AppShell)
- **Phase 2 final state:** `docs/PHASE2-HANDOFF.md`
- **Phase 2 constraints pattern:** `docs/PHASE-2-CONSTRAINTS.md`
- **Reviewer rules:** `docs/REVIEWER-CONSTITUTION.md` (v1.3, 2026-05-04)
- **Phase 4 что блокируется:** `docs/NATIVE-MIGRATION-PLAN.md` § Phase 4 (Onboarding flow)
- **Bridge:** `packages/react-native-rustok-bridge/` — 24 commands via WalletHandle
- **Mobile root:** `mobile/`

> **Удалены устаревшие refs:** `docs/COMPONENTS.md` (помечен в CLAUDE.md как deprecated, удаляется в Phase 8), `docs/REDESIGN.md` (status неясен — verify в M1 spike перед использованием).
