/**
 * Spinner — Phase 3 M2 Commit 1.
 *
 * Thin wrapper around <ActivityIndicator>. Default color is
 * accent.periwinkle (theme-agnostic — same in both modes per palette).
 *
 * Note: <ActivityIndicator> only accepts 'small' | 'large' | number.
 * Plan size tokens map to native values: sm/md → 'small', lg → 'large'.
 * For finer control, pass `color` explicitly.
 */

import React from 'react';
import { ActivityIndicator } from 'react-native';
import { palette } from '../theme/tokens';

type SpinnerSize = 'sm' | 'md' | 'lg';

const NATIVE_SIZE: Record<SpinnerSize, 'small' | 'large'> = {
  sm: 'small',
  md: 'small',
  lg: 'large',
};

interface SpinnerProps {
  size?: SpinnerSize;
  color?: string;
  accessibilityLabel?: string;
}

export function Spinner({
  size = 'md',
  color = palette.light.accent.periwinkle,
  accessibilityLabel = 'Loading',
}: SpinnerProps) {
  return (
    <ActivityIndicator
      size={NATIVE_SIZE[size]}
      color={color}
      accessibilityLabel={accessibilityLabel}
    />
  );
}
