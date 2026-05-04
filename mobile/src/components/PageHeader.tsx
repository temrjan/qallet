/**
 * PageHeader — Phase 3 M2 Commit 2.
 *
 * Shared screen header with title, optional back button (left), and
 * optional right action. Respects safe-area top inset.
 *
 * a11y:
 * - Title: accessibilityRole="header"
 * - Back / right action: accessibilityRole="button" with explicit labels
 */

import React from 'react';
import { Text, TouchableOpacity, View } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

interface RightAction {
  label: string;
  onPress: () => void;
}

interface PageHeaderProps {
  title: string;
  onBack?: () => void;
  rightAction?: RightAction;
}

// SLOT_WIDTH balances the left/right slots so the centred title stays
// visually centred. 64px fits "← Back" and 6-letter actions like "Cancel".
const SLOT_WIDTH = 64;

export function PageHeader({ title, onBack, rightAction }: PageHeaderProps) {
  const insets = useSafeAreaInsets();

  return (
    <View
      className="flex-row items-center justify-between px-4 pb-3 bg-canvas border-b border-ink-muted"
      style={{ paddingTop: insets.top + 8 }}
    >
      {/* Left slot: Back or filler for alignment */}
      {onBack !== undefined ? (
        <TouchableOpacity
          onPress={onBack}
          accessibilityRole="button"
          accessibilityLabel="Back"
          style={{ width: SLOT_WIDTH }}
        >
          <Text className="text-accent-periwinkle text-base">← Back</Text>
        </TouchableOpacity>
      ) : (
        <View style={{ width: SLOT_WIDTH }} />
      )}

      <Text
        accessibilityRole="header"
        className="text-ink-primary text-lg font-semibold"
      >
        {title}
      </Text>

      {/* Right slot: action or filler for alignment */}
      {rightAction !== undefined ? (
        <TouchableOpacity
          onPress={rightAction.onPress}
          accessibilityRole="button"
          accessibilityLabel={rightAction.label}
          style={{ width: SLOT_WIDTH }}
        >
          <Text className="text-accent-periwinkle text-base text-right">
            {rightAction.label}
          </Text>
        </TouchableOpacity>
      ) : (
        <View style={{ width: SLOT_WIDTH }} />
      )}
    </View>
  );
}
