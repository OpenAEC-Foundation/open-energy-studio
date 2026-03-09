import { useState, useEffect } from 'react';
import { useI18n } from '../../i18n/i18n';
import {
  FilePlus, FolderOpen, Save,
  Download, Upload, Printer, FileText, Box,
  Info,
} from 'lucide-react';
import './AppMenu.css';

interface AppMenuProps {
  isOpen: boolean;
  onClose: () => void;
  onNewProject: () => void;
  onOpenProject: () => void;
  onSaveProject: () => void;
  onSaveAsProject: () => void;
  onExportReport: () => void;
  onExportIFC: () => void;
  onExportModelIFC: () => void;
  onPrintReport: () => void;
  onExportUNIEC3: () => void;
  onImportUNIEC3: () => void;
  onExportVABI: () => void;
  onImportVABI: () => void;
  onOpenDialog: (type: string) => void;
}

type MenuView = 'none' | 'export' | 'import' | 'about';

function MenuItem({
  icon, label, shortcut, onClick, onMouseEnter, active,
}: {
  icon?: React.ReactNode;
  label: string;
  shortcut?: string;
  onClick?: () => void;
  onMouseEnter?: () => void;
  active?: boolean;
}) {
  return (
    <button
      className={`app-menu-item${active ? ' active' : ''}`}
      onClick={onClick}
      onMouseEnter={onMouseEnter}
    >
      <span className="app-menu-item-icon">{icon}</span>
      <span className="app-menu-item-label">{label}</span>
      {shortcut && <span className="app-menu-item-shortcut">{shortcut}</span>}
    </button>
  );
}

function Separator() {
  return <div className="app-menu-separator" />;
}

export function AppMenu({
  isOpen,
  onClose,
  onNewProject,
  onOpenProject,
  onSaveProject,
  onSaveAsProject,
  onExportReport,
  onExportIFC,
  onExportModelIFC,
  onPrintReport,
  onExportUNIEC3,
  onImportUNIEC3,
  onExportVABI,
  onImportVABI,
  onOpenDialog,
}: AppMenuProps) {
  const { t } = useI18n();
  const [activeView, setActiveView] = useState<MenuView>('none');

  useEffect(() => {
    if (!isOpen) setActiveView('none');
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const action = (fn: () => void) => () => {
    onClose();
    fn();
  };

  const clearView = () => setActiveView('none');

  return (
    <div className="app-menu-overlay">
      <div className="app-menu">
        {/* Sidebar */}
        <div className="app-menu-sidebar">
          <button className="app-menu-sidebar-header" onClick={onClose}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="14" y1="8" x2="2" y2="8" />
              <polyline points="8,2 2,8 8,14" />
            </svg>
            {t('ribbon.file')}
          </button>

          <div className="app-menu-items">
            <MenuItem icon={<FilePlus size={16} />} label={t('ribbon.new')} shortcut="Ctrl+N" onClick={action(onNewProject)} onMouseEnter={clearView} />
            <MenuItem icon={<FolderOpen size={16} />} label={t('ribbon.open')} shortcut="Ctrl+O" onClick={action(onOpenProject)} onMouseEnter={clearView} />
            <MenuItem icon={<Save size={16} />} label={t('ribbon.save')} shortcut="Ctrl+S" onClick={action(onSaveProject)} onMouseEnter={clearView} />
            <MenuItem icon={<Save size={16} />} label={t('ribbon.saveAs')} shortcut="Ctrl+Shift+S" onClick={action(onSaveAsProject)} onMouseEnter={clearView} />
            <Separator />
            <MenuItem icon={<Info size={16} />} label={t('ribbon.projectInfo')} onClick={action(() => onOpenDialog('project-info'))} onMouseEnter={clearView} />
            <Separator />
            <MenuItem icon={<Download size={16} />} label={t('ribbon.import')} onClick={() => setActiveView('import')} onMouseEnter={() => setActiveView('import')} active={activeView === 'import'} />
            <MenuItem icon={<Upload size={16} />} label={t('ribbon.export')} onClick={() => setActiveView('export')} onMouseEnter={() => setActiveView('export')} active={activeView === 'export'} />
            <Separator />
            <MenuItem icon={<Printer size={16} />} label={t('report.print')} onClick={action(onPrintReport)} onMouseEnter={clearView} />
          </div>

          <div className="app-menu-items-bottom">
            <MenuItem label={t('ribbon.about')} onClick={() => setActiveView('about')} onMouseEnter={() => setActiveView('about')} active={activeView === 'about'} />
          </div>
        </div>

        {/* Content area */}
        <div className="app-menu-content">
          {activeView === 'export' && (
            <div className="app-menu-panel">
              <h3 className="app-menu-panel-title">{t('ribbon.export')}</h3>
              <MenuItem icon={<FileText size={16} />} label={t('report.export')} onClick={action(onExportReport)} />
              <MenuItem icon={<Box size={16} />} label="IFC (BENG)" onClick={action(onExportIFC)} />
              <MenuItem icon={<Box size={16} />} label="IFC (Model)" onClick={action(onExportModelIFC)} />
              <Separator />
              <MenuItem icon={<Upload size={16} />} label="UNIEC3" onClick={action(onExportUNIEC3)} />
              <MenuItem icon={<Upload size={16} />} label="VABI Elements" onClick={action(onExportVABI)} />
            </div>
          )}

          {activeView === 'import' && (
            <div className="app-menu-panel">
              <h3 className="app-menu-panel-title">{t('ribbon.import')}</h3>
              <MenuItem icon={<Download size={16} />} label="UNIEC3" onClick={action(onImportUNIEC3)} />
              <MenuItem icon={<Download size={16} />} label="VABI Elements" onClick={action(onImportVABI)} />
            </div>
          )}

          {activeView === 'about' && (
            <div className="app-menu-panel">
              <h3 className="app-menu-panel-title">{t('ribbon.about')}</h3>
              <p className="app-menu-about-text">Open Energy Studio</p>
              <p className="app-menu-about-sub">Open-source energy performance calculation tool</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
