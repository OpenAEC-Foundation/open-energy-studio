/**
 * Results dashboard (UI redesign F7): chart data mapping against the kernel
 * totals, the gauges against the Bbl check, the sub tabs, the stale state and
 * the withheld fallback.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.kernel-output.json';
import office from '../../training-data/nta8800-example-office.kernel-output.json';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import type { KernelState } from '../context/KernelProvider';
import { kernelVerdict } from '../core/nta/KernelVerdict';
import {
  energySeries, gauges, monthlyNeed, monthlyPv, seriesTotal, zoneDemands,
} from '../components/shell/pages/results/resultsData';
import { ResultsDashboard } from '../components/shell/pages/results/ResultsDashboard';
import { renderWithProviders, userEvent } from './test-utils';

const kernel = vi.hoisted(() => ({ state: null as KernelState | null }));
vi.mock('../context/KernelProvider', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../context/KernelProvider')>()),
  useKernel: () => kernel.state,
}));

const examples: Array<[string, ProjectPerformanceAssessment]> = [
  ['terraced dwelling', terraced as unknown as ProjectPerformanceAssessment],
  ['office', office as unknown as ProjectPerformanceAssessment],
];

function state(assessment: ProjectPerformanceAssessment, phase: KernelState['phase'] = 'current'): KernelState {
  return {
    project: {} as KernelState['project'],
    query: { kind: 'done', assessment } as KernelState['query'],
    settled: assessment,
    verdict: kernelVerdict(assessment),
    phase,
    error: null,
    refresh: () => {},
  };
}

afterEach(() => {
  kernel.state = null;
  vi.unstubAllGlobals();
});

describe('results dashboard data mapping', () => {
  it.each(examples)('the chart series of the %s sum to the kernel totals', (_name, assessment) => {
    const performance = assessment.performance!;
    // Primary fossil: services minus export credit and storage correction = EPTot.
    expect(seriesTotal(energySeries(performance, 'primary'))).toBeCloseTo(performance.annualPrimaryFossilKwh!, 3);
    // Delivered energy per service and per carrier both equal the kernel's carrier totals.
    const delivered = performance.carriers.reduce((sum, row) => sum + row.deliveredKwh, 0);
    expect(seriesTotal(energySeries(performance, 'delivered'), 'positive')).toBeCloseTo(delivered, 3);
    expect(seriesTotal(energySeries(performance, 'carrier'), 'positive')).toBeCloseTo(delivered, 3);
    // PV below zero equals the chapter 16 production.
    const pv = energySeries(performance, 'delivered').find((item) => item.key === 'pv');
    const production = (performance.pvSystems ?? []).reduce((sum, system) => sum + system.annualKwh, 0);
    expect(-(pv?.values.reduce((sum, value) => sum + value, 0) ?? 0)).toBeCloseTo(production, 3);
    expect(monthlyPv(performance).reduce((sum, value) => sum + value, 0)).toBeCloseTo(production, 3);
    // Net need per month sums to the annual need of every zone.
    const need = monthlyNeed(performance);
    const zones = zoneDemands(performance);
    expect(need.heat.reduce((sum, value) => sum + value, 0))
      .toBeCloseTo(zones.reduce((sum, zone) => sum + (zone.annualHeatingNeedKwh ?? 0), 0), 3);
    expect(need.cold.reduce((sum, value) => sum + value, 0))
      .toBeCloseTo(zones.reduce((sum, zone) => sum + (zone.annualCoolingNeedKwh ?? 0), 0), 3);
  });

  it('builds the gauges from the Bbl check, with TOjuli for dwellings only', () => {
    const dwelling = gauges(examples[0][1]);
    expect(dwelling.map((gauge) => gauge.key)).toEqual(['beng1', 'beng2', 'beng3', 'tojuli']);
    const beng2 = dwelling.find((gauge) => gauge.key === 'beng2')!;
    expect(beng2.meets).toBe(false);
    expect(beng2.margin).toBeCloseTo(30 - 80.56, 2);
    const beng3 = dwelling.find((gauge) => gauge.key === 'beng3')!;
    expect(beng3.higherIsBetter).toBe(true);
    expect(beng3.margin).toBeCloseTo(22.3 - 50, 1);
    expect(dwelling.find((gauge) => gauge.key === 'tojuli')!.limit).toBe(1.2);
    const utility = gauges(examples[1][1]);
    expect(utility.map((gauge) => gauge.key)).toEqual(['beng1', 'beng2', 'beng3']);
    expect(utility.map((gauge) => gauge.meets)).toEqual([true, false, true]);
  });
});

describe('results dashboard', () => {
  it('shows the label, the gauges with verdicts and the charts', () => {
    kernel.state = state(examples[0][1]);
    renderWithProviders(<ResultsDashboard sub="overview" onNavigate={() => {}} />);
    expect(screen.getByTestId('results-dashboard')).toBeInTheDocument();
    expect(screen.getByTestId('results-label-class')).toHaveTextContent('A+');
    expect(screen.getByTestId('results-gauge-beng2')).toHaveTextContent('80.56');
    expect(screen.getAllByText('Does not meet').length).toBeGreaterThanOrEqual(2);
    expect(screen.getByTestId('results-energy-chart')).toBeInTheDocument();
    expect(screen.getByTestId('results-need-chart')).toBeInTheDocument();
    expect(screen.getByText(/BENG 2 exceeded by 50.56/)).toBeInTheDocument();
  });

  it('switches the chart mode and offers a table view of the same numbers', async () => {
    const user = userEvent.setup();
    kernel.state = state(examples[0][1]);
    renderWithProviders(<ResultsDashboard sub="overview" onNavigate={() => {}} />);
    const chart = screen.getByTestId('results-energy-chart');
    expect(within(chart).getAllByRole('img').length).toBe(12);
    await user.click(screen.getByRole('radio', { name: 'By carrier' }));
    expect(within(chart).getByText('Natural gas')).toBeInTheDocument();
    await user.click(within(chart).getByRole('button', { name: 'Table' }));
    const table = within(chart).getByRole('table');
    expect(within(table).getAllByRole('row')).toHaveLength(14);
  }, 60000);

  it('says TOjuli does not apply to a utility building', () => {
    kernel.state = state(examples[1][1]);
    renderWithProviders(<ResultsDashboard sub="overview" onNavigate={() => {}} />);
    expect(screen.queryByTestId('results-gauge-tojuli')).not.toBeInTheDocument();
    expect(screen.getByText(/Not applicable to non-residential/)).toBeInTheDocument();
  });

  it('renders each sub tab from the same kernel answer', () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve({ ok: true, json: async () => [] })));
    const assessment = examples[0][1];
    kernel.state = state(assessment);
    const { rerender } = renderWithProviders(<ResultsDashboard sub="services" onNavigate={() => {}} />);
    expect(screen.getByTestId('results-eptot')).toHaveTextContent('8,055');
    rerender(<ResultsDashboard sub="zones" onNavigate={() => {}} />);
    expect(screen.getAllByText('Need per calculation zone').length).toBeGreaterThan(0);
    rerender(<ResultsDashboard sub="monthly" onNavigate={() => {}} />);
    expect(screen.getAllByRole('row').length).toBeGreaterThanOrEqual(13);
    rerender(<ResultsDashboard sub="provenance" onNavigate={() => {}} />);
    const provenance = screen.getByTestId('results-provenance');
    expect(provenance).toHaveTextContent(assessment.kernelVersion);
    expect(provenance).toHaveTextContent(assessment.inputFingerprint);
  });

  it('marks a result of the previous input as stale', () => {
    kernel.state = state(examples[0][1], 'stale');
    renderWithProviders(<ResultsDashboard sub="overview" onNavigate={() => {}} />);
    expect(screen.getByText(/belongs to the previous input/)).toBeInTheDocument();
  });

  it('withholds every number when the kernel refused the input', async () => {
    const refused = {
      ...examples[0][1], status: 'invalid', performance: null,
      gaps: [{ code: 'u_value_out_of_range', path: 'zones[0].surfaces[0]' }],
    } as unknown as ProjectPerformanceAssessment;
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve({ ok: true, json: async () => refused })));
    kernel.state = state(refused);
    renderWithProviders(<ResultsDashboard sub="overview" onNavigate={() => {}} />);
    expect(screen.queryByTestId('results-dashboard')).not.toBeInTheDocument();
    expect(await screen.findByTestId('results-withheld')).toBeInTheDocument();
    expect(screen.queryByTestId('beng-cards-indicative')).not.toBeInTheDocument();
  });
});
