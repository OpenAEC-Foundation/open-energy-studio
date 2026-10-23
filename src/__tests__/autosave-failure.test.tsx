/**
 * Issue #13: a write the browser storage refuses (full or blocked) is not
 * counted as saved. The autosave and the project library write the same,
 * unchanged document again once the storage accepts it, and the status bar
 * shows the failure meanwhile.
 */
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { createDefaultProject, restoreFrom, saveTo, type DocumentEntry, type DocumentManagerState } from '../context/EnergyContext';
import { readLibraryProject, saveDocumentsToLibrary } from '../core/io/projectLibrary';
import { StatusBar } from '../components/StatusBar/StatusBar';
import type { IProject } from '../core/energy/types';
import { renderWithProviders } from './test-utils';

/** A store whose `setItem` throws while `full` is set. */
function flakyStore() {
  const data = new Map<string, string>();
  const store = {
    data,
    full: false,
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => {
      if (store.full) throw new DOMException('quota', 'QuotaExceededError');
      data.set(key, value);
    },
    removeItem: (key: string) => { data.delete(key); },
  };
  return store;
}

const project = (id: string, name: string): IProject => ({ ...createDefaultProject(), id, name });
const entry = (id: string, name: string): DocumentEntry => ({
  id, filePath: null,
  state: { ...restoreFrom(flakyStore()).documents[0].state, project: project(id, name), isDirty: true },
});
const tab = (...documents: DocumentEntry[]): DocumentManagerState => ({ documents, activeDocumentId: documents[0]?.id ?? null });

describe('autosave when the storage refuses a write', () => {
  it('writes the unchanged document again once the storage accepts it', () => {
    const store = flakyStore();
    const doc = entry('a', 'Woning');
    store.full = true;
    const failed: string[] = [];
    const seen = saveTo(store, tab(doc), new Map(), failed);
    expect(failed.length).toBeGreaterThan(0);
    expect(restoreFrom(store).documents.some((item) => item.id === 'a')).toBe(false);
    // Storage back; the same document, not edited, is written now.
    store.full = false;
    const retry: string[] = [];
    saveTo(store, tab(doc), seen, retry);
    expect(retry).toEqual([]);
    expect(restoreFrom(store).documents.map((item) => item.state.project.name)).toEqual(['Woning']);
  });

  it('keeps an older recovery copy until the newer one is written', () => {
    const store = flakyStore();
    const doc = entry('a', 'Versie 1');
    const seen = saveTo(store, tab(doc), new Map());
    const edited: DocumentEntry = { ...doc, state: { ...doc.state, project: project('a', 'Versie 2') } };
    store.full = true;
    const failed: string[] = [];
    const after = saveTo(store, tab(edited), seen, failed);
    expect(failed).toContain('a');
    store.full = false;
    saveTo(store, tab(edited), after);
    expect(restoreFrom(store).documents.map((item) => item.state.project.name)).toEqual(['Versie 2']);
  });

  it('retries a library snapshot that failed', () => {
    const store = flakyStore();
    const documents = [{ id: 'doc', filePath: null, project: project('p1', 'Kantoor') }];
    store.full = true;
    const failed: string[] = [];
    const seen = saveDocumentsToLibrary(store, documents, new Map(), '2026-10-23T10:00:00Z', failed);
    expect(failed).toEqual(['doc']);
    expect(readLibraryProject(store, 'p1')).toBeNull();
    store.full = false;
    const retry: string[] = [];
    saveDocumentsToLibrary(store, documents, seen, '2026-10-23T10:00:05Z', retry);
    expect(retry).toEqual([]);
    expect(readLibraryProject(store, 'p1')?.name).toBe('Kantoor');
  });

  it('says nothing in the status bar while the autosave works', () => {
    renderWithProviders(<StatusBar />);
    expect(screen.queryByText('Recovery copy failed')).toBeNull();
  });
});
