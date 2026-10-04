import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { isTauri } from '@tauri-apps/api/core';
import { useEnergy } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { ResultsView } from '../components/ResultsView/ResultsView';
import { KernelCode } from '../components/KernelCode/KernelCode';
import { formatNumber, kernelCodeLabel } from '../i18n/format';
import { kernelEnergyBreakdown } from '../core/nta/KernelBreakdown';
import { defaultValueLabel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { table713Setpoints } from '../components/NtaPerformancePanel/NtaCalculationForm';
import type { BuildingPerformanceAssessment } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(isTauri).mockReturnValue(false);
});

const month = (value: number, index: number) => ({
  month: index + 1, heatingNeedKwh: value, emissionLossKwh: 0, emissionInputKwh: value,
  distributionLossKwh: 0, generatorOutputKwh: value, naturalGasKwh: value, generatorElectricityKwh: 0,
  auxiliaryElectricityKwh: 0, collectiveSourceHeatKwh: 0,
});

const demandMonth = (index: number) => ({
  month: index + 1, hours: 720, outdoorTemperatureC: 5, internalGainsKwh: 100, windowSolarGainsKwh: 50,
  windowSolarCoolingKwh: 0, opaqueSolarGainsKwh: 10, groundConductanceWPerK: 0,
  heating: { transmissionKwh: 400, ventilationKwh: 200, needKwh: 300 },
  cooling: { needKwh: 0 },
});

const performance = {
  status: 'calculated_unverified', issues: [], warnings: [],
  needIndicatorKwhPerM2Year: 51.37, primaryFossilIndicatorKwhPerM2Year: 48.33, renewableSharePercent: 49.6,
  indicativeLabelClass: 'A+++', labelSource: 'annex IX',
  bblCheck: { source: 'Bbl', function: 'office', lossAreaRatio: 1.2,
    limits: { energyNeedMaxKwhPerM2: 93, primaryFossilMaxKwhPerM2: 40, renewableShareMinPercent: 30,
      lightConstructionAllowanceApplied: false },
    energyNeedMeets: true, primaryFossilMeets: false, renewableShareMeets: true },
  tojuliMaxK: 0.4, tojuliMeetsBblLimit: true, tojuli: [],
  annualPrimaryFossilKwh: 6000, annualRenewablePrimaryKwh: 1500,
  pvSystems: [{ id: 'pv-1', monthlyKwh: [], annualKwh: 1200 }, { id: 'pv-2', monthlyKwh: [], annualKwh: 345.4 }],
  energyByService: {
    months: [{ service: 'heating', carrier: 'gas', month: 1, usedKwh: 3600, deliveredKwh: 3600, primaryFossilKwh: 3600 }],
    renewable: [], adjustments: [],
    annual: [
      { service: 'heating', carrier: 'gas', usedKwh: 3600, deliveredKwh: 3600, primaryFossilKwh: 3600 },
      { service: 'lighting', carrier: 'el', usedKwh: 700, deliveredKwh: 700, primaryFossilKwh: 1015 },
    ],
  },
  spaceHeating: {
    omittedTerms: [], additionalZoneDemands: [],
    monthly: Array.from({ length: 12 }, (_, index) => month(300, index)),
    demand: {
      annualHeatingNeedKwh: 3600, annualCoolingNeedKwh: 0, omittedCorrections: [],
      monthly: Array.from({ length: 12 }, (_, index) => demandMonth(index)),
    },
  },
};

function stubKernel(body: unknown) {
  vi.stubGlobal('fetch', vi.fn((url: string) => (String(url).includes('project/performance')
    ? Promise.resolve({ ok: true, json: async () => body })
    : new Promise(() => {}))));
}

function Harness() {
  const { state, dispatch } = useEnergy();
  return <>
    <button type="button" onClick={() => dispatch({ type: 'SET_RESULT', payload: calculateBENGMonthly(state.project) })}>Calculate</button>
    <button type="button" onClick={() => dispatch({ type: 'UPDATE_HEATING_SYSTEM', payload: {
      id: state.project.heatingSystems[0].id, data: { cop: 4.1 },
    } })}>Change heating</button>
    <ResultsView />
  </>;
}

describe('results view', () => {
  it('shows only kernel BENG values and the kernel PV yield when the kernel calculated', async () => {
    stubKernel({ status: 'calculated_unverified', gaps: [], derivedInput: { calculationScope: 'utility' }, performance });
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(screen.getByRole('button', { name: 'Calculate' }));
    const cards = within(await screen.findByTestId('beng-cards-kernel'));
    expect(cards.getByText('48.33')).toBeInTheDocument();
    expect(cards.getByText('51.37')).toBeInTheDocument();
    expect(cards.getAllByText('NTA kernel · unverified')).toHaveLength(3);
    expect(screen.queryByTestId('beng-cards-indicative')).not.toBeInTheDocument();
    // TOjuli has no Bbl limit for utility: no TOjuli card.
    expect(screen.queryByTestId('to-juli-kernel')).not.toBeInTheDocument();
    expect(screen.queryByText(/GTO:/)).not.toBeInTheDocument();
    expect(screen.getByTestId('breakdown-source')).toHaveTextContent('From the NTA 8800 kernel');
    expect(screen.getByText('1,545 kWh')).toBeInTheDocument();
  });

  it('keeps the tailored-advice panel and marks the simplified result stale after an edit', async () => {
    stubKernel({ status: 'incomplete', gaps: [{ code: 'nta_calculation_block_missing', path: 'ntaCalculation' }],
      derivedInput: null, performance: null });
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(screen.getByRole('button', { name: 'Calculate' }));
    expect(await screen.findByTestId('beng-cards-indicative')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Start tailored advice' }));
    const advice = screen.getByRole('region', { name: 'Tailored advice (maatwerkadvies)' });
    expect(within(advice).queryByRole('button', { name: 'Start tailored advice' })).not.toBeInTheDocument();
    // The advice start is a project edit: the result is cleared but stays visible as stale.
    expect(screen.getByTestId('results-stale')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Change heating' }));
    expect(screen.getByRole('region', { name: 'Tailored advice (maatwerkadvies)' })).toBeInTheDocument();
    expect(screen.getByTestId('results-stale')).toBeInTheDocument();
    expect(await screen.findByTestId('beng-cards-indicative')).toBeInTheDocument();
  }, 60000);
});

describe('results withheld when the kernel refuses the input', () => {
  for (const [status, gap] of [['invalid', 'surface_area_out_of_range'], ['incomplete', 'ventilation_flow_required']] as const) {
    it(`shows no simplified BENG values when the kernel result is ${status}`, async () => {
      stubKernel({ status, gaps: [{ code: gap, path: 'zones[0]' }], derivedInput: null, performance: null });
      const user = userEvent.setup();
      renderWithProviders(<Harness />);
      await user.click(screen.getByRole('button', { name: 'Calculate' }));
      expect(await screen.findByTestId('results-withheld')).toHaveTextContent('Results withheld');
      expect(screen.queryByTestId('beng-cards-indicative')).not.toBeInTheDocument();
    }, 60000);
  }
});

describe('labels and number formatting', () => {
  it('formats numbers in the UI language', () => {
    expect(formatNumber(4997.4, 'en')).toBe('4,997');
    expect(formatNumber(4997.4, 'nl')).toBe('4.997');
    expect(formatNumber(48.333, 'nl', 2)).toBe('48,33');
    expect(formatNumber(null, 'en')).toBe('–');
  });

  it('translates kernel codes and keeps unknown ones as code', () => {
    const t = (key: string) => (key === 'nta.gap.known_code' ? 'Known code' : key);
    expect(kernelCodeLabel(t, 'known_code')).toEqual({ text: 'Known code', known: true });
    expect(kernelCodeLabel(t, 'unknown_code')).toEqual({ text: 'unknown_code', known: false });
    renderWithProviders(<><KernelCode code="setpoints_table_7_13_mismatch" /><KernelCode code="no_such_code_xyz" /></>);
    expect(screen.getByText('no_such_code_xyz').tagName).toBe('CODE');
    expect(screen.getByText('setpoints_table_7_13_mismatch')).toHaveClass('kernel-code-ref');
  });

  it('labels recorded survey default values', () => {
    const t = (key: string) => (key === 'opname.value.bath_geyser' ? 'Bath geyser' : key);
    renderWithProviders(<p>{defaultValueLabel(t, 'bath_geyser')} / {defaultValueLabel(t, 'none_known_value')} / {defaultValueLabel(t, '1.5')}</p>);
    expect(screen.getByText(/Bath geyser/)).toBeInTheDocument();
    expect(screen.getByText(/none known value/)).toBeInTheDocument();
    expect(screen.getByText(/1\.5/)).toBeInTheDocument();
  });

  it('knows the table 7.13 setpoints the kernel requires', () => {
    expect(table713Setpoints('residential')).toEqual({ heatingC: 20, coolingC: 24 });
    expect(table713Setpoints('sport')).toEqual({ heatingC: 16, coolingC: 24 });
    expect(table713Setpoints('healthcare_with_beds')).toEqual({ heatingC: 22, coolingC: 24 });
    expect(table713Setpoints('office')).toEqual({ heatingC: 21, coolingC: 24 });
    expect(table713Setpoints(null)).toBeNull();
  });

  it('builds the chart balance from the kernel', () => {
    const breakdown = kernelEnergyBreakdown(performance as unknown as BuildingPerformanceAssessment);
    expect(breakdown.transmissionLoss).toBe(4800);
    expect(breakdown.ventilationLoss).toBe(2400);
    expect(breakdown.solarGain).toBe(720);
    expect(breakdown.heatingEnergy).toBe(3600);
    expect(breakdown.lightingEnergy).toBe(700);
    expect(breakdown.pvProduction).toBeCloseTo(1545.4);
  });
});
