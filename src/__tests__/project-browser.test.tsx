/**
 * ProjectBrowser — functional tests
 *
 * Tests tree rendering, node expansion/collapse, selection, and panel collapse.
 */
import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { ProjectBrowser } from '../components/ProjectBrowser/ProjectBrowser';

describe('ProjectBrowser', () => {
  it('renders the panel header', () => {
    renderWithProviders(<ProjectBrowser />);
    expect(screen.getByText(/Project/i, { selector: '.project-browser-header span' })).toBeInTheDocument();
  });

  it('renders the Zones tree node (default open)', () => {
    renderWithProviders(<ProjectBrowser />);
    expect(screen.getByText(/Zones/i)).toBeInTheDocument();
  });

  it('renders default zone name from the demo project', () => {
    renderWithProviders(<ProjectBrowser />);
    expect(screen.getByText(/Woonfunctie/)).toBeInTheDocument();
  });

  it('renders Constructions tree node', () => {
    renderWithProviders(<ProjectBrowser />);
    expect(screen.getByText(/Constructions/i)).toBeInTheDocument();
  });

  it('renders Installations tree node', () => {
    renderWithProviders(<ProjectBrowser />);
    expect(screen.getByText(/Installations/i)).toBeInTheDocument();
  });

  it('renders Renewables tree node', () => {
    renderWithProviders(<ProjectBrowser />);
    expect(screen.getByText(/Renewables/i)).toBeInTheDocument();
  });

  it('collapses the panel when collapse button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProjectBrowser />);
    // Find the collapse button in the header
    const collapseBtn = screen.getByRole('button', { name: '' });
    // After clicking, the collapsed label should appear
    await user.click(collapseBtn);
    // The panel-collapsed-label should be in the document
    const collapsed = document.querySelector('.project-browser.collapsed');
    expect(collapsed).toBeTruthy();
  });

  it('expands a collapsed panel when clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProjectBrowser />);
    // Collapse first
    const collapseBtn = document.querySelector('.panel-collapse-btn') as HTMLElement;
    await user.click(collapseBtn);
    // Now click the collapsed panel to expand
    const collapsedPanel = document.querySelector('.project-browser.collapsed') as HTMLElement;
    await user.click(collapsedPanel);
    // Should no longer have .collapsed
    expect(document.querySelector('.project-browser.collapsed')).toBeFalsy();
    expect(screen.getByText(/Zones/i)).toBeInTheDocument();
  });

  it('renders surfaces under a zone (when zone node is expanded)', () => {
    renderWithProviders(<ProjectBrowser />);
    // Default project has surfaces like "Gevel Noord"
    expect(screen.getByText(/Surfaces/i)).toBeInTheDocument();
  });

  it('renders heating systems under Installations', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProjectBrowser />);
    // Click "Heating" to expand
    const heatingNode = screen.getByText(/Heating/i);
    await user.click(heatingNode);
    // Default project has a heating system
    expect(screen.getByText(/Warmtepomp lucht/i)).toBeInTheDocument();
  });
});
