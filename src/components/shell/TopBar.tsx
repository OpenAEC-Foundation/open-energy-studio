/**
 * Top bar of the app shell (ontwerp §3.1): brand, document tabs, command
 * search, the File menu, Save, Recalculate, the inspector toggle and the
 * window controls. It replaces TitleBar, Ribbon and the separate tab strip;
 * the empty parts drag the window in the desktop build.
 */
import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react';
import {
  ChevronDown, FilePlus, FolderOpen, MessageSquare, Minus, PanelRight, RefreshCw, Save, Search,
  Settings, Square, Copy, Upload, X, XSquare,
} from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import type { NewProjectKind } from '../WelcomeScreen/WelcomeScreen';
import { Button, IconButton, Kbd } from '../ui';
import { isTauri } from '@tauri-apps/api/core';
import { version } from '../../../package.json';

async function currentWindow() {
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  return getCurrentWindow();
}
const minimizeWindow = () => currentWindow().then((win) => win.minimize()).catch(() => { /* not in Tauri */ });
const toggleMaximize = () => currentWindow().then((win) => win.toggleMaximize()).catch(() => { /* not in Tauri */ });
const closeWindow = () => currentWindow().then((win) => win.close()).catch(() => { /* not in Tauri */ });
const startDragging = () => currentWindow().then((win) => win.startDragging()).catch(() => { /* not in Tauri */ });

/** Clicks on these never drag the window. */
const INTERACTIVE = 'button, a, input, select, textarea, [role="menu"], .document-tabs-bar';

export interface TopBarProps {
  hasDocument: boolean;
  onNewProject: () => void;
  /** The "+" menu offers the four project kinds of the welcome screen. */
  onNewProjectOf?: (kind: NewProjectKind) => void;
  /** Shows the project library while documents stay open. */
  onShowLibrary?: () => void;
  onOpenProject: () => void;
  onSaveProject: () => void;
  onSaveAsProject: () => void;
  onImportUNIEC3: () => void;
  onImportVABI: () => void;
  onCloseTab: (id: string) => void;
  onCloseActiveTab: () => void;
  onOpenPalette: () => void;
  onOpenSettings: () => void;
  onOpenFeedback: () => void;
  onRecalculate?: () => void;
  onToggleInspector?: () => void;
  inspectorOpen?: boolean;
  recalculating?: boolean;
}

function MenuItem({ icon, label, shortcut, onSelect }: { icon: ReactNode; label: string; shortcut?: string; onSelect: () => void }) {
  return (
    <button type="button" role="menuitem" className="shell-menu-item" onClick={onSelect}>
      {icon}<span>{label}</span>{shortcut && <Kbd>{shortcut}</Kbd>}
    </button>
  );
}

/** Keyboard behaviour shared by the shell menus: arrows move, Escape closes and returns focus. */
export function useMenu() {
  const [open, setOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    menuRef.current?.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
    const onDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (!menuRef.current?.contains(target) && !buttonRef.current?.contains(target)) setOpen(false);
    };
    document.addEventListener('mousedown', onDown);
    return () => document.removeEventListener('mousedown', onDown);
  }, [open]);

  const onKeyDown = (event: React.KeyboardEvent) => {
    const items = Array.from(menuRef.current?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []);
    const index = items.indexOf(document.activeElement as HTMLElement);
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      setOpen(false);
      buttonRef.current?.focus();
    } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      const step = event.key === 'ArrowDown' ? 1 : -1;
      items[(index + step + items.length) % items.length]?.focus();
    } else if (event.key === 'Home' || event.key === 'End') {
      event.preventDefault();
      items[event.key === 'Home' ? 0 : items.length - 1]?.focus();
    } else if (event.key === 'Tab') {
      setOpen(false);
    }
  };
  /** Run a menu command after closing the menu. */
  const select = (run: () => void) => () => { setOpen(false); run(); };
  return { open, setOpen, buttonRef, menuRef, onKeyDown, select };
}

export function TopBar(props: TopBarProps) {
  const { t } = useI18n();
  const [maximized, setMaximized] = useState(false);
  const fileMenu = useMenu();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    (async () => {
      try {
        const win = await currentWindow();
        if (cancelled) return;
        setMaximized(await win.isMaximized());
        unlisten = await win.onResized(async () => setMaximized(await win.isMaximized()));
      } catch { /* not in Tauri */ }
    })();
    return () => { cancelled = true; unlisten?.(); };
  }, []);

  const isModalOpen = () => Boolean(document.querySelector('.dialog-overlay, .palette-backdrop'));
  const onMouseDown = useCallback((event: React.MouseEvent) => {
    if (event.button !== 0 || (event.target as HTMLElement).closest(INTERACTIVE) || isModalOpen()) return;
    void startDragging();
  }, []);
  const onDoubleClick = useCallback((event: React.MouseEvent) => {
    if ((event.target as HTMLElement).closest(INTERACTIVE) || isModalOpen()) return;
    void toggleMaximize();
  }, []);

  return (
    <header className="top-bar" onMouseDown={onMouseDown} onDoubleClick={onDoubleClick}>
      {/* Logo and name lead to the main menu, the project library (feedback 8 Oct 2026). */}
      <button type="button" className="top-bar-brand" onClick={props.onShowLibrary} disabled={!props.onShowLibrary}
        aria-label={t('shell.home')} title={t('shell.home')}>
        {/* The logo of the logo package (9 Oct 2026). */}
        <img className="top-bar-logo" src="/logo-mark.png" alt="" aria-hidden="true" />
        <span className="top-bar-app-name">{t('app.title')}<small>{t('shell.brandTagline')} · v{version}</small></span>
      </button>
      <div className="top-bar-drag" data-tauri-drag-region />

      <button type="button" className="top-bar-search" onClick={props.onOpenPalette} title={t('shell.search')} aria-keyshortcuts="Control+K"
        aria-haspopup="dialog">
        <Search aria-hidden="true" /><span>{t('shell.search')}</span><Kbd>Ctrl K</Kbd>
      </button>

      <div className="top-bar-actions">
        <div className="top-bar-menu">
          <Button ref={fileMenu.buttonRef} variant="ghost" size="sm" aria-haspopup="menu" aria-expanded={fileMenu.open}
            onClick={() => fileMenu.setOpen(!fileMenu.open)}>
            {t('shell.fileMenu')} <ChevronDown aria-hidden="true" />
          </Button>
          {fileMenu.open && (
            <div ref={fileMenu.menuRef} role="menu" aria-label={t('shell.fileMenu')} className="shell-menu shell-menu--down"
              onKeyDown={fileMenu.onKeyDown}>
              <MenuItem icon={<FilePlus aria-hidden="true" />} label={t('ribbon.new')} shortcut="Ctrl N" onSelect={fileMenu.select(props.onNewProject)} />
              <MenuItem icon={<FolderOpen aria-hidden="true" />} label={t('ribbon.open')} shortcut="Ctrl O" onSelect={fileMenu.select(props.onOpenProject)} />
              {props.hasDocument && <>
                <MenuItem icon={<Save aria-hidden="true" />} label={t('ribbon.save')} shortcut="Ctrl S" onSelect={fileMenu.select(props.onSaveProject)} />
                <MenuItem icon={<Save aria-hidden="true" />} label={t('ribbon.saveAs')} shortcut="Ctrl Shift S" onSelect={fileMenu.select(props.onSaveAsProject)} />
                <MenuItem icon={<XSquare aria-hidden="true" />} label={t('shell.closeTab')} shortcut="Ctrl W" onSelect={fileMenu.select(props.onCloseActiveTab)} />
              </>}
              <div className="shell-menu-sep" role="separator" />
              <MenuItem icon={<Upload aria-hidden="true" />} label={t('ribbon.importUNIEC3')} onSelect={fileMenu.select(props.onImportUNIEC3)} />
              <MenuItem icon={<Upload aria-hidden="true" />} label={t('ribbon.importVABI')} onSelect={fileMenu.select(props.onImportVABI)} />
              <div className="shell-menu-sep" role="separator" />
              <MenuItem icon={<Settings aria-hidden="true" />} label={t('nav.settings')} shortcut="Ctrl ," onSelect={fileMenu.select(props.onOpenSettings)} />
              <MenuItem icon={<MessageSquare aria-hidden="true" />} label={t('nav.feedback')} onSelect={fileMenu.select(props.onOpenFeedback)} />
            </div>
          )}
        </div>
        {props.hasDocument && <>
          <Button icon={<Save aria-hidden="true" />} onClick={props.onSaveProject} title={`${t('ribbon.save')} (Ctrl+S)`}
            aria-keyshortcuts="Control+S">{t('shell.save')}</Button>
          <Button variant="primary" icon={<RefreshCw aria-hidden="true" />} kbd="Ctrl ↵" onClick={props.onRecalculate}
            loading={props.recalculating} aria-keyshortcuts="Control+Enter">{t('shell.recalculate')}</Button>
          <IconButton icon={<PanelRight aria-hidden="true" />} aria-label={t('shell.inspector')} aria-pressed={props.inspectorOpen}
            aria-keyshortcuts="Control+." onClick={props.onToggleInspector} />
        </>}
      </div>

      {/* Window controls only exist in the desktop build; the browser has its own. */}
      <div className="top-bar-winctl" hidden={!isTauri()}>
        <button type="button" title={t('shell.window.minimize')} aria-label={t('shell.window.minimize')} onClick={() => void minimizeWindow()}><Minus aria-hidden="true" /></button>
        <button type="button" title={t(maximized ? 'shell.window.restore' : 'shell.window.maximize')} aria-label={t(maximized ? 'shell.window.restore' : 'shell.window.maximize')}
          onClick={() => void toggleMaximize()}>{maximized ? <Copy aria-hidden="true" /> : <Square aria-hidden="true" />}</button>
        <button type="button" className="close" title={t('shell.window.close')} aria-label={t('shell.window.close')} onClick={() => void closeWindow()}><X aria-hidden="true" /></button>
      </div>
    </header>
  );
}
