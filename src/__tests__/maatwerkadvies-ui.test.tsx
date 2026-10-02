import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { createDefaultProject } from '../context/EnergyContext';
import { MaatwerkadviesPanel, parseMonthly } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { RelabelPanel } from '../components/MaatwerkadviesPanel/RelabelPanel';
import { buildMaatwerkadviesInput, type MaatwerkadviesAssessment } from '../core/nta/KernelClient';
import { generateMaatwerkadviesReportHTML } from '../core/report/MaatwerkadviesReport';
import { renderWithProviders, userEvent } from './test-utils';

function Editor() {
  const { state } = useEnergy();
  return <><MaatwerkadviesPanel /><RelabelPanel />
    <output data-testid="mwa">{JSON.stringify(state.project.maatwerkadvies ?? null)}</output></>;
}

describe('maatwerkadvies panel', () => {
  it('parses monthly readings with gaps', () => {
    expect(parseMonthly('')).toBeUndefined();
    expect(parseMonthly('300; 280; -; 150,5')).toEqual([300, 280, null, 150.5]);
  });

  it('starts an advice and adds measures and packages to the project', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    expect(screen.getByRole('region', { name: 'Relabelling (BRL 9500 annex 6a/6b)' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Start tailored advice' }));
    await user.click(screen.getByRole('button', { name: 'Add measure' }));
    await user.click(screen.getByRole('button', { name: 'Add package' }));
    await user.click(screen.getByRole('checkbox', { name: 'm1' }));
    const stored = JSON.parse(screen.getByTestId('mwa').textContent ?? 'null');
    expect(stored.measures).toHaveLength(1);
    expect(stored.packages[0].measureIds).toEqual(['m1']);
    expect(stored.tariffs.gasEurPerM3).toBe(1.4);
  });
});

describe('maatwerkadvies input and report', () => {
  it('uses the project without its own advice block as the base', () => {
    const project = createDefaultProject();
    const definition = {
      measures: [], packages: [],
      tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, sourceReference: 'x' },
    };
    const input = buildMaatwerkadviesInput({ ...project, maatwerkadvies: definition }, definition) as {
      base: { kind: string; project: Record<string, unknown> };
    };
    expect(input.base.kind).toBe('project');
    expect(input.base.project.maatwerkadvies).toBeUndefined();
  });

  it('renders packages, savings and advice in the report', () => {
    const project = createDefaultProject();
    const variant = (id: string, name: string, kind: 'current' | 'package') => ({
      id, name, kind, measureIds: kind === 'package' ? ['m1'] : [], valid: true,
      label: { labelClass: 'C', needIndicatorKwhPerM2: null, primaryFossilIndicatorKwhPerM2: 200, renewableSharePercent: 0, tojuliMaxK: null },
      actualUse: {
        gasKwh: 9769, gasM3: 1000, electricityImportKwh: 2500, electricityExportKwh: 0, electricityProducedKwh: 0,
        districtHeatKwh: 0, districtColdKwh: 0, oilKwh: 0, biomassKwh: 0, primaryFossilKwh: 20000, co2Kg: 2500,
        energyCostEur: 2150, monthlyGasM3: new Array(12).fill(83),
        monthlyElectricityImportKwh: new Array(12).fill(208), monthlyHeatKwh: new Array(12).fill(0),
        monthlyElectricityExportKwh: new Array(12).fill(0),
      },
      savings: kind === 'package' ? { gasM3: 300, electricityKwh: 0, heatKwh: 0, primaryFossilKwh: 3000, co2Kg: 550, energyCostEur: 420 } : null,
      investmentEur: kind === 'package' ? 8000 : 0, maintenanceEurPerYear: 0,
      simplePaybackYears: kind === 'package' ? 19 : null, netPresentValueEur: kind === 'package' ? -500 : null,
      horizonYears: 40, phasing: [], issues: [],
      systemChecks: [
        { system: 'space_heating' as const, value: 1.42, limit: 1.31, unit: '-', meets: false, note: null },
        { system: 'ventilation' as const, value: null, limit: null, unit: 'kWh/(m3/h)', meets: null, note: null },
      ],
    });
    const assessment: MaatwerkadviesAssessment = {
      status: 'calculated_unverified', scope: 's', targetNormVersion: 'NTA', kernelVersion: 'k', inputFingerprint: 'f',
      attestStatus: 'unattested', current: variant('current', 'Huidige situatie', 'current'), measures: [],
      packages: [variant('p1', 'Spouwmuur', 'package')], fitCheck: null,
      advice: { packageId: 'p1', chosenBy: 'adviser', motivation: 'past bij budget', warnings: ['let op ventilatie'], specialistNotes: [], notes: [] },
      renovationPassport: {
        steps: [variant('passport-step-1', 'Stap 1: beperken warmte- en koudevraag', 'package')],
        requirements: [{ code: 'natural_gas_free_main_heating', met: false, detail: null }],
        eligible: false,
        requiredStatements: [],
      },
      interpretations: ['npv'], issues: [],
    };
    const html = generateMaatwerkadviesReportHTML(project, {
      measures: [{ id: 'm1', name: 'Spouwmuurisolatie', category: 'insulation', target: 'project', patch: [], investmentEur: 8000, costSource: 'RVO', lifetimeYears: 40 }],
      packages: [{ id: 'p1', name: 'Spouwmuur', measureIds: ['m1'] }],
      tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, sourceReference: 'tarieven 2026' },
    }, assessment);
    expect(html).toContain('Spouwmuurisolatie');
    expect(html).toContain('keuze adviseur');
    expect(html).toContain('let op ventilatie');
    expect(html).toContain('tarieven 2026');
    expect(html).toContain('Systeemeisen Bbl art. 4.248');
    expect(html).toContain('Ruimteverwarming');
    expect(html).toContain('Renovatiepaspoort');
    expect(html).toContain('Hoofdverwarming aardgasvrij');
  });
});
