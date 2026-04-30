# Rustok — AI Session Quick Start

**Актуальная точка входа — `docs/NATIVE-MIGRATION-PLAN.md` секции A-O (Onboarding).** Прочитай ПОЛНОСТЬЮ перед работой. Затем `docs/POC-FOUNDATION.md`.

---

## 30-second context

Production Ethereum wallet (Android + iOS). React Native 0.85.2 + uniffi-bindgen-react-native + Rust core (rustok-core + txguard). Мигрировали с Tauri+Leptos на 2026-04-28.

**Текущая фаза:** Phase 1 — Foundation. **M1+M2+M3+M4 closed.** M3 closed 2026-04-29 (PR #10 merged); M4 closed 2026-04-30 — Android E2E на физ. устройстве (Xiaomi), <100ms cold call, FLAG_SECURE применён. M5 next (iOS parity на Mac). Working branch: `main`. См. `docs/M3-RETROSPECTIVE.md` + `docs/M4-PROGRESS.md`.

## Start every session with

```bash
# Путь ТОЛЬКО ASCII — AGP не поддерживает кириллицу на Windows
cd C:/Claude/projects/rustok
git status
git log --oneline -10
cargo test --workspace
```

## Workflow (см. NATIVE-MIGRATION-PLAN.md §C и §D)

```
/workflow "задача" → /check → /rust или /typescript → код → /rust-review или /typescript-review → коммит
```

Между каждым шагом — пауза, ждать "да" от пользователя.

## Mandatory skills

- `/rust` — ВСЕГДА перед Rust кодом (загрузка стандартов)
- `/typescript` — ВСЕГДА перед TS/RN кодом (загрузка стандартов)
- `/check` — adversarial review плана (≥5 проблем, 5 категорий)
- `/rust-review` — перед коммитом Rust (НИКОГДА не пропускать)
- `/typescript-review` — перед коммитом TS (НИКОГДА не пропускать)
- `/security-review` — при любых изменениях в txguard/crypto/auth
- `/workflow` — для отслеживания состояния задачи (compaction-safe)

## Gates перед коммитом

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# React Native
cd mobile && npm run lint && npm run typecheck && npm run test
```

## Android dev (Windows — PowerShell!)

```powershell
# local.properties нужен вручную (gitignored):
# sdk.dir=C\:\\Users\\omadg\\AppData\\Local\\Android\\Sdk

# Metro (отдельный терминал):
cd mobile && npx react-native start --port 8081

# Сборка + установка:
cd mobile/android && .\gradlew.bat app:installDebug -PreactNativeDevServerPort=8081

# Физ. устройство — reverse port:
adb reverse tcp:8081 tcp:8081
```

## Links

- Strategy: `docs/NATIVE-MIGRATION-PLAN.md`
- Phase 1 plan: `docs/POC-FOUNDATION.md`
- Repo: https://github.com/temrjan/rustok
- CI: https://github.com/temrjan/rustok/actions

## Устаревшие docs (не выполнять!)

- `docs/SESSION.md` — старый стек Tauri+Leptos
- `docs/COMPONENTS.md`, `docs/TECHNICAL.md`, `docs/LEPTOS-GUIDE.md` — удаляются в Phase 8
