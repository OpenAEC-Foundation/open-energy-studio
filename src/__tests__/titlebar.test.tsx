/**
 * TitleBar — functional tests
 */
import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { TitleBar } from '../components/TitleBar/TitleBar';

describe('TitleBar', () => {
  it('renders the app title', () => {
    renderWithProviders(<TitleBar />);
    expect(screen.getByText(/Open Energy Studio/)).toBeInTheDocument();
  });

  it('renders window control buttons (minimize, maximize, close)', () => {
    renderWithProviders(<TitleBar />);
    expect(screen.getByTitle('Minimize')).toBeInTheDocument();
    expect(screen.getByTitle(/Maximize/)).toBeInTheDocument();
    expect(screen.getByTitle('Close')).toBeInTheDocument();
  });

  it('calls onNewProject when New button is clicked', async () => {
    const user = userEvent.setup();
    const onNew = vi.fn();
    renderWithProviders(<TitleBar onNewProject={onNew} />);
    const btn = screen.getByTitle(/New/i);
    await user.click(btn);
    expect(onNew).toHaveBeenCalledOnce();
  });

  it('calls onOpenProject when Open button is clicked', async () => {
    const user = userEvent.setup();
    const onOpen = vi.fn();
    renderWithProviders(<TitleBar onOpenProject={onOpen} />);
    const btn = screen.getByTitle(/Open/i);
    await user.click(btn);
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it('calls onSaveProject when Save button is clicked', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    renderWithProviders(<TitleBar onSaveProject={onSave} />);
    const btn = screen.getByTitle(/Save/i);
    await user.click(btn);
    expect(onSave).toHaveBeenCalledOnce();
  });

  it('opens the Settings dialog when settings button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<TitleBar />);
    const btn = screen.getByTitle(/Settings/i);
    await user.click(btn);
    // SettingsDialog should now be visible
    expect(screen.getByText(/Settings/i, { selector: '.dialog-header-title' })).toBeInTheDocument();
  });
});
