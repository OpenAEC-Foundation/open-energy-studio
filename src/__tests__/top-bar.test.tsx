/**
 * TopBar — the title-bar tests rewritten for the app shell (UI redesign F4):
 * title, window controls, New/Open/Save (now in the File menu and the Save
 * button), Settings, plus the new search, Recalculate and inspector toggle.
 */
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { TopBar, type TopBarProps } from '../components/shell/TopBar';
import { SettingsDialog } from '../components/SettingsDialog/SettingsDialog';

function props(overrides: Partial<TopBarProps> = {}): TopBarProps {
  return {
    hasDocument: true,
    onNewProject: vi.fn(), onOpenProject: vi.fn(), onSaveProject: vi.fn(), onSaveAsProject: vi.fn(),
    onImportUNIEC3: vi.fn(), onImportVABI: vi.fn(), onCloseTab: vi.fn(), onCloseActiveTab: vi.fn(),
    onOpenPalette: vi.fn(), onOpenSettings: vi.fn(), onOpenFeedback: vi.fn(), onRecalculate: vi.fn(),
    onToggleInspector: vi.fn(), inspectorOpen: true,
    ...overrides,
  };
}

const fileMenu = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.click(screen.getByRole('button', { name: /^File/ }));
  return screen.getByRole('menu', { name: 'File' });
};

describe('TopBar', () => {
  it('renders the app title in a banner landmark', () => {
    renderWithProviders(<TopBar {...props()} />);
    expect(within(screen.getByRole('banner')).getByText(/Open Energy Studio/)).toBeInTheDocument();
  });

  it('renders window control buttons (minimize, maximize, close)', () => {
    renderWithProviders(<TopBar {...props()} />);
    expect(screen.getByTitle('Minimize')).toBeInTheDocument();
    expect(screen.getByTitle(/Maximize/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Close' })).toHaveAttribute('title', 'Close');
  });

  it('calls onNewProject and onOpenProject from the File menu', async () => {
    const user = userEvent.setup();
    const p = props();
    renderWithProviders(<TopBar {...p} />);
    await user.click(within(await fileMenu(user)).getByRole('menuitem', { name: /^New/ }));
    expect(p.onNewProject).toHaveBeenCalledOnce();
    expect(screen.queryByRole('menu')).toBeNull();
    await user.click(within(await fileMenu(user)).getByRole('menuitem', { name: /^Open/ }));
    expect(p.onOpenProject).toHaveBeenCalledOnce();
  });

  it('calls onSaveProject from the Save button and Save As from the File menu', async () => {
    const user = userEvent.setup();
    const p = props();
    renderWithProviders(<TopBar {...p} />);
    await user.click(screen.getByTitle(/Save \(Ctrl\+S\)/));
    expect(p.onSaveProject).toHaveBeenCalledOnce();
    await user.click(within(await fileMenu(user)).getByRole('menuitem', { name: /^Save As/ }));
    expect(p.onSaveAsProject).toHaveBeenCalledOnce();
    await user.click(within(await fileMenu(user)).getByRole('menuitem', { name: /^Close tab/ }));
    expect(p.onCloseActiveTab).toHaveBeenCalledOnce();
  });

  it('opens the Settings dialog from the File menu', async () => {
    const user = userEvent.setup();
    function WithSettings() {
      const [open, setOpen] = useState(false);
      return <><TopBar {...props({ onOpenSettings: () => setOpen(true) })} />{open && <SettingsDialog onClose={() => setOpen(false)} />}</>;
    }
    renderWithProviders(<WithSettings />);
    await user.click(within(await fileMenu(user)).getByRole('menuitem', { name: /^Settings/ }));
    expect(screen.getByText(/Settings/i, { selector: '.dialog-header-title' })).toBeInTheDocument();
  });

  it('offers search, Recalculate and the inspector toggle', async () => {
    const user = userEvent.setup();
    const p = props();
    renderWithProviders(<TopBar {...p} />);
    await user.click(screen.getByRole('button', { name: /Search field, step or command/ }));
    expect(p.onOpenPalette).toHaveBeenCalledOnce();
    await user.click(screen.getByRole('button', { name: /^Recalculate/ }));
    expect(p.onRecalculate).toHaveBeenCalledOnce();
    const toggle = screen.getByRole('button', { name: 'Context panel' });
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    await user.click(toggle);
    expect(p.onToggleInspector).toHaveBeenCalledOnce();
  });

  it('offers only file commands without a document', async () => {
    const user = userEvent.setup();
    renderWithProviders(<TopBar {...props({ hasDocument: false })} />);
    expect(screen.queryByRole('button', { name: /^Recalculate/ })).toBeNull();
    const menu = await fileMenu(user);
    expect(within(menu).queryByRole('menuitem', { name: /^Save/ })).toBeNull();
    expect(within(menu).getByRole('menuitem', { name: 'UNIEC3 Import' })).toBeInTheDocument();
  });

  it('moves through the File menu with the arrow keys and closes it on Escape', async () => {
    const user = userEvent.setup();
    renderWithProviders(<TopBar {...props()} />);
    const menu = await fileMenu(user);
    const items = within(menu).getAllByRole('menuitem');
    expect(items[0]).toHaveFocus();
    await user.keyboard('{ArrowDown}');
    expect(items[1]).toHaveFocus();
    await user.keyboard('{ArrowUp}{ArrowUp}');
    expect(items[items.length - 1]).toHaveFocus();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(screen.getByRole('button', { name: /^File/ })).toHaveFocus();
  });
});
