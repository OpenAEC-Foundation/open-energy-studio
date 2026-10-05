import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { HeatPumpForfaitDiagnosticPanel } from '../components/HeatPumpForfaitDiagnosticPanel/HeatPumpForfaitDiagnosticPanel';
import type { INtaHeatPumpInput } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

const pump: INtaHeatPumpInput = {
  id: 'hp-forfait', source: 'outdoor_air', sink: 'hydronic', drive: 'electric_compression',
  reversible: false, hybrid: false, booster: false,
  performanceEvidence: { kind: 'normative_default', reference: null },
};

afterEach(() => vi.unstubAllGlobals());

describe('draft COP table UI', () => {
  it('requires evidence, uses the Rust lookup and restores saved table input', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({ json: async () => ({
      status: 'input_valid', table: '9.27', temperatureBand: '>30–35 °C',
      tableCop: 2.4, correctedCop: 2.4, inputFingerprint: 'sha256:table',
      finalEditionVerified: false, applicabilityVerified: false,
      annualPerformanceAvailable: false, bengCalculationAvailable: false, issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const view = renderWithProviders(<HeatPumpForfaitDiagnosticPanel pump={pump} buildingFunction="residential" onSave={onSave} />);
    await user.click(screen.getByRole('button', { name: 'Look up draft COP' }));
    expect(screen.getByRole('alert')).toHaveTextContent('traceable evidence');
    expect(fetchMock).not.toHaveBeenCalled();

    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'design sheet' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '25' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'manufacturer sheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    await user.click(screen.getByRole('button', { name: 'Look up draft COP' }));
    expect(await screen.findByText('Draft table COP: 2.40')).toBeInTheDocument();
    expect(screen.getByText(/final edition, applicability, seasonal performance and BENG calculation are unverified/i)).toBeInTheDocument();
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose');
    const input = JSON.parse(options.body).input;
    expect(input).toMatchObject({ generatorId: 'hp-forfait', scope: 'residential_at_most25_kw',
      source: 'outdoor_air', sink: 'hydronic', designSupplyTemperatureC: 35,
      classificationSourceReference: 'design sheet', thermalCapacityKw: 25,
      collectiveBuildingInstallation: false });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(input));
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '36' } });
    expect(screen.queryByText('Draft table COP: 2.40')).not.toBeInTheDocument();
    view.unmount();
    renderWithProviders(<HeatPumpForfaitDiagnosticPanel pump={{ ...pump, forfaitHeatPumpDraft: input }}
      buildingFunction="residential" onSave={vi.fn()} />);
    expect(screen.getByLabelText('Design supply temperature (°C)')).toHaveValue(35);
    expect(screen.getByText(/Table input saved in this project/)).toBeInTheDocument();
  });

  it('requires an explicit source correction for a residential ground loop', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ json: async () => ({
      status: 'input_valid', table: '9.27', temperatureBand: '≤30 °C',
      tableCop: 4, correctedCop: 4, inputFingerprint: 'sha256:ground', issues: [],
    }) }));
    renderWithProviders(<HeatPumpForfaitDiagnosticPanel pump={{ ...pump, source: 'ground' }}
      buildingFunction="residential" onSave={onSave} />);
    await user.selectOptions(screen.getByLabelText('Table source class'), 'ground');
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '30' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'system design' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '10' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'datasheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    expect(onSave).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText('Supplied source correction csource'), { target: { value: '1' } });
    fireEvent.change(screen.getByLabelText('Source correction evidence'), { target: { value: 'no regeneration, design' } });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ sourceCorrectionFactor: 1,
      sourceCorrectionReference: 'no regeneration, design' })));
  });

  it('keeps a Rust-rejected correction out of the saved project', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ json: async () => ({
      status: 'invalid', table: '9.27', correctedCop: null, issues: [{ code: 'cop_overflow', path: 'sourceCorrectionFactor' }],
    }) }));
    renderWithProviders(<HeatPumpForfaitDiagnosticPanel pump={{ ...pump, source: 'ground' }}
      buildingFunction="residential" onSave={onSave} />);
    await user.selectOptions(screen.getByLabelText('Table source class'), 'ground');
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '30' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'design sheet' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '10' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'datasheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    fireEvent.change(screen.getByLabelText('Supplied source correction csource'), { target: { value: '1e308' } });
    fireEvent.change(screen.getByLabelText('Source correction evidence'), { target: { value: 'source report' } });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    expect(await screen.findByText(/cop_overflow/)).toBeInTheDocument();
    expect(onSave).not.toHaveBeenCalled();
  });

  it('requires all three strictly passing outdoor-air tests for the higher row', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({ json: async () => ({
      status: 'input_valid', table: '9.27', rowVariant: 'table_9_28_high_efficiency',
      temperatureBand: '>30–35 °C', tableCop: 3.35, correctedCop: 3.35,
      inputFingerprint: 'sha256:high-row', issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    renderWithProviders(<HeatPumpForfaitDiagnosticPanel pump={pump} buildingFunction="residential" onSave={onSave} />);
    await user.selectOptions(screen.getByLabelText('Table row'), 'table_9_28_high_efficiency');
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'design' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '8' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'datasheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    fireEvent.change(screen.getByLabelText('Tested product or combination'), { target: { value: 'model A' } });
    fireEvent.change(screen.getByLabelText('NEN-EN 14511-2:2022 test report'), { target: { value: 'lab report p4' } });
    fireEvent.change(screen.getByLabelText('A7(6)/W45 · COP > 2.75'), { target: { value: '2.75' } });
    fireEvent.change(screen.getByLabelText('A7(6)/W35 · COP > 2.85'), { target: { value: '2.86' } });
    fireEvent.change(screen.getByLabelText('A−7(−8)/W45 · COP > 1.9'), { target: { value: '1.91' } });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    expect(onSave).not.toHaveBeenCalled();
    expect(fetchMock).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText('A7(6)/W45 · COP > 2.75'), { target: { value: '2.76' } });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
      rowVariant: 'table_9_28_high_efficiency',
      highEfficiencyEvidence: expect.objectContaining({
        testStandardEdition: 'NEN-EN 14511-2:2022',
        points: expect.arrayContaining([{ condition: 'a7_wet6_w45', measuredCop: 2.76 }]),
      }),
    })));
    expect(screen.getByRole('status')).toHaveTextContent('Higher efficiency row (table 9.28 test)');
  });

  it('uses the groundwater row without a source declaration and the collective row with one', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const fetchMock = vi.fn()
      .mockResolvedValueOnce({ json: async () => ({
        status: 'input_valid', table: '9.27', rowVariant: 'base', correctedCop: 4.5,
        selectedSource: 'groundwater_below15_c', sourceFallbackApplied: true,
        temperatureBand: '>30–35 °C', inputFingerprint: 'sha256:fallback', issues: [],
      }) })
      .mockResolvedValueOnce({ json: async () => ({
        status: 'input_valid', table: '9.27', rowVariant: 'base', correctedCop: 5.1,
        selectedSource: 'collective20_to40_c', sourceFallbackApplied: false,
        temperatureBand: '>30–35 °C', inputFingerprint: 'sha256:collective', issues: [],
      }) });
    vi.stubGlobal('fetch', fetchMock);
    renderWithProviders(<HeatPumpForfaitDiagnosticPanel pump={{ ...pump, source: 'ground' }}
      buildingFunction="residential" onSave={onSave} />);
    await user.selectOptions(screen.getByLabelText('Table source class'), 'collective20_to40_c');
    fireEvent.change(screen.getByLabelText('Declared source temperature (°C)'), { target: { value: '20' } });
    fireEvent.change(screen.getByLabelText('Source temperature evidence'), { target: { value: 'source design' } });
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'system schedule' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '8' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'datasheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(screen.getByText(/groundwater-below-15 °C row/)).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText('Supplied source correction csource'), { target: { value: '1' } });
    fireEvent.change(screen.getByLabelText('Source correction evidence'), { target: { value: 'no regeneration' } });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
      source: 'collective20_to40_c', sourceCorrectionFactor: 1,
      sourceQualityDeclarationReference: null,
    })));
    expect(screen.getByRole('status')).toHaveTextContent('Fallback table source:');
    fireEvent.change(screen.getByLabelText('Source quality declaration reference (if available)'), { target: { value: 'QD-1' } });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
      source: 'collective20_to40_c', sourceTemperatureC: 20,
      sourceQualityDeclarationReference: 'QD-1', sourceCorrectionFactor: null,
    })));
  });

  it('exposes the ground fallback when ground or groundwater details are unknown', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ json: async () => ({
      status: 'input_valid', table: '9.27', rowVariant: 'base', correctedCop: 3.8,
      selectedSource: 'ground', sourceFallbackApplied: true,
      sourceFallbackReason: 'ground_or_groundwater_unknown',
      temperatureBand: '>30–35 °C', inputFingerprint: 'sha256:unknown-ground', issues: [],
    }) }));
    renderWithProviders(<HeatPumpForfaitDiagnosticPanel pump={{ ...pump, source: 'groundwater' }}
      buildingFunction="residential" onSave={onSave} />);
    await user.selectOptions(screen.getByLabelText('Table source class'), 'ground_or_groundwater_unknown');
    expect(screen.getByText(/uses the ground row/)).toBeInTheDocument();
    expect(screen.queryByLabelText('Declared source temperature (°C)')).not.toBeInTheDocument();
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'source survey' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '8' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'manufacturer sheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    fireEvent.change(screen.getByLabelText('Supplied source correction csource'), { target: { value: '1' } });
    fireEvent.change(screen.getByLabelText('Source correction evidence'), { target: { value: 'no regeneration' } });
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
      source: 'ground_or_groundwater_unknown', sourceTemperatureC: null, sourceCorrectionFactor: 1,
    })));
    expect(screen.getByRole('status')).toHaveTextContent('Fallback table source: Ground loop');
  });
});
