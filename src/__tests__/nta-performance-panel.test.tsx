import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { useEnergy } from '../context/EnergyContext';
import { NtaPerformancePanel } from '../components/NtaPerformancePanel/NtaPerformancePanel';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(isTauri).mockReturnValue(false);
  vi.mocked(invoke).mockReset();
});

function Harness() {
  const { state } = useEnergy();
  return <><NtaPerformancePanel />
    <output data-testid="block">{JSON.stringify(state.project.ntaCalculation ?? null)}</output>
  </>;
}

const month = (value: number, index: number) => ({
  month: index + 1, heatingNeedKwh: value, emissionLossKwh: value * 0.15,
  emissionInputKwh: value * 1.15, distributionLossKwh: 0, generatorOutputKwh: value * 1.15,
  naturalGasKwh: value * 1.18, generatorElectricityKwh: 0, auxiliaryElectricityKwh: 10,
  collectiveSourceHeatKwh: 0,
});

describe('NTA performance panel', () => {
  it('lists input gaps and stores an edited NTA block in the project', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({
        status: 'incomplete', inputFingerprint: 'sha256:x', attestStatus: 'unattested',
        gaps: [{ code: 'nta_calculation_block_missing', path: 'ntaCalculation' },
          { code: 'nta_calculation_block_invalid', path: 'ntaCalculation',
            detail: 'ventilationFlows[0].months[3].conductanceWPerK: invalid type: null' }],
        derivedInput: null, performance: null,
      }),
    });
    vi.stubGlobal('fetch', fetchMock);
    renderWithProviders(<Harness />);
    const panel = within(await screen.findByRole('region', { name: 'NTA 8800 calculation (Rust kernel)' }));
    expect(await panel.findByText('NTA input block missing')).toBeInTheDocument();
    expect(panel.getByText(/months\[3\]\.conductanceWPerK/)).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/nta8800/project/performance', expect.anything());
    expect(panel.getByText('Unverified')).toBeInTheDocument();

    await user.click(panel.getByRole('button', { name: 'Start NTA input' }));
    const editor = panel.getByLabelText('NTA input block (JSON)') as HTMLTextAreaElement;
    const template = JSON.parse(editor.value);
    expect(template.calculationScope).toBe('residential');
    expect(template.setpoints).toEqual({ heatingC: 20, coolingC: 24, sourceReference: '' });
    expect(template.ventilationFlows[0].months).toHaveLength(12);
    expect(template.ventilationFlows[0].months[0].conductanceWPerK).toBeNull();

    await user.clear(editor);
    await user.type(editor, 'not json');
    await user.click(panel.getByRole('button', { name: 'Save' }));
    expect(panel.getByRole('alert')).toHaveTextContent('Not a valid JSON object');

    await user.clear(editor);
    await user.click(editor);
    await user.paste('{"calculationScope":"residential"}');
    await user.click(panel.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('block').textContent ?? 'null'))
      .toEqual({ calculationScope: 'residential' });
    // The saved block triggers a new kernel run.
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('shows unverified indicators, BENG 1 condition and monthly values', async () => {
    const need = [800, 650, 500, 250, 50, 0, 0, 0, 40, 250, 500, 750];
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'calculated_unverified', inputFingerprint: 'sha256:abc', attestStatus: 'unattested',
        gaps: [], derivedInput: null,
        performance: {
          status: 'calculated_unverified', issues: [],
          needIndicatorKwhPerM2Year: null, primaryFossilIndicatorKwhPerM2Year: 66.24,
          renewableSharePercent: 17.6, indicativeLabelClass: 'A+', labelSource: 'annex IX',
          bblCheck: { source: 'Bbl', function: 'other_residential', lossAreaRatio: 2.12,
            limits: { energyNeedMaxKwhPerM2: 78.6, primaryFossilMaxKwhPerM2: 30, renewableShareMinPercent: 50,
              lightConstructionAllowanceApplied: true },
            energyNeedMeets: null, primaryFossilMeets: false, renewableShareMeets: false }, annualPrimaryFossilKwh: 6623.2, annualRenewablePrimaryKwh: 1421,
          spaceHeating: {
            omittedTerms: ['9.2.3 node losses and node gains (including solar thermal)'],
            monthly: need.map(month),
            demand: {
              annualHeatingNeedKwh: 3790, annualCoolingNeedKwh: 420,
              omittedCorrections: ['7.9.2 intermittent heating reduction a_H;red'],
              monthly: need.map(() => ({ cooling: { needKwh: 35 } })),
            },
          },
        },
      }),
    }));
    renderWithProviders(<NtaPerformancePanel />);
    const indicators = within(await screen.findByRole('group', { name: 'Indicators' }));
    expect(indicators.getByText('66.24')).toBeInTheDocument();
    expect(indicators.getByText('17.6')).toBeInTheDocument();
    expect(indicators.getByText('A+')).toBeInTheDocument();
    expect(indicators.getByText('indicative, not registered (Omgevingsregeling annex IX/X)')).toBeInTheDocument();
    expect(indicators.getByText('requires fixed ventilation system C1 (§5.4)')).toBeInTheDocument();
    expect(screen.getByText('7.9.2 intermittent heating reduction a_H;red')).toBeInTheDocument();
    expect(screen.getByText('sha256:abc')).toBeInTheDocument();
    const bbl = within(screen.getByRole('group', { name: 'Bbl article 4.149 check (table 4.148A)' }));
    expect(bbl.getByText('cannot be checked')).toBeInTheDocument();
    expect(bbl.getAllByText('does not meet')).toHaveLength(2);
    expect(bbl.getByText('BENG 2 ≤ 30.0')).toBeInTheDocument();
    expect(screen.getAllByRole('row')).toHaveLength(13);
  });
});
