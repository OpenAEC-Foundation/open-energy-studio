import { useRef, useCallback, useEffect, useId, useState } from 'react';
import i18next from 'i18next';

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), '
  + 'select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/** Visible, enabled focusable elements inside the dialog, in tab order. */
function focusableIn(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE))
    .filter((element) => !element.closest('[hidden], [inert]') && element.getAttribute('aria-hidden') !== 'true');
}

/**
 * Open dialogs, innermost last. Keyboard handling and the focus trap act
 * only for the topmost one, so a nested dialog owns Tab and Escape.
 */
const dialogStack: HTMLElement[] = [];

/**
 * The last dialog pushed, but a dialog nested in the DOM of another always
 * wins over its parent. React runs a child's effects before its parent's,
 * so when both mount in one commit the parent is pushed last.
 */
function topmostDialog(): HTMLElement | undefined {
  let current = dialogStack[dialogStack.length - 1];
  if (!current) return undefined;
  for (let changed = true; changed;) {
    changed = false;
    for (const other of dialogStack) {
      if (other !== current && current.contains(other)) { current = other; changed = true; break; }
    }
  }
  return current;
}

function isTopmost(dialog: HTMLElement): boolean {
  return topmostDialog() === dialog;
}

function focusFirst(dialog: HTMLElement) {
  const items = focusableIn(dialog);
  const first = items.find((element) => !element.classList.contains('dialog-close-btn')) ?? items[0];
  (first ?? dialog).focus();
}

interface DialogShellProps {
  title: string;
  onClose: () => void;
  onSubmit?: () => void;
  submitLabel?: string;
  cancelLabel?: string;
  children: React.ReactNode;
  className?: string;
  bodyClassName?: string;
  style?: React.CSSProperties;
  footer?: React.ReactNode;
}

export function DialogShell({
  title,
  onClose,
  onSubmit,
  submitLabel,
  cancelLabel,
  children,
  className,
  bodyClassName,
  style,
  footer,
}: DialogShellProps) {
  const titleId = useId();
  // ── Drag support ──
  const dialogRef = useRef<HTMLDivElement>(null);
  const dragState = useRef({ isDragging: false, offsetX: 0, offsetY: 0 });
  const [position, setPosition] = useState<{ x: number; y: number } | null>(null);

  const handleHeaderMouseDown = useCallback((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.dialog-close-btn')) return;
    const dialog = dialogRef.current;
    if (!dialog) return;
    const rect = dialog.getBoundingClientRect();
    dragState.current = {
      isDragging: true,
      offsetX: e.clientX - rect.left,
      offsetY: e.clientY - rect.top,
    };
    if (!position) {
      setPosition({ x: rect.left, y: rect.top });
    }
    e.preventDefault();
  }, [position]);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!dragState.current.isDragging) return;
      setPosition({
        x: e.clientX - dragState.current.offsetX,
        y: e.clientY - dragState.current.offsetY,
      });
    };
    const handleMouseUp = () => {
      dragState.current.isDragging = false;
    };
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  // ── Keyboard: focus into the dialog, trap Tab, Escape closes, focus returns to the trigger ──
  // The key handler sits on the document, so Tab and Escape still work when
  // the focused field unmounts (focus falls to <body>); only the topmost
  // open dialog reacts. Focus is not pulled back on `focusin`: the overlay
  // already blocks the page, and panels shown beside a dialog keep focus.
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialogStack.push(dialog);
    // A parent mounted in the same commit as a nested dialog must not take
    // the focus the nested one already moved into itself.
    if (isTopmost(dialog)) focusFirst(dialog);

    const onKeyDown = (event: KeyboardEvent) => {
      if (!isTopmost(dialog) || event.defaultPrevented) return;
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        onCloseRef.current();
        return;
      }
      if (event.key !== 'Tab') return;
      const items = focusableIn(dialog);
      if (items.length === 0) { event.preventDefault(); dialog.focus(); return; }
      const firstItem = items[0];
      const lastItem = items[items.length - 1];
      const active = document.activeElement;
      const inside = active instanceof Node && dialog.contains(active);
      if (event.shiftKey && (active === firstItem || active === dialog || !inside)) {
        event.preventDefault();
        lastItem.focus();
      } else if (!event.shiftKey && (active === lastItem || !inside)) {
        event.preventDefault();
        firstItem.focus();
      }
    };
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('keydown', onKeyDown);
      const index = dialogStack.lastIndexOf(dialog);
      if (index >= 0) dialogStack.splice(index, 1);
      if (trigger && trigger.isConnected) trigger.focus();
    };
  }, []);

  const [shake, setShake] = useState(false);
  const justFocused = useRef(false);

  // Suppress shake/beep when the click is just bringing the window into focus
  useEffect(() => {
    const onBlur = () => { justFocused.current = true; };
    const onFocus = () => {
      // The focus event fires before the click, so keep the flag briefly
      setTimeout(() => { justFocused.current = false; }, 300);
    };
    window.addEventListener('blur', onBlur);
    window.addEventListener('focus', onFocus);
    return () => {
      window.removeEventListener('blur', onBlur);
      window.removeEventListener('focus', onFocus);
    };
  }, []);

  const playBeep = useCallback(async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('play_system_beep');
    } catch { /* ignore errors */ }
  }, []);

  const handleOverlayClick = useCallback((e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      if (justFocused.current) return; // Window was just activated, ignore
      setShake(true);
      playBeep();
      setTimeout(() => setShake(false), 200);
    }
  }, [playBeep]);

  const blockEvent = useCallback((e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      e.stopPropagation();
      e.preventDefault();
    }
  }, []);

  const positionStyle: React.CSSProperties | undefined = position
    ? { position: 'fixed', left: position.x, top: position.y }
    : undefined;

  return (
    <div className="dialog-overlay" onClick={handleOverlayClick} onMouseDown={blockEvent} onDoubleClick={blockEvent}>
      <div
        ref={dialogRef}
        className={`${className ? `dialog ${className}` : 'dialog'}${shake ? ' dialog-shake' : ''}`}
        style={{ ...style, ...positionStyle }}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
      >
        <div className="dialog-header" onMouseDown={handleHeaderMouseDown}>
          <span className="dialog-header-title" id={titleId}>{title}</span>
          <button type="button" className="dialog-close-btn" onClick={onClose} aria-label={i18next.t('dialog.close')}>&times;</button>
        </div>
        <div className={bodyClassName ?? 'dialog-body'}>
          {children}
        </div>
        {footer !== undefined ? footer : (
          <div className="dialog-footer">
            <button className="btn" onClick={onClose}>
              {cancelLabel ?? 'Cancel'}
            </button>
            {onSubmit && (
              <button className="btn btn-primary" onClick={onSubmit}>
                {submitLabel ?? 'Save'}
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
