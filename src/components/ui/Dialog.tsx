import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from 'react';
import { DialogShell } from '../dialogs/DialogShell';
import { useI18n } from '../../i18n/i18n';
import { Button } from './Button';

export interface DialogProps {
  title: string;
  onClose: () => void;
  children: ReactNode;
  /** Buttons in the fixed footer, right aligned. */
  footer?: ReactNode;
  width?: number;
  className?: string;
}

/**
 * Modal dialog of the design system. Built on `DialogShell`, so it shares the
 * dialog stack: focus trap, Escape closes the topmost dialog, focus returns
 * to the trigger.
 */
export function Dialog({ title, onClose, children, footer, width = 560, className }: DialogProps) {
  return (
    <DialogShell
      title={title}
      onClose={onClose}
      className={['ui-dialog', className].filter(Boolean).join(' ')}
      style={{ width: `min(${width}px, calc(100vw - 32px))` }}
      footer={footer === undefined ? null : <div className="dialog-footer ui-dialog__footer">{footer}</div>}
    >
      {children}
    </DialogShell>
  );
}

/** Side sheet docked right; the inspector variant of a dialog. */
export function SideSheet({ title, onClose, children, footer, width = 420, className }: DialogProps) {
  return (
    <DialogShell
      variant="sheet"
      title={title}
      onClose={onClose}
      className={['ui-sheet', className].filter(Boolean).join(' ')}
      style={{ width: `min(${width}px, 100vw)` }}
      footer={footer === undefined ? null : <div className="dialog-footer ui-dialog__footer">{footer}</div>}
    >
      {children}
    </DialogShell>
  );
}

// ── Confirm ──────────────────────────────────────────────────────────

/** `confirm` = primary action, `deny` = secondary destructive choice ("Niet opslaan"), `cancel` = back out. */
export type ConfirmChoice = 'confirm' | 'deny' | 'cancel';

export interface ConfirmOptions {
  title: string;
  message: ReactNode;
  confirmLabel?: string;
  /** Optional middle button, e.g. "Niet opslaan". */
  denyLabel?: string;
  cancelLabel?: string;
  /** Colour the confirm button as destructive. */
  danger?: boolean;
}

export interface ConfirmDialogProps extends ConfirmOptions {
  onChoose: (choice: ConfirmChoice) => void;
}

/** Accessible replacement for `confirm()` and the English Tauri `ask` dialog. */
export function ConfirmDialog({ title, message, confirmLabel, denyLabel, cancelLabel, danger, onChoose }: ConfirmDialogProps) {
  const { t } = useI18n();
  return (
    <Dialog
      title={title}
      onClose={() => onChoose('cancel')}
      width={460}
      className="ui-confirm"
      footer={(
        <>
          <Button variant="ghost" onClick={() => onChoose('cancel')}>{cancelLabel ?? t('dialog.cancel')}</Button>
          {denyLabel && <Button variant="danger" onClick={() => onChoose('deny')}>{denyLabel}</Button>}
          <Button variant={danger ? 'danger' : 'primary'} onClick={() => onChoose('confirm')}>
            {confirmLabel ?? t('ui.confirm')}
          </Button>
        </>
      )}
    >
      <div className="ui-confirm__message">{message}</div>
    </Dialog>
  );
}

type ConfirmFn = (options: ConfirmOptions) => Promise<ConfirmChoice>;

const ConfirmContext = createContext<ConfirmFn | null>(null);

/** Provides `useConfirm()`: an awaitable confirm dialog. */
export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [request, setRequest] = useState<ConfirmOptions | null>(null);
  const resolver = useRef<((choice: ConfirmChoice) => void) | null>(null);

  const confirm = useCallback<ConfirmFn>((options) => new Promise<ConfirmChoice>((resolve) => {
    // A second request while one is open answers the first with "cancel".
    resolver.current?.('cancel');
    resolver.current = resolve;
    setRequest(options);
  }), []);

  const choose = useCallback((choice: ConfirmChoice) => {
    const resolve = resolver.current;
    resolver.current = null;
    setRequest(null);
    resolve?.(choice);
  }, []);

  const value = useMemo(() => confirm, [confirm]);
  return (
    <ConfirmContext.Provider value={value}>
      {children}
      {request && <ConfirmDialog {...request} onChoose={choose} />}
    </ConfirmContext.Provider>
  );
}

/** Awaitable confirm; without a provider it resolves to "cancel" (safe default). */
export function useConfirm(): ConfirmFn {
  return useContext(ConfirmContext) ?? (async () => 'cancel');
}
