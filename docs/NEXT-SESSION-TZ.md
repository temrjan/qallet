# ТЗ — Next Session

## Цель

Theme parity: Settings → toggle Light/Dark, единая тема на recurring
screens, фикс перепада «light Unlock → dark Home».

## Старт сессии

```bash
cd /Users/avangard/Workspace/projects/rustok
git status && git log --oneline -5 && cargo test --workspace
```

Прочитать **`docs/REDESIGN-AUDIT.md`** целиком — это single source of truth.

Запустить скиллы: **`/codex`** → план → **`/check`** → **`/rust web/leptos`** → код → **`/rust-review`** → коммит.

## Стратегия (гибрид, одобрено)

| Категория | Экраны | Подход |
|---|---|---|
| **One-time** | Splash, Welcome, Wallet wizard (5 step + Success), Restore | static **light** (как в rust-design) |
| **Recurring** | Unlock + Home + Receive + Activity + Settings + Send + TxGuard | **switchable** через `ThemeKind` |

Settings toggle переключает только recurring. Onboarding всегда светлый
(seed phrase contrast + first-impression brand).

## Чек-лист (8 атомарных коммитов)

- [ ] **A.** `index.html` + `tokens::css` + `app.rs` ThemeKind + persist в `localStorage` + Anti-FOUC script.
- [ ] **B.** Migrate 8 recurring файлов (`dark_shell`, `home`, `receive`, `activity`, `settings`, `send`, `analyze`, `unlock`) с `t::*_DARK` → `t::css::*`. После — `grep -rn "BG_DARK\|SURFACE_DARK\|TEXT_LIGHT\|CARD_DARK\|BORDER_DARK"` пуст в этих файлах.
- [ ] **C.** Settings → Appearance section с `ToggleRow "Light mode"` через context.
- [ ] **D.** `main.css`: body + `.tab-bar` на `var(--rw-*)`.
- [ ] **E.** `pages/splash.rs` с auto-advance 1.4s + WalletState routing.
- [ ] **F.** `Step::Success` в wallet.rs + restore.rs (зелёный check + Continue).
- [ ] **G.** Manual QA в обеих темах (см. §5G в audit).
- [ ] **H.** `docs/REDESIGN.md` § 5 + `SESSION-NEXT.md` + `COMPONENTS.md` + `README.md` обновить.

## Файлы которые **НЕ трогать**

`pages/welcome.rs`, `pages/wallet.rs` (только step Success добавить),
`pages/restore.rs` (только step Success), `pages/balance.rs` — остаются
на статических `t::*` константах.

## Acceptance

`cargo test`/`clippy` зелёные, CI green, `grep` чистый, toggle работает live, тема в `localStorage` сохраняется, нет FOUC, Splash → Home/Welcome/Unlock по `WalletState`, CreateSuccess после wizard.

## Тестовое окружение

Pixel_8 emulator, PIN `111111`, адрес `0x542E…B1B0`.

## Не делать

- Не возвращать `rustls-platform-verifier`.
- Не editить через SSH.
- Не пушить на красном CI.
- Не мигрировать onboarding (welcome/wallet/restore) на CSS vars.

## После

`docs/SESSION-NEXT.md` → следующее: Phase 4 (Across) / iOS TestFlight /
price feed (CoinGecko) / v0.1.3 release.
