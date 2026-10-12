import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { ErrorState } from '../components/ui';
import { ResultsView } from '../components/ResultsView/ResultsView';
import { ReportBuilder } from '../components/ReportView/ReportBuilder';
import { RegistrationForm } from '../components/shell/pages/RegistrationForm';
import { GasChainReferencePanel } from '../components/GasChainReferencePanel/GasChainReferencePanel';
import { fieldLabelIn } from '../components/shell/focusPath';
import { createDefaultProject } from '../context/EnergyContext';
import { nl } from '../i18n/nl';
import { kernelCodeLabelsNl } from '../i18n/kernelCodeLabels';
import i18next from '../i18n/i18n';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

// The UX findings of 10 October 2026 (docs/ui-redesign/ontwerp.md, "UX-bevindingen").

const root = resolve(__dirname, '../..');

afterEach(async () => {
  vi.unstubAllGlobals();
  await i18next.changeLanguage('en');
});

describe('UX findings of 10 October 2026', () => {
  it('1: folds the raw technical error away behind "Technical details"', () => {
    renderWithProviders(<ErrorState title="No connection" details="Rust API: HTTP 500">The kernel could not be reached.</ErrorState>);
    const alert = screen.getByRole('alert');
    const details = alert.querySelector('details');
    expect(details).not.toBeNull();
    expect(details!.open).toBe(false);
    expect(within(alert).getByText('Technical details').tagName).toBe('SUMMARY');
    // The message itself does not carry the raw error.
    expect(within(alert).getByText('The kernel could not be reached.').textContent).not.toContain('HTTP 500');
    expect(details!.textContent).toContain('Rust API: HTTP 500');
  });

  it('1: shows no disclosure without a technical detail', () => {
    renderWithProviders(<ErrorState title="No connection">Try again.</ErrorState>);
    expect(screen.getByRole('alert').querySelector('details')).toBeNull();
  });

  it('2: offers Recalculate and the check overview when the kernel refused the input', async () => {
    vi.stubGlobal('fetch', vi.fn((url: string) => (String(url).includes('project/performance')
      ? Promise.resolve({ ok: true, json: async () => ({ status: 'incomplete',
        gaps: [{ code: 'ventilation_flow_required', path: 'ventilation' }], derivedInput: null, performance: null }) })
      : new Promise(() => {}))));
    renderWithProviders(<ResultsView />);
    expect(await screen.findByRole('button', { name: 'Go to Check › Check overview' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Recalculate' })).toBeInTheDocument();
  }, SLOW_UI_TEST_MS);

  it('3: keeps report export and print off without a kernel result, and says why', () => {
    renderWithProviders(<ReportBuilder project={createDefaultProject()} assessment={null} pending={false} />);
    for (const name of ['Download report (pdf)', 'Print or save as PDF']) {
      const button = screen.getByRole('button', { name });
      expect(button).toBeDisabled();
      expect(button).toHaveAccessibleDescription('Export and print become available once the kernel has a result.');
    }
    expect(screen.getByText('Export and print become available once the kernel has a result.')).toBeInTheDocument();
  });

  it('4: reads the form label of a reference field for Bron & bewijs', () => {
    const container = document.createElement('div');
    container.innerHTML = `
      <section data-path="ntaCalculation.generator">
        <label data-path="ntaCalculation.generator.boiler.equipmentReference">Bron toestel
          <input type="text" /></label>
        <div data-path="ntaCalculation.generator.boiler"><input id="other" /></div>
      </section>`;
    document.body.append(container);
    expect(fieldLabelIn(container, 'ntaCalculation.generator.boiler.equipmentReference')).toBe('Bron toestel');
    // A block that only contains the path is not the field's label.
    expect(fieldLabelIn(container, 'ntaCalculation.generator.boiler.temperatureReference')).toBeNull();
    container.remove();
  });

  it('5: counts with plural forms instead of "fout(en)"', async () => {
    await i18next.changeLanguage('nl');
    expect(i18next.t('nav.state.errors', { count: 0 })).toBe('geen fouten');
    expect(i18next.t('nav.state.errors', { count: 1 })).toBe('1 fout');
    expect(i18next.t('nav.state.errors', { count: 3 })).toBe('3 fouten');
    expect(i18next.t('ntaStep.check.warnings', { count: 1 })).toBe('1 aandachtspunt');
    expect(i18next.t('opname.takeover.count', { count: 2 })).toBe('2 wijzigingen');
    await i18next.changeLanguage('en');
    expect(i18next.t('nav.state.errors', { count: 1 })).toBe('1 error');
    expect(i18next.t('opname.resultCard.issues', { count: 4 })).toBe('4 errors in the survey');
  });

  it('6: writes the source texts of the example projects in Dutch', () => {
    for (const name of ['nta8800-example-terraced-dwelling.json', 'nta8800-example-office.json']) {
      const text = readFileSync(resolve(root, 'training-data', name), 'utf8');
      expect(text, name).not.toMatch(/"[^"]*\bsynthetic\b[^"]*"/);
      expect(text, name).not.toContain('"one dwelling"');
    }
  });

  it('7: uses "instelwaarde" instead of "setpoint" in the Dutch interface', () => {
    const dutch = [...Object.values(nl), ...Object.values(kernelCodeLabelsNl)];
    expect(dutch.filter((text) => /\bsetpoints?\b/i.test(text))).toEqual([]);
    expect(nl['nta.form.heatingSetpoint']).toBe('Instelwaarde verwarming °C');
  });

  it('8: keeps Save off on Registratie until something changed', async () => {
    const user = userEvent.setup();
    renderWithProviders(<RegistrationForm />);
    const save = screen.getByRole('button', { name: 'Save' });
    expect(save).toBeDisabled();
    expect(save).toHaveAccessibleDescription('No changes to save');
    await user.type(screen.getByLabelText('BAG object id'), '0363010000000001');
    expect(save).toBeEnabled();
    expect(save).not.toHaveAccessibleDescription('No changes to save');
  }, SLOW_UI_TEST_MS);

  it('9: says why the gas reference comparison is off', async () => {
    const user = userEvent.setup();
    renderWithProviders(<GasChainReferencePanel />);
    await user.click(screen.getByText('Gas heat pump reference comparison'));
    const compare = screen.getByRole('button', { name: 'Compare draft values' });
    expect(compare).toBeDisabled();
    expect(compare).toHaveAccessibleDescription('Load a case first (example or JSON file); then you can compare.');
    await user.click(screen.getByRole('button', { name: 'Load synthetic example' }));
    expect(compare).toBeEnabled();
    expect(compare).not.toHaveAccessibleDescription(/Load a case first/);
  }, SLOW_UI_TEST_MS);
});
