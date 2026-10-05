/**
 * Settings (UI redesign F9): tabs Algemeen · Berekening · Over, the preview
 * toggle and the default edition for new calculations.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { SettingsDialog } from '../components/SettingsDialog/SettingsDialog';
import { DEFAULT_EDITION_KEY, readDefaultEdition } from '../core/nta/defaultEdition';
import { buildNtaCalculationTemplate } from '../core/nta/NtaCalculationTemplate';
import { createDefaultProject } from '../context/EnergyContext';

afterEach(() => {
  localStorage.removeItem(DEFAULT_EDITION_KEY);
});

describe('SettingsDialog', () => {
  it('has General, Calculation and About tabs with theme and language as radio groups', async () => {
    const user = userEvent.setup();
    renderWithProviders(<SettingsDialog onClose={vi.fn()} />);
    expect(screen.getAllByRole('tab').map((tab) => tab.textContent)).toEqual(['General', 'Calculation', 'About']);
    expect(screen.getByRole('radiogroup', { name: 'Theme' })).toBeInTheDocument();
    const language = screen.getByRole('radiogroup', { name: 'Select language' });
    expect(language.querySelector('[aria-checked="true"]')?.textContent).toContain('EN');
    await user.click(screen.getByRole('tab', { name: 'About' }));
    expect(screen.getByRole('tabpanel')).toHaveTextContent('Open Energy Studio');
  });

  it('commits the preview run and the default edition only on OK', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    const onClose = vi.fn();
    renderWithProviders(<SettingsDialog onClose={onClose} previewSetting={{ enabled: true, onChange }} />);
    await user.click(screen.getByRole('tab', { name: 'Calculation' }));
    await user.click(screen.getByRole('switch'));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Edition for new calculations' }), '2024');
    expect(screen.getByRole('note')).toHaveTextContent('cannot be registered');
    expect(readDefaultEdition()).toBe('2025+C1');
    await user.click(screen.getByRole('button', { name: 'OK' }));
    expect(onChange).toHaveBeenCalledWith(false);
    expect(onClose).toHaveBeenCalledOnce();
    expect(readDefaultEdition()).toBe('2024');
  });

  it('disables the preview toggle without an open document', async () => {
    const user = userEvent.setup();
    renderWithProviders(<SettingsDialog onClose={vi.fn()} />);
    await user.click(screen.getByRole('tab', { name: 'Calculation' }));
    expect(screen.getByRole('switch')).toBeDisabled();
  });
});

describe('default edition', () => {
  it('starts new NTA input in the chosen edition; the current edition stays implicit', () => {
    const project = createDefaultProject();
    expect(buildNtaCalculationTemplate(project)).not.toHaveProperty('normVersion');
    localStorage.setItem(DEFAULT_EDITION_KEY, '2024');
    expect(buildNtaCalculationTemplate(project)).toMatchObject({ normVersion: '2024' });
    localStorage.setItem(DEFAULT_EDITION_KEY, '2020+A1');
    // Not implemented by the kernel: falls back to the current edition.
    expect(readDefaultEdition()).toBe('2025+C1');
    expect(buildNtaCalculationTemplate(project)).not.toHaveProperty('normVersion');
  });
});
