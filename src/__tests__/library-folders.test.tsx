/**
 * Project library folders and trash (feedback 9 Oct 2026: "een mappenstructuur
 * zoals in Uniec3"): empty folders, rename and delete, and a trash from which a
 * project is restored or deleted for good.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { createDefaultProject } from '../context/EnergyContext';
import {
  createFolder, deleteFolder, emptyTrash, libraryFolders, readLibrary, readLibraryEntry, renameFolder, saveDocumentsToLibrary, updateLibraryEntry,
} from '../core/io/projectLibrary';
import { WelcomeScreen } from '../components/WelcomeScreen/WelcomeScreen';

afterEach(() => { localStorage.clear(); vi.restoreAllMocks(); });

const add = (id: string, name: string) => {
  const project = { ...createDefaultProject(), id, name };
  saveDocumentsToLibrary(localStorage, [{ id, filePath: null, project }], new Map());
};

describe('library folders', () => {
  it('keeps an empty folder, renames it with its projects and deletes it without losing them', () => {
    add('a', 'Woning A');
    expect(createFolder(localStorage, 'Domera Projecten')).toBe(true);
    expect(createFolder(localStorage, 'Domera Projecten')).toBe(false);
    expect(libraryFolders(readLibrary(localStorage), localStorage)).toEqual(['Domera Projecten']);
    updateLibraryEntry(localStorage, 'a', { folder: 'Domera Projecten' });
    expect(renameFolder(localStorage, 'Domera Projecten', '3BM Projecten')).toBe(true);
    expect(readLibraryEntry(localStorage, 'a')?.folder).toBe('3BM Projecten');
    expect(libraryFolders(readLibrary(localStorage), localStorage)).toEqual(['3BM Projecten']);
    deleteFolder(localStorage, '3BM Projecten');
    expect(readLibraryEntry(localStorage, 'a')?.folder).toBeUndefined();
    expect(libraryFolders(readLibrary(localStorage), localStorage)).toEqual([]);
  });

  it('keeps a trashed project out of the folders until the trash is emptied', () => {
    add('a', 'Woning A');
    add('b', 'Woning B');
    updateLibraryEntry(localStorage, 'a', { trashed: '2026-10-09T10:00:00Z' });
    // A later save of the project keeps it in the trash.
    add('a', 'Woning A');
    expect(readLibraryEntry(localStorage, 'a')?.trashed).toBe('2026-10-09T10:00:00Z');
    emptyTrash(localStorage);
    expect(readLibrary(localStorage).map((entry) => entry.id)).toEqual(['b']);
  });
});

describe('library folders on the start screen', () => {
  const props = () => ({ onNewProject: vi.fn(), onOpenProject: vi.fn(), onOpenExample: vi.fn(), onTrashEntry: vi.fn(), onDeleteEntry: vi.fn() });

  it('makes a folder from the Folders menu and shows it while empty', async () => {
    vi.spyOn(window, 'prompt').mockReturnValue('W25.056 Lineaire');
    const user = userEvent.setup();
    renderWithProviders(<WelcomeScreen {...props()} />);
    await user.click(screen.getByRole('button', { name: 'Folders' }));
    await user.click(screen.getByRole('menuitem', { name: /New folder/ }));
    const side = within(screen.getByRole('complementary', { name: 'Folders' }));
    expect(side.getByRole('button', { name: /^W25\.056 Lineaire/ })).toHaveAttribute('aria-current', 'true');
    expect(screen.getByText(/This folder is empty/)).toBeInTheDocument();
    expect(side.getByRole('button', { name: 'Folder W25.056 Lineaire' })).toBeInTheDocument();
  });

  it('moves a project to the trash and restores or deletes it from there', async () => {
    add('a', 'Woning A');
    const user = userEvent.setup();
    const handlers = props();
    renderWithProviders(<WelcomeScreen {...handlers} />);
    const card = screen.getByRole('button', { name: 'Open Woning A' }).closest('article')!;
    await user.click(within(card).getByRole('button', { expanded: false }));
    await user.click(screen.getByRole('menuitem', { name: /Move to trash/ }));
    expect(handlers.onTrashEntry).toHaveBeenCalledWith('a', true);

    act(() => { updateLibraryEntry(localStorage, 'a', { trashed: '2026-10-09T10:00:00Z' }); });
    expect(screen.queryByRole('button', { name: 'Open Woning A' })).toBeNull();
    await user.click(screen.getByRole('button', { name: /^Trash/ }));
    const trashed = screen.getByRole('button', { name: 'Open Woning A' }).closest('article')!;
    await user.click(within(trashed).getByRole('button', { expanded: false }));
    expect(screen.queryByRole('menuitem', { name: /Duplicate/ })).toBeNull();
    await user.click(screen.getByRole('menuitem', { name: /Restore/ }));
    expect(handlers.onTrashEntry).toHaveBeenLastCalledWith('a', false);
    await user.click(within(trashed).getByRole('button', { expanded: false }));
    await user.click(screen.getByRole('menuitem', { name: /Delete for good/ }));
    expect(handlers.onDeleteEntry).toHaveBeenCalledWith('a');
  });
});
