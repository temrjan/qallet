# Rustok — AI Session Quick Start

> **Актуальная точка входа — `docs/SESSION.md`.** Прочитай его ПОЛНОСТЬЮ перед работой.
> Этот файл — краткая напоминалка, не замена SESSION.md.

---

## 30-second context

Production Ethereum wallet (iOS + Android + Desktop). Full Rust: Tauri 2.0 backend + Leptos 0.7 UI (WASM) + rustok-core + txguard security engine.

## Start every session with

```bash
cd /Users/avangard/Workspace/projects/rustok
cargo test --workspace       # 110+ green?
git log --oneline -10        # what changed?
```

## Workflow

| Mode | Steps |
|------|-------|
| **LIGHT** (1 file, config, docs) | Study → Do → `/check` → diff → Commit → Push → CI |
| **FULL** (features, multi-file) | Study → `/codex` → Plan → `/check` → `/rust` → Implement → `/rust-review` → diff → Commit → Push → CI |

## Mandatory skills

- `/codex` — load stack standards
- `/rust` — load Rust + web/leptos tentacles
- `/check` — self-critique after every plan
- `/rust-review` — review before commit

## 4 gates (all green before commit)

```bash
cd app/src && cargo check --target wasm32-unknown-unknown
RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets --all-features
cargo fmt --all --check
cargo test --workspace
```

## Links

- Master doc: `docs/SESSION.md`
- Architecture: `docs/COMPONENTS.md`
- Tech details: `docs/TECHNICAL.md`
- Vision: `docs/VISION.md`
- Repo: https://github.com/temrjan/rustok
- CI: https://github.com/temrjan/rustok/actions
