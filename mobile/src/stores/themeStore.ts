/**
 * Theme store — Phase 3 M1 Commit 2.
 *
 * Single source of truth for the user's preferred theme mode.
 * Persists to MMKV (UI prefs only — no encryption; secrets go through
 * Keychain/Keystore in Phase 4+).
 *
 * Hydration is synchronous on module load (top-level `mmkv.getString`)
 * so the first render already has the persisted mode — no FOIT on
 * cold-start with a saved dark preference.
 */

import { createMMKV } from 'react-native-mmkv';
import { create } from 'zustand';

export type ThemeMode = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'themeMode';
export const VALID_MODES: readonly ThemeMode[] = ['light', 'dark', 'system'];

// MMKV v4 exposes a factory; `MMKV` itself is a type-only export.
const mmkv = createMMKV();

function parseMode(value: string | undefined): ThemeMode {
  return value !== undefined && (VALID_MODES as readonly string[]).includes(value)
    ? (value as ThemeMode)
    : 'system';
}

const persistedMode = parseMode(mmkv.getString(STORAGE_KEY));

interface ThemeState {
  mode: ThemeMode;
  setMode: (mode: ThemeMode) => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  mode: persistedMode,
  setMode: (mode) => {
    mmkv.set(STORAGE_KEY, mode);
    set({ mode });
  },
}));
