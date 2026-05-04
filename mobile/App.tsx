/**
 * Rustok Wallet — mobile UI shell.
 *
 * Wires `generateMnemonic` from react-native-rustok-bridge (uniffi
 * turbo-module) into the UI in M3. End-to-end run on physical device
 * is M4. Phase 2 commit 10 adds a `__DEV__`-gated FFI DevHarness
 * button for runtime smoke testing of the full WalletHandle surface
 * (24 commands across wallet lifecycle, signing, swap, history).
 *
 * Phase 3 M1 Commit 1: NativeWind v4 + design tokens added side-by-side.
 * Existing StyleSheet / useColorScheme path remains intact (migrates in M2).
 * Smoke verification = the `<View className>` strip below (compiles iff
 * NativeWind babel/metro pipeline works).
 */

// Side-effect import: required by react-native-gesture-handler on Android
// for system back gesture / native handler registration. Must be first.
import 'react-native-gesture-handler';
import './global.css';
import { useState } from 'react';
import {
  StatusBar,
  StyleSheet,
  Text,
  TouchableOpacity,
  useColorScheme,
  View,
} from 'react-native';
import {
  SafeAreaProvider,
  useSafeAreaInsets,
} from 'react-native-safe-area-context';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { BottomSheetModalProvider } from '@gorhom/bottom-sheet';
import { generateMnemonic } from 'react-native-rustok-bridge';
import DevHarness from './src/screens/_DevHarness';
import ComponentsScreen from './src/screens/_ComponentsScreen';
import { ThemeProvider } from './src/components/ThemeProvider';
import { ToastProvider } from './src/components';

function App() {
  const isDarkMode = useColorScheme() === 'dark';
  return (
    <ThemeProvider>
      <GestureHandlerRootView style={styles.rootFlex}>
        <BottomSheetModalProvider>
          <SafeAreaProvider>
            <StatusBar
              barStyle={isDarkMode ? 'light-content' : 'dark-content'}
            />
            <AppContent isDarkMode={isDarkMode} />
            {/* ToastProvider mounts inside SafeAreaProvider so toasts
                respect device notches; rendered last so it overlays
                everything above. */}
            <ToastProvider />
          </SafeAreaProvider>
        </BottomSheetModalProvider>
      </GestureHandlerRootView>
    </ThemeProvider>
  );
}

function AppContent({ isDarkMode }: { isDarkMode: boolean }) {
  const insets = useSafeAreaInsets();
  const [phrase, setPhrase] = useState<string | null>(null);
  const [showDevHarness, setShowDevHarness] = useState(false);
  const [showComponentsScreen, setShowComponentsScreen] = useState(false);

  if (__DEV__ && showDevHarness) {
    return <DevHarness onBack={() => setShowDevHarness(false)} />;
  }

  if (__DEV__ && showComponentsScreen) {
    return <ComponentsScreen onBack={() => setShowComponentsScreen(false)} />;
  }

  const onGenerate = async () => {
    try {
      const result = await generateMnemonic();
      setPhrase(result);
    } catch (e) {
      setPhrase(`Error: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const containerStyle = [
    styles.container,
    isDarkMode ? styles.containerDark : styles.containerLight,
    { paddingTop: insets.top, paddingBottom: insets.bottom },
  ];
  const titleStyle = [
    styles.title,
    isDarkMode ? styles.textPrimaryDark : styles.textPrimaryLight,
  ];
  const subtitleStyle = [
    styles.subtitle,
    isDarkMode ? styles.textMutedDark : styles.textMutedLight,
  ];
  const phraseStyle = [
    styles.phrase,
    isDarkMode ? styles.textMutedDark : styles.textMutedLight,
  ];

  return (
    <View style={containerStyle}>
      <Text style={titleStyle}>Rustok</Text>
      <Text style={subtitleStyle}>Phase 1 — POC Foundation</Text>

      {/* Phase 3 M1 smoke: this strip renders iff NativeWind compile works. */}
      <View
        accessible={false}
        className="h-2 w-32 bg-accent-periwinkle rounded-full my-3"
      />

      <TouchableOpacity style={styles.button} onPress={onGenerate}>
        <Text style={styles.buttonText}>Generate mnemonic</Text>
      </TouchableOpacity>

      {phrase !== null && <Text style={phraseStyle}>{phrase}</Text>}

      {__DEV__ && (
        <TouchableOpacity
          style={styles.devButton}
          onPress={() => setShowDevHarness(true)}
        >
          <Text style={styles.devButtonText}>Open FFI DevHarness</Text>
        </TouchableOpacity>
      )}

      {__DEV__ && (
        <TouchableOpacity
          style={styles.devButton}
          onPress={() => setShowComponentsScreen(true)}
        >
          <Text style={styles.devButtonText}>Open Components Screen</Text>
        </TouchableOpacity>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  rootFlex: {
    flex: 1,
  },
  container: {
    flex: 1,
    paddingHorizontal: 24,
    alignItems: 'center',
    justifyContent: 'center',
  },
  containerLight: {
    backgroundColor: '#FFFFFF',
  },
  containerDark: {
    backgroundColor: '#0A1123',
  },
  title: {
    fontSize: 32,
    fontWeight: '700',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    marginBottom: 32,
  },
  textPrimaryLight: {
    color: '#0A1123',
  },
  textPrimaryDark: {
    color: '#FFFFFF',
  },
  textMutedLight: {
    color: '#3A3E6C',
  },
  textMutedDark: {
    color: '#8A8CAC',
  },
  button: {
    backgroundColor: '#8387C3',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 12,
  },
  buttonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: '600',
  },
  phrase: {
    marginTop: 24,
    fontSize: 14,
    textAlign: 'center',
  },
  devButton: {
    marginTop: 32,
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#8387C3',
  },
  devButtonText: {
    color: '#8387C3',
    fontSize: 12,
  },
});

export default App;
