/**
 * BRL 9501 readiness tasks B8 and B9: the NL-EPBD mark and attest number only
 * for an attested program, and the adviser's statements for label elements k
 * and l (Omgevingsregeling art. 5.13a lid 1).
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { attestMark, softwareAttestNumber } from '../core/nta/Attest';
import { cleanRegistration } from '../core/nta/Registration';
import { generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';
import { buildEpOnlineOverview } from '../core/report/EpOnlineOverview';
import { RegistrationForm } from '../components/shell/pages/RegistrationForm';
import { renderWithProviders, userEvent } from './test-utils';

const root = resolve(__dirname, '../..');
const load = () => ({
  project: JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject,
  assessment: JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.kernel-output.json'), 'utf8')) as ProjectPerformanceAssessment,
});
const at = new Date('2026-10-08T10:00:00Z');

describe('attest mark (BRL 9501 §8.2, §8.4)', () => {
  it('shows no mark until the program carries an attest number', () => {
    expect(softwareAttestNumber()).toBeNull();
    expect(attestMark()).toEqual({ attested: false, attestNumber: null, markText: null });
    expect(attestMark('  ')).toEqual({ attested: false, attestNumber: null, markText: null });
    expect(attestMark('K-2026-01')).toEqual({
      attested: true, attestNumber: 'K-2026-01', markText: 'NL-EPBD · BRL 9501-attest K-2026-01',
    });
  });

  it('puts the attest number and the mark slot on the report only for an attested program', () => {
    const { project, assessment } = load();
    const unattested = generateEnergyPerformanceReportHTML(project, assessment, { level: 'summary', generatedAt: at });
    expect(unattested).toContain('(niet geattesteerd)');
    expect(unattested).toContain('geen officieel energielabel, niet geattesteerd.');
    expect(unattested).not.toContain('data-mark="nl-epbd"');
    const attested = generateEnergyPerformanceReportHTML(project, assessment, {
      level: 'summary', generatedAt: at, attestNumber: 'K-2026-01',
    });
    expect(attested).toContain('(BRL 9501-attest K-2026-01)');
    expect(attested).toContain('· attest K-2026-01');
    expect(attested).toContain('<div class="attest-mark" data-mark="nl-epbd">NL-EPBD · BRL 9501-attest K-2026-01</div>');
    expect(attested).not.toContain('niet geattesteerd)');
  });
});

describe('label elements k and l (Omgevingsregeling art. 5.13a lid 1)', () => {
  it('keeps a "no" as an answer and drops an unanswered statement', () => {
    const cleaned = cleanRegistration({ labelStatements: { respondsToExternalSignals: false, lowTemperatureHeating: undefined } });
    expect(cleaned?.labelStatements).toEqual({ respondsToExternalSignals: false });
    expect(cleanRegistration({ labelStatements: {} })).toBeUndefined();
  });

  it('lists the statements in the report and apart from the export schema in the EP-Online overview', () => {
    const { project, assessment } = load();
    const withStatements: IProject = {
      ...project,
      registration: { ...project.registration, labelStatements: { respondsToExternalSignals: false, lowTemperatureHeating: true } },
    };
    const html = generateEnergyPerformanceReportHTML(withStatements, assessment, { level: 'summary', generatedAt: at });
    expect(html).toContain('k. Reageert op externe signalen (verklaring adviseur)</th><td colspan="2">nee');
    expect(html).toContain('l. Afgiftesysteem ontworpen voor lage temperatuur (verklaring adviseur)</th><td colspan="2">ja');
    const overview = buildEpOnlineOverview(withStatements, assessment);
    expect(overview.labelStatements).toMatchObject({ k_reageertOpExterneSignalen: false, l_afgiftesysteemLageTemperatuur: true });
    expect(Object.keys(overview.pandcertificaat)).not.toContain('k_reageertOpExterneSignalen');
    const unanswered = buildEpOnlineOverview({ ...project, registration: {} }, assessment);
    expect(unanswered.labelStatements).toMatchObject({ k_reageertOpExterneSignalen: null, l_afgiftesysteemLageTemperatuur: null });
  });

  it('prefers the kernel label elements k and l and shows them once', () => {
    const { project, assessment } = load();
    const elements = assessment.labelData!.indicators!.elements!;
    const kernel: ProjectPerformanceAssessment = {
      ...assessment,
      labelData: {
        ...assessment.labelData!,
        indicators: {
          ...assessment.labelData!.indicators!,
          elements: { ...elements, respondsToExternalSignals: true, lowTemperatureHeating: false },
        },
      },
    };
    // The registration says the opposite: the kernel's elements win.
    const withStatements: IProject = {
      ...project,
      registration: { ...project.registration, labelStatements: { respondsToExternalSignals: false, lowTemperatureHeating: true } },
    };
    const html = generateEnergyPerformanceReportHTML(withStatements, kernel, { level: 'summary', generatedAt: at });
    expect(html).toContain('k. Reageert op externe signalen (verklaring adviseur)</th><td colspan="2">ja');
    expect(html).toContain('l. Afgiftesysteem ontworpen voor lage temperatuur (verklaring adviseur)</th><td colspan="2">nee');
    expect(html.match(/k\. Reageert op externe signalen/g)).toHaveLength(1);
    expect(html).not.toContain('respondsToExternalSignals');
    expect(buildEpOnlineOverview(withStatements, kernel).labelStatements)
      .toMatchObject({ k_reageertOpExterneSignalen: true, l_afgiftesysteemLageTemperatuur: false });
  });

  function Harness() {
    const { state } = useEnergy();
    return <>
      <RegistrationForm />
      <output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output>
    </>;
  }

  it('stores the answers from the registration form', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.selectOptions(screen.getByRole('combobox', {
      name: 'k. Can the building respond to external signals and adapt its energy use?',
    }), 'no');
    await user.selectOptions(screen.getByRole('combobox', {
      name: 'l. Is the heating distribution system designed for low temperature?',
    }), 'yes');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const stored = JSON.parse(screen.getByTestId('registration').textContent ?? 'null');
    expect(stored.labelStatements).toEqual({ respondsToExternalSignals: false, lowTemperatureHeating: true });
  }, 60000);
});
