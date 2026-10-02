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
        geometry: { usableFloorAreaM2: 96, lossAreaM2: 232.8, envelopeAreaM2: 247.2, lossAreaRatio: 2.425, unclassifiedSurfaceCount: 0 },
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
    expect(panel.getByText(/232\.8 m²/)).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/nta8800/project/performance', expect.anything());
    expect(panel.getByText('Unverified')).toBeInTheDocument();

    await user.click(panel.getByRole('button', { name: 'Advanced (JSON)' }));
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
            energyNeedMeets: null, primaryFossilMeets: false, renewableShareMeets: false },
          tojuliMaxK: 1.34, tojuliMeetsBblLimit: false,
          tojuli: [{ zoneId: 'z1', status: 'calculated_unverified', activeCooling: false, maxTojuliK: 1.34, meetsBblLimit: false, issues: [],
            orientations: [
              { orientation: 'south', areaM2: 82, share: 0.5, assessed: true, conductanceWPerK: 45, coolingNeedJulyKwh: 44, tojuliK: 1.34 },
              { orientation: 'north_east', areaM2: 0, share: 0, assessed: false, conductanceWPerK: 0, coolingNeedJulyKwh: 0, tojuliK: null },
            ] }], annualPrimaryFossilKwh: 6623.2, annualRenewablePrimaryKwh: 1421,
          spaceHeating: {
            omittedTerms: ['9.2.3 node losses and node gains (including solar thermal)'],
            monthly: need.map(month),
            additionalZoneDemands: [],
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
    const tojuli = within(screen.getByRole('group', { name: 'TOjuli' }));
    expect(tojuli.getByText('South')).toBeInTheDocument();
    expect(tojuli.getByText('1.34 K')).toBeInTheDocument();
    expect(tojuli.queryByText('North-east')).not.toBeInTheDocument();
    expect(tojuli.getByText('does not meet')).toBeInTheDocument();
    expect(screen.getAllByRole('row')).toHaveLength(13);
  });
  it('edits the NTA block through the structured form', async () => {
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({ status: 'incomplete', inputFingerprint: 'sha256:x', attestStatus: 'unattested',
        gaps: [{ code: 'nta_calculation_block_missing', path: 'ntaCalculation' }], derivedInput: null, performance: null }),
    }));
    renderWithProviders(<Harness />);
    const panel = within(await screen.findByRole('region', { name: 'NTA 8800 calculation (Rust kernel)' }));
    await user.click(await panel.findByRole('button', { name: 'Start NTA input' }));
    const form = within(panel.getByRole('form', { name: 'NTA input' }));
    expect(form.getByLabelText('Heating setpoint °C')).toHaveValue(20);
    await user.type(form.getByLabelText('Source of usable floor area'), 'floor plan A-01');
    await user.type(form.getAllByLabelText('Source')[0], 'table 7.13');
    await user.selectOptions(form.getByLabelText('Floors'), 'very_heavy');
    await user.type(form.getByLabelText('Or: ventilation flow incl. infiltration m³/h (H_ve = q·ρ·c/3600)'), '100');
    expect(form.getByLabelText('Ventilation conductance H_ve W/K (all months)')).toHaveValue(100 * 1.205 * 1005 / 3600);
    await user.clear(form.getByLabelText('Ventilation conductance H_ve W/K (all months)'));
    await user.type(form.getByLabelText('Ventilation conductance H_ve W/K (all months)'), '42');
    await user.selectOptions(form.getByLabelText('Emission system'), 'floor_heating');
    await user.selectOptions(form.getByLabelText('Generator type'), 'external_heat');
    await user.type(form.getByLabelText('Proof of supply (invoice/contract)'), 'contract 42');
    await user.click(form.getByLabelText('All energy uses are included'));
    await user.click(form.getByRole('button', { name: 'Save' }));
    const block = JSON.parse(screen.getByTestId('block').textContent ?? 'null');
    expect(block.areaSourceReference).toBe('floor plan A-01');
    expect(block.thermalMass.floor).toBe('very_heavy');
    expect(block.ventilationFlows[0].months).toHaveLength(12);
    expect(block.ventilationFlows[0].months.every((month: { conductanceWPerK: number }) => month.conductanceWPerK === 42)).toBe(true);
    expect(block.emission.system).toBe('floor_heating');
    expect(block.useInventoryComplete).toBe(true);
    expect(block.generator).toEqual({
      kind: 'external_heat', supplierReference: 'contract 42', qualityDeclarationPresent: false,
      auxiliary: { electricallyConnectedDevices: null, nominalPowerKw: null, sourceReference: '' },
    });
    expect(block.setpoints.sourceReference).toBe('table 7.13');
    expect(block.thermalMass.sourceReference).toBe('');
  });
});
