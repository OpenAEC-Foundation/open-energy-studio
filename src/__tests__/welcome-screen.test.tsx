/**
 * WelcomeScreen — functional tests
 */
import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { WelcomeScreen } from '../components/WelcomeScreen/WelcomeScreen';

describe('WelcomeScreen', () => {
  it('renders the app title', () => {
    renderWithProviders(<WelcomeScreen onNewProject={vi.fn()} onOpenProject={vi.fn()} onOpenExample={vi.fn()} />);
    expect(screen.getByText(/Open Energy Studio/)).toBeInTheDocument();
  });

  it('renders the subtitle', () => {
    renderWithProviders(<WelcomeScreen onNewProject={vi.fn()} onOpenProject={vi.fn()} onOpenExample={vi.fn()} />);
    expect(screen.getByText(/Create or open a project/i)).toBeInTheDocument();
  });

  it('renders New and Open buttons', () => {
    renderWithProviders(<WelcomeScreen onNewProject={vi.fn()} onOpenProject={vi.fn()} onOpenExample={vi.fn()} />);
    expect(screen.getByText(/New Project/i)).toBeInTheDocument();
    expect(screen.getByText(/^Open$/, { selector: '.welcome-btn' })).toBeInTheDocument();
  });

  it('calls onNewProject when New button is clicked', async () => {
    const user = userEvent.setup();
    const onNew = vi.fn();
    renderWithProviders(<WelcomeScreen onNewProject={onNew} onOpenProject={vi.fn()} onOpenExample={vi.fn()} />);
    await user.click(screen.getByText(/New Project/i));
    expect(onNew).toHaveBeenCalledOnce();
  });

  it('calls onOpenProject when Open button is clicked', async () => {
    const user = userEvent.setup();
    const onOpen = vi.fn();
    renderWithProviders(<WelcomeScreen onNewProject={vi.fn()} onOpenProject={onOpen} onOpenExample={vi.fn()} />);
    await user.click(screen.getByText(/^Open$/i));
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it('opens an example project from the start screen', async () => {
    const user = userEvent.setup();
    const onExample = vi.fn();
    renderWithProviders(<WelcomeScreen onNewProject={vi.fn()} onOpenProject={vi.fn()} onOpenExample={onExample} />);
    await user.click(screen.getByText(/Example: terraced dwelling/i));
    await user.click(screen.getByText(/Example: small office/i));
    expect(onExample.mock.calls).toEqual([['terraced_dwelling'], ['small_office']]);
  });
});
