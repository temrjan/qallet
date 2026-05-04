/**
 * Design tokens — Phase 3 M1.
 *
 * Single source of truth for non-NativeWind code (StyleSheet, native APIs).
 * For className-based usage, see mobile/tailwind.config.js + mobile/global.css.
 *
 * Mirrors the values in global.css exactly. If you change one, change both.
 */

// Semantic and accent palettes are theme-agnostic for now.
// If a future design pass diverges them between modes, inline the values back
// into light/dark blocks.
const semantic = {
  success: '#22C55E',
  warn: '#F59E0B',
  danger: '#EF4444',
} as const;

const accent = {
  periwinkle: '#8387C3',
  deep: '#3A3E6C',
} as const;

export const palette = {
  light: {
    canvas: '#FFFFFF',
    ink: {
      primary: '#0A1123',
      muted: '#3A3E6C',
    },
    accent,
    semantic,
  },
  dark: {
    canvas: '#0A1123',
    ink: {
      primary: '#FFFFFF',
      muted: '#8A8CAC',
    },
    accent,
    semantic,
  },
} as const;

export type ThemeMode = 'light' | 'dark';
export type Palette = (typeof palette)[ThemeMode];
