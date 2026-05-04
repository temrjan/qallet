/**
 * ThemeProvider — Phase 3 M1 Commit 2.
 *
 * Pushes the current `themeStore.mode` into NativeWind's `colorScheme`.
 * NativeWind v4 handles system-mode listening internally when given
 * `'system'`, so we do NOT register our own `Appearance.addChangeListener`
 * here (would double-fire on system theme switch).
 *
 * Wrapped around <SafeAreaProvider> in App.tsx.
 */

import React, { useEffect } from 'react';
import { colorScheme } from 'nativewind';
import { useThemeStore } from '../stores/themeStore';

interface ThemeProviderProps {
  children: React.ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps): React.ReactNode {
  const mode = useThemeStore((s) => s.mode);

  useEffect(() => {
    colorScheme.set(mode);
  }, [mode]);

  return children;
}
