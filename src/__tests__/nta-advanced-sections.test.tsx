import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import { HotWaterGeneratorFields, SpaceGeneratorFields } from '../components/NtaPerformancePanel/NtaSystemSections';
import {
  AirHeatersFields, BacsAndSupplyFields, BBL_FUNCTIONS, FunctionAreasFields, LABEL_FUNCTIONS,
} from '../components/NtaPerformancePanel/NtaAdvancedSections';
import { NumberField, TriStateField, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { EmitterFields, PassiveCoolingFields } from '../components/BasisopnamePanel/BasisopnamePanel';
import {
  airHeatersTemplate, bacsTemplate, hotWaterGeneratorTemplate, microChpTemplate, regenerationTemplate, spaceGeneratorTemplate,
} from '../core/nta/NtaSystemTemplates';
import { en } from '../i18n/en';
import { renderWithProviders, userEvent } from './test-utils';

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'residential', address: '', city: '',
  zones: [], heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [],
  solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

const t = (key: string) => en[key] ?? key;

type Body = (draft: Draft, change: (path: Path, value: unknown) => void) => ReactElement;

function Harness({ initial, body }: { initial: Draft; body: Body }) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    {body(draft, change)}
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const current = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');

describe('advanced NTA templates', () => {
  it('use the kernel field names', () => {
    expect(spaceGeneratorTemplate('gas_heat_pump')).toEqual({
      kind: 'gas_heat_pump', table: 'residential_at_most25_kw', source: 'outdoor_air', designSupplyTemperatureC: null,
      sourceCorrectionFactor: null, equipmentReference: '',
      auxiliary: { electricallyConnectedDevices: null, nominalPowerKw: null, sourceReference: '' },
    });
    expect(regenerationTemplate()).toEqual({ freeCoolingFromSource: false, solar: [], sourceReference: '' });
    expect(airHeatersTemplate('indirect')).toEqual({
      kind: { kind: 'indirect', roomHeightAbove8M: null, warmAirReturn: null, ecMotor: null },
      designHeatLoadW: null, sourceReference: '',
    });
    expect(microChpTemplate()).toMatchObject({
      kind: 'stirling_engine', fuel: 'natural_gas', location: 'heated_space',
      fullLoad: { thermalPowerKw: null }, chpOnly: { thermalPowerKw: null }, testReportReference: '',
    });
    expect(bacsTemplate(false)).toMatchObject({
      buildingUse: 'utility', systemInventoryComplete: false,
      systems: [{ service: 'heating', generators: [{ nominalThermalCapacityKw: null }] }],
      bacs: { present: false, sourceReference: '' },
    });
  });
});

describe('advanced NTA calculation fields', () => {
  it('edits a gas heat pump in the kernel shape', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ generator: spaceGeneratorTemplate('gas_boiler') }}
      body={(draft, change) => <SpaceGeneratorFields draft={draft} change={change} base={['generator']} project={project} />} />);
    await user.selectOptions(screen.getByRole('combobox', { name: 'Generator type' }), 'gas_heat_pump');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Table' }), 'utility_collective_or_above25_kw');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source' }), 'ground');
    await user.type(screen.getByRole('spinbutton', { name: 'Design supply temperature (°C)' }), '45');
    expect(current().generator).toEqual({
      kind: 'gas_heat_pump', table: 'utility_collective_or_above25_kw', source: 'ground', designSupplyTemperatureC: 45,
      sourceCorrectionFactor: null, equipmentReference: '',
      auxiliary: { electricallyConnectedDevices: null, nominalPowerKw: null, sourceReference: '' },
    });
  });

  it('adds annex V regeneration to a forfait heat pump and removes it again', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ generator: spaceGeneratorTemplate('heat_pump_forfait') }}
      body={(draft, change) => <SpaceGeneratorFields draft={draft} change={change} base={['generator']} project={project} />} />);
    await user.click(screen.getByRole('checkbox', { name: 'Ground-source regeneration (annex V)' }));
    await user.click(screen.getByRole('checkbox', { name: 'Free cooling from the same source' }));
    await user.click(screen.getByRole('button', { name: 'Add collector field' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Collector area (m²)' }), '6');
    expect(current().generator.regeneration).toEqual({
      freeCoolingFromSource: true, sourceReference: '',
      solar: [{ collectorAreaM2: 6, azimuthDeg: 180, tiltDeg: 45, declaredEfficiency: null, sourceReference: '' }],
    });
    await user.click(screen.getByRole('checkbox', { name: 'Ground-source regeneration (annex V)' }));
    expect(current().generator).not.toHaveProperty('regeneration');
  });

  it('switches a CHP between method 2 and method 1 (exclusive)', { timeout: 60000 }, async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ generator: spaceGeneratorTemplate('chp') }}
      body={(draft, change) => <SpaceGeneratorFields draft={draft} change={change} base={['generator']} project={project} />} />);
    await user.selectOptions(screen.getByRole('combobox', { name: 'CHP method' }), 'method1');
    expect(current().generator).toMatchObject({ kind: 'chp', chp: null, method1: { kind: 'stirling_engine' } });
    await user.selectOptions(screen.getByRole('combobox', { name: 'Micro-CHP type' }), 'pem_fuel_cell');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Hydraulic connection' }), 'decoupled');
    const thermal = screen.getAllByRole('spinbutton', { name: 'Thermal power, kW' });
    await user.type(thermal[0], '6');
    await user.type(thermal[1], '5');
    await user.type(screen.getAllByRole('spinbutton', { name: 'Electric efficiency' })[0], '0.35');
    await user.click(screen.getByRole('checkbox', { name: 'Storage outside the test configuration (9.6.6.2.2.8)' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Storage heat loss, W/K' }), '1.8');
    await user.type(screen.getByRole('textbox', { name: 'Test report (NEN-EN 50465)' }), 'TR-1');
    expect(current().generator.method1).toMatchObject({
      kind: 'pem_fuel_cell', fuel: 'natural_gas', hydraulics: 'decoupled', testReportReference: 'TR-1',
      fullLoad: { thermalPowerKw: 6, electricEfficiency: 0.35 }, chpOnly: { thermalPowerKw: 5 },
      storage: { lossWPerK: 1.8, setTemperatureC: null, chargingAuxiliaryW: null, sourceReference: '' },
    });
    await user.click(screen.getByRole('checkbox', { name: 'Storage outside the test configuration (9.6.6.2.2.8)' }));
    expect(current().generator.method1.storage).toBeNull();
    await user.selectOptions(screen.getByRole('combobox', { name: 'CHP method' }), 'method2');
    expect(current().generator).toMatchObject({ chp: { powerKw: null, builtAfter2006: true }, method1: null });
  });

  it('marks a hot-water heat pump on the same ground source', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ hotWater: { generator: hotWaterGeneratorTemplate('heat_pump') } }}
      body={(draft, change) => <HotWaterGeneratorFields draft={draft} change={change} base={['hotWater', 'generator']} />} />);
    await user.click(screen.getByRole('checkbox', { name: 'same regenerated ground source as space heating (annex V)' }));
    expect(current().hotWater.generator).toEqual({
      kind: 'heat_pump', exhaustAirSource: false, measuredClass: 'class4', sameGroundSource: true });
  });

  it('edits air heaters with tri-state answers', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ emission: { system: 'air_heating' } }}
      body={(draft, change) => <AirHeatersFields draft={draft} change={change} />} />);
    await user.selectOptions(screen.getByRole('combobox', { name: 'Air heaters (tables 9.12/9.13)' }), 'indirect');
    await user.selectOptions(screen.getByRole('combobox', { name: 'EC motor' }), 'true');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Warm-air return' }), 'false');
    await user.type(screen.getByRole('spinbutton', { name: /Design heat load/ }), '12000');
    expect(current().emission).toEqual({ system: 'air_heating', airHeaters: {
      kind: { kind: 'indirect', roomHeightAbove8M: null, warmAirReturn: false, ecMotor: true },
      designHeatLoadW: 12000, sourceReference: '' } });
    await user.selectOptions(screen.getByRole('combobox', { name: 'Air heaters (tables 9.12/9.13)' }), 'direct');
    expect(current().emission.airHeaters.kind).toEqual({ kind: 'direct', radialFan: null });
    expect(current().emission.airHeaters.designHeatLoadW).toBe(12000);
    await user.selectOptions(screen.getByRole('combobox', { name: 'Air heaters (tables 9.12/9.13)' }), '');
    expect(current().emission).toEqual({ system: 'air_heating' });
  });

  it('edits label and Bbl use functions with their areas', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}} body={(draft, change) => <>
      <FunctionAreasFields draft={draft} change={change} base="labelFunctions" functions={LABEL_FUNCTIONS} prefix="nta.form.labelFn" />
      <FunctionAreasFields draft={draft} change={change} base="bblFunctions" functions={BBL_FUNCTIONS} prefix="nta.form.bblFn" />
      <NumberField draft={draft} onChange={change} path={['identicalSystems']} label="identical" step="1" />
      <NumberField draft={draft} onChange={change} path={['constructionYear']} label="year" step="1" />
      <TriStateField draft={draft} onChange={change} path={['fossilAppliancesOutsideCalculation']} label="fossil" yes="yes" no="no" />
    </>} />);
    await user.click(screen.getByRole('button', { name: 'Add label function' }));
    await user.click(screen.getByRole('button', { name: 'Add label function' }));
    await user.selectOptions(screen.getAllByRole('combobox', { name: 'Label function (§5.3.1)' })[1], 'office');
    const areas = screen.getAllByRole('spinbutton', { name: 'Usable area (m²)' });
    await user.type(areas[0], '120');
    await user.type(areas[1], '80');
    await user.click(screen.getByRole('button', { name: 'Add Bbl use function' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Bbl use function (art. 4.149(2))' }), 'lodging_in_lodging_building');
    await user.type(screen.getByRole('spinbutton', { name: 'identical' }), '3');
    await user.type(screen.getByRole('spinbutton', { name: 'year' }), '1975');
    await user.selectOptions(screen.getByRole('combobox', { name: 'fossil' }), 'false');
    expect(current()).toEqual({
      labelFunctions: [{ function: 'residential', areaM2: 120 }, { function: 'office', areaM2: 80 }],
      bblFunctions: [{ function: 'lodging_in_lodging_building', areaM2: null }],
      identicalSystems: 3, constructionYear: 1975, fossilAppliancesOutsideCalculation: false,
    });
    await user.click(screen.getAllByRole('button', { name: 'Remove' })[0]);
    expect(current().labelFunctions).toEqual([{ function: 'office', areaM2: 80 }]);
  });

  it('edits BACS systems and evidence, and annex P external supply as JSON', { timeout: 60000 }, async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}}
      body={(draft, change) => <BacsAndSupplyFields draft={draft} change={change} residential />} />);
    await user.click(screen.getByRole('checkbox', { name: /Systems and BACS evidence/ }));
    expect(current().bacs).toEqual(bacsTemplate(true));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Building use for §5.5.8' }), 'utility');
    await user.click(screen.getByRole('checkbox', { name: 'All heating and cooling systems are listed' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Nominal thermal capacity, kW' }), '60');
    await user.click(screen.getByRole('button', { name: 'Add system' }));
    await user.selectOptions(screen.getAllByRole('combobox', { name: 'Service' })[1], 'cooling');
    await user.click(screen.getByRole('checkbox', { name: 'BACS present' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Automatic controls class (NEN-EN 15232)' }), 'B');
    expect(current().bacs).toEqual({
      buildingUse: 'utility', systemInventoryComplete: true,
      bacs: { present: true, automaticControlsClass: 'B', sourceReference: '' },
      systems: [
        { id: 'heating-1', service: 'heating', sourceReference: '',
          generators: [{ id: 'generator-1', nominalThermalCapacityKw: 60, sourceReference: '' }] },
        { id: 'system-2', service: 'cooling', sourceReference: '',
          generators: [{ id: 'generator-1', nominalThermalCapacityKw: null, sourceReference: '' }] },
      ],
    });
    await user.click(screen.getAllByRole('button', { name: 'Remove system' })[1]);
    expect(current().bacs.systems).toHaveLength(1);
    const supply = screen.getByRole('textbox', { name: 'External supply, annex P (JSON)' });
    const value = { heating: { method: 'forfait' }, areaElectricity: [] };
    fireEvent.change(supply, { target: { value: JSON.stringify(value) } });
    fireEvent.blur(supply);
    expect(current().externalSupply).toEqual(value);
    fireEvent.change(supply, { target: { value: '' } });
    fireEvent.blur(supply);
    expect(current()).not.toHaveProperty('externalSupply');
    await user.click(screen.getByRole('checkbox', { name: /Systems and BACS evidence/ }));
    expect(current()).not.toHaveProperty('bacs');
  });
});

describe('basisopname air heating and passive cooling', () => {
  it('records the table 9.16 air-heater type only with air heating', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ heating: { emitters: 'radiators', sourceReference: '' } }}
      body={(draft, change) => <EmitterFields draft={draft} change={change} t={t} />} />);
    expect(screen.queryByRole('combobox', { name: /Air-heating type/ })).toBeNull();
    await user.selectOptions(screen.getByRole('combobox', { name: 'Emitters' }), 'air_heating');
    expect(current().heating).toEqual({ emitters: 'air_heating', airHeating: null, sourceReference: '' });
    await user.selectOptions(screen.getByRole('combobox', { name: /Air-heating type/ }), 'direct');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Radial fan' }), 'true');
    await user.type(screen.getByRole('spinbutton', { name: 'Number of air heaters' }), '4');
    expect(current().heating.airHeating).toEqual({ kind: 'direct', radialFan: true, count: 4 });
    expect(screen.getByRole('combobox', { name: /Air-heating type/ })).toHaveValue('direct');
    await user.selectOptions(screen.getByRole('combobox', { name: /Air-heating type/ }), 'via_air_handling_unit');
    expect(current().heating.airHeating).toEqual({ kind: 'via_air_handling_unit' });
    await user.selectOptions(screen.getByRole('combobox', { name: 'Emitters' }), 'floor_heating');
    expect(current().heating).toEqual({ emitters: 'floor_heating', sourceReference: '' });
  });

  it('records passive cooling with its evidence and installed capacity', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ ventilation: { principle: 'balanced', sourceReference: '' } }}
      body={(draft, change) => <PassiveCoolingFields draft={draft} change={change} t={t} />} />);
    await user.click(screen.getByRole('checkbox', { name: /Passive cooling proven/ }));
    await user.type(screen.getByRole('textbox', { name: /Project document/ }), 'PD-12');
    await user.type(screen.getByRole('spinbutton', { name: /Installed capacity/ }), '150');
    expect(current().ventilation).toEqual({ principle: 'balanced', sourceReference: '',
      passiveCooling: { evidenceReference: 'PD-12', installedCapacityDm3PerS: 150 } });
    await user.click(screen.getByRole('checkbox', { name: /Passive cooling proven/ }));
    expect(current().ventilation.passiveCooling).toBeNull();
  });
});
