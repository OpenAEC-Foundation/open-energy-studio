import { useRef, useCallback, useEffect, useId, useState } from 'react';
import i18next from 'i18next';

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
