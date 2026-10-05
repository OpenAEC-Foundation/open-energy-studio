import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { HybridHeatPumpMonthlyPanel } from '../components/HeatPumpForfaitMonthlyPanel/HybridHeatPumpMonthlyPanel';
import type { INtaHeatPumpInput } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

const pump: INtaHeatPumpInput = {
  id: 'hp-hybrid', source: 'district_water', sink: 'hydronic', drive: 'electric_compression',
  reversible: false, hybrid: true, booster: false,
  performanceEvidence: { kind: 'normative_default', reference: null },
  forfaitHeatPumpDraft: {
    generatorId: 'hp-hybrid', classificationSourceReference: 'source design',
    scope: 'residential_at_most25_kw', source: 'collective15_to20_c', sink: 'hydronic',
    designSupplyTemperatureC: 35, sourceCorrectionFactor: null, sourceCorrectionReference: null,
    sourceTemperatureC: 15, sourceTemperatureEvidenceReference: 'source meter',
    thermalCapacityKw: 4, capacitySourceReference: 'hp plate', collectiveBuildingInstallation: false,
  },
  heatingAuxMeasuredDraft: {
    generatorId: 'hp-hybrid', generatorSourceReference: 'hp plate',
    measurements: { standbyElectronicsW: 10, deliveryPumpDuringCompressorW: 200,
      deliveryPumpPrePostW: 90, pumpPreRunSeconds: 300, pumpPostRunSeconds: 300,
      averageCompressorOnSeconds: 600, meanCompressorModulation: 0.5,
      nominalElectricDriveKw: 2, measurementSourceReference: 'power measurements',
      timingSourceReference: 'cycle measurements' },
    inputEnergySourceReference: 'old supplied monthly values',
    months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
      generatorInputElectricityKwh: 999 })),
  },
};

afterEach(() => vi.unstubAllGlobals());

describe('new-build hybrid draft UI', () => {
  it('passes node demand, provenance and matched COP to one Rust chain', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn()
      .mockResolvedValueOnce({ json: async () => ({ status: 'input_valid', correctedCop: 4.8,
        inputFingerprint: 'sha256:cop' }) })
      .mockResolvedValueOnce({ json: async () => ({ status: 'diagnostic_valid', generationEfficiency: 0.95,
        inputFingerprint: 'sha256:boiler' }) })
      .mockResolvedValueOnce({ json: async () => ({
        status: 'diagnostic_valid', inputFingerprint: 'sha256:hybrid', issues: [],
        dispatch: { monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
          generators: [{ generatorId: 'hp-hybrid', deliveredHeatKwh: index === 4 ? 980 : 750 },
            { generatorId: 'boiler', deliveredHeatKwh: index === 4 ? 20 : 250 }] })) },
        heatPump: { monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
          generatorOutputKwh: index === 4 ? 980 : 750, generatorInputElectricityKwh: 143.19 })) },
        heatPumpAuxiliary: { auxiliary: { monthlyAuxiliaryElectricityKwh: Array.from({ length: 12 }, (_, index) => ({
          month: index + 1, electricityKwh: 34.51 })) } },
        boiler: { monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
          inputNaturalGasKwh: index === 4 ? 20 / 0.95 : 250 / 0.95,
          auxiliaryElectricityKwh: 7.27 })) },
      }) })
      .mockResolvedValueOnce({ json: async () => ({ status: 'input_valid', correctedCop: 4.8,
        inputFingerprint: 'sha256:cop' }) })
      .mockResolvedValueOnce({ json: async () => ({ status: 'invalid', generationEfficiency: null,
        issues: [{ code: 'pilot_flame_route_unavailable', path: 'pilotFlamePresent' }] }) });
    vi.stubGlobal('fetch', fetchMock);
    renderWithProviders(<HybridHeatPumpMonthlyPanel pump={pump} />);
    await user.click(screen.getByRole('button', { name: 'Calculate draft hybrid months' }));
    expect(fetchMock).not.toHaveBeenCalled();
    await user.click(screen.getByLabelText('This is a new-build case with standard parallel operation'));
    fireEvent.change(screen.getByLabelText('Node heat source reference'), { target: { value: 'node balance' } });
    fireEvent.change(screen.getByLabelText('Source system evidence'), { target: { value: 'source design' } });
    fireEvent.change(screen.getByLabelText('Boiler ID'), { target: { value: 'boiler' } });
    fireEvent.change(screen.getByLabelText('Boiler type evidence'), { target: { value: 'system design' } });
    fireEvent.change(screen.getByLabelText('Rated boiler power (kW)'), { target: { value: '6' } });
    fireEvent.change(screen.getByLabelText('Boiler power source'), { target: { value: 'boiler plate' } });
    fireEvent.change(screen.getByLabelText('Boiler location evidence'), { target: { value: 'site inspection' } });
    fireEvent.change(screen.getByLabelText('Average design emission temperature (°C)'), { target: { value: '45' } });
    fireEvent.change(screen.getByLabelText('Temperature and circuit evidence'), { target: { value: 'heating design' } });
    fireEvent.change(screen.getByLabelText('Boiler installation year (blank = unknown)'), { target: { value: '2015' } });
    fireEvent.change(screen.getByLabelText('Installation year evidence'), { target: { value: 'commissioning certificate' } });
    for (let month = 1; month <= 12; month += 1) {
      fireEvent.change(screen.getByLabelText(`Node heat (kWh) ${month}`), { target: { value: '1000' } });
    }
    await user.click(screen.getByRole('button', { name: 'Calculate draft hybrid months' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(3));
    expect(fetchMock.mock.calls[1][0]).toBe('/api/v1/nta8800/boilers/forfait-draft/diagnose');
    expect(fetchMock.mock.calls[2][0]).toBe('/api/v1/nta8800/heat-pumps/hybrid-monthly-draft/diagnose');
    const input = JSON.parse(fetchMock.mock.calls[2][1].body).input;
    expect(input.dispatch.generators[0]).toMatchObject({ id: 'hp-hybrid', nominalThermalPowerKw: 4,
      priorityEfficiency: 4.8, powerReference: 'hp plate' });
    expect(input.dispatch.generators[1]).toMatchObject({ id: 'boiler', nominalThermalPowerKw: 6,
      priorityEfficiency: 0.95, efficiencyReference: 'boiler_forfait_sha256:sha256:boiler' });
    expect(input.boiler).toMatchObject({ role: 'individual_supplementary', kind: 'hr107',
      averageDesignEmissionTemperatureC: 45, fuel: 'natural_gas', installationYear: 2015,
      installationYearReference: 'commissioning certificate' });
    expect(input.heatPumpAuxiliaryMeasurements).toMatchObject({ generatorId: 'hp-hybrid',
      measurements: { standbyElectronicsW: 10, nominalElectricDriveKw: 2 } });
    expect(input.heatPumpAuxiliaryMeasurements).not.toHaveProperty('months');
    expect(input.dispatch.nodeInputKwh).toHaveLength(12);
    expect(input.declaredOperatingLimitsPresent).toBe(false);
    expect(screen.getByRole('status')).toHaveTextContent('980.00');
    expect(screen.getByRole('status')).toHaveTextContent('263.16');
    expect(screen.getByRole('status')).toHaveTextContent('7.27');
    expect(screen.getByRole('status')).toHaveTextContent('34.51');
    expect(screen.getByRole('status')).toHaveTextContent('source pump/fan, pilot flame');
    await user.click(screen.getByLabelText('Pilot flame present (this draft route will reject it)'));
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Calculate draft hybrid months' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(5));
    expect(JSON.parse(fetchMock.mock.calls[4][1].body).input.pilotFlamePresent).toBe(true);
    expect(screen.getByRole('alert')).toHaveTextContent('draft boiler table input is invalid');
  });
});
