# Rustok — AI Session Quick Start

**Актуальная точка входа — `docs/NATIVE-MIGRATION-PLAN.md` секции A-O (Onboarding).** Прочитай ПОЛНОСТЬЮ перед работой. Затем `docs/POC-FOUNDATION.md`.

---

## 30-second context

Production Ethereum wallet (Android + iOS). React Native 0.85.2 + uniffi-bindgen-react-native + Rust core (rustok-core + txguard). Мигрировали с Tauri+Leptos на 2026-04-28.

**Текущая фаза:** Phase 1 — Foundation. M1+M2 done, M3 next (uniffi-bindgen-react-native setup).

## Start every session with

```bash
# Путь ТОЛЬКО ASCII — AGP не поддерживает кириллицу на Windows
cd C:/Claude/projects/rustok
git status
git log --oneline -10
cargo test --workspace
```

## Workflow (8 шагов — см. NATIVE-MIGRATION-PLAN.md §C)

Изучаю → План → /check → Исправляю → /codex + /rust или /typescript → Реализую → Ревьюю → Коммит

Между каждым шагом — пауза, ждать "да" от пользователя.

## Mandatory skills

- `/codex` — ВСЕГДА перед кодом
- `/rust` — Rust изменения
- `/typescript` — TypeScript/React Native изменения
- `/check` — после каждого плана
- `/rust-review` — перед коммитом Rust
- `/typescript-review` — перед коммитом TS

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
