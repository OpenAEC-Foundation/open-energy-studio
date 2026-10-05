import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { GasHeatPumpMonthlyPanel } from '../components/GasHeatPumpMonthlyPanel/GasHeatPumpMonthlyPanel';
import type { INtaHeatPumpInput } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => vi.unstubAllGlobals());

describe('gas heat pump draft monthly terms UI', () => {
  it('sends collective temperature and declaration evidence to the dh diagnostic and clears stale results', async () => {
    const pump: INtaHeatPumpInput = {
      id: 'ga-1', drive: 'absorption', source: 'groundwater', sink: 'hydronic',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: { kind: 'normative_default', reference: null },
      gasHeatPumpForfaitDraft: {
        generatorId: 'ga-1', drive: 'absorption', application: 'utility', applicationReference: 'office',
        collectiveBuildingInstallation: false, externalHeatSupply: false, thermalCapacityKw: 20,
        capacityReference: 'plate', source: 'groundwater_aquifer', sourceReference: 'source design',
        designSupplyTemperatureC: 35, designSupplyReference: 'design',
      },
      gasHeatPumpAuxDraft: {
        generatorId: 'ga-1', drive: 'absorption', nominalThermalCapacityKw: 20,
        capacityReference: 'plate', standbyElectronicsW: 10, burnerAuxiliaryWPerKw: 1,
        solutionPumpWPerKw: 0, coefficientsReference: 'draft', meanModulation: 1,
        modulationReference: 'draft', buildingShare: 1, buildingShareReference: 'whole building',
        forfaitCopUsed: true, monthHoursReference: 'hours', generatorOutputReference: 'meter',
        months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, hours: 730, generatorOutputKwh: 1000 })),
      },
    };
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({
      status: 'diagnostic_valid', annualDeliveredSourceHeatKwh: 5000,
      primaryFossilFactor: 1.45 / 23, primaryRenewableFactor: 0.95,
      annualDraftPrimaryFossilKwh: 5000 * 1.45 / 23, annualDraftPrimaryRenewableKwh: 4750,
      inputFingerprint: 'sha256:source', issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<GasHeatPumpMonthlyPanel pump={pump} />);
    await user.selectOptions(screen.getByLabelText('Source system'), 'collective_groundwater_surface_or_at_least15_c');
    fireEvent.change(screen.getByLabelText('Source-system evidence'), { target: { value: 'invoice' } });
    await user.selectOptions(screen.getByLabelText('Source temperature class'), 'below20_c');
    fireEvent.change(screen.getByLabelText('Temperature class evidence'), { target: { value: 'source design' } });
    await user.click(screen.getByLabelText('No quality declaration applies (checked)'));
    fireEvent.change(screen.getByLabelText('Quality declaration check evidence'), { target: { value: 'register check' } });
    await user.click(screen.getByRole('button', { name: 'Calculate collective source contribution' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    expect(fetchMock.mock.calls[0][0]).toBe('/api/v1/nta8800/heat-pumps/gas-collective-source-draft/diagnose');
    expect(JSON.parse(fetchMock.mock.calls[0][1].body).input).toMatchObject({
      sourceTemperatureClass: 'below20_c', noQualityDeclarationConfirmed: true,
      noQualityDeclarationReference: 'register check', chain: { monthly: {
        sourceSystem: 'collective_groundwater_surface_or_at_least15_c',
      } },
    });
    expect(screen.getByRole('status')).toHaveTextContent('5000.00 kWh (dh)');
    fireEvent.change(screen.getByLabelText('Temperature class evidence'), { target: { value: 'revised design' } });
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
  it('links saved auxiliary input to the same twelve heat values without assigning the input carrier', async () => {
    const pump: INtaHeatPumpInput = {
      id: 'ga-1', drive: 'absorption', source: 'outdoor_air', sink: 'hydronic',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: { kind: 'normative_default', reference: null },
      gasHeatPumpForfaitDraft: {
        generatorId: 'ga-1', drive: 'absorption', application: 'utility',
        applicationReference: 'office', collectiveBuildingInstallation: false,
        externalHeatSupply: false, thermalCapacityKw: 20, capacityReference: 'plate',
        source: 'outdoor_air', sourceReference: 'plan', designSupplyTemperatureC: 35,
        designSupplyReference: 'design',
      },
      gasHeatPumpAuxDraft: {
        generatorId: 'ga-1', drive: 'absorption', nominalThermalCapacityKw: 20,
        capacityReference: 'plate', standbyElectronicsW: 10,
        burnerAuxiliaryWPerKw: 1, solutionPumpWPerKw: 0,
        coefficientsReference: 'draft', meanModulation: 1, modulationReference: 'draft',
        buildingShare: 1, buildingShareReference: 'building', forfaitCopUsed: true,
        monthHoursReference: 'hours', generatorOutputReference: 'meter',
        months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, hours: 730, generatorOutputKwh: 1000 })),
      },
    };
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({
      status: 'diagnostic_valid', annualEquipmentAuxiliaryElectricityKwh: 100.8,
      monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
        equation962UnallocatedInputTermKwh: 625, equipmentAuxiliaryElectricityKwh: 8.4 })),
      inputFingerprint: 'sha256:linked', carrierAllocationAvailable: false,
      gasInputEnergyAvailable: false, bengCalculationAvailable: false, issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<GasHeatPumpMonthlyPanel pump={pump} />);
    fireEvent.change(screen.getByLabelText('Source-system evidence'), { target: { value: 'plan' } });
    await user.click(screen.getByRole('button', { name: 'Calculate linked 9.62 and equipment electricity' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    expect(fetchMock.mock.calls[0][0]).toBe('/api/v1/nta8800/heat-pumps/gas-chain-draft/diagnose');
    const sent = JSON.parse(fetchMock.mock.calls[0][1].body).input;
    expect(sent.monthly.generatorOutputKwh[0].energyKwh).toBe(sent.auxiliary.months[0].generatorOutputKwh);
    expect(screen.getByRole('status')).toHaveTextContent('100.80 kWh');
    expect(screen.getByRole('status')).toHaveTextContent('625.00');
    expect(screen.getByRole('status')).toHaveTextContent('8.40');
    fireEvent.change(screen.getByLabelText('Generator heat (kWh) 1'), { target: { value: '900' } });
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
  it('shows collective source heat and an unallocated 9.62 term only after complete input', async () => {
    const pump: INtaHeatPumpInput = {
      id: 'ga-1', drive: 'absorption', source: 'groundwater', sink: 'hydronic',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: { kind: 'normative_default', reference: null },
      gasHeatPumpForfaitDraft: {
        generatorId: 'ga-1', drive: 'absorption', application: 'utility',
        applicationReference: 'office design', collectiveBuildingInstallation: false,
        externalHeatSupply: false, thermalCapacityKw: 40, capacityReference: 'plate',
        source: 'groundwater_aquifer', sourceReference: 'source design',
        designSupplyTemperatureC: 35, designSupplyReference: 'heating design',
      },
    };
    const sourceHeat = 1000 * (1 - 1 / 2.1);
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({
      status: 'diagnostic_valid', correctedCop: 2.1, collectiveSourceCorrectionFactor: 0.022,
      collectiveSourceHeatDerived: true,
      monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
        collectiveSourceHeatKwh: sourceHeat,
        equation962InputTermKwh: 1000 / 2.1 - sourceHeat * 0.022 })),
      inputFingerprint: 'sha256:gas-monthly', carrierAllocationAvailable: false,
      gasInputEnergyAvailable: false, bengCalculationAvailable: false, issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<GasHeatPumpMonthlyPanel pump={pump} />);
    await user.click(screen.getByRole('button', { name: 'Calculate draft 9.62 terms' }));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('twelve generator heat values');
    await user.selectOptions(screen.getByLabelText('Source system'), 'collective_groundwater_surface_or_at_least15_c');
    fireEvent.change(screen.getByLabelText('Source-system evidence'), { target: { value: 'invoices' } });
    fireEvent.change(screen.getByLabelText('Monthly generator heat evidence'), { target: { value: 'heat meter' } });
    for (let month = 1; month <= 12; month += 1) {
      fireEvent.change(screen.getByLabelText(`Generator heat (kWh) ${month}`), { target: { value: '1000' } });
    }
    await user.click(screen.getByRole('button', { name: 'Calculate draft 9.62 terms' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/nta8800/heat-pumps/gas-forfait-monthly-draft/diagnose');
    expect(JSON.parse(options.body).input).toMatchObject({
      sourceSystem: 'collective_groundwater_surface_or_at_least15_c',
      generatorOutputKwh: expect.arrayContaining([{ month: 1, energyKwh: 1000 }]),
    });
    expect(screen.getByRole('status')).toHaveTextContent('523.81');
    expect(screen.getByRole('status')).toHaveTextContent('464.67');
    expect(screen.getByRole('status')).toHaveTextContent('not assigned to gas or electricity');
    fireEvent.change(screen.getByLabelText('Generator heat (kWh) 1'), { target: { value: '900' } });
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});
