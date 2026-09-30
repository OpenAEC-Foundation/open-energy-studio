import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { GasHeatPumpForfaitPanel } from '../components/GasHeatPumpForfaitPanel/GasHeatPumpForfaitPanel';
import { HeatPumpInventoryPanel } from '../components/HeatPumpInventoryPanel/HeatPumpInventoryPanel';
import { useEnergy } from '../context/EnergyContext';
import type { INtaHeatPumpInput } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => vi.unstubAllGlobals());

describe('gas heat pump draft COP UI', () => {
  it('requires traceable evidence and shows a COP without implying gas use', async () => {
    const pump: INtaHeatPumpInput = {
      id: 'gas-1', drive: 'absorption', source: 'exhaust_air', sink: 'hydronic',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: { kind: 'normative_default', reference: null },
    };
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({
      status: 'diagnostic_valid', table: '9.29', forfaitCop: 2.4, correctedCop: 2.4, temperatureBandUpperC: 40,
      gasInputEnergyAvailable: false, auxiliaryEnergyAvailable: false, bengCalculationAvailable: false,
      issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    const onSave = vi.fn();
    const view = renderWithProviders(<GasHeatPumpForfaitPanel pump={pump} buildingFunction="office" onSave={onSave} />);
    await user.click(screen.getByRole('button', { name: 'Look up gas draft COP' }));
    expect(screen.getByRole('alert')).toHaveTextContent('traceable evidence');
    expect(fetchMock).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText('Application evidence'), { target: { value: 'building schedule' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity (kW)'), { target: { value: '40' } });
    fireEvent.change(screen.getByLabelText('Capacity evidence'), { target: { value: 'plate' } });
    fireEvent.change(screen.getByLabelText('Source evidence'), { target: { value: 'drawing' } });
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '40' } });
    fireEvent.change(screen.getByLabelText('Supply temperature evidence'), { target: { value: 'design' } });
    await user.click(screen.getByRole('button', { name: 'Look up gas draft COP' }));
    expect(await screen.findByText(/Draft table COP 9.29: 2.40/)).toBeInTheDocument();
    expect(screen.getByText(/No gas use, auxiliary electricity, BENG or energy label is calculated/)).toBeInTheDocument();
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/nta8800/heat-pumps/gas-forfait-cop-draft/diagnose');
    expect(JSON.parse(options.body).input).toMatchObject({ drive: 'absorption', source: 'exhaust_air', designSupplyTemperatureC: 40 });
    await user.click(screen.getByRole('button', { name: 'Save gas table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ generatorId: 'gas-1', application: 'utility' })));
    view.unmount();
    renderWithProviders(<GasHeatPumpForfaitPanel pump={{ ...pump, gasHeatPumpForfaitDraft: onSave.mock.calls[0][0] }}
      buildingFunction="office" onSave={vi.fn()} />);
    expect(screen.getByLabelText('Design supply temperature (°C)')).toHaveValue(40);
    expect(screen.getByText(/Gas table input saved in this project/)).toBeInTheDocument();
  });

  it('persists and removes the gas table input in the inventory', async () => {
    function InventoryState() {
      const { state } = useEnergy();
      return <><HeatPumpInventoryPanel /><output data-testid="gas-input">{JSON.stringify(state.project.ntaHeatPumps?.[0]?.gasHeatPumpForfaitDraft ?? null)}</output></>;
    }
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => ({
      status: 'diagnostic_valid', table: '9.27', forfaitCop: 1.2, correctedCop: 1.2, temperatureBandUpperC: 35,
      gasInputEnergyAvailable: false, auxiliaryEnergyAvailable: false,
      finalEditionVerified: false, bengCalculationAvailable: false, issues: [],
    }) }));
    renderWithProviders(<InventoryState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Heat source'), 'outdoor_air');
    await user.selectOptions(screen.getByLabelText('Drive'), 'gas_engine');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    await user.click(screen.getByRole('button', { name: 'Gas heat pump draft COP' }));
    await user.click(screen.getByLabelText('Collective building installation'));
    fireEvent.change(screen.getByLabelText('Application evidence'), { target: { value: 'collective schedule' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity (kW)'), { target: { value: '25' } });
    fireEvent.change(screen.getByLabelText('Capacity evidence'), { target: { value: 'plate' } });
    fireEvent.change(screen.getByLabelText('Source evidence'), { target: { value: 'system design' } });
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Supply temperature evidence'), { target: { value: 'heating design' } });
    await user.click(screen.getByRole('button', { name: 'Save gas table input' }));
    await waitFor(() => expect(JSON.parse(screen.getByTestId('gas-input').textContent ?? 'null')).toMatchObject({
      drive: 'gas_engine', application: 'residential_collective_at_most25_kw', source: 'outdoor_air', designSupplyTemperatureC: 35,
    }));
    expect(screen.getByRole('button', { name: 'Remove gas table input' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Remove gas table input' }));
    expect(screen.getByTestId('gas-input')).toHaveTextContent('null');
  });

  it('requires a sourced correction for residential collective ground', async () => {
    const pump: INtaHeatPumpInput = {
      id: 'ground-gas', drive: 'gas_engine', source: 'ground', sink: 'hydronic',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: { kind: 'normative_default', reference: null },
    };
    const onSave = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({
      status: 'diagnostic_valid', table: '9.27', forfaitCop: 1.3, correctedCop: 1.43,
      temperatureBandUpperC: 35, issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<GasHeatPumpForfaitPanel pump={pump} buildingFunction="residential" onSave={onSave} />);
    await user.click(screen.getByLabelText('Collective building installation'));
    fireEvent.change(screen.getByLabelText('Application evidence'), { target: { value: 'building schedule' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity (kW)'), { target: { value: '25' } });
    fireEvent.change(screen.getByLabelText('Capacity evidence'), { target: { value: 'plate' } });
    fireEvent.change(screen.getByLabelText('Source evidence'), { target: { value: 'ground loop drawing' } });
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Supply temperature evidence'), { target: { value: 'heating design' } });
    await user.click(screen.getByRole('button', { name: 'Save gas table input' }));
    expect(onSave).not.toHaveBeenCalled();
    expect(fetchMock).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText('Supplied source correction csource'), { target: { value: '1.1' } });
    fireEvent.change(screen.getByLabelText('Source correction evidence'), { target: { value: 'appendix V calculation' } });
    await user.click(screen.getByRole('button', { name: 'Save gas table input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
      application: 'residential_collective_at_most25_kw', sourceCorrectionFactor: 1.1,
    })));
    expect(await screen.findByText(/COP with supplied source correction: 1.43/)).toBeInTheDocument();
  });
});
