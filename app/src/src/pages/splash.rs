//! Splash overlay — full-screen brand frame shown while `WalletState`
//! resolves on cold start.
//!
//! Rendered conditionally over `HomePage` — the host component owns the
//! `splash_done` timer and decides when to dismiss this view. Splash is
//! intentionally locked to the static dark palette (`t::BG_DARK`); the
//! first-impression brand surface stays consistent regardless of the
//! user's chosen theme, and the user has no way to reach the toggle
//! before this overlay disappears anyway.

use leptos::prelude::*;

use crate::components::RustokLogo;
use crate::tokens::{self as t, rw_type};

/// Logo + wordmark + three pulsing dots, centered on a navy backdrop.
///
/// Pure presentation: no state, no side effects. The animation comes
/// from the `rw-pulse-dot` class defined in `app/src/styles/main.css`.
#[component]
pub fn SplashView() -> impl IntoView {
    view! {
        <div style=format!(
            "position:fixed;inset:0;z-index:9999;\
             display:flex;flex-direction:column;align-items:center;\
             justify-content:center;gap:24px;background:{bg};",
            bg = t::BG_DARK,
        )>
            <RustokLogo size=128 />

            <div style=format!(
                "font-family:{family};font-size:24px;font-weight:700;\
                 letter-spacing:-0.4px;color:{text};",
                family = rw_type::FAMILY,
                text = t::WHITE,
            )>"Rustok"</div>

            <div style="display:flex;gap:8px;">
                <span class="rw-pulse-dot" style="animation-delay:0ms;"/>
                <span class="rw-pulse-dot" style="animation-delay:150ms;"/>
                <span class="rw-pulse-dot" style="animation-delay:300ms;"/>
            </div>
        </div>
    }
}
