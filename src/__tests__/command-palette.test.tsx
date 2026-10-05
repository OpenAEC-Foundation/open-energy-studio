/**
 * Command palette (Ctrl K) and the shell keyboard navigation (UI redesign F4).
 */
import { describe, expect, it, vi } from 'vitest';
import { act, render, screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { CommandPalette, buildPaletteEntries, filterEntries } from '../components/shell/CommandPalette';
import type { ShellActions } from '../components/shell/ShellActions';
import { createDefaultProject } from '../context/EnergyContext';
import { en } from '../i18n/en';
import { adjacentStep } from '../core/navigation/routes';
import App from '../App';

const t = (key: string) => en[key] ?? key;

function makeActions(): ShellActions {
  return {
    newProject: vi.fn(), openProject: vi.fn(), saveProject: vi.fn(), saveAsProject: vi.fn(), calculate: vi.fn(),
    openDialog: vi.fn(), navigate: vi.fn(), exportReport: vi.fn(), printReport: vi.fn(), exportIFC: vi.fn(),
    exportModelIFC: vi.fn(), exportUNIEC3: vi.fn(), importUNIEC3: vi.fn(), exportVABI: vi.fn(), importVABI: vi.fn(),
    openSettings: vi.fn(), openFeedback: vi.fn(), openPalette: vi.fn(), toggleInspector: vi.fn(), togglePreview: vi.fn(),
  };
}

describe('palette entries', () => {
  const project = createDefaultProject();
  const actions = makeActions();
  const select = vi.fn();
  const entries = buildPaletteEntries(t, project, actions, select);

  it('covers steps, commands, fields and project items', () => {
    const groups = new Set(entries.map((entry) => entry.group));
    expect([...groups].sort()).toEqual(['commands', 'fields', 'items', 'steps']);
    expect(entries.some((entry) => entry.label === 'Building')).toBe(true);
    expect(entries.some((entry) => entry.label === 'Add Zone')).toBe(true);
    expect(entries.some((entry) => entry.label === 'Raam Zuid 2')).toBe(true);
    expect(entries.some((entry) => entry.label === 'U-value Calculator')).toBe(true);
  });

  it('finds by every word, case-insensitively, prefix matches first', () => {
    expect(filterEntries(entries, 'add zo')[0].label).toBe('Add Zone');
    expect(filterEntries(entries, 'raam zuid 2').map((entry) => entry.label)).toContain('Raam Zuid 2');
    expect(filterEntries(entries, 'qqqxyz')).toEqual([]);
    // Without a query only steps and commands are listed.
    expect(filterEntries(entries, '').every((entry) => entry.group === 'steps' || entry.group === 'commands')).toBe(true);
  });

  it('selects a project item through the shell', () => {
    filterEntries(entries, 'raam zuid 2').find((entry) => entry.label === 'Raam Zuid 2')!.run();
    expect(select).toHaveBeenCalledWith(expect.objectContaining({ itemType: 'window', route: { step: 'building', sub: 'envelope' } }));
  });
});

describe('CommandPalette', () => {
  it('runs the highlighted entry with Enter and closes', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    const onClose = vi.fn();
    const entries = buildPaletteEntries(t, createDefaultProject(), actions, vi.fn());
    renderWithProviders(<CommandPalette entries={entries} onClose={onClose} />);
    const input = screen.getByRole('combobox');
    expect(input).toHaveFocus();
    await user.type(input, 'add heating');
    const options = screen.getAllByRole('option');
    expect(options[0]).toHaveAttribute('aria-selected', 'true');
    expect(input).toHaveAttribute('aria-activedescendant', options[0].id);
    await user.keyboard('{Enter}');
    expect(onClose).toHaveBeenCalled();
    expect(actions.openDialog).toHaveBeenCalledWith('heating-system');
  });

  it('moves the selection with the arrow keys and closes on Escape', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    renderWithProviders(<CommandPalette entries={buildPaletteEntries(t, null, makeActions(), vi.fn())} onClose={onClose} />);
    const options = () => screen.getAllByRole('option');
    await user.keyboard('{ArrowDown}');
    expect(options()[1]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{ArrowUp}{ArrowUp}');
    expect(options()[options().length - 1]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalledOnce();
  });

  it('says so when nothing matches', async () => {
    const user = userEvent.setup();
    renderWithProviders(<CommandPalette entries={[]} onClose={vi.fn()} />);
    await user.type(screen.getByRole('combobox'), 'zzz');
    expect(screen.getByRole('status')).toHaveTextContent('No results');
  });
});

describe('keyboard navigation of the shell', () => {
  it('steps through the workflow with Alt ↑/↓ order', () => {
    expect(adjacentStep('project', 1)).toBe('building');
    expect(adjacentStep('building', -1)).toBe('project');
    expect(adjacentStep('registration', 1)).toBe('registration');
    expect(adjacentStep('tool', 1)).toBe('project');
  });

  it('opens the palette with Ctrl K, navigates with Alt ↓ and has landmarks and a skip link', async () => {
    const user = userEvent.setup();
    window.history.replaceState(null, '', '/');
    render(<App />);
    // Open an example through the welcome screen's New action (Ctrl N).
    await user.keyboard('{Control>}n{/Control}');
    expect(await screen.findByRole('navigation', { name: 'Workflow steps' })).toBeInTheDocument();
    expect(screen.getByRole('main')).toHaveAttribute('id', 'main-content');
    // (jsdom also counts the card headers inside <main> as banners; the first is the top bar.)
    expect(screen.getAllByRole('banner')[0]).toHaveClass('top-bar');
    expect(screen.getByRole('contentinfo')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Skip to content' })).toHaveAttribute('href', '#main-content');

    const nav = screen.getByRole('navigation', { name: 'Workflow steps' });
    expect(within(nav).getByRole('button', { name: /^Project/ })).toHaveAttribute('aria-current', 'page');
    await act(async () => { document.body.focus(); });
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    expect(within(nav).getByRole('button', { name: /^Building/ })).toHaveAttribute('aria-current', 'page');
    expect(window.location.hash).toBe('#/gebouw');

    await user.keyboard('{Control>}k{/Control}');
    const dialog = screen.getByRole('dialog', { name: 'Command palette' });
    await user.type(within(dialog).getByRole('combobox'), 'tailored');
    await user.keyboard('{Enter}');
    expect(screen.queryByRole('dialog', { name: 'Command palette' })).toBeNull();
    expect(within(nav).getByRole('button', { name: /^Tailored advice/ })).toHaveAttribute('aria-current', 'page');
    window.history.replaceState(null, '', '/');
  }, 60000);
});
