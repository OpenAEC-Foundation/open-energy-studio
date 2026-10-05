import {
  createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode,
} from 'react';
import { AlertTriangle, CheckCircle2, Info, X, XCircle } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { cx } from './Button';

export type ToastTone = 'success' | 'info' | 'warn' | 'error';

export interface ToastInput {
  tone?: ToastTone;
  title: string;
  message?: string;
  /** Milliseconds before an automatic close; errors stay until closed. */
  duration?: number;
}

interface ToastItem extends Required<Pick<ToastInput, 'tone' | 'title'>> {
  id: number;
  message?: string;
  duration: number | null;
}

interface ToastApi {
  show: (toast: ToastInput) => number;
  dismiss: (id: number) => void;
}

const ToastContext = createContext<ToastApi | null>(null);

/** Max toasts on screen (ontwerp.md §5.2); older ones make room. */
const MAX_TOASTS = 3;
const SUCCESS_MS = 6000;

const ICONS: Record<ToastTone, ReactNode> = {
  success: <CheckCircle2 aria-hidden="true" />,
  info: <Info aria-hidden="true" />,
  warn: <AlertTriangle aria-hidden="true" />,
  error: <XCircle aria-hidden="true" />,
};

/** Toast messages bottom right; replace `alert()`. */
export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const nextId = useRef(1);

  const dismiss = useCallback((id: number) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const show = useCallback((input: ToastInput) => {
    const id = nextId.current++;
    const tone = input.tone ?? 'info';
    const duration = input.duration ?? (tone === 'error' || tone === 'warn' ? null : SUCCESS_MS);
    setToasts((current) => [...current, { id, tone, title: input.title, message: input.message, duration }].slice(-MAX_TOASTS));
    return id;
  }, []);

  const api = useMemo(() => ({ show, dismiss }), [show, dismiss]);

  return (
    <ToastContext.Provider value={api}>
      {children}
      <ToastRegion toasts={toasts} onDismiss={dismiss} />
    </ToastContext.Provider>
  );
}

function ToastRegion({ toasts, onDismiss }: { toasts: ToastItem[]; onDismiss: (id: number) => void }) {
  const { t } = useI18n();
  return (
    <div className="ui-toasts" aria-live="polite" aria-relevant="additions">
      {toasts.map((toast) => (
        <ToastView key={toast.id} toast={toast} onDismiss={onDismiss} closeLabel={t('dialog.close')} />
      ))}
    </div>
  );
}

function ToastView({ toast, onDismiss, closeLabel }: { toast: ToastItem; onDismiss: (id: number) => void; closeLabel: string }) {
  useEffect(() => {
    if (toast.duration == null) return undefined;
    const timer = window.setTimeout(() => onDismiss(toast.id), toast.duration);
    return () => window.clearTimeout(timer);
  }, [toast.id, toast.duration, onDismiss]);
  return (
    <div className={cx('ui-toast', `ui-toast--${toast.tone}`)} role={toast.tone === 'error' ? 'alert' : 'status'}>
      <span className="ui-toast__icon">{ICONS[toast.tone]}</span>
      <div className="ui-toast__text">
        <strong>{toast.title}</strong>
        {toast.message && <p>{toast.message}</p>}
      </div>
      <button type="button" className="ui-iconbtn ui-iconbtn--sm" aria-label={closeLabel} onClick={() => onDismiss(toast.id)}>
        <X aria-hidden="true" />
      </button>
    </div>
  );
}

/**
 * Show a toast. Outside a provider (isolated component tests) it falls back
 * to a no-op so components never crash on a missing provider.
 */
export function useToast(): ToastApi {
  return useContext(ToastContext) ?? FALLBACK;
}

const FALLBACK: ToastApi = { show: () => 0, dismiss: () => undefined };
