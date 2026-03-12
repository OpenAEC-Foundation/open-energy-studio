/**
 * DocumentTabs — functional tests
 *
 * Tests tab rendering, tab switching, close button, and + menu.
 */
import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { DocumentTabs } from '../components/DocumentTabs/DocumentTabs';

describe('DocumentTabs', () => {
  const defaultProps = {
    onCloseTab: vi.fn(),
    onNewProject: vi.fn(),
    onOpenProject: vi.fn(),
  };

  it('renders the + (add) button', () => {
    renderWithProviders(<DocumentTabs {...defaultProps} />);
    const addBtn = document.querySelector('.document-tab-add');
    expect(addBtn).toBeTruthy();
  });

  it('opens dropdown menu when + button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DocumentTabs {...defaultProps} />);
    const addBtn = document.querySelector('.document-tab-add') as HTMLElement;
    await user.click(addBtn);
    // The dropdown menu should appear
    expect(document.querySelector('.document-tab-menu')).toBeTruthy();
  });

  it('calls onNewProject from the dropdown menu', async () => {
    const user = userEvent.setup();
    const props = { ...defaultProps, onNewProject: vi.fn() };
    renderWithProviders(<DocumentTabs {...props} />);
    const addBtn = document.querySelector('.document-tab-add') as HTMLElement;
    await user.click(addBtn);
    // Click "New" in the menu
    const menuItems = document.querySelectorAll('.document-tab-menu-item');
    await user.click(menuItems[0] as HTMLElement); // First item is "New"
    expect(props.onNewProject).toHaveBeenCalledOnce();
  });

  it('calls onOpenProject from the dropdown menu', async () => {
    const user = userEvent.setup();
    const props = { ...defaultProps, onOpenProject: vi.fn() };
    renderWithProviders(<DocumentTabs {...props} />);
    const addBtn = document.querySelector('.document-tab-add') as HTMLElement;
    await user.click(addBtn);
    const menuItems = document.querySelectorAll('.document-tab-menu-item');
    await user.click(menuItems[1] as HTMLElement); // Second item is "Open"
    expect(props.onOpenProject).toHaveBeenCalledOnce();
  });

  it('closes dropdown on Escape key', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DocumentTabs {...defaultProps} />);
    const addBtn = document.querySelector('.document-tab-add') as HTMLElement;
    await user.click(addBtn);
    expect(document.querySelector('.document-tab-menu')).toBeTruthy();
    await user.keyboard('{Escape}');
    expect(document.querySelector('.document-tab-menu')).toBeFalsy();
  });
});
