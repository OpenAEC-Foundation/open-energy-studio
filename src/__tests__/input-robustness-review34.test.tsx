// Review 34: untrusted input on the open and import paths. A project file
// with a byte-order mark, a file that is JSON but no object, and an import
// file far larger than any UNIEC3 or VABI file.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import App from '../App';
import { deserializeProjectFile, serializeProject } from '../core/io/ProjectSerializer';
import { importFile, MAX_IMPORT_FILE_BYTES } from '../core/io/importFile';
import { ImportFormatError, ImportIssues } from '../core/io/importIssues';
import { importErrorText } from '../core/io/importText';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';
import type { IProject } from '../core/energy/types';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { SLOW_UI_TEST_MS } from './timeouts';

const project = terraced as unknown as IProject;

function translate(table: Record<string, string>) {
  return (key: string, params?: Record<string, string | number>) => {
    const text = table[key];
    if (text === undefined) return key;
    return text.replace(/\{\{(\w+)\}\}/g, (_, name) => String(params?.[name] ?? ''));
  };
}

describe('opening a project file', () => {
  it('reads a file with a UTF-8 byte-order mark (the desktop read keeps it)', () => {
    const text = serializeProject(project);
    expect(deserializeProjectFile(`\uFEFF${text}`)).toEqual(deserializeProjectFile(text));
  });

  it('refuses JSON that is not an object with a format error', () => {
    for (const text of ['null', '[]', '5', '"x"', 'true']) {
      expect(() => deserializeProjectFile(text), text).toThrow('Invalid file format: expected a JSON object');
    }
  });
});

describe('importing a foreign file', () => {
  it('refuses a file larger than any import needs before reading it', async () => {
    const arrayBuffer = vi.fn(async () => new ArrayBuffer(0));
    const read = vi.fn(() => ({ project, issues: new ImportIssues() }));
    const file = { name: 'film.mp4', size: MAX_IMPORT_FILE_BYTES + 1, arrayBuffer };
    await expect(importFile(file, read)).rejects.toThrow(ImportFormatError);
    await expect(importFile(file, read)).rejects.toMatchObject({ code: 'file_too_large' });
    expect(arrayBuffer).not.toHaveBeenCalled();
    expect(read).not.toHaveBeenCalled();
  });

  it('names the file and the limit in NL and EN', async () => {
    const file = { name: 'film.mp4', size: MAX_IMPORT_FILE_BYTES + 1, arrayBuffer: async () => new ArrayBuffer(0) };
    const error = await importFile(file, () => ({ project, issues: new ImportIssues() })).catch((reason) => reason);
    for (const table of [nl, en]) {
      const text = importErrorText(error, translate(table));
      expect(text).not.toContain('import.error');
      expect(text).toContain('film.mp4');
      expect(text).toContain(String(MAX_IMPORT_FILE_BYTES));
    }
  });
});

describe('opening in the desktop app', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(isTauri).mockReturnValue(true);
  });

  afterEach(() => {
    vi.mocked(isTauri).mockReturnValue(false);
    vi.mocked(invoke).mockReset();
    vi.mocked(open).mockReset();
  });

  it('opens a project file that starts with a byte-order mark', async () => {
    vi.mocked(open).mockResolvedValue('/projecten/woning.oes.json');
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'read_project_file') return `\uFEFF${serializeProject({ ...project, name: 'Woning met BOM' })}`;
      throw new Error(`no kernel in this test: ${command}`);
    });
    render(<App />);
    fireEvent.keyDown(window, { key: 'o', ctrlKey: true });
    await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalledWith('read_project_file', { path: '/projecten/woning.oes.json' }));
    expect((await screen.findAllByText(/Woning met BOM/, undefined, { timeout: 10_000 })).length).toBeGreaterThan(0);
  }, SLOW_UI_TEST_MS);
});
