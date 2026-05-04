/**
 * _ComponentsScreen — Phase 3 M1 Commit 2.
 *
 * Dev-only surface for smoke-testing the theme switcher and verifying
 * that NativeWind className compilation is alive end-to-end. Not a
 * production screen — gated by `__DEV__` in App.tsx, follows the `_`
 * prefix pattern of `_DevHarness`.
 *
 * Smoke calls `generateMnemonic` (top-level bridge function, no
 * WalletHandle instance required) to confirm the rustok-mobile-bindings
 * FFI is still wired after the NativeWind installation. WalletHandle
 * surface is exercised in `_DevHarness` separately.
 */

import { useEffect, useState } from 'react';
import { Text, TouchableOpacity, View } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { generateMnemonic } from 'react-native-rustok-bridge';
import { useThemeStore, VALID_MODES } from '../stores/themeStore';

interface ComponentsScreenProps {
  onBack: () => void;
}

type BridgeStatus = 'idle' | 'ok' | 'fail';

function ComponentsScreen({ onBack }: ComponentsScreenProps) {
  const insets = useSafeAreaInsets();
  const mode = useThemeStore((s) => s.mode);
  const setMode = useThemeStore((s) => s.setMode);
  const [bridgeStatus, setBridgeStatus] = useState<BridgeStatus>('idle');

  useEffect(() => {
    let isMounted = true;
    // generateMnemonic returns a string synchronously; wrap in Promise to
    // get unified .then/.catch surface and to keep the smoke check
    // resilient if uniffi later migrates this binding to async.
    Promise.resolve(generateMnemonic())
      .then(() => {
        if (isMounted) {
          setBridgeStatus('ok');
        }
      })
      .catch(() => {
        if (isMounted) {
          setBridgeStatus('fail');
        }
      });
    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <View
      className="flex-1 bg-canvas px-6"
      style={{ paddingTop: insets.top, paddingBottom: insets.bottom }}
    >
      <TouchableOpacity onPress={onBack} className="py-3">
        <Text className="text-accent-periwinkle text-base">← Back</Text>
      </TouchableOpacity>

      <Text className="text-ink-primary text-xl font-bold mb-2">
        Components
      </Text>
      <Text className="text-ink-muted text-sm mb-6">
        Phase 3 M1 — theme switcher smoke
      </Text>

      <Text className="text-ink-muted text-xs uppercase mb-2">Theme mode</Text>
      <View
        accessibilityRole="radiogroup"
        accessibilityLabel="Theme mode"
        className="mb-6"
      >
        {VALID_MODES.map((m) => {
          const selected = mode === m;
          return (
            <TouchableOpacity
              key={m}
              accessibilityRole="radio"
              accessibilityState={{ selected }}
              accessibilityLabel={`Theme mode ${m}`}
              onPress={() => setMode(m)}
              className="py-3 flex-row items-center"
            >
              <Text className="text-ink-primary text-base">
                {selected ? '● ' : '○ '}
                {m}
              </Text>
            </TouchableOpacity>
          );
        })}
      </View>

      <Text className="text-ink-muted text-xs uppercase mb-2">
        Visual sample (auto-themed)
      </Text>
      <View
        accessible={false}
        className="bg-canvas border border-ink-muted rounded-lg h-24 mb-6"
      />

      <Text className="text-ink-muted text-xs uppercase mb-2">Bridge</Text>
      <Text className="text-ink-primary text-sm">
        generateMnemonic: {bridgeStatus}
      </Text>
    </View>
  );
}

export default ComponentsScreen;
