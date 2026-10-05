/**
 * WelcomeScreen — functional tests (UI redesign F9: new dwelling or utility,
 * open, import, recent projects and the examples)
 */
import { afterEach, describe, it, expect, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { WelcomeScreen } from '../components/WelcomeScreen/WelcomeScreen';
import {
  RECENT_PROJECTS_KEY, RECENT_PROJECTS_MAX, forgetRecentProject, readRecentProjects, recordRecentProject,
} from '../core/io/recentProjects';

const noop = () => ({ onNewProject: vi.fn(), onOpenProject: vi.fn(), onOpenExample: vi.fn() });

afterEach(() => localStorage.removeItem(RECENT_PROJECTS_KEY));

describe('WelcomeScreen', () => {
  it('renders the app title', () => {
    renderWithProviders(<WelcomeScreen {...noop()} />);
    expect(screen.getByRole('heading', { level: 1, name: /Open Energy Studio/ })).toBeInTheDocument();
  });

  it('renders the subtitle', () => {
    renderWithProviders(<WelcomeScreen {...noop()} />);
    expect(screen.getByText(/Create or open a project/i)).toBeInTheDocument();
  });

  it('renders New and Open buttons', () => {
    renderWithProviders(<WelcomeScreen {...noop()} />);
    expect(screen.getByRole('button', { name: /New dwelling/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /New utility building/ })).toBeInTheDocument();
    expect(screen.getByText(/^Open$/, { selector: '.welcome-btn' })).toBeInTheDocument();
  });

  it('calls onNewProject with the building kind', async () => {
    const user = userEvent.setup();
    const props = noop();
    renderWithProviders(<WelcomeScreen {...props} />);
    await user.click(screen.getByRole('button', { name: /New dwelling/ }));
    await user.click(screen.getByRole('button', { name: /New utility building/ }));
    expect(props.onNewProject.mock.calls).toEqual([['residential'], ['utility']]);
  });

  it('calls onOpenProject when Open button is clicked', async () => {
    const user = userEvent.setup();
    const props = noop();
    renderWithProviders(<WelcomeScreen {...props} />);
    await user.click(screen.getByText(/^Open$/i));
    expect(props.onOpenProject).toHaveBeenCalledOnce();
  });

  it('imports UNIEC3 and VABI from the start screen', async () => {
    const user = userEvent.setup();
    const onImportUNIEC3 = vi.fn();
    const onImportVABI = vi.fn();
    renderWithProviders(<WelcomeScreen {...noop()} onImportUNIEC3={onImportUNIEC3} onImportVABI={onImportVABI} />);
    await user.click(screen.getByRole('button', { name: 'UNIEC3 Import' }));
    await user.click(screen.getByRole('button', { name: 'VABI Import' }));
    expect(onImportUNIEC3).toHaveBeenCalledOnce();
    expect(onImportVABI).toHaveBeenCalledOnce();
  });

  it('opens an example project from the start screen', async () => {
    const user = userEvent.setup();
    const props = noop();
    renderWithProviders(<WelcomeScreen {...props} />);
    await user.click(screen.getByText(/Example: terraced dwelling/i));
    await user.click(screen.getByText(/Example: small office/i));
    expect(props.onOpenExample.mock.calls).toEqual([['terraced_dwelling'], ['small_office']]);
    expect(screen.getByText(/Fictional practice projects/)).toHaveTextContent('no registered energy label');
  });

  it('lists recent projects, opens one and forgets one', async () => {
    const user = userEvent.setup();
    const onOpenRecent = vi.fn();
    const onForgetRecent = vi.fn();
    const recent = [
      { path: '/home/a/Woning.oes.json', name: 'Woning', buildingFunction: 'residential' as const, at: '2026-10-05T10:00:00Z' },
      { path: '/home/a/Kantoor.oes.json', name: 'Kantoor', at: '2026-10-04T10:00:00Z' },
    ];
    renderWithProviders(<WelcomeScreen {...noop()} recent={recent} onOpenRecent={onOpenRecent} onForgetRecent={onForgetRecent} />);
    const list = screen.getByRole('list');
    expect(within(list).getAllByRole('listitem')).toHaveLength(2);
    expect(within(list).getByText('/home/a/Woning.oes.json')).toBeInTheDocument();
    await user.click(within(list).getByRole('button', { name: /^Woning/ }));
    expect(onOpenRecent).toHaveBeenCalledWith('/home/a/Woning.oes.json');
    await user.click(within(list).getByRole('button', { name: 'Remove Kantoor from the list' }));
    expect(onForgetRecent).toHaveBeenCalledWith('/home/a/Kantoor.oes.json');
  });

  it('says where recent projects come from when there are none', () => {
    renderWithProviders(<WelcomeScreen {...noop()} />);
    expect(screen.getByText('Projects you open or save appear here.')).toBeInTheDocument();
  });
});

describe('recent projects store', () => {
  it('keeps one entry per path, newest first, at most eight', () => {
    recordRecentProject({ path: '/a.oes.json', name: 'A' }, new Date('2026-10-01T00:00:00Z'));
    recordRecentProject({ path: '/b.oes.json', name: 'B' }, new Date('2026-10-02T00:00:00Z'));
    recordRecentProject({ path: '/a.oes.json', name: 'A2' }, new Date('2026-10-03T00:00:00Z'));
    expect(readRecentProjects().map((entry) => [entry.path, entry.name])).toEqual([['/a.oes.json', 'A2'], ['/b.oes.json', 'B']]);
    for (let i = 0; i < 10; i += 1) recordRecentProject({ path: `/p${i}.oes.json`, name: `P${i}` });
    expect(readRecentProjects()).toHaveLength(RECENT_PROJECTS_MAX);
    expect(readRecentProjects()[0].path).toBe('/p9.oes.json');
    expect(forgetRecentProject('/p9.oes.json').map((entry) => entry.path)).not.toContain('/p9.oes.json');
  });

  it('ignores a corrupt or foreign list', () => {
    localStorage.setItem(RECENT_PROJECTS_KEY, '{not json');
    expect(readRecentProjects()).toEqual([]);
    localStorage.setItem(RECENT_PROJECTS_KEY, JSON.stringify([{ path: 3 }, { path: '/ok.json', name: 'Ok', at: 'x' }]));
    expect(readRecentProjects().map((entry) => entry.path)).toEqual(['/ok.json']);
  });
});
