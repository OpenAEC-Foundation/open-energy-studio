import { describe, expect, it, vi } from 'vitest';
import {
  confirmCloseAll,
  documentsNeedingConfirmation,
  isMissingFileError,
  saveProjectText,
  type ClosingDocument,
  type SaveDeps,
} from '../core/io/projectFiles';
import {
  deserializeProjectFile,
  FILE_VERSION,
  parseFileVersion,
  ProjectFileTooNewError,
  serializeProject,
} from '../core/io/ProjectSerializer';
import type { IProject } from '../core/energy/types';

function deps(overrides: Partial<SaveDeps> = {}): SaveDeps & { written: Array<[string, string]>; downloads: string[] } {
  const written: Array<[string, string]> = [];
  const downloads: string[] = [];
  return {
    desktop: true,
    pickPath: vi.fn(async () => '/projects/nieuw.oes.json'),
    write: vi.fn(async (path: string, text: string) => { written.push([path, text]); }),
    download: vi.fn((name: string) => { downloads.push(name); }),
    written,
    downloads,
    ...overrides,
  };
}

describe('saving a project file', () => {
  it('writes to the known path without asking', async () => {
    const d = deps();
    const outcome = await saveProjectText('{}', 'woning.oes.json', '/projects/woning.oes.json', false, d);
    expect(outcome).toEqual({ kind: 'saved', path: '/projects/woning.oes.json' });
    expect(d.pickPath).not.toHaveBeenCalled();
    expect(d.written).toEqual([['/projects/woning.oes.json', '{}']]);
  });

  it('asks for a path for a new file or Save As', async () => {
    const d = deps();
    expect(await saveProjectText('{}', 'woning.oes.json', null, false, d)).toEqual({ kind: 'saved', path: '/projects/nieuw.oes.json' });
    expect(await saveProjectText('{}', 'woning.oes.json', '/projects/woning.oes.json', true, d))
      .toEqual({ kind: 'saved', path: '/projects/nieuw.oes.json' });
    expect(d.pickPath).toHaveBeenLastCalledWith('/projects/woning.oes.json');
  });

  it('reports a cancelled dialog as cancelled', async () => {
    const d = deps({ pickPath: vi.fn(async () => null) });
    expect(await saveProjectText('{}', 'woning.oes.json', null, false, d)).toEqual({ kind: 'cancelled' });
    expect(d.written).toEqual([]);
  });

  it('reports a failed desktop write as a failure, never as a download', async () => {
    // Disk full, read-only folder, permission denied: the Rust command rejects
    // with a string. The document must stay dirty, so this is no save.
    const d = deps({ write: vi.fn(async () => { throw 'project_save_failed: No space left on device'; }) });
    const known = await saveProjectText('{}', 'woning.oes.json', '/projects/woning.oes.json', false, d);
    expect(known).toEqual({ kind: 'failed', message: 'project_save_failed: No space left on device' });
    const picked = await saveProjectText('{}', 'woning.oes.json', null, false, d);
    expect(picked.kind).toBe('failed');
    expect(d.download).not.toHaveBeenCalled();
    expect(d.pickPath).toHaveBeenCalledTimes(1);
  });

  it('offers a download in the browser build', async () => {
    const d = deps({ desktop: false });
    expect(await saveProjectText('{}', 'woning.oes.json', null, false, d)).toEqual({ kind: 'downloaded' });
    expect(d.downloads).toEqual(['woning.oes.json']);
    expect(d.write).not.toHaveBeenCalled();
  });
});

describe('the recent list', () => {
  it('forgets an entry only when the file is gone', () => {
    expect(isMissingFileError('project_file_not_found: /projects/weg.oes.json')).toBe(true);
    expect(isMissingFileError('project_path_not_allowed: /projects/woning.oes.json')).toBe(false);
    expect(isMissingFileError(new Error('forbidden path: /projects/woning.oes.json'))).toBe(false);
    expect(isMissingFileError('project_file_unreadable: Permission denied')).toBe(false);
  });
});

describe('closing the window', () => {
  const clean: ClosingDocument = { id: 'a', name: 'A', isDirty: false, draftDirty: false };
  const dirty: ClosingDocument = { id: 'b', name: 'B', isDirty: true, draftDirty: false };
  const draft: ClosingDocument = { id: 'c', name: 'C', isDirty: false, draftDirty: true };

  function closeDeps(choice: 'save' | 'discard' | 'cancel', saved = true, discardDraft = true) {
    return {
      confirmDiscardDraft: vi.fn(async () => discardDraft),
      askUnsaved: vi.fn(async () => choice),
      save: vi.fn(async () => saved),
    };
  }

  it('closes at once when nothing is unsaved', async () => {
    const d = closeDeps('cancel');
    expect(documentsNeedingConfirmation([clean])).toEqual([]);
    expect(await confirmCloseAll([clean], d)).toBe(true);
    expect(d.askUnsaved).not.toHaveBeenCalled();
  });

  it('asks per unsaved document and saves on request', async () => {
    const d = closeDeps('save');
    expect(await confirmCloseAll([clean, dirty, { ...dirty, id: 'd' }], d)).toBe(true);
    expect(d.askUnsaved).toHaveBeenCalledTimes(2);
    expect(d.save).toHaveBeenCalledTimes(2);
  });

  it('stays open when the user cancels', async () => {
    expect(await confirmCloseAll([dirty], closeDeps('cancel'))).toBe(false);
  });

  it('stays open when a save fails or is cancelled', async () => {
    const d = closeDeps('save', false);
    expect(await confirmCloseAll([dirty, { ...dirty, id: 'd' }], d)).toBe(false);
    expect(d.save).toHaveBeenCalledTimes(1);
  });

  it('closes without saving on "Niet opslaan"', async () => {
    const d = closeDeps('discard');
    expect(await confirmCloseAll([dirty], d)).toBe(true);
    expect(d.save).not.toHaveBeenCalled();
  });

  it('asks about unapplied NTA input first', async () => {
    const keep = closeDeps('discard', true, false);
    expect(await confirmCloseAll([draft], keep)).toBe(false);
    const discard = closeDeps('discard', true, true);
    expect(await confirmCloseAll([draft], discard)).toBe(true);
    expect(discard.askUnsaved).not.toHaveBeenCalled();
  });
});

describe('file versions', () => {
  const project = { id: 'p', name: 'Woning' } as unknown as IProject;

  it('writes and reads version 1.0', () => {
    const loaded = deserializeProjectFile(serializeProject(project));
    expect(loaded.fileVersion).toBe(FILE_VERSION);
    expect(loaded.unknownFields).toBeUndefined();
  });

  it('reads a file without a version as the original 1.0', () => {
    const loaded = deserializeProjectFile(JSON.stringify({ type: 'open-energy-studio', project }));
    expect(loaded.fileVersion).toBe('1.0');
    expect(parseFileVersion(undefined)).toEqual({ major: 1, minor: 0, text: '1.0' });
  });

  it('refuses a newer major version instead of reading it as 1.0', () => {
    const json = JSON.stringify({ version: '9.0', type: 'open-energy-studio', project });
    expect(() => deserializeProjectFile(json)).toThrow(ProjectFileTooNewError);
    try {
      deserializeProjectFile(json);
    } catch (error) {
      expect((error as ProjectFileTooNewError).fileVersion).toBe('9.0');
    }
  });

  it('opens a newer 1.x file and names the fields it would drop', () => {
    const json = JSON.stringify({ version: '1.4', type: 'open-energy-studio', project, attachments: [], signature: 'x' });
    const loaded = deserializeProjectFile(json);
    expect(loaded.fileVersion).toBe('1.4');
    expect(loaded.unknownFields).toEqual(['attachments', 'signature']);
  });

  it('refuses a version that is not a number', () => {
    expect(() => deserializeProjectFile(JSON.stringify({ version: 'beta', type: 'open-energy-studio', project })))
      .toThrow(/unknown version/);
  });
});
