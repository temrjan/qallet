/**
 * Button — Phase 3 M2 Commit 1.
 *
 * 4 variants × 3 sizes. Variants compose via class-variance-authority.
 * Loading state shows Spinner instead of label and disables the button.
 *
 * Tokens used (see tailwind.config.js + global.css):
 * - bg-accent-periwinkle / bg-canvas / bg-semantic-danger
 * - text-canvas / text-accent-periwinkle
 */

import React from 'react';
import {
  ActivityIndicator,
  Text,
  TouchableOpacity,
  type GestureResponderEvent,
} from 'react-native';
import { cva, type VariantProps } from 'class-variance-authority';
import clsx from 'clsx';

const buttonVariants = cva(
  'rounded-md flex-row items-center justify-center',
  {
    variants: {
      variant: {
        primary: 'bg-accent-periwinkle',
        secondary: 'bg-canvas border border-accent-periwinkle',
        ghost: 'bg-transparent',
        danger: 'bg-semantic-danger',
      },
      size: {
        sm: 'px-3 py-1.5',
        md: 'px-4 py-2.5',
        lg: 'px-6 py-3.5',
      },
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
    },
  },
);

const labelVariants = cva('font-medium text-center', {
  variants: {
    variant: {
      primary: 'text-canvas',
      secondary: 'text-accent-periwinkle',
      ghost: 'text-accent-periwinkle',
      danger: 'text-canvas',
    },
    size: {
      sm: 'text-sm',
      md: 'text-base',
      lg: 'text-lg',
    },
  },
  defaultVariants: {
    variant: 'primary',
    size: 'md',
  },
});

interface ButtonProps extends VariantProps<typeof buttonVariants> {
  onPress: (event: GestureResponderEvent) => void;
  disabled?: boolean;
  loading?: boolean;
  children: React.ReactNode;
  accessibilityLabel?: string;
}

export function Button({
  variant,
  size,
  onPress,
  disabled = false,
  loading = false,
  children,
  accessibilityLabel,
}: ButtonProps) {
  const isDisabled = disabled || loading;
  return (
    <TouchableOpacity
      onPress={isDisabled ? undefined : onPress}
      disabled={isDisabled}
      accessibilityRole="button"
      accessibilityState={{ disabled: isDisabled, busy: loading }}
      accessibilityLabel={accessibilityLabel}
      className={clsx(
        buttonVariants({ variant, size }),
        isDisabled && 'opacity-50',
      )}
    >
      {loading ? (
        <ActivityIndicator size={size === 'lg' ? 'large' : 'small'} />
      ) : (
        <Text className={labelVariants({ variant, size })}>{children}</Text>
      )}
    </TouchableOpacity>
  );
}
