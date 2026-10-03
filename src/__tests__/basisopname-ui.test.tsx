import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, waitFor } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { renderWithProviders, userEvent } from './test-utils';

function Editor() {
  const { state } = useEnergy();
  return <><BasisopnamePanel />
    <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output></>;
}

function stored() {
  return JSON.parse(screen.getByTestId('survey').textContent ?? 'null') as {
    kind: string; survey: Record<string, any>;
  } | null;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('basisopname panel', () => {
  it('starts a dwelling survey and edits generators, solar and PV in the kernel shape', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    expect(stored()).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    expect(stored()?.kind).toBe('residential');

    // Heating: CHP with a diesel engine and a peak boiler.
    const generators = screen.getAllByRole('combobox', { name: 'Generator' });
    await user.selectOptions(generators[0], 'chp');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Prime mover' }), 'diesel_engine');
    await user.click(screen.getAllByRole('button', { name: 'Add generator' })[0]);
    let survey = stored()!.survey;
    expect(survey.heating.generator).toEqual({ kind: 'chp', electricalPowerKw: 20, engine: 'diesel_engine' });
    expect(survey.heating.additionalGenerators).toEqual([
      { generator: { kind: 'boiler', boilerType: 'hr107', insideThermalBoundary: true }, nominalPowerKw: 20 },
    ]);

    // Hot water: collective, solar water heater and PV with a roof edge.
    await user.click(screen.getByRole('checkbox', { name: 'Collective hot-water system' }));
    await user.click(screen.getByRole('button', { name: 'Add solar water heater' }));
    await user.click(screen.getByRole('button', { name: 'Add PV system' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Obstruction (§15.4.7)' }), 'roof_edge');
    survey = stored()!.survey;
    expect(survey.hotWater.collective).toEqual({});
    expect(survey.hotWater.solar[0]).toMatchObject({ id: 'zb1', collector: 'glazed', orientation: 'south' });
    expect(survey.pv[0].shading).toEqual({ method: 'roof_edge', heightM: 1, distanceM: 0.5 });

    // Vertical pipes: explicitly none.
    await user.selectOptions(screen.getByRole('combobox', { name: 'Vertical pipes (§7.2.4)' }), 'none');
    expect(stored()!.survey.verticalPipes).toEqual([]);
  }, 60000);

  it('edits ventilation controls, system E, heating strips, solar-control glass and the 90/70 declaration', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));

    // ISSO 82.1 tables 11.4–11.6.
    await user.click(screen.getByRole('checkbox', { name: 'Controls established (tables 11.4–11.6)' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'CO₂ measurement' }), 'living_room_and_main_bedroom');
    await user.selectOptions(screen.getByRole('combobox', { name: 'CO₂ control' }), 'extract');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Zoning' }), 'true');
    await user.type(screen.getByRole('textbox', { name: 'Evidence for the controls' }), 'datasheet');
    // §11.3.6 and §11.3.7.
    await user.click(screen.getByRole('checkbox', { name: 'Combined system E (decentral heat recovery in part of the zone)' }));
    await user.click(screen.getByRole('checkbox', { name: 'Grilles with electric heating strips (§11.3.7)' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Maximum temperature rise, K' }), '8');
    let survey = stored()!.survey;
    expect(survey.ventilation.controls).toEqual({
      evidenceReference: 'datasheet', co2Measurement: 'living_room_and_main_bedroom', co2Control: 'extract', zoning: true,
    });
    expect(survey.ventilation.combined).toEqual({ decentralAreaM2: 0, totalResidenceAreaM2: 0 });
    expect(survey.ventilation.grilleHeatingStrips).toEqual({ sourceReference: '', maxTemperatureRiseK: 8 });

    // p. 94: product g for solar-control glass.
    await user.click(screen.getAllByRole('checkbox', { name: 'Solar-control glass or film with product data' })[0]);
    await user.type(screen.getByRole('spinbutton', { name: 'g-value from the product data' }), '0.3');
    // Table 9.9 / erratum §4.
    await user.selectOptions(screen.getByRole('combobox', { name: 'Design temperature class (table 9.9)' }), 'c90_70');
    await user.type(screen.getByRole('textbox', { name: 'Controlled declaration for a heat pump above 70 °C' }), 'BCRG 1');
    survey = stored()!.survey;
    expect(survey.envelope.windows[0].solarControl).toEqual({ gValue: 0.3, sourceReference: '' });
    expect(survey.heating.designClass).toBe('c90_70');
    expect(survey.heating.heatPumpAbove70Declaration).toBe('BCRG 1');
  }, 60000);

  it('edits table 9.3/9.6 generators and the table 9.12 distribution', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    const generator = () => screen.getAllByRole('combobox', { name: 'Generator' })[0];

    // Collective groundwater doublet with a gas-absorption heat pump.
    await user.selectOptions(generator(), 'heat_pump');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Drive' }), 'gas_absorption');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source' }), 'groundwater');
    await user.click(screen.getByRole('checkbox', { name: 'Collective heat-pump source (invoices or design data)' }));
    await user.type(screen.getByRole('textbox', { name: 'Evidence of the collective source' }), 'invoice');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Groundwater source system (unknown: recirculation)' }), 'doublet');
    await user.type(screen.getByRole('spinbutton', { name: 'Source temperature, °C (unknown: ground)' }), '11');
    expect(stored()!.survey.heating.generator).toMatchObject({
      kind: 'heat_pump', drive: 'gas_absorption', source: 'groundwater', collectiveSourceReference: 'invoice',
      groundwaterSystem: 'doublet', sourceTemperatureC: 11,
    });

    // Local gas heating without a flue, and gas air heaters.
    await user.selectOptions(generator(), 'local_fired');
    await user.click(screen.getByRole('checkbox', { name: 'With flue-gas exhaust' }));
    expect(stored()!.survey.heating.generator).toEqual({
      kind: 'local_fired', appliance: 'gas_heater', flueGasExhaust: false, electricityConnected: null,
    });
    await user.selectOptions(generator(), 'gas_air_heater');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Air heater type' }), 'hr107');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Pilot flame (unknown: yes)' }), 'false');
    expect(stored()!.survey.heating.generator).toEqual({
      kind: 'gas_air_heater', heaterType: 'hr107', pilotFlame: false, count: null,
    });

    // One-pipe loop with insulated pipes from 1990.
    await user.selectOptions(screen.getByRole('combobox', { name: 'Distribution system (unknown: two-pipe)' }), 'one_pipe');
    const count = screen.getByRole('spinbutton', { name: 'Emitters on the one-pipe loop' });
    await user.clear(count);
    await user.type(count, '8');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Pipes insulated (unknown: no)' }), 'true');
    await user.type(screen.getByRole('spinbutton', { name: 'Year of insulation (unknown: construction year)' }), '1990');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Valves and brackets insulated (unknown: no)' }), 'true');
    const heating = stored()!.survey.heating;
    expect(heating.distributionType).toEqual({ kind: 'one_pipe', emitterCount: 8 });
    expect(heating.pipeInsulation).toEqual({ insulated: true, insulationYear: 1990, fittingsInsulated: true });
  }, 60000);

  it('clears hidden heat-pump and local-heater answers when the source or appliance changes', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    const generator = () => screen.getAllByRole('combobox', { name: 'Generator' })[0];

    // A collective groundwater source, then an air source: the water-only
    // answers go (collective_source_water_based_only otherwise).
    await user.selectOptions(generator(), 'heat_pump');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source' }), 'groundwater');
    await user.click(screen.getByRole('checkbox', { name: 'Collective heat-pump source (invoices or design data)' }));
    await user.type(screen.getByRole('textbox', { name: 'Evidence of the collective source' }), 'invoice');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Groundwater source system (unknown: recirculation)' }), 'doublet');
    await user.type(screen.getByRole('spinbutton', { name: 'Source temperature, °C (unknown: ground)' }), '11');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source' }), 'outdoor_air');
    let heatPump = stored()!.survey.heating.generator;
    expect(heatPump.source).toBe('outdoor_air');
    expect(heatPump).not.toHaveProperty('collectiveSourceReference');
    expect(heatPump).not.toHaveProperty('groundwaterSystem');
    expect(heatPump).not.toHaveProperty('sourceTemperatureC');

    // Unticking the collective source of surface water drops its temperature.
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source' }), 'surface_water');
    await user.click(screen.getByRole('checkbox', { name: 'Collective heat-pump source (invoices or design data)' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Source temperature, °C (unknown: ground)' }), '9');
    await user.click(screen.getByRole('checkbox', { name: 'Collective heat-pump source (invoices or design data)' }));
    heatPump = stored()!.survey.heating.generator;
    expect(heatPump).not.toHaveProperty('collectiveSourceReference');
    expect(heatPump).not.toHaveProperty('sourceTemperatureC');

    // A steam boiler's fuel goes with the appliance (local_heater_fuel_contradiction otherwise).
    await user.selectOptions(generator(), 'local_fired');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Appliance' }), 'steam_boiler');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Fuel' }), 'oil');
    expect(stored()!.survey.heating.generator.fuel).toBe('oil');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Appliance' }), 'oil_heater');
    const local = stored()!.survey.heating.generator;
    expect(local.appliance).toBe('oil_heater');
    expect(local).not.toHaveProperty('fuel');
  }, 60000);

  it('edits houseboats, sunrooms and rooflights in the dwelling survey', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Building type' }), 'floating');
    await user.click(screen.getByRole('checkbox', { name: 'New berth from 2018' }));
    await user.selectOptions(screen.getAllByRole('combobox', { name: 'Boundary' })[0], 'sunroom');
    await user.click(screen.getByRole('button', { name: 'Add rooflight with quality declaration' }));
    const survey = stored()!.survey;
    expect(survey.envelope.buildingKind).toEqual({ kind: 'floating', newBerthSince2018: true });
    expect(survey.envelope.surfaces[0].boundary).toEqual({ kind: 'sunroom' });
    expect(survey.envelope.rooflights[0]).toMatchObject({ areaM2: 1, uValue: 2.5, glass: 'double', qualityDeclarationReference: '' });
  });

  it('adds a utility hot-water system with served areas and AHU coils', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    await user.click(screen.getByRole('button', { name: 'Add hot-water system' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Served function' }), 'assembly_without_day_care');
    await user.click(screen.getByRole('checkbox', { name: 'Heating connected (reheating coil)' }));
    const survey = stored()!.survey;
    expect(survey.additionalHotWaterSystems[0]).toMatchObject({
      generator: { kind: 'electric_instantaneous' }, showerHeatRecovery: 'none',
      servedAreas: [{ function: 'assembly_without_day_care', areaM2: 0 }],
    });
    expect(survey.ventilation.ahu.heatingConnected).toBe(true);
  });

  it('adds a dwelling cooling system in the ISSO 82.1 chapter 10 shape', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.click(screen.getByRole('checkbox', { name: 'Building-bound cooling system present' }));
    let survey = stored()!.survey;
    expect(survey.coolingPresent).toBe(true);
    expect(survey.cooling).toMatchObject({ generator: 'room_air_conditioner', emitter: 'split_indoor_units_on_wall', waterBased: false });
    await user.selectOptions(screen.getByRole('combobox', { name: 'Cooling generator' }), 'closed_ground_loop');
    await user.click(screen.getByRole('checkbox', { name: 'Collective cooling generator (several dwellings)' }));
    survey = stored()!.survey;
    expect(survey.cooling.generator).toBe('closed_ground_loop');
    expect(survey.coolingCollective).toBe(true);
    await user.click(screen.getByRole('checkbox', { name: 'Building-bound cooling system present' }));
    survey = stored()!.survey;
    expect(survey.cooling).toBeNull();
    expect(survey.coolingPresent).toBe(false);
  });

  it('runs the kernel route and records a reason per applied default', async () => {
    const assessment = {
      status: 'calculated_unverified', scope: 'x', source: 'ISSO 82.1',
      appliedDefaults: [{ rule: 'chp_engine_unknown_gas', path: 'heating.generator.engine', value: 'gas_engine', source: 'ISSO 82.1 p. 113 (table 9.7)' }],
      warnings: [], issues: [], derivedInput: null,
      performance: { indicativeLabelClass: 'C', needIndicatorKwhPerM2Year: 120, primaryFossilIndicatorKwhPerM2Year: 210.5, renewableSharePercent: 4 },
      referenceVerified: false,
    };
    const fetchMock = vi.fn(async () => ({ json: async () => assessment }));
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    await user.click(screen.getByRole('button', { name: 'Calculate survey' }));
    expect(await screen.findByText('ISSO 82.1 p. 113 (table 9.7)')).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/nta8800/opname/utility', expect.anything());
    expect(screen.getByText('C')).toBeInTheDocument();
    await user.type(screen.getByRole('textbox', { name: 'Reason for the forfait heating.generator.engine' }), 'plate missing');
    expect(stored()!.survey.inklapRedenen).toEqual({ 'heating.generator.engine': 'plate missing' });
    // The result stays while the reason is typed.
    expect(screen.getByText('ISSO 82.1 p. 113 (table 9.7)')).toBeInTheDocument();
  });

  it('does not show an assessment for a survey changed during the request', async () => {
    let resolveResponse!: (value: { json: () => Promise<unknown> }) => void;
    const fetchMock = vi.fn(() => new Promise<{ json: () => Promise<unknown> }>((resolve) => {
      resolveResponse = resolve;
    }));
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.click(screen.getByRole('button', { name: 'Calculate survey' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    await user.clear(screen.getByRole('spinbutton', { name: 'Construction year' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Construction year' }), '2000');
    await act(async () => resolveResponse({ json: async () => ({
      status: 'calculated_unverified', appliedDefaults: [], warnings: [], issues: [],
      performance: { indicativeLabelClass: 'A' }, referenceVerified: false,
    }) }));
    expect(screen.queryByText('A')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Survey result')).not.toBeInTheDocument();
  });
});
