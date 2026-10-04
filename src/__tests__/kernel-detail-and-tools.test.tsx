import { afterEach, describe, expect, it } from 'vitest';
import { act, screen } from '@testing-library/react';
import i18next from '../i18n/i18n';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';
import { kernelDetailText } from '../i18n/kernelDetail';
import { dutchDetailHtml } from '../core/report/DutchReportText';
import { UValueCalculator } from '../components/UValueCalculator/UValueCalculator';
import { ThermalBridgeCalculator } from '../components/ThermalBridgeCalculator/ThermalBridgeCalculator';
import { renderWithProviders } from './test-utils';

const lookup = (table: Record<string, string>) => (key: string) => table[key] ?? key;

describe('kernel detail texts', () => {
  it('translates a known detail with locale numbers', () => {
    expect(kernelDetailText('A_ls/A_g 200001.4', lookup(nl), 'nl')).toEqual({ text: 'A_ls/A_g = 200.001,4', translated: true });
    expect(kernelDetailText('A_ls/A_g 200001.4', lookup(en), 'en')).toEqual({ text: 'A_ls/A_g = 200,001.4', translated: true });
    expect(kernelDetailText('12.5 m² per dwelling', lookup(nl), 'nl').text).toBe('12,5 m² per woning');
    expect(kernelDetailText('1500000 > 1000000', lookup(nl), 'nl').text).toBe('1.500.000 is groter dan 1.000.000');
    expect(kernelDetailText('registration 1975 against ntaCalculation 1980', lookup(nl), 'nl').text)
      .toBe('Registratie 1975, rekeninvoer 1980');
  });

  it('translates the fixed vertical-pipe hint', () => {
    const detail = '7.3.3: zone list [] (none) conflicts with the project-level pipes; remove one of the two';
    const result = kernelDetailText(detail, lookup(nl), 'nl');
    expect(result.translated).toBe(true);
    expect(result.text).toContain('zonelijst');
  });

  it('shows a nested code by its label', () => {
    const result = kernelDetailText('ventilation_flow_required', lookup(nl), 'nl');
    expect(result.translated).toBe(true);
    expect(result.text).toContain('(ventilation_flow_required)');
  });

  it('keeps an unknown detail as it is', () => {
    expect(kernelDetailText('something unforeseen 3.2', lookup(nl), 'nl')).toEqual({ text: 'something unforeseen 3.2', translated: false });
    expect(dutchDetailHtml('something unforeseen')).toBe('<span lang="en">Technisch detail: something unforeseen</span>');
    expect(dutchDetailHtml(undefined)).toBe('');
  });
});

describe('calculator labels', () => {
  afterEach(async () => {
    await act(async () => { await i18next.changeLanguage('en'); });
  });

  it('the U-value calculator shows translated labels and Dutch decimals', async () => {
    await act(async () => { await i18next.changeLanguage('nl'); });
    const { container } = renderWithProviders(<UValueCalculator />);
    expect(screen.getByRole('heading', { name: 'U-waardecalculator' })).toBeInTheDocument();
    expect(container.textContent).not.toMatch(/uvalue\./);
    // Rsi of a wall, 0,13, in Dutch notation.
    expect(container.textContent).toContain('0,13');
  });

  it('the thermal-bridge calculator shows translated labels', async () => {
    await act(async () => { await i18next.changeLanguage('nl'); });
    const { container } = renderWithProviders(<ThermalBridgeCalculator />);
    expect(screen.getByRole('heading', { name: 'Koudebrugcalculator' })).toBeInTheDocument();
    expect(container.textContent).not.toMatch(/\btb\./);
  });
});
