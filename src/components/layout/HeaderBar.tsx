/**
 * HeaderBar — top strip with New / Open / Save / Save As. Uses the Tauri
 * dialog plugin for native file pickers. Web mode (browser preview without
 * Tauri) falls back to download/upload via the File API later in Phase B.
 */
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';

import { useProjectStore } from '../../store/projectStore';
import { isTauri } from '../../lib/backend';

const FILE_FILTERS = [
  { name: 'Open Energy Studio project', extensions: ['oes.json', 'oes', 'json'] },
];

export function HeaderBar() {
  const project = useProjectStore((s) => s.project);
  const filePath = useProjectStore((s) => s.filePath);
  const isDirty = useProjectStore((s) => s.isDirty);
  const isSaving = useProjectStore((s) => s.isSaving);
  const isLoading = useProjectStore((s) => s.isLoading);
  const newProject = useProjectStore((s) => s.newProject);
  const loadProject = useProjectStore((s) => s.loadProject);
  const saveProject = useProjectStore((s) => s.saveProject);

  const onOpen = async () => {
    if (!isTauri()) {
      console.warn('Open is only available in the desktop app for now.');
      return;
    }
    const picked = await openDialog({
      multiple: false,
      directory: false,
      filters: FILE_FILTERS,
    });
    if (typeof picked === 'string') {
      await loadProject(picked);
    }
  };

  const onSave = async () => {
    if (filePath) {
      await saveProject();
      return;
    }
    await onSaveAs();
  };

  const onSaveAs = async () => {
    if (!isTauri()) {
      console.warn('Save is only available in the desktop app for now.');
      return;
    }
    const picked = await saveDialog({
      defaultPath: project ? `${getProjectName(project) ?? 'project'}.oes.json` : 'project.oes.json',
      filters: FILE_FILTERS,
    });
    if (typeof picked === 'string') {
      await saveProject(picked);
    }
  };

  return (
    <header style={barStyle}>
      <div style={leftStyle}>
        <button type="button" style={btnStyle} onClick={() => newProject()} disabled={isLoading}>
          Nieuw
        </button>
        <button type="button" style={btnStyle} onClick={onOpen} disabled={isLoading}>
          Openen…
        </button>
        <button
          type="button"
          style={btnStyle}
          onClick={onSave}
          disabled={!project || isSaving}
        >
          {isSaving ? 'Opslaan…' : filePath ? 'Opslaan' : 'Opslaan…'}
        </button>
        <button
          type="button"
          style={btnStyle}
          onClick={onSaveAs}
          disabled={!project || isSaving}
        >
          Opslaan als…
        </button>
      </div>
      <div style={statusStyle}>
        {project ? (
          <>
            <span>{getProjectName(project) ?? 'Naamloos project'}</span>
            {filePath ? <span style={pathStyle}>· {trimPath(filePath)}</span> : null}
            {isDirty ? <span style={dirtyStyle}>· gewijzigd</span> : null}
          </>
        ) : (
          <span style={pathStyle}>Geen project geladen</span>
        )}
      </div>
    </header>
  );
}

function getProjectName(project: unknown): string | null {
  if (
    project &&
    typeof project === 'object' &&
    'shared' in project &&
    (project as { shared: unknown }).shared &&
    typeof (project as { shared: { name?: unknown } }).shared === 'object'
  ) {
    const name = (project as { shared: { name?: unknown } }).shared.name;
    if (typeof name === 'string' && name.length > 0) return name;
  }
  return null;
}

function trimPath(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] ?? path;
}

const barStyle: React.CSSProperties = {
  display: 'flex',
  justifyContent: 'space-between',
  alignItems: 'center',
  padding: '8px 16px',
  borderBottom: '1px solid var(--oes-border, #232830)',
  background: 'var(--oes-bg-elev, #14171d)',
  gap: 16,
};

const leftStyle: React.CSSProperties = { display: 'flex', gap: 6 };

const btnStyle: React.CSSProperties = {
  padding: '6px 12px',
  border: '1px solid var(--oes-border, #232830)',
  background: 'transparent',
  color: 'var(--oes-fg, #e6e9ee)',
  borderRadius: 5,
  fontSize: 12.5,
  cursor: 'pointer',
};

const statusStyle: React.CSSProperties = {
  fontSize: 12,
  color: 'var(--oes-fg-muted, #8a93a3)',
  display: 'flex',
  gap: 6,
};

const pathStyle: React.CSSProperties = {
  color: 'var(--oes-fg-muted, #8a93a3)',
};

const dirtyStyle: React.CSSProperties = {
  color: '#e9c46a',
};
