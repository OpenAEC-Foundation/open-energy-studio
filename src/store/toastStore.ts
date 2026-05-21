/**
 * Non-blocking toast notifications. Queued; consumers render them via a
 * dedicated toast container component (TBD in Phase B).
 */
import { create } from 'zustand';

export type ToastSeverity = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
  id: string;
  severity: ToastSeverity;
  title: string;
  body?: string;
  /** Milliseconds before auto-dismiss; null means sticky. */
  ttlMs: number | null;
}

interface ToastState {
  toasts: Toast[];
  push: (toast: Omit<Toast, 'id'>) => string;
  dismiss: (id: string) => void;
  clear: () => void;
}

export const useToastStore = create<ToastState>((set) => ({
  toasts: [],

  push: (toast) => {
    const id = generateId();
    const full: Toast = { id, ...toast };
    set((s) => ({ toasts: [...s.toasts, full] }));
    if (full.ttlMs !== null) {
      setTimeout(() => {
        set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
      }, full.ttlMs);
    }
    return id;
  },

  dismiss: (id) =>
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),

  clear: () => set({ toasts: [] }),
}));

function generateId(): string {
  return `toast_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;
}
