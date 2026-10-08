/**
 * Autosave per document (feedback 8 Oct 2026: work was lost): two tabs do not
 * overwrite each other, a document that cannot be restored is kept, and the
 * single-key autosave of before is taken over.
 */
import { describe, expect, it } from 'vitest';
import { createDefaultProject, restoreFrom, saveTo, type DocumentEntry, type DocumentManagerState } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';

function memoryStore() {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => { data.set(key, value); },
    removeItem: (key: string) => { data.delete(key); },
  };
}

const project = (id: string, name: string): IProject => ({ ...createDefaultProject(), id, name });
const entry = (id: string, name: string): DocumentEntry => ({
  id, filePath: null,
  state: { ...restoreFrom(memoryStore()).documents[0].state, project: project(id, name), isDirty: true },
});
const tab = (...documents: DocumentEntry[]): DocumentManagerState => ({ documents, activeDocumentId: documents[0]?.id ?? null });

describe('autosave', () => {
  it('keeps the documents of two tabs apart', () => {
    const store = memoryStore();
    const a = entry('a', 'B01 survey');
    const b = entry('b', 'Other tab');
    let savedA = saveTo(store, tab(a), new Map());
    saveTo(store, tab(b), new Map());
    // Tab A edits its document again; tab B's document stays.
    const a2: DocumentEntry = { ...a, state: { ...a.state, project: project('a', 'B01 survey, edited') } };
    savedA = saveTo(store, tab(a2), savedA);
    const restored = restoreFrom(store);
    expect(restored.documents.map((doc) => doc.state.project.name).sort()).toEqual(['B01 survey, edited', 'Other tab']);
    // Closing a document in tab A removes only that one.
    saveTo(store, tab(), savedA);
    expect(restoreFrom(store).documents.map((doc) => doc.id)).toEqual(['b']);
  });

  it('skips a document it cannot restore and keeps it in storage', () => {
    const store = memoryStore();
    saveTo(store, tab(entry('good', 'Good')), new Map());
    store.setItem('oes.autosave.v2.doc.bad', JSON.stringify({ id: 'bad', filePath: null, project: { id: '' }, route: { step: 'project' }, isDirty: true }));
    store.setItem('oes.autosave.v2.index', JSON.stringify(['good', 'bad']));
    const restored = restoreFrom(store);
    expect(restored.documents.map((doc) => doc.id)).toEqual(['good']);
    // A save of the tab does not touch the document it could not open.
    saveTo(store, restored, new Map(restored.documents.map((doc) => [doc.id, doc])));
    expect(store.data.has('oes.autosave.v2.doc.bad')).toBe(true);
    expect(JSON.parse(store.getItem('oes.autosave.v2.index')!)).toContain('bad');
  });

  it('takes over the single-key autosave of before and keeps it as a backup', () => {
    const store = memoryStore();
    const old = { activeDocumentId: 'x', documents: [{ id: 'x', filePath: null, project: project('x', 'Old'), route: { step: 'project' }, isDirty: true }] };
    store.setItem('oes.autosave.v1', JSON.stringify(old));
    const restored = restoreFrom(store);
    expect(restored.documents.map((doc) => doc.state.project.name)).toEqual(['Old']);
    expect(store.getItem('oes.autosave.v1')).toBeNull();
    expect(store.getItem('oes.autosave.v1.backup')).toContain('"Old"');
  });
});
