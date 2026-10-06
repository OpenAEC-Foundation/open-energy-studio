import { useCallback, useState, useRef, useEffect } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useDocumentManager } from '../../context/EnergyContext';
import type { DocumentEntry } from '../../context/EnergyContext';
import './DocumentTabs.css';
import type { NewProjectKind } from '../WelcomeScreen/WelcomeScreen';

function getTabLabel(doc: DocumentEntry): string {
  if (doc.filePath) {
    const name = doc.filePath.replace(/\\/g, '/').split('/').pop() || '';
    return name.replace(/\.oes\.json$/i, '').replace(/\.json$/i, '') || 'Untitled';
  }
  return doc.state.project.name || 'Untitled';
}

interface DocumentTabsProps {
  onCloseTab: (id: string) => void;
  onNewProject: () => void;
  /** With this, "+" offers the project kinds (existing dwelling first) instead of one "New". */
  onNewProjectOf?: (kind: NewProjectKind) => void;
  onOpenProject: () => void;
}

const NEW_KINDS: Array<{ kind: NewProjectKind; labelKey: string }> = [
  { kind: 'existing_residential', labelKey: 'welcome.existingResidential' },
  { kind: 'existing_utility', labelKey: 'welcome.existingUtility' },
  { kind: 'residential', labelKey: 'welcome.newResidential' },
  { kind: 'utility', labelKey: 'welcome.newUtility' },
];

export function DocumentTabs({ onCloseTab, onNewProject, onNewProjectOf, onOpenProject }: DocumentTabsProps) {
  const { t } = useI18n();
  const { docState, docDispatch } = useDocumentManager();
  const [menuPos, setMenuPos] = useState<{ x: number; y: number } | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const btnRef = useRef<HTMLButtonElement>(null);

  const handleTabClick = useCallback((id: string) => {
    docDispatch({ type: 'DOC_SET_ACTIVE', payload: id });
  }, [docDispatch]);

  const handleMiddleClick = useCallback((e: React.MouseEvent, id: string) => {
    if (e.button === 1) {
      e.preventDefault();
      onCloseTab(id);
    }
  }, [onCloseTab]);

  const toggleMenu = useCallback(() => {
    if (menuPos) {
      setMenuPos(null);
    } else if (btnRef.current) {
      const rect = btnRef.current.getBoundingClientRect();
      setMenuPos({ x: rect.left, y: rect.bottom + 4 });
    }
  }, [menuPos]);

  // Close dropdown on outside click or Escape
  useEffect(() => {
    if (!menuPos) return;
    const handleClick = (e: MouseEvent) => {
      if (
        menuRef.current && !menuRef.current.contains(e.target as Node) &&
        btnRef.current && !btnRef.current.contains(e.target as Node)
      ) {
        setMenuPos(null);
      }
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setMenuPos(null);
    };
    document.addEventListener('mousedown', handleClick);
    document.addEventListener('keydown', handleKey);
    return () => {
      document.removeEventListener('mousedown', handleClick);
      document.removeEventListener('keydown', handleKey);
    };
  }, [menuPos]);

  return (
    <div className="document-tabs-bar">
      {docState.documents.map(doc => (
        <div
          key={doc.id}
          className={`document-tab${doc.id === docState.activeDocumentId ? ' active' : ''}`}
          onClick={() => handleTabClick(doc.id)}
          onMouseDown={(e) => handleMiddleClick(e, doc.id)}
        >
          {/* A real button so the tab is reachable by keyboard; the click bubbles to the tab. */}
          <button type="button" className="document-tab-label" aria-current={doc.id === docState.activeDocumentId ? 'page' : undefined}>
            {doc.state.isDirty && <span className="document-tab-dirty">*</span>}
            {getTabLabel(doc)}
          </button>
          <button
            type="button"
            className="document-tab-close"
            onClick={(e) => { e.stopPropagation(); onCloseTab(doc.id); }}
            title={t('shell.tab.close')}
            aria-label={`${t('shell.tab.close')}: ${getTabLabel(doc)}`}
          >
            &times;
          </button>
        </div>
      ))}

      {/* + button right after the last tab */}
      <button
        ref={btnRef}
        className="document-tab-add"
        title={t('app.newProject')}
        onClick={toggleMenu}
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
          <line x1="6" y1="1" x2="6" y2="11" />
          <line x1="1" y1="6" x2="11" y2="6" />
        </svg>
      </button>

      {/* Dropdown rendered with fixed position so it's never clipped */}
      {menuPos && (
        <div
          ref={menuRef}
          className="document-tab-menu"
          style={{ left: menuPos.x, top: menuPos.y }}
        >
          {onNewProjectOf && NEW_KINDS.map((item) => (
            <button key={item.kind} type="button" className="document-tab-menu-item"
              onClick={() => { setMenuPos(null); onNewProjectOf(item.kind); }}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M3 10.5 12 3l9 7.5" /><path d="M5 9v11h14V9" />
              </svg>
              {t(item.labelKey)}
            </button>
          ))}
          {!onNewProjectOf && <button
            className="document-tab-menu-item"
            onClick={() => { setMenuPos(null); onNewProject(); }}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="12" y1="18" x2="12" y2="12" />
              <line x1="9" y1="15" x2="15" y2="15" />
            </svg>
            {t('ribbon.new')}
          </button>}
          <button
            className="document-tab-menu-item"
            onClick={() => { setMenuPos(null); onOpenProject(); }}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            {t('ribbon.open')}
          </button>
        </div>
      )}
    </div>
  );
}
