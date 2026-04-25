# Rustok — Full Audit (Kimi K2.5, 2026-04-25)

## Executive Summary

Rustok is a production-quality Ethereum wallet with a Tauri + Leptos stack that demonstrates strong architectural decisions and security-conscious implementation. The codebase shows clear separation between the secure backend (keyring, txguard) and reactive frontend, with the txguard transaction security engine being a particularly well-designed standalone crate.

**Strengths:** The cryptographic implementation follows best practices with AES-256-GCM, Argon2id, and proper Zeroize usage. The Leptos reactive patterns are mostly correct, and the theming system with CSS variables is well-architected. The security model for biometric storage is appropriately documented and scoped.

**Weaknesses:** The most significant issues are in the Leptos frontend's resource lifecycle management — specifically missing `on_cleanup` handlers for intervals and event listeners that lead to memory leaks. The CSS has hardcoded colors that predate the theme system. Error handling in the Tauri bridge could be more structured than `String` errors.

**Highest-leverage investment:** Addressing the Leptos resource cleanup issues should be the immediate priority, as these cause memory leaks during navigation. Following that, standardizing error types across the Tauri bridge would improve maintainability.

---

## Scoreboard

| Category | Coverage | Health | Top Issue |
|----------|----------|--------|-----------|
| Rust Correctness & Safety | HIGH | HIGH | std::sync::Mutex usage acceptable but could document invariants better |
| Security (wallet-specific) | HIGH | HIGH | Static biometric key appropriately scoped; nonce handling correct |
| Async / Leptos-Specific | HIGH | MED | Missing on_cleanup for intervals/listeners causes memory leaks |
| Error Handling | MED | MED | Tauri bridge uses String errors; could use structured types |
| Performance | MED | HIGH | Minor String allocations in hot paths; no blocking issues |
| Architecture | HIGH | HIGH | Clean crate boundaries; good separation of concerns |
| Design / UX / Visual | HIGH | MED | Hardcoded colors in CSS; theme not applied to onboarding |

---

## Findings

### 1. RUST CORRECTNESS & SAFETY

#### [SUGGESTION] Debug impl for LocalKeyring could leak metadata
- **File:** `crates/core/src/keyring/local.rs:38-45`
- **Problem:** The `Debug` impl for `LocalKeyring` prints the address. While not a key leak, wallet addresses are pseudonymous identifiers.
- **Fix:** Consider redacting the address or using `finish_non_exhaustive()` without the address field in release builds.

#### [SUGGESTION] Document why std::sync::Mutex is safe
- **File:** `app/src-tauri/src/commands.rs:22`
- **Problem:** The comment notes the lock "must never be held across .await points" but this is a critical security invariant that should be enforced or more prominently documented.
- **Fix:** Add `#[deny(clippy::await_holding_lock)]` to the crate or module level, and document the invariant in the struct-level doc comment.

#### [SUGGESTION] `unwrap_or_default()` on SystemTime could hide clock issues
- **File:** `crates/core/src/keyring/local.rs:267-270`
- **Problem:** If system clock is before UNIX_EPOCH, `now_unix()` returns 0, which could cause issues with key metadata.
- **Fix:** Consider propagating this error or at least logging a warning when the clock appears incorrect.

---

### 2. SECURITY (wallet-specific)

#### [SUGGESTION] Biometric key is static but appropriately documented
- **File:** `app/src-tauri/src/commands.rs:480`
- **Problem:** The `BIOMETRIC_KEY` is a static 32-byte string compiled into the binary.
- **Assessment:** This is NOT a critical issue. The module-level documentation correctly states: "AES-256-GCM provides at-rest obfuscation, NOT a cryptographic boundary. The key is app-static — the real protection is the sandbox + biometric." This is an accurate threat model for mobile biometric storage.

#### [WARNING] Biometric password stored as String before encryption
- **File:** `app/src-tauri/src/commands.rs:496-514`
- **Problem:** The password parameter is a `String` that is not zeroized before the function returns. While it is encrypted, the plaintext password remains in memory until the function returns and the String is dropped.
- **Fix:** Accept `Zeroizing<String>` or `&[u8]` and ensure explicit zeroization after encryption.

#### [SUGGESTION] No rate limiting on unlock attempts
- **File:** `app/src-tauri/src/commands.rs:335-346`
- **Problem:** `unlock_wallet` accepts passwords with no rate limiting or backoff. Argon2id provides some protection, but an attacker with access to the desktop app could brute-force weak passwords.
- **Fix:** Consider adding exponential backoff or a counter for failed attempts stored in app state.

#### [SUGGESTION] Keystore enumeration possible
- **File:** `app/src-tauri/src/commands.rs:312-324`
- **Problem:** `has_wallet` checks for `.json` files, which could allow an attacker to detect wallet presence.
- **Fix:** This is a minor information leak; consider using a fixed filename or storing presence in a separate config file.

---

### 3. ASYNC / LEPTOS-SPECIFIC

#### [ERROR] Interval not cleaned up on component unmount
- **File:** `app/src/src/pages/home.rs:87-93`
- **Problem:** The `gloo_timers::callback::Interval` created for balance auto-refresh is `.forget()`'d and never cleaned up. When the user navigates away from Home and back, multiple intervals will be running.
- **Fix:** Store the interval handle and register cleanup:
  ```rust
  let interval_handle = RwSignal::new(None::<Interval>);
  on_cleanup(move || {
      if let Some(i) = interval_handle.get_untracked() { i.cancel(); }
  });
  ```

#### [ERROR] visibilitychange event listener leaked
- **File:** `app/src/src/pages/home.rs:96-106`
- **Problem:** The `Closure` for the `visibilitychange` event is `.forget()`'d and never removed. Each mount of HomePage adds another listener.
- **Fix:** Use `on_cleanup` to call `remove_event_listener_with_callback` with the same closure reference, or store closures in a context-level collection.

#### [WARNING] Splash timeout not cancellable
- **File:** `app/src/src/app.rs:120`
- **Problem:** The splash timeout is `.forget()`'d. While this only runs once per app lifecycle, it's still an uncontrolled resource.
- **Fix:** Store the timeout handle and cancel on app teardown (less critical since it's once per app, not per component).

#### [WARNING] `spawn_local` without JoinHandle tracking
- **File:** `app/src/src/pages/settings.rs:63-77`
- **Problem:** Multiple `spawn_local` calls for address/biometric loading are fire-and-forget. If the component unmounts before they complete, the callbacks will still run and may set signals on a defunct component.
- **Fix:** Use an abortable task or check component liveness before updating signals, or use `LocalResource` which handles this automatically.

#### [SUGGESTION] `set_timeout` in unlock creates closure each call
- **File:** `app/src/src/pages/unlock.rs:90-98`
- **Problem:** The error reset timeout creates a new closure each time. Not a leak (runs once), but could be more efficient.
- **Fix:** Use a single `Effect` watching `error` and `shake` signals to auto-reset after delay.

---

### 4. ERROR HANDLING

#### [WARNING] Tauri bridge uses String errors exclusively
- **File:** `app/src/src/bridge.rs:31-43`
- **Problem:** The `tauri_invoke` function returns `Result<R, String>`, losing structured error information from the backend.
- **Fix:** Define a shared `BridgeError` enum in `rustok-types` with variants like `Serialization`, `Invocation`, `Deserialization` and use it consistently.

#### [SUGGESTION] Error messages contain sensitive context
- **File:** `app/src-tauri/src/commands.rs:293-294`
- **Problem:** Decryption error message says "wrong password or corrupted keystore" — this distinction could be useful to attackers.
- **Fix:** Use a single generic message: "unlock failed" and log details server-side only.

#### [SUGGESTION] `.ok().flatten()` hides storage errors
- **File:** `app/src/src/app.rs:66-77`
- **Problem:** LocalStorage errors are silently ignored with multiple `.ok()` calls.
- **Fix:** At minimum, log these errors with `web_sys::console::error_1` for debugging.

---

### 5. PERFORMANCE

#### [WARNING] `format!` in render closures creates allocations
- **File:** `app/src/src/pages/home.rs:186-189`
- **Problem:** The style string is reformatted on every render. While not in a tight loop, this is unnecessary work.
- **Fix:** Use static strings where possible, or use Leptos `class:` and `style:` directives which are optimized.

#### [SUGGESTION] `collect_view()` on large lists
- **File:** `app/src/src/pages/activity.rs:165`
- **Problem:** The transaction list uses `.map().collect_view()` which creates a new collection.
- **Fix:** For very long lists, use `<For keyed=...>` which efficiently diffs and updates only changed items.

#### [SUGGESTION] Chain colors as &'static str could be more robust
- **File:** `app/src/src/pages/home.rs:642-651`
- **Problem:** `chain_color` returns `&'static str` based on string matching. This is fragile if chain names change.
- **Fix:** Use a `HashMap` or match on `chain_id` (u64) instead of name.

---

### 6. ARCHITECTURE

#### [SUGGESTION] CSS variable tokens not used everywhere
- **File:** `app/src/styles/main.css:103-121`
- **Problem:** Many colors are hardcoded (e.g., `#f59e0b`, `#1A1A1A`) rather than using the CSS variables defined in `index.html`.
- **Fix:** Migrate all color values to use `var(--rw-*)` references for consistency.

#### [SUGGESTION] `Button` component has unused `SecondaryButton`
- **File:** `app/src/src/components/button.rs:88-111`
- **Problem:** `SecondaryButton` is defined but marked with `#![allow(dead_code)]` and not exported.
- **Fix:** Either export it or remove if truly unused.

#### [SUGGESTION] `tokens.rs` constants could be const fn
- **File:** `app/src/src/tokens.rs`
- **Problem:** All color constants are `&'static str` which requires runtime indirection.
- **Fix:** These could be `const` values, though the current approach is idiomatic for CSS-in-Rust.

---

### 7. DESIGN / UX / VISUAL

#### [ERROR] Onboarding screens don't follow theme toggle
- **File:** `app/src/src/tokens.rs:153-160`
- **Problem:** The docs state onboarding (Welcome, Wallet wizard, Restore) is "locked to the static light palette" by design. This is a UX inconsistency.
- **Fix:** While intentional, consider applying at least the dark theme to the Unlock screen for users who prefer dark mode during the unlock flow.

#### [WARNING] Touch targets may be insufficient on some elements
- **File:** `app/src/styles/main.css:122-131`
- **Problem:** Apple HIG recommends 44pt minimum touch targets. The CSS sets `min-height: 44px` but some elements (like the back button at 44×44) are at the absolute minimum.
- **Fix:** Increase to at least 48px for better accessibility, especially on Android where Material Design recommends 48dp.

#### [WARNING] Color contrast on muted text may fail WCAG AA
- **File:** `app/src/src/tokens.rs:39`
- **Problem:** `NEUTRAL_MID = "#959BB5"` on light backgrounds (`#F6F7FB`) yields a contrast ratio of approximately 2.9:1, which fails WCAG AA (requires 4.5:1 for normal text).
- **Fix:** Darken `NEUTRAL_MID` to at least `#7A8099` for light mode, or use a darker variant specifically for light surfaces.

#### [WARNING] No focus-visible styles for keyboard navigation
- **File:** `app/src/styles/main.css`
- **Problem:** No `:focus-visible` styles are defined, making keyboard navigation invisible.
- **Fix:** Add visible focus rings:
  ```css
  :focus-visible { outline: 2px solid var(--rw-accent); outline-offset: 2px; }
  ```

#### [SUGGESTION] Splash screen lacks reduced-motion support
- **File:** `app/src/styles/main.css:12-23`
- **Problem:** The pulse animation runs unconditionally.
- **Fix:** Respect `prefers-reduced-motion`:
  ```css
  @media (prefers-reduced-motion: reduce) {
    .rw-pulse-dot { animation: none; opacity: 1; }
  }
  ```

#### [SUGGESTION] No empty state for transaction history
- **File:** `app/src/src/pages/activity.rs:127-149`
- **Problem:** The empty state exists but is basic text. No illustration or CTA to receive funds.
- **Fix:** Add a visual illustration and a "Receive your first funds" CTA button.

#### [SUGGESTION] Tab bar lacks active state indicator beyond color
- **File:** `app/src/src/app.rs:192-214`
- **Problem:** The tab bar uses only color (`#8387C3`) to indicate active state. This fails WCAG 1.4.1 (Use of Color).
- **Fix:** Add a subtle underline, pill background, or icon fill change to distinguish active tab.

#### [SUGGESTION] No loading skeleton for QR code
- **File:** `app/src/src/pages/receive.rs:114-138`
- **Problem:** While there's a "…" placeholder, a skeleton that mimics the QR code shape would reduce layout shift.
- **Fix:** Add a pulsing placeholder with the same dimensions as the expected QR code.

---

## Design Proposals

1. **Unified Theme System for All Screens**
   - **Files:** `app/src/src/tokens.rs`, `app/src/src/pages/welcome.rs`, `app/src/src/pages/wallet.rs`
   - **Current:** Onboarding screens are hardcoded to light palette.
   - **Proposed:** Extend theme context to onboarding with a subtle variant (slightly off-white for dark mode to maintain readability while respecting user preference).
   - **Rationale:** Users expect consistent theming; sudden light-to-dark transition during onboarding is jarring.

2. **Enhanced Focus States for Accessibility**
   - **File:** `app/src/styles/main.css`
   - **Current:** No visible focus indicators.
   - **Proposed:** Add 2px periwinkle outline with 2px offset on all interactive elements.
   - **Rationale:** WCAG 2.2 compliance; keyboard navigation is essential for accessibility.

3. **Reduced Motion Support**
   - **File:** `app/src/styles/main.css`
   - **Current:** Pulse animation runs unconditionally.
   - **Proposed:** Wrap in `prefers-reduced-motion: no-preference` and provide static fallback.
   - **Rationale:** Respect user system preferences; prevents vestibular issues.

4. **Improved Empty States**
   - **File:** `app/src/src/pages/activity.rs`
   - **Current:** "No transactions yet" with small subtitle.
   - **Proposed:** Add centered illustration (SVG), larger text, and primary CTA button "Receive ETH".
   - **Rationale:** Empty states are onboarding moments; guide the user to first action.

5. **Better Tab Bar Active Indicator**
   - **File:** `app/src/src/app.rs`, `app/src/styles/main.css`
   - **Current:** Color change only (`#8387C3` vs muted gray).
   - **Proposed:** Add 2px periwinkle underline or filled pill background on active tab.
   - **Rationale:** WCAG 1.4.1 compliance; don't rely solely on color.

6. **Contrast-Compliant Muted Text**
   - **File:** `app/src/src/tokens.rs`
   - **Current:** `#959BB5` on `#F6F7FB` is ~2.9:1 contrast.
   - **Proposed:** Use `#6B7088` for muted text on light surfaces (4.6:1 contrast).
   - **Rationale:** WCAG AA compliance for text readability.

7. **Toast Notification System**
   - **File:** New `app/src/src/components/toast.rs`
   - **Current:** Copy confirmation uses inline icon swap (fleeting).
   - **Proposed:** Add a toast stack for transient confirmations (copied, unlocked, etc.).
   - **Rationale:** Consistent feedback pattern; inline changes can be missed.

8. **Biometric Enable Inline Flow**
   - **File:** `app/src/src/pages/settings.rs:79-93`
   - **Current:** Biometric enable is deferred to next unlock (no-op in settings).
   - **Proposed:** Trigger biometric enrollment inline with password confirmation.
   - **Rationale:** User expectation: toggle should work immediately, not require leaving settings.

9. **Transaction Confirmation Modal**
   - **File:** `app/src/src/pages/send.rs`
   - **Current:** 3-step wizard is good, but BLOCK verdict could be more prominent.
   - **Proposed:** Full-screen interstitial for BLOCK with explicit "I understand the risks" checkbox.
   - **Rationale:** Security-critical action deserves friction; prevents accidental override.

10. **Network Offline Indicator**
    - **File:** `app/src/src/app.rs`
    - **Current:** Balance errors show inline but no global connectivity status.
    - **Proposed:** Subtle offline banner at top when all RPC endpoints fail.
    - **Rationale:** Users should know when data is stale due to connectivity.

---

## What I Did NOT Audit

- **Mobile-specific behaviors:** Safe area insets are used but I couldn't verify actual iOS/Android rendering without running the app on devices.
- **Biometric plugin internals:** The `tauri-plugin-biometric` is an external dependency; only its usage was reviewed.
- **txguard rule engine comprehensiveness:** Rules are implemented but the completeness of scam detection vs. production threats wasn't evaluated.
- **Network security:** TLS certificate pinning, RPC endpoint verification — assumed handled by `reqwest` defaults.
- **Keystore backup/restore flows:** The import/export JSON format exists but the full UX flow wasn't audited.
- **Performance under load:** No profiling was done; findings are based on code review only.
- **GoPlus API integration:** The enrichment module makes external calls but error handling and rate limiting weren't deeply analyzed.

---

## Confidence

This audit is **high confidence** on:
- Rust correctness and safety patterns (thorough review of keyring, commands, core)
- Leptos reactive patterns and identified memory leaks (clear anti-patterns found)
- Security architecture (encryption, KDF, key handling)
- CSS/design token system

**Medium confidence** on:
- Mobile-specific UX issues (based on code analysis, not device testing)
- Performance hot paths (no profiling data)
- Complete coverage of all error paths (focused on main flows)

The findings in the "Async / Leptos-Specific" category are the most actionable and were verified by examining the exact lifecycle patterns. The design section is based on WCAG guidelines and established UX patterns, but actual user testing would validate priorities.

---

*Report generated: 2026-04-25*
*Auditor: Kimi K2.5*
*Scope: Full codebase read-only audit*
