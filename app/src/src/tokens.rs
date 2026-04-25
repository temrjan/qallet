// Tokens are the design-system foundation shared across all screens.
// Only a subset is used today (Welcome); the rest land as the dark screens
// (home/send/receive/settings/txguard) get ported.
#![allow(dead_code)]

//! Design tokens for Rustok Wallet UI.
//!
//! All colors derive from the brand palette:
//!
//! - `#0A1123` — primary dark (brand)
//! - `#3A3E6C` — accent deep
//! - `#8387C3` — primary accent (periwinkle)
//! - `#959BB5` — neutral mid
//! - `#8A8CAC` — neutral soft
//!
//! Plus derived light surfaces, semantic colors, gradients, and shadows.
//!
//! Tokens are exposed as `&'static str` constants so they can be embedded
//! directly into `style=` attributes in [`leptos::view!`] macros.

// ─── Brand core ─────────────────────────────────────────────────

/// Primary dark — app background, primary CTA on light.
pub const BRAND: &str = "#0A1123";

/// Even deeper brand tone for subtle layering on dark.
pub const BRAND_DEEP: &str = "#070D1B";

/// Indigo accent — hover, active strokes, depth.
pub const ACCENT_DEEP: &str = "#3A3E6C";

/// Periwinkle — primary interactive, links, active icons.
pub const ACCENT: &str = "#8387C3";

/// Lighter periwinkle — highlights, decorative.
pub const ACCENT_SOFT: &str = "#9EA3D1";

/// Muted text, placeholders, secondary labels.
pub const NEUTRAL_MID: &str = "#959BB5";

/// Dividers, inactive icons, outline hints.
pub const NEUTRAL_SOFT: &str = "#8A8CAC";

// ─── Light surfaces (onboarding mode) ───────────────────────────

/// Pure white surface.
pub const WHITE: &str = "#FFFFFF";

/// Light background — inputs, subtle fills.
pub const SURFACE_ALT: &str = "#F6F7FB";

/// Hairlines on light surfaces.
pub const SURFACE_BORDER: &str = "#E5E8F2";

// ─── Dark surfaces (main app mode) ──────────────────────────────

/// Dark background — main shell.
pub const BG_DARK: &str = "#0A1123";

/// Cards on dark (balance card, list items).
pub const SURFACE_DARK: &str = "#141A33";

/// Elevated cards on dark.
pub const SURFACE_DARK_2: &str = "#1C2244";

/// Borders on dark surfaces.
pub const BORDER_DARK: &str = "#242B4C";

// ─── Text ───────────────────────────────────────────────────────

/// Text on light backgrounds.
pub const TEXT_DARK: &str = "#0A1123";

/// Text on dark backgrounds.
pub const TEXT_LIGHT: &str = "#FFFFFF";

/// Muted text (neutral mid).
pub const TEXT_MUTED: &str = "#959BB5";

/// Softer muted text.
pub const TEXT_SOFT: &str = "#8A8CAC";

// ─── Semantic ───────────────────────────────────────────────────

/// Success / positive change.
pub const SUCCESS: &str = "#4AB37B";

/// Success tinted background (12% alpha).
pub const SUCCESS_BG: &str = "rgba(74,179,123,0.12)";

/// Danger / negative change / error.
pub const DANGER: &str = "#E06B6B";

/// Danger tinted background (12% alpha).
pub const DANGER_BG: &str = "rgba(224,107,107,0.12)";

/// Warning / attention.
pub const WARN: &str = "#D9A562";

/// Warning tinted background (12% alpha).
pub const WARN_BG: &str = "rgba(217,165,98,0.12)";

// ─── Gradients ──────────────────────────────────────────────────

/// Sky header gradient for onboarding shells.
pub const SKY_HEADER: &str = "linear-gradient(180deg, #8387C3 0%, #6E73B3 100%)";

/// Softer sky header variant.
pub const SKY_HEADER_SOFT: &str = "linear-gradient(180deg, #A7AAD6 0%, #8387C3 100%)";

/// Dark card gradient (balance, chart cards).
pub const CARD_DARK: &str = "linear-gradient(160deg, #141A33 0%, #0D1328 100%)";

// ─── Shadows ────────────────────────────────────────────────────

/// Soft shadow for subtle cards.
pub const SHADOW_SOFT: &str = "0 8px 24px rgba(10,17,35,0.08)";

/// Elevated card shadow.
pub const SHADOW_CARD: &str = "0 12px 32px rgba(10,17,35,0.12)";

/// Button shadow (dark CTA on light).
pub const SHADOW_BTN: &str = "0 6px 16px rgba(10,17,35,0.22)";

// ─── Typography ─────────────────────────────────────────────────

/// Typography tokens — font stacks and weights.
pub mod rw_type {
    /// System sans stack for UI text.
    pub const FAMILY: &str =
        "Roboto, -apple-system, \"SF Pro Display\", \"SF Pro Text\", system-ui, sans-serif";

    /// Monospace stack for addresses, hashes, numbers.
    pub const MONO: &str = "ui-monospace, \"SF Mono\", Menlo, monospace";

    /// Regular weight (400).
    pub const REGULAR: u16 = 400;

    /// Medium weight (500).
    pub const MEDIUM: u16 = 500;

    /// Semibold weight (600).
    pub const SEMIBOLD: u16 = 600;

    /// Bold weight (700).
    pub const BOLD: u16 = 700;
}

// ─── Theme-aware CSS variables ──────────────────────────────────

/// CSS custom-property references for the switchable theme.
///
/// Use these on recurring app surfaces (Unlock + main app screens) where
/// the user expects light/dark to follow the Settings toggle. One-time
/// onboarding screens (Welcome / Wallet wizard / Restore) keep the static
/// `t::*` constants because their first-impression contrast is fixed.
///
/// The variables themselves live in `app/src/index.html` `<style>` block;
/// dark is the default and `:root[data-theme="light"]` overrides them.
pub mod css {
    /// Page background — replaces `BG_DARK` on switchable screens.
    pub const BG: &str = "var(--rw-bg)";
    /// Cards / sections — replaces `SURFACE_DARK`.
    pub const SURFACE: &str = "var(--rw-surface-1)";
    /// Elevated surfaces — replaces `SURFACE_DARK_2`.
    pub const SURFACE_2: &str = "var(--rw-surface-2)";
    /// Hairlines on theme surfaces — replaces `BORDER_DARK`.
    pub const BORDER: &str = "var(--rw-border)";
    /// Primary text — replaces `TEXT_LIGHT`.
    pub const TEXT: &str = "var(--rw-text)";
    /// Hero card gradient — replaces `CARD_DARK`.
    pub const CARD: &str = "var(--rw-card)";
    /// Switch OFF track — separate from generic border to keep the toggle
    /// visually subtle on light surfaces. Used by `settings::Switch`.
    pub const SWITCH_OFF: &str = "var(--rw-switch-off)";
    /// Bottom tab bar background (semi-transparent for blur backdrop).
    pub const TAB_BG: &str = "var(--rw-tab-bg)";
    /// Muted text / inactive icons — theme-aware variant of `NEUTRAL_MID`.
    pub const NEUTRAL_MID: &str = "var(--rw-neutral-mid)";
}

// ─── Radii ──────────────────────────────────────────────────────

/// Border radius scale tokens (in pixels).
pub mod rw_radius {
    /// Small radius — 10px.
    pub const SM: u16 = 10;

    /// Medium radius — 14px.
    pub const MD: u16 = 14;

    /// Large radius — 18px.
    pub const LG: u16 = 18;

    /// Extra-large radius — 24px.
    pub const XL: u16 = 24;

    /// Pill / full rounding.
    pub const PILL: u16 = 9999;
}
