/**
 * Project library — opening the project that is already open from the library
 * goes back to it (feedback 9 Oct 2026: "ik klik daarna weer op het project
 * gebeurt er niks").
 */
import { afterEach, describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { userEvent } from './test-utils';
import { createDefaultProject } from '../context/EnergyContext';
import { saveDocumentsToLibrary } from '../core/io/projectLibrary';
import App from '../App';

afterEach(() => localStorage.clear());

describe('project library', () => {
  it('opens the open project again after going to all projects', async () => {
    const project = { ...createDefaultProject(), name: 'Bibliotheekwoning' };
    saveDocumentsToLibrary(localStorage, [{ id: project.id, filePath: null, project }], new Map());
    const user = userEvent.setup();
    render(<App />);
    const openCard = () => screen.getByRole('button', { name: 'Open Bibliotheekwoning' });
    await user.click(screen.getByRole('button', { name: 'To all projects' }));
    await user.click(openCard());
    expect(await screen.findByRole('navigation', { name: 'Workflow steps' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'To all projects' }));
    expect(screen.queryByRole('navigation', { name: 'Workflow steps' })).toBeNull();
    await user.click(openCard());
    expect(await screen.findByRole('navigation', { name: 'Workflow steps' })).toBeInTheDocument();
  });
});
