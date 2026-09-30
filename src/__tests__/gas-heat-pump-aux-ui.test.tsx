import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { GasHeatPumpAuxPanel } from '../components/GasHeatPumpAuxPanel/GasHeatPumpAuxPanel';
import { HeatPumpInventoryPanel } from '../components/HeatPumpInventoryPanel/HeatPumpInventoryPanel';
import { useEnergy } from '../context/EnergyContext';
import type { INtaHeatPumpInput } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

const pump: INtaHeatPumpInput = {
  id: 'ga-1', drive: 'absorption', source: 'outdoor_air', sink: 'hydronic',
  reversible: false, hybrid: false, booster: false,
  performanceEvidence: { kind: 'normative_default', reference: null },
  gasHeatPumpForfaitDraft: {
    generatorId: 'ga-1', drive: 'absorption', application: 'utility',
    applicationReference: 'office design', collectiveBuildingInstallation: false,
    externalHeatSupply: false, thermalCapacityKw: 20, capacityReference: 'plate',
    source: 'outdoor_air', sourceReference: 'source drawing',
    designSupplyTemperatureC: 35, designSupplyReference: 'heating design',
  },
};

afterEach(() => vi.unstubAllGlobals());

describe('gas heat pump draft auxiliary UI', () => {
  it('requires twelve sourced months, shows a limited result and saves the input', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({
      status: 'diagnostic_valid', annualAuxiliaryElectricityKwh: 100.8,
      monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
        cappedOnHours: 55, auxiliaryElectricityKwh: 8.4 })),
      inputFingerprint: 'sha256:gas-aux', bengCalculationAvailable: false, issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const view = renderWithProviders(<GasHeatPumpAuxPanel pump={pump} onSave={onSave} />);
    await user.click(screen.getByRole('button', { name: 'Calculate draft auxiliary electricity' }));
    expect(screen.getByRole('alert')).toHaveTextContent('twelve valid months');
    expect(fetchMock).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText('Building-share evidence'), { target: { value: 'whole building' } });
    fireEvent.change(screen.getByLabelText('Month-hours evidence'), { target: { value: 'hour schedule' } });
    fireEvent.change(screen.getByLabelText('Generator heat evidence'), { target: { value: 'heat ledger' } });
    for (let month = 1; month <= 12; month += 1) {
      fireEvent.change(screen.getByLabelText(`Month hours (h) ${month}`), { target: { value: '730' } });
      fireEvent.change(screen.getByLabelText(`Generator heat (kWh) ${month}`), { target: { value: '1000' } });
    }
    await user.click(screen.getByRole('button', { name: 'Calculate draft auxiliary electricity' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/nta8800/heat-pumps/gas-aux-draft/diagnose');
    const input = JSON.parse(options.body).input;
    expect(input.months).toHaveLength(12);
    expect(input).toMatchObject({ forfaitCopUsed: true, solutionPumpWPerKw: 0, nominalThermalCapacityKw: 20 });
    expect(screen.getByRole('status')).toHaveTextContent('100.80 kWh');
    expect(screen.getByRole('status')).toHaveTextContent('Source pump/fan, gas use, final norm, BENG and label remain unverified');
    await user.click(screen.getByRole('button', { name: 'Save gas auxiliary input' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ generatorId: 'ga-1' })));
    fireEvent.change(screen.getByLabelText('Generator heat (kWh) 1'), { target: { value: '900' } });
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    view.unmount();
    const saved = onSave.mock.calls[0][0];
    renderWithProviders(<GasHeatPumpAuxPanel pump={{ ...pump, gasHeatPumpAuxDraft: { ...saved, months: [...saved.months].reverse() } }} onSave={vi.fn()} />);
    expect(screen.getByLabelText('Generator heat (kWh) 1')).toHaveValue(1000);
    expect(screen.getByText(/Gas auxiliary input saved in this project/)).toBeInTheDocument();
  });

  it('persists and removes the input from the gas pump inventory', async () => {
    function InventoryState() {
      const { state } = useEnergy();
      return <><HeatPumpInventoryPanel /><output data-testid="gas-aux-input">
        {JSON.stringify(state.project.ntaHeatPumps?.[0]?.gasHeatPumpAuxDraft ?? null)}
      </output></>;
    }
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockImplementation(async (url: string) => ({ ok: true, json: async () =>
      url.includes('gas-aux-draft') ? {
        status: 'diagnostic_valid', annualAuxiliaryElectricityKwh: 100.8,
        monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, cappedOnHours: 55, auxiliaryElectricityKwh: 8.4 })),
        inputFingerprint: 'sha256:gas-aux', issues: [],
      } : { status: 'diagnostic_valid', table: '9.27', forfaitCop: 1.2, correctedCop: 1.2,
        temperatureBandUpperC: 35, issues: [] },
    })));
    renderWithProviders(<InventoryState />);
    await user.click(screen.getByRole('button', { name: 'Add heat pump' }));
    await user.selectOptions(screen.getByLabelText('Drive'), 'gas_engine');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    await user.click(screen.getByRole('button', { name: 'Gas heat pump draft COP' }));
    await user.click(screen.getByLabelText('Collective building installation'));
    fireEvent.change(screen.getByLabelText('Application evidence'), { target: { value: 'building design' } });
    fireEvent.change(screen.getByLabelText('Thermal capacity (kW)'), { target: { value: '20' } });
    fireEvent.change(screen.getByLabelText('Capacity evidence'), { target: { value: 'plate' } });
    fireEvent.change(screen.getByLabelText('Source evidence'), { target: { value: 'source plan' } });
    fireEvent.change(screen.getByLabelText('Design supply temperature (°C)'), { target: { value: '35' } });
    fireEvent.change(screen.getByLabelText('Supply temperature evidence'), { target: { value: 'design' } });
    await user.click(screen.getByRole('button', { name: 'Save gas table input' }));
    await screen.findByRole('button', { name: 'Gas heat pump draft auxiliary electricity' });
    expect(screen.getByRole('button', { name: 'Gas heat pump draft monthly terms' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Gas heat pump draft auxiliary electricity' }));
    fireEvent.change(screen.getByLabelText('Building-share evidence'), { target: { value: 'whole building' } });
    fireEvent.change(screen.getByLabelText('Month-hours evidence'), { target: { value: 'schedule' } });
    fireEvent.change(screen.getByLabelText('Generator heat evidence'), { target: { value: 'ledger' } });
    for (let month = 1; month <= 12; month += 1) {
      fireEvent.change(screen.getByLabelText(`Month hours (h) ${month}`), { target: { value: '730' } });
      fireEvent.change(screen.getByLabelText(`Generator heat (kWh) ${month}`), { target: { value: '1000' } });
    }
    await user.click(screen.getByRole('button', { name: 'Save gas auxiliary input' }));
    await waitFor(() => expect(JSON.parse(screen.getByTestId('gas-aux-input').textContent ?? 'null')).toMatchObject({
      generatorId: expect.any(String), nominalThermalCapacityKw: 20, months: expect.arrayContaining([{ month: 1, hours: 730, generatorOutputKwh: 1000 }]),
    }));
    await user.click(screen.getByRole('button', { name: 'Remove gas auxiliary input' }));
    expect(screen.getByTestId('gas-aux-input')).toHaveTextContent('null');
  });
});
