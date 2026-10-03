import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, within } from '@testing-library/react';
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

function ProjectSwitchHarness() {
  const { state, dispatch } = useEnergy();
  return <><NtaPerformancePanel />
    <button type="button" onClick={() => dispatch({ type: 'SET_PROJECT', payload: {
      ...state.project, id: 'second-project', ntaCalculation: undefined,
    } })}>Switch project</button>
    <output data-testid="active-project">{state.project.id}</output>
  </>;
}

const month = (value: number, index: number) => ({
  month: index + 1, heatingNeedKwh: value, emissionLossKwh: value * 0.15,
  emissionInputKwh: value * 1.15, distributionLossKwh: 0, generatorOutputKwh: value * 1.15,
  naturalGasKwh: value * 1.18, generatorElectricityKwh: 0, auxiliaryElectricityKwh: 10,
  collectiveSourceHeatKwh: 0,
});

describe('NTA performance panel', () => {
  it('discards an open NTA editor when another project is loaded', async () => {
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({ status: 'incomplete', gaps: [], performance: null, derivedInput: null }),
    }));
    renderWithProviders(<ProjectSwitchHarness />);
    const panel = within(await screen.findByRole('region', { name: 'NTA 8800 calculation (Rust kernel)' }));
    await user.click(panel.getByRole('button', { name: 'Advanced (JSON)' }));
    const editor = panel.getByLabelText('NTA input block (JSON)') as HTMLTextAreaElement;
    await user.clear(editor);
    await user.type(editor, 'stale draft');
    await user.click(screen.getByRole('button', { name: 'Switch project' }));
    expect(screen.getByTestId('active-project')).toHaveTextContent('second-project');
    expect(panel.queryByLabelText('NTA input block (JSON)')).not.toBeInTheDocument();
    await user.click(panel.getByRole('button', { name: 'Advanced (JSON)' }));
    expect((panel.getByLabelText('NTA input block (JSON)') as HTMLTextAreaElement).value)
      .not.toContain('stale draft');
  });

  it('lists input gaps and stores an edited NTA block in the project', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({
        status: 'incomplete', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: 'test-kernel',
        inputFingerprint: 'sha256:x', attestStatus: 'unattested',
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
    await user.click(panel.getByText('Input check provenance'));
    const provenance = within(panel.getByLabelText('Input check provenance'));
    expect(provenance.getByText('NTA 8800:2025+C1:2026')).toBeInTheDocument();
    expect(provenance.getByText('test-kernel')).toBeInTheDocument();
    expect(provenance.getByText('sha256:x')).toBeInTheDocument();
    expect(provenance.getByText('Unattested')).toBeInTheDocument();

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
    const user = userEvent.setup();
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
          annualZebPrimaryTotalKwh: 7432.5, zebPrimaryTotalIndicatorKwhPerM2: 77.43,
          annualZebCo2Kg: 1234.25,
          annualFinalEnergyKwh: 6080.25, annualFinalEnergyEedKwh: 6330.25,
          finalEnergyByCarrier: [{ carrier: 'gas', annualKwh: 5900 }, { carrier: 'el', annualKwh: 180.25 }],
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
    const zeb = within(screen.getByRole('region', { name: 'ZEB (Annex AB)' }));
    expect(zeb.getByText('7432.50 kWh')).toBeInTheDocument();
    expect(zeb.getByText('77.43 kWh/m²')).toBeInTheDocument();
    expect(zeb.getByText('1234.25 kg/yr')).toBeInTheDocument();
    expect(zeb.getByText('Informative results, unattested and not a registered energy label.')).toBeInTheDocument();
    await user.click(screen.getByText('Final energy use (§5.9)'));
    const finalEnergy = within(screen.getByLabelText('Final energy use'));
    expect(finalEnergy.getByText('6080.25 kWh/yr')).toBeInTheDocument();
    expect(finalEnergy.getByText('6330.25 kWh/yr')).toBeInTheDocument();
    expect(finalEnergy.getByRole('row', { name: 'gas 5900.00' })).toBeInTheDocument();
    expect(finalEnergy.getByRole('row', { name: 'el 180.25' })).toBeInTheDocument();
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
    expect(screen.getAllByRole('row')).toHaveLength(16);
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
    fireEvent.change(form.getByLabelText('Source of usable floor area'), { target: { value: 'floor plan A-01' } });
    fireEvent.change(form.getAllByLabelText('Source')[0], { target: { value: 'table 7.13' } });
    await user.selectOptions(form.getByLabelText('Floors'), 'very_heavy');
    fireEvent.change(form.getByLabelText('Or: ventilation flow incl. infiltration m³/h (H_ve = q·ρ·c/3600)'), { target: { value: '100' } });
    expect(form.getByLabelText('Ventilation conductance H_ve W/K (all months)')).toHaveValue(100 * 1.205 * 1005 / 3600);
    await user.clear(form.getByLabelText('Ventilation conductance H_ve W/K (all months)'));
    fireEvent.change(form.getByLabelText('Ventilation conductance H_ve W/K (all months)'), { target: { value: '42' } });
    await user.selectOptions(form.getByLabelText('Emission system'), 'floor_heating');
    await user.selectOptions(form.getByLabelText('External heat delivery temperature for ZEB (Annex AB)'), 'from40_to60');
    await user.selectOptions(form.getByLabelText('Generator type'), 'external_heat');
    fireEvent.change(form.getByLabelText('Proof of supply (invoice/contract)'), { target: { value: 'contract 42' } });
    await user.click(form.getByLabelText('All energy uses are included'));
    await user.click(form.getByRole('button', { name: 'Save' }));
    const block = JSON.parse(screen.getByTestId('block').textContent ?? 'null');
    expect(block.areaSourceReference).toBe('floor plan A-01');
    expect(block.thermalMass.floor).toBe('very_heavy');
    expect(block.ventilationFlows[0].months).toHaveLength(12);
    expect(block.ventilationFlows[0].months.every((month: { conductanceWPerK: number }) => month.conductanceWPerK === 42)).toBe(true);
    expect(block.emission.system).toBe('floor_heating');
    expect(block.zebHeatDeliveryTemperature).toBe('from40_to60');
    expect(block.useInventoryComplete).toBe(true);
    expect(block.generator).toEqual({
      kind: 'external_heat', supplierReference: 'contract 42', qualityDeclarationPresent: false,
      auxiliary: { electricallyConnectedDevices: null, nominalPowerKw: null, sourceReference: '' },
    });
    expect(block.setpoints.sourceReference).toBe('table 7.13');
    expect(block.thermalMass.sourceReference).toBe('');
  });

  it('states the vertical pipes and lists plausibility warnings', async () => {
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({ status: 'incomplete', inputFingerprint: 'sha256:x', attestStatus: 'unattested',
        geometry: null, gaps: [{ code: 'vertical_pipes_unknown', path: 'ntaCalculation.verticalPipes' }],
        warnings: [{ code: 'declared_hot_water_efficiency_above_one', path: 'ntaCalculation.declaredUses[0].monthlyKwh',
          detail: 'η_W ≈ 1.08 > 1' }],
        derivedInput: null, performance: null }),
    }));
    renderWithProviders(<Harness />);
    const panel = within(await screen.findByRole('region', { name: 'NTA 8800 calculation (Rust kernel)' }));
    expect(await panel.findByText('Vertical pipes unknown: list them, or none (7.3.3)')).toBeInTheDocument();
    const warnings = within(panel.getByTestId('nta-plausibility-warnings'));
    expect(warnings.getByText('Declared hot-water use implies an efficiency above 1')).toBeInTheDocument();
    expect(warnings.getByText('η_W ≈ 1.08 > 1')).toBeInTheDocument();

    await user.click(panel.getByRole('button', { name: 'Start NTA input' }));
    const form = within(panel.getByRole('form', { name: 'NTA input' }));
    // The template leaves the pipes unknown (a kernel gap until stated).
    expect(form.getByLabelText('Pipes through the envelope')).toHaveValue('unknown');
    await user.selectOptions(form.getByLabelText('Pipes through the envelope'), 'listed');
    fireEvent.change(form.getByLabelText('Storeys of the zone'), { target: { value: '2' } });
    await user.click(form.getByRole('button', { name: 'Add pipe' }));
    expect(form.getAllByLabelText('Storeys of the zone')).toHaveLength(2);
    await user.click(form.getAllByRole('button', { name: 'Remove' })[1]);
    await user.click(form.getByRole('button', { name: 'Save' }));
    let block = JSON.parse(screen.getByTestId('block').textContent ?? 'null');
    expect(block.verticalPipes).toEqual([{ id: 'leiding-1', storeys: 2, insulated: false, sourceReference: '' }]);

    await user.click(panel.getByRole('button', { name: 'Edit NTA input' }));
    const edit = within(panel.getByRole('form', { name: 'NTA input' }));
    await user.selectOptions(edit.getByLabelText('Pipes through the envelope'), 'none');
    await user.click(edit.getByRole('button', { name: 'Save' }));
    block = JSON.parse(screen.getByTestId('block').textContent ?? 'null');
    expect(block.verticalPipes).toEqual([]);
  });

  it('switches the form to chapter 11 ventilation and saves a mirrored block', async () => {
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({ status: 'incomplete', inputFingerprint: 'sha256:x', attestStatus: 'unattested',
        geometry: null, gaps: [], derivedInput: null, performance: null }),
    }));
    renderWithProviders(<Harness />);
    const panel = within(await screen.findByRole('region', { name: 'NTA 8800 calculation (Rust kernel)' }));
    await user.click(await panel.findByRole('button', { name: 'Start NTA input' }));
    await user.selectOptions(panel.getByLabelText('Ventilation input'), 'chapter11');
    expect(panel.getByText('Ventilation chapter 11 – building')).toBeInTheDocument();
    await user.selectOptions(panel.getByLabelText('System variant'), 'd5c');
    await user.click(panel.getByLabelText('Heat recovery (HRU)'));
    await user.click(panel.getByRole('button', { name: 'Save' }));
    const block = JSON.parse(screen.getByTestId('block').textContent ?? 'null');
    expect(block.ventilationFlows).toEqual([]);
    expect(block.ventilation.system.unit.variant).toBe('d5c');
    expect(block.ventilation.system.unit.heatRecovery.bypass).toEqual({ kind: 'full' });
    expect(block.ventilation.heatingSetpointC).toBe(block.setpoints.heatingC);
    expect(block.ventilation.functions[0].function).toBe('residential');
  });

  it('shows the BENG 1 basis and chapter 11 outputs', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'calculated_unverified', inputFingerprint: 'sha256:abc', attestStatus: 'unattested', gaps: [], derivedInput: null,
        performance: {
          status: 'calculated_unverified', issues: [], needIndicatorKwhPerM2Year: 48.1, primaryFossilIndicatorKwhPerM2Year: 20,
          renewableSharePercent: 60, indicativeLabelClass: 'A++', labelSource: 'annex IX', bblCheck: null, a0Check: null,
          tojuli: [], tojuliMaxK: null, tojuliMeetsBblLimit: null, annualPrimaryFossilKwh: 2000, annualRenewablePrimaryKwh: 3000,
          annualCo2Kg: 812, co2KgPerM2: 6.5, annualStorageCorrectionKwh: 0,
          spaceHeating: {
            omittedTerms: [], monthly: Array.from({ length: 12 }, (_, index) => ({ ...month(100, index), recoverableLossKwh: 5 })),
            additionalZoneDemands: [],
            demand: {
              annualHeatingNeedKwh: 3000, annualCoolingNeedKwh: 200, omittedCorrections: [], recoverableLossesApplied: true,
              monthly: Array.from({ length: 12 }, () => ({ cooling: { needKwh: 10 } })),
              fixedC1: { status: 'calculated_unverified', annualHeatingNeedKwh: 5500, annualCoolingNeedKwh: 300 },
              ventilation: { zoneId: 'z1', annualFanElectricityKwh: 210, annualFrostProtectionElectricityKwh: 0,
                annualGrillePreheatingElectricityKwh: 0,
                months: [{ heating: { requiredOutdoorAirM3PerH: 191, infiltrationM3PerH: 38, conductanceWPerK: 90.06 } }] },
            },
          },
        },
      }),
    }));
    renderWithProviders(<NtaPerformancePanel />);
    expect(await screen.findByText(/separate run with the fixed C1 system \(§5\.4\.2\) · 5[.,]?800 kWh/)).toBeInTheDocument();
    expect(screen.getByText(/H_ve January \(heating\): 90\.1 W\/K/)).toBeInTheDocument();
    expect(screen.getByText(/Q_H;ls;rbl per year: 60 kWh/)).toBeInTheDocument();
    expect(screen.getByText(/812 kg · 6\.5 kg\/m²/)).toBeInTheDocument();
  });
});
