/**
 * The project library (feedback 8 Oct 2026): cards with kind, outcome and
 * state kept per project in this browser; folders, search and the ways to
 * open, duplicate, move and delete.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { createDefaultProject } from '../context/EnergyContext';
import {
  libraryEntryFrom, libraryKindOf, libraryDwellingOf, readLibrary, rememberOutcome, removeFromLibrary, saveDocumentsToLibrary,
  sourceHashOf, updateLibraryEntry,
} from '../core/io/projectLibrary';
import { surveyTemplate } from '../core/nta/SurveyTemplates';
import { WelcomeScreen } from '../components/WelcomeScreen/WelcomeScreen';
import type { IProject } from '../core/energy/types';

afterEach(() => { localStorage.clear(); });

function surveyProject(name: string, extra: Record<string, unknown> = {}): IProject {
  const stored = surveyTemplate('residential');
  stored.survey = { ...stored.survey, ...extra };
  return { ...createDefaultProject(), id: `p-${name}`, name, zones: [], basisopname: { ...stored, progress: { done: ['woning.adres'] } } } as IProject;
}

describe('project library store', () => {
  it('derives kind, dwelling and state from a project', () => {
    const boat = surveyProject('Boot', { envelope: { surfaces: [], windows: [], doors: [], buildingKind: { kind: 'floating' } } });
    expect(libraryKindOf(boat)).toBe('existing_residential');
    expect(libraryDwellingOf(boat)).toBe('floating');
    expect(libraryKindOf(createDefaultProject())).toBe('new_residential');
    const entry = libraryEntryFrom(boat, null, null, '2026-10-08T10:00:00Z');
    expect(entry).toMatchObject({ id: 'p-Boot', kind: 'existing_residential', dwelling: 'floating', status: { surveyDone: 0, surveyTotal: expect.any(Number) } });
    expect(entry.status.openAnswers).toBeGreaterThan(0);
  });

  it('keeps a card after the document is gone, with folder, archive and a stale outcome', () => {
    const project = surveyProject('Woning');
    let seen = saveDocumentsToLibrary(localStorage, [{ id: 'doc', filePath: null, project }], new Map(), '2026-10-08T10:00:00Z');
    expect(readLibrary(localStorage).map((entry) => entry.name)).toEqual(['Woning']);
    // Unchanged project: nothing rewritten.
    seen = saveDocumentsToLibrary(localStorage, [{ id: 'doc', filePath: null, project }], seen, '2026-10-08T11:00:00Z');
    expect(readLibrary(localStorage)[0].updatedAt).toBe('2026-10-08T10:00:00Z');
    rememberOutcome(localStorage, project, { label: 'C', ep2: 180.5, beng1: 120, beng3: 10, tojuli: 1 }, '2026-10-08T11:30:00Z');
    expect(readLibrary(localStorage)[0]).toMatchObject({ outcome: { label: 'C', ep2: 180.5, sourceHash: sourceHashOf(project) }, outcomeStale: false });
    // The input changes afterwards: the outcome is kept but marked.
    const changed = { ...project, basisopname: { ...project.basisopname!, survey: { ...project.basisopname!.survey, constructionYear: 1930 } } } as IProject;
    saveDocumentsToLibrary(localStorage, [{ id: 'doc', filePath: null, project: changed }], seen, '2026-10-08T12:00:00Z');
    expect(readLibrary(localStorage)[0]).toMatchObject({ outcome: { label: 'C' }, outcomeStale: true, updatedAt: '2026-10-08T12:00:00Z' });
    updateLibraryEntry(localStorage, project.id, { folder: 'Van Dorp', archived: true });
    expect(readLibrary(localStorage)[0]).toMatchObject({ folder: 'Van Dorp', archived: true });
    // The document closes: the card and snapshot stay until removed.
    saveDocumentsToLibrary(localStorage, [], seen, '2026-10-08T13:00:00Z');
    expect(readLibrary(localStorage)).toHaveLength(1);
    removeFromLibrary(localStorage, project.id);
    expect(readLibrary(localStorage)).toHaveLength(0);
  });
});

describe('project library screen', () => {
  it('shows the cards with outcome and state, filters by folder and search, and opens a project', async () => {
    const user = userEvent.setup();
    const boat = surveyProject('Woonboot Maas', { envelope: { surfaces: [], windows: [], doors: [], buildingKind: { kind: 'floating' } } });
    const house = surveyProject('Tussenwoning Delft');
    saveDocumentsToLibrary(localStorage, [{ id: 'a', filePath: null, project: boat }, { id: 'b', filePath: null, project: house }], new Map(), '2026-10-08T10:00:00Z');
    rememberOutcome(localStorage, house, { label: 'E', ep2: 332.2, beng1: 169, beng3: 6.6, tojuli: 0.8, defaultsWithoutReason: 39 }, '2026-10-08T10:30:00Z');
    updateLibraryEntry(localStorage, boat.id, { folder: 'Van Dorp' });
    const onOpenEntry = vi.fn();
    const onNewProject = vi.fn();
    renderWithProviders(<WelcomeScreen onNewProject={onNewProject} onOpenProject={vi.fn()} onOpenExample={vi.fn()} onOpenEntry={onOpenEntry} />);
    expect(screen.getByRole('heading', { name: 'Projects' })).toBeInTheDocument();
    const houseCard = screen.getByRole('button', { name: 'Open Tussenwoning Delft' });
    expect(houseCard).toHaveTextContent('E');
    expect(houseCard).toHaveTextContent('EP2 332.2');
    expect(houseCard).toHaveTextContent('39 defaults without a reason');
    expect(screen.getByRole('button', { name: 'Open Woonboot Maas' })).toHaveTextContent('Houseboat');
    // Folder filter.
    await user.click(screen.getByRole('button', { name: /^Van Dorp/ }));
    expect(screen.queryByRole('button', { name: 'Open Tussenwoning Delft' })).toBeNull();
    await user.click(screen.getByRole('button', { name: /^All projects/ }));
    // Search.
    await user.type(screen.getByRole('searchbox', { name: 'Search by name, address, city or folder' }), 'delft');
    expect(screen.queryByRole('button', { name: 'Open Woonboot Maas' })).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Open Tussenwoning Delft' }));
    expect(onOpenEntry).toHaveBeenCalledWith(house.id);
    // New project menu.
    await user.click(screen.getByRole('button', { name: 'New project', expanded: false }));
    await user.click(within(screen.getByRole('menu')).getByRole('menuitem', { name: /Energy label existing dwelling|existing dwelling/i }));
    expect(onNewProject).toHaveBeenCalledWith('existing_residential');
  }, 60000);
});
