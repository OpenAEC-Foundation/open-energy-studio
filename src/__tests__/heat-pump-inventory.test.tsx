import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { HeatPumpInventoryPanel } from '../components/HeatPumpInventoryPanel/HeatPumpInventoryPanel';
import { PreviewPanel } from '../components/PreviewPanel/PreviewPanel';
import { renderWithProviders, userEvent } from './test-utils';

function InventoryWithState() {
  const { state } = useEnergy();
  return <><HeatPumpInventoryPanel /><PreviewPanel />
    <output data-testid="assets">{state.project.ntaHeatPumps?.map((pump) => `${pump.source}:${pump.servedZoneIds?.join(',')}`).join('|') ?? ''}</output>
    <output data-testid="points">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.performancePoints ?? [])}</output>
    <output data-testid="dhw-test-points">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.dhwTestPoints ?? [])}</output>
    <output data-testid="registry">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.performanceEvidence.registryRecord ?? null)}</output>
    <output data-testid="operating-limits">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.declaredOperatingLimits ?? null)}</output>
    <output data-testid="auxiliaries">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.auxiliaryComponents ?? [])}</output>
    <output data-testid="system-links">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.systemLinks ?? [])}</output>
    <output data-testid="forfait-input">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.forfaitHeatPumpDraft ?? null)}</output>
  </>;
}

afterEach(() => vi.unstubAllGlobals());

describe('standalone NTA heat pump inventory', () => {
  it('keeps a hybrid pump table input and opens the draft dispatch without an energy label', async () => {
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ json: async () => ({
      status: 'input_valid', table: '9.27', correctedCop: 2.4,
      inputFingerprint: 'sha256:hybrid-table', issues: [],
    }) }));
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.click(screen.getByLabelText('Hybrid'));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    await user.click(screen.getByRole('button', { name: 'Draft COP table' }));
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'design sheet' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '8' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'manufacturer sheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Draft hybrid monthly dispatch' })).toBeInTheDocument());
    await user.click(screen.getByRole('button', { name: 'Draft hybrid monthly dispatch' }));
    expect(screen.getByRole('region', { name: 'Hybrid heat pump: monthly dispatch' })).toBeInTheDocument();
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
  });

  it('saves and reopens a COP table classification without creating a result', async () => {
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ json: async () => ({
      status: 'input_valid', table: '9.27', temperatureBand: '>30–35 °C',
      tableCop: 2.4, correctedCop: 2.4, inputFingerprint: 'sha256:table', issues: [],
    }) }));
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    await user.click(screen.getByRole('button', { name: 'Draft COP table' }));
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Scope and source evidence'), { target: { value: 'design sheet' } });
    fireEvent.change(screen.getByLabelText('Declared thermal capacity (kW)'), { target: { value: '8' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity evidence'), { target: { value: 'manufacturer sheet' } });
    await user.selectOptions(screen.getByLabelText('Building installation'), 'false');
    await user.click(screen.getByRole('button', { name: 'Save table input' }));
    await waitFor(() => expect(JSON.parse(screen.getByTestId('forfait-input').textContent ?? 'null')).toMatchObject({
      source: 'outdoor_air', designSupplyTemperatureC: 35,
    }));
    await user.click(screen.getByRole('button', { name: 'Draft COP table' }));
    await user.click(screen.getByRole('button', { name: 'Draft COP table' }));
    expect(screen.getByLabelText('Design supply temperature (°C)')).toHaveValue(35);
    expect(screen.getByText(/Table input saved in this project/)).toBeInTheDocument();
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Remove table input' }));
    expect(screen.getByTestId('forfait-input')).toHaveTextContent('null');
  }, 60000);
  it('does not offer an air-to-air COP row for a non-air source', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Heat source'), 'ground');
    await user.selectOptions(screen.getByLabelText('Sink / service'), 'indoor_air');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.queryByRole('button', { name: 'Draft COP table' })).not.toBeInTheDocument();
  });
  it('opens the scoped measured-auxiliary diagnostic for an eligible pump', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    await user.click(screen.getByRole('button', { name: 'Auxiliary concept diagnosis' }));
    expect(screen.getByRole('region', { name: 'Heat pump auxiliary electricity · draft' })).toBeInTheDocument();
    expect(screen.getByText(/Save measured inputs and sources in the project/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Auxiliary concept diagnosis' }));
    expect(screen.queryByRole('region', { name: 'Heat pump auxiliary electricity · draft' })).not.toBeInTheDocument();
  });
  it('preserves declared hybrid shutoff limits and blocks a declaration mismatch', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.type(screen.getByLabelText('Quality declaration reference'), 'BCRG 20240234GK');
    await user.click(screen.getByRole('button', { name: 'Add limits' }));
    await user.type(screen.getByLabelText('Minimum operating COP'), '2');
    await user.type(screen.getByLabelText('Maximum supply temperature (°C)'), '55');
    await user.type(screen.getByLabelText('Declaration norm edition'), 'NTA 8800:2024');
    await user.type(screen.getByLabelText('Limit source reference'), 'BCRG 20240234GK p2');
    await user.click(screen.getByRole('button', { name: 'Save limits' }));
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'normative_default');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByRole('alert')).toHaveTextContent('controlled declaration');
    expect(screen.getByTestId('assets')).toBeEmptyDOMElement();
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('operating-limits').textContent ?? 'null')).toEqual({
      minimumOperatingCop: 2, maximumSupplyTemperatureC: 55,
      declarationNormVersion: 'NTA 8800:2024', sourceReference: 'BCRG 20240234GK p2',
    });
  }, 60000);
  it('keeps referenced DHW test-profile inputs separate from annual performance', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Sink / service'), 'combined_hydronic_and_hot_water');
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.type(screen.getByLabelText('Quality declaration reference'), 'BCRG 20240123GK');
    await user.click(screen.getByRole('button', { name: 'Add test profile' }));
    await user.type(screen.getByLabelText('Tap profile'), 'M');
    await user.type(screen.getByLabelText('Useful test energy (kWh/day)'), '5.865');
    await user.type(screen.getByLabelText('Input test energy (kWh/day)'), '2.52');
    await user.type(screen.getByLabelText('Nominal capacity (kW)'), '4.71');
    await user.type(screen.getByLabelText('Supplied practice factor'), '0.9');
    await user.type(screen.getByLabelText('Test setpoint (°C)'), '49.1');
    await user.type(screen.getByLabelText('Design setpoint (°C)'), '55');
    await user.type(screen.getByLabelText('Declaration norm edition'), 'NTA 8800:2020');
    await user.type(screen.getByLabelText('Declaration and table/page'), 'BCRG 20240123GK p3');
    await user.click(screen.getByRole('button', { name: 'Save test profile' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('dhw-test-points').textContent ?? '[]')).toMatchObject([{
      tapProfile: 'M', usefulEnergyKwhPerDay: 5.865, inputEnergyKwhPerDay: 2.52,
      declarationNormVersion: 'NTA 8800:2020',
    }]);
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
  }, 60000);
  it('records a water-source asset with its served zone and removes it', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Heat source'), 'surface_water');
    await user.click(screen.getByRole('checkbox', { name: 'Woonfunctie' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('assets')).toHaveTextContent('surface_water:zone-main');
    expect(screen.getByRole('alert')).toHaveTextContent(/indicative calculator cannot include/i);
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /Remove:/ }));
    expect(screen.getByTestId('assets')).toBeEmptyDOMElement();
  });

  it('requires a reference for a controlled quality declaration', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Enter a quality declaration reference.');
    expect(screen.getByTestId('assets')).toBeEmptyDOMElement();
  });

  it('keeps an exact user-entered registry identity with the declared heat pump', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.type(screen.getByLabelText('Quality declaration reference'), '20260214GK');
    await user.click(screen.getByRole('checkbox', { name: 'Record registry identity' }));
    await user.type(screen.getByLabelText('Registration number'), '20260214GK');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Complete the declaration registration');
    expect(screen.getByTestId('assets')).toBeEmptyDOMElement();
    await user.type(screen.getByLabelText('Registered product or combination'), 'Model A + tank B');
    await user.type(screen.getByLabelText('Manufacturer or supplier'), 'Supplier');
    await user.type(screen.getByLabelText('Declaration source URL'), 'https://bcrg.nl/declaration/example');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('registry').textContent ?? 'null')).toEqual({
      registrationNumber: '20260214GK', productName: 'Model A + tank B',
      manufacturer: 'Supplier', sourceUrl: 'https://bcrg.nl/declaration/example',
    });
  });

  it('stores a referenced operating point with explicit units and carrier', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.click(screen.getByRole('button', { name: 'Add operating point' }));
    await user.type(screen.getByLabelText('Source temperature (°C)'), '7');
    await user.type(screen.getByLabelText('Sink temperature (°C)'), '35');
    await user.type(screen.getByLabelText('Useful capacity (kW)'), '5');
    await user.type(screen.getByLabelText('Input power (kW)'), '1.5');
    await user.type(screen.getByLabelText('Test or declaration reference'), 'Test report 42');
    await user.click(screen.getByRole('button', { name: 'Save operating point' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const points = JSON.parse(screen.getByTestId('points').textContent ?? '[]');
    expect(points).toMatchObject([{
      service: 'space_heating', sourceTemperatureC: 7, sinkTemperatureC: 35,
      usefulCapacityKw: 5, inputPowerKw: 1.5,
      inputEnergyCarrier: 'electricity', testReference: 'Test report 42',
    }]);
  });

  it('rejects a gas operating point for an electric compressor in the editor', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.click(screen.getByRole('button', { name: 'Add operating point' }));
    await user.type(screen.getByLabelText('Source temperature (°C)'), '7');
    await user.type(screen.getByLabelText('Sink temperature (°C)'), '35');
    await user.type(screen.getByLabelText('Useful capacity (kW)'), '5');
    await user.type(screen.getByLabelText('Input power (kW)'), '1.5');
    await user.type(screen.getByLabelText('Test or declaration reference'), 'Test report 42');
    await user.selectOptions(screen.getByLabelText('Input energy carrier'), 'gas');
    await user.click(screen.getByRole('button', { name: 'Save operating point' }));
    expect(screen.getByRole('alert')).toHaveTextContent('electric compressor');
    expect(screen.queryByText(/5 kW \/ 1.5 kW/)).not.toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText('Input energy carrier'), 'electricity');
    await user.click(screen.getByRole('button', { name: 'Save operating point' }));
    expect(screen.getByText(/5 kW \/ 1.5 kW/)).toBeInTheDocument();
  });

  it('blocks saving an electric compressor after changing drive with a gas point', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Drive'), 'gas_engine');
    await user.click(screen.getByRole('button', { name: 'Add operating point' }));
    await user.type(screen.getByLabelText('Source temperature (°C)'), '7');
    await user.type(screen.getByLabelText('Sink temperature (°C)'), '35');
    await user.type(screen.getByLabelText('Useful capacity (kW)'), '5');
    await user.type(screen.getByLabelText('Input power (kW)'), '1.5');
    await user.type(screen.getByLabelText('Test or declaration reference'), 'Test report 42');
    await user.click(screen.getByRole('button', { name: 'Save operating point' }));
    await user.selectOptions(screen.getByLabelText('Drive'), 'electric_compression');
    expect(screen.getByText(/An electric compressor operating point/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByRole('alert')).toHaveTextContent('electric compressor');
    expect(screen.getByTestId('assets')).toBeEmptyDOMElement();
  });

  it('stores auxiliary power with an explicit measurement boundary', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.click(screen.getByRole('button', { name: 'Add component' }));
    await user.type(screen.getByLabelText('Rated power (W)'), '80');
    await user.selectOptions(screen.getByLabelText('Measurement boundary'), 'additional');
    await user.type(screen.getByLabelText('Source or product reference'), 'Datasheet 42');
    await user.click(screen.getByRole('button', { name: 'Save component' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const auxiliaries = JSON.parse(screen.getByTestId('auxiliaries').textContent ?? '[]');
    expect(auxiliaries).toMatchObject([{
      kind: 'source_pump', service: 'space_heating', nominalPowerW: 80,
      energyCarrier: 'electricity', measurementBoundary: 'additional',
      evidenceReference: 'Datasheet 42',
    }]);
  });

  it('stores an explicit backup-generator link and keeps it after editing', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InventoryWithState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.click(screen.getByRole('button', { name: 'Add link' }));
    await user.selectOptions(screen.getByLabelText('Linked system'), 'heat-1');
    await user.type(screen.getByLabelText('Link source reference'), 'Hydraulic scheme 1');
    await user.click(screen.getByRole('button', { name: 'Save link' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('system-links').textContent ?? '[]')).toMatchObject([{
      role: 'backup_generator', targetKind: 'heating_system',
      targetId: 'heat-1', evidenceReference: 'Hydraulic scheme 1',
    }]);
    await user.click(screen.getByRole('button', { name: 'Edit' }));
    expect(screen.getByText(/heat-1 · Hydraulic scheme 1/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('system-links').textContent ?? '[]')).toHaveLength(1);
  });
});
