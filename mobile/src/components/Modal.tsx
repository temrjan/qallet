/**
 * Modal — Phase 3 M2 Commit 2.
 *
 * Declarative wrapper over @gorhom/bottom-sheet's imperative
 * BottomSheetModal API. Adapter: useRef + useEffect maps `isOpen` →
 * `present()/dismiss()` calls.
 *
 * Two variants:
 * - 'sheet'      → 50% snap point (drawer-style)
 * - 'fullscreen' → 100% snap point (modal-style)
 *
 * Custom snap points override the variant default.
 *
 * Requires <BottomSheetModalProvider> wrapped at the app root (see App.tsx).
 */

import React, { useEffect, useMemo, useRef } from 'react';
import {
  BottomSheetModal,
  BottomSheetView,
} from '@gorhom/bottom-sheet';
import { cssInterop } from 'nativewind';

// Register className → style passthrough for gorhom's BottomSheetView so
// NativeWind utility classes (`bg-canvas`, padding helpers) actually
// apply. Without this, BottomSheetView's underlying Animated.View is
// not seen by NativeWind's compile-time pre-pass and className is dropped.
cssInterop(BottomSheetView, { className: 'style' });

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
  variant?: 'sheet' | 'fullscreen';
  snapPoints?: (string | number)[];
}

export function Modal({
  isOpen,
  onClose,
  children,
  variant = 'sheet',
  snapPoints,
}: ModalProps) {
  const ref = useRef<BottomSheetModal>(null);
  const lastIsOpen = useRef(false);

  const resolvedSnapPoints = useMemo(
    () => snapPoints ?? (variant === 'fullscreen' ? ['100%'] : ['50%']),
    [snapPoints, variant],
  );

  useEffect(() => {
    // Skip redundant dismiss/present when state didn't actually flip
    // (e.g. swipe-down → onDismiss → consumer setState(false) → here).
    if (isOpen === lastIsOpen.current) {
      return;
    }
    lastIsOpen.current = isOpen;
    if (isOpen) {
      ref.current?.present();
    } else {
      ref.current?.dismiss();
    }
  }, [isOpen]);

  return (
    <BottomSheetModal
      ref={ref}
      snapPoints={resolvedSnapPoints}
      onDismiss={onClose}
      enablePanDownToClose
    >
      <BottomSheetView className="flex-1 bg-canvas px-6 pt-2 pb-8">
        {children}
      </BottomSheetView>
    </BottomSheetModal>
  );
}
