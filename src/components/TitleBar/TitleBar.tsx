import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useDocumentManager } from '../../context/EnergyContext';
import { SettingsDialog } from '../SettingsDialog/SettingsDialog';
import { FeedbackDialog } from '../dialogs/FeedbackDialog/FeedbackDialog';
import { version } from '../../../package.json';
import './TitleBar.css';

function isModalOpen() {
  return !!document.querySelector('.dialog-overlay');
}

async function minimizeWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().minimize();
  } catch { /* not in Tauri */ }
}

async function toggleMaximize() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().toggleMaximize();
  } catch { /* not in Tauri */ }
}

async function closeWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().close();
  } catch { /* not in Tauri */ }
}

async function startDragging() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().startDragging();
  } catch { /* not in Tauri */ }
}

interface TitleBarProps {
  onNewProject?: () => void;
  onOpenProject?: () => void;
  onSaveProject?: () => void;
}

export function TitleBar({ onNewProject, onOpenProject, onSaveProject }: TitleBarProps) {
  const { t } = useI18n();
  const { docState } = useDocumentManager();
  const activeDoc = docState.documents.find(d => d.id === docState.activeDocumentId);
  const projectName = activeDoc ? (activeDoc.state.project.name || t('app.untitledProject')) : null;
  const isDirty = activeDoc?.state.isDirty ?? false;
  const [maximized, setMaximized] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [feedbackOpen, setFeedbackOpen] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const win = getCurrentWindow();
        setMaximized(await win.isMaximized());
        unlisten = await win.onResized(async () => {
          setMaximized(await win.isMaximized());
        });
      } catch { /* not in Tauri */ }
    })();

    return () => { unlisten?.(); };
  }, []);

  const handleTitleBarMouseDown = useCallback((e: React.MouseEvent) => {
    // Don't drag if clicking on interactive elements or if a modal is open
    if ((e.target as HTMLElement).closest('.title-bar-left, .window-controls')) return;
    if (isModalOpen()) return;
    startDragging();
  }, []);

  const handleTitleBarDoubleClick = useCallback((e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.title-bar-left, .window-controls')) return;
    if (isModalOpen()) return;
    toggleMaximize();
  }, []);

  return (
    <>
      <div className="title-bar" onMouseDown={handleTitleBarMouseDown} onDoubleClick={handleTitleBarDoubleClick}>
        <div className="title-bar-left">
          <img src="/icon.png" alt="" className="title-bar-icon" />

          <div className="qat-buttons">
            <button className="qat-btn" title={`${t('ribbon.new')} (Ctrl+N)`} onClick={onNewProject}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                <polyline points="14 2 14 8 20 8" />
                <line x1="12" y1="18" x2="12" y2="12" />
                <line x1="9" y1="15" x2="15" y2="15" />
              </svg>
            </button>
            <button className="qat-btn" title={`${t('ribbon.open')} (Ctrl+O)`} onClick={onOpenProject}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
              </svg>
            </button>
            <button className="qat-btn" title={`${t('ribbon.save')} (Ctrl+S)`} onClick={onSaveProject}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                <polyline points="17 21 17 13 7 13 7 21" />
                <polyline points="7 3 7 8 15 8" />
              </svg>
            </button>

            <div className="qat-separator" />

            <button className="qat-btn" title={t('settings.title')} onClick={() => setSettingsOpen(true)}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
            </button>
          </div>
        </div>

        <div className="title-bar-center">
          {isDirty && <span className="title-bar-dirty">*</span>}
          <span className="title-bar-app-name">{t('app.title')} v{version}</span>
          {projectName !== null && (
            <>
              <span className="title-bar-separator">&ndash;</span>
              <span className="title-bar-project">{projectName}</span>
            </>
          )}
        </div>

        <div className="window-controls">
          <button className="send-feedback-btn" onClick={() => setFeedbackOpen(true)}>
            {t('feedback.sendFeedback')}
          </button>
          <button className="window-btn" title="Minimize" onClick={minimizeWindow}>
            <svg width="10" height="1" viewBox="0 0 10 1">
              <rect width="10" height="1" fill="currentColor" />
            </svg>
          </button>
          <button className="window-btn" title={maximized ? 'Restore' : 'Maximize'} onClick={toggleMaximize}>
            {maximized ? (
              <svg width="10" height="10" viewBox="0 0 10 10">
                <rect x="2.5" y="0.5" width="7" height="7" fill="none" stroke="currentColor" strokeWidth="1.2" />
                <rect x="0.5" y="2.5" width="7" height="7" fill="var(--titlebar-bg-end, #1a1a2e)" stroke="currentColor" strokeWidth="1.2" />
              </svg>
            ) : (
              <svg width="10" height="10" viewBox="0 0 10 10">
                <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" strokeWidth="1.2" />
              </svg>
            )}
          </button>
          <button className="window-btn window-btn-close" title="Close" onClick={closeWindow}>
            <svg width="10" height="10" viewBox="0 0 10 10">
              <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" strokeWidth="1.2" />
              <line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" strokeWidth="1.2" />
            </svg>
          </button>
        </div>
      </div>

      {settingsOpen && <SettingsDialog onClose={() => setSettingsOpen(false)} />}
      {feedbackOpen && <FeedbackDialog onClose={() => setFeedbackOpen(false)} />}
    </>
  );
}
