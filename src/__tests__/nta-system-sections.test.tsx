import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import {
  CoolingPerformanceFields, HotWaterGeneratorFields, HotWaterGeneratorsFields, SolarWaterHeaterFields, SpaceGeneratorFields,
  WindowObstructionFields,
} from '../components/NtaPerformancePanel/NtaSystemSections';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import {
  collectorObstructionTemplate, coolingPerformanceTemplate, multipleGeneratorsTemplate, solarWaterHeaterTemplate,
  testedSolarMethod, windowObstructionTemplate,
} from '../core/nta/NtaSystemTemplates';
import { renderWithProviders, userEvent } from './test-utils';

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'residential', address: '', city: '',
  zones: [], heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [],
  solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

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

describe('NTA system templates', () => {
  it('use the kernel field names', () => {
    expect(multipleGeneratorsTemplate()).toMatchObject({
      kind: 'multiple', addedPreferredGenerator: false, sourceReference: '',
      generators: [
        { preference: 1, nominalPowerKw: null, generator: { kind: 'heat_pump_forfait', sourceSystem: 'individual' } },
        { preference: 2, nominalPowerKw: null, generator: { kind: 'gas_boiler', boiler: { fuel: 'natural_gas' } } },
      ],
    });
    expect(solarWaterHeaterTemplate()).toEqual({
      id: 'solar-1', solarUse: 'water_heating', count: 1, pvt: null, sourceReference: '',
      method: {
        method: 'calculated', solarType: 'preheater',
        collectors: { moduleAreaM2: null, moduleCount: 1, orientation: 'south', tiltDeg: 45, obstruction: { method: 'minimal' },
          efficiency: { method: 'forfait', collector: 'glazed' }, loopPipes: { method: 'forfait' } },
        storage: { totalVolumeL: null, loss: { method: 'unknown_label', producedFrom2018: false } },
      },
    });
    expect(testedSolarMethod()).toMatchObject({ method: 'tested', testPoints: [{ annualDemandKwh: null, auxiliaryKwh: null }] });
    expect(collectorObstructionTemplate('roof_edge')).toEqual({ method: 'roof_edge', heightM: null, distanceM: null });
    expect(windowObstructionTemplate('side_obstruction')).toEqual({
      method: 'side_obstruction', side: 'both', relativeWidth: null, coolingHeightCondition: false });
    expect(coolingPerformanceTemplate('en14825', false)).toMatchObject({
      method: 'en14825', testPoints: [{ partLoadPercent: 100 }, { partLoadPercent: 74 }, { partLoadPercent: 47 }, { partLoadPercent: 21 }] });
    expect(coolingPerformanceTemplate('en14511', true)).toMatchObject({ method: 'en14511', roomUnitType: 'split_inverter' });
    expect(coolingPerformanceTemplate('en14511', false)).not.toHaveProperty('roomUnitType');
    expect(coolingPerformanceTemplate('', false)).toBeNull();
  });
});

describe('NTA system sections', () => {
  it('edits several space-heating generators (9.6.1)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ generator: { kind: 'gas_boiler' } }}
      body={(draft, change) => <SpaceGeneratorFields draft={draft} change={change} base={['generator']} project={project} allowMultiple />} />);
    await user.selectOptions(screen.getByLabelText('Generator type'), 'multiple');
    const powers = screen.getAllByLabelText('Nominal heating power, kW');
    await user.type(powers[0], '6');
    await user.type(powers[1], '18');
    // The nested generator of the second part becomes external heat.
    await user.selectOptions(screen.getAllByLabelText('Generator type')[2], 'external_heat');
    await user.click(screen.getByRole('button', { name: 'Add generator' }));
    await user.click(screen.getByLabelText('Preferred generator added later (renovation, 9.58/9.59)'));
    const generator = current().generator;
    expect(generator.kind).toBe('multiple');
    expect(generator.addedPreferredGenerator).toBe(true);
    expect(generator.generators.map((part: Draft) => [part.preference, part.nominalPowerKw, (part.generator as Draft).kind]))
      .toEqual([[1, 6, 'heat_pump_forfait'], [2, 18, 'external_heat'], [3, null, 'gas_boiler']]);
    // A nested generator cannot itself be split.
    expect(screen.getAllByLabelText('Generator type')[1].querySelector('option[value="multiple"]')).toBeNull();
    await user.click(screen.getAllByRole('button', { name: 'Remove' })[2]);
    expect(current().generator.generators).toHaveLength(2);
  });

  it('adds hot-water generators with a series arrangement (13.8.2)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ hotWater: { generator: { kind: 'gas_appliance', appliance: null } } }}
      body={(draft, change) => <HotWaterGeneratorsFields draft={draft} change={change} />} />);
    await user.type(screen.getByLabelText('Nominal power, kW (13.141)'), '24');
    await user.click(screen.getByRole('button', { name: 'Add hot-water generator (13.8.2)' }));
    await user.selectOptions(screen.getByLabelText('Hot-water generator'), 'heat_pump');
    await user.selectOptions(screen.getByLabelText('Series arrangement (13.141a–d)'), 'hotfill_electric_boiler');
    const hotWater = current().hotWater;
    expect(hotWater.nominalPowerKw).toBe(24);
    expect(hotWater.additionalGenerators).toEqual([{
      generator: { kind: 'heat_pump', exhaustAirSource: false, measuredClass: 'class4' }, nominalPowerKw: null, equipmentReference: '' }]);
    expect(hotWater.series).toEqual({ kind: 'hotfill_electric_boiler' });
    await user.selectOptions(screen.getByLabelText('Series arrangement (13.141a–d)'), 'collective_first_also_heating');
    expect(current().hotWater.series).toEqual({ kind: 'collective_first_also_heating', maximumSupplyC: Array(12).fill(null) });
  });

  it('edits a two-profile appliance, the heating-system generator, exhaust air and a declared share', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ hotWater: { generator: { kind: 'electric_boiler' } } }}
      body={(draft, change) => <>
        <HotWaterGeneratorFields draft={draft} change={change} base={['hotWater', 'generator']} />
        <HotWaterGeneratorsFields draft={draft} change={change} />
      </>} />);
    await user.selectOptions(screen.getAllByLabelText('Hot-water generator')[0], 'measured_two_profiles');
    await user.selectOptions(screen.getByLabelText('Test standard'), 'en16147_heat_pump');
    const inputs = screen.getAllByLabelText('Electricity Q_elec, kWh/day');
    await user.type(inputs[0], '2.6');
    await user.type(inputs[1], '4.4');
    await user.click(screen.getByLabelText('Combi appliance'));
    await user.click(screen.getByLabelText('exhaust-air source'));
    await user.selectOptions(screen.getByLabelText('Mixed-air correction (combi heat pump on outdoor and return air)'), 'en14511');
    await user.type(screen.getByLabelText('COP at condition 2 (A7/W55)'), '3');
    let generator = current().hotWater.generator;
    expect(generator).toMatchObject({
      kind: 'measured_two_profiles', standard: 'en16147_heat_pump', combi: true,
      low: { profile: 'm', deliveredKwhPerDay: 5.845, inputKwhPerDay: 2.6 },
      high: { profile: 'l', deliveredKwhPerDay: 11.655, inputKwhPerDay: 4.4 },
      mixedAir: { method: 'en14511', copCondition2: 3, condenserOutC: 55, evaporatorInC: 7 },
    });
    // 13.144a/13.148: exhaust-air use with the declared flow.
    await user.click(screen.getByLabelText('Uses ventilation return air (13.144a/13.148)'));
    await user.type(screen.getByLabelText('Ventilation flow q_ve;hp;W from a quality declaration, m³/h'), '150');
    expect(current().hotWater.exhaustAir).toMatchObject({ ventilationSuitable: true, declaredFlowM3PerH: 150 });
    // 13.146 declared share with two classes.
    await user.click(screen.getByLabelText('Energy share from a quality declaration (13.146)'));
    await user.type(screen.getByLabelText('Q_W;dis;nren;an of the class, kWh'), '1000');
    await user.type(screen.getByLabelText('Share F_W;gen'), '0.6');
    await user.click(screen.getByRole('button', { name: 'Add class' }));
    expect(current().hotWater.declaredShare.points).toEqual([{ annualKwh: 1000, share: 0.6 }, { annualKwh: null, share: null }]);
    // PFHRD only for gas appliances.
    await user.selectOptions(screen.getByLabelText('Test standard'), 'en13203_gas');
    await user.click(screen.getByLabelText('PFHRD (flue heat recovery) present'));
    await user.type(screen.getByLabelText('Q_gas;indirect, kWh/day (net value)'), '0.5');
    generator = current().hotWater.generator;
    expect(generator.pfhrd).toEqual({ indirectGasKwhPerDay: 0.5, heatingGasKwhPerDay: null, sourceReference: '' });
    // 13.8.4.9.3: hot water from the heating system has no own fields.
    await user.selectOptions(screen.getAllByLabelText('Hot-water generator')[0], 'heating_system');
    expect(current().hotWater.generator).toEqual({ kind: 'heating_system' });
  });

  it('edits a calculated and a tested solar water heater (13.7)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ hotWater: { generator: { kind: 'electric_boiler' } } }}
      body={(draft, change) => <SolarWaterHeaterFields draft={draft} change={change} />} />);
    await user.click(screen.getByRole('button', { name: 'Add solar water heater' }));
    await user.type(screen.getByLabelText('Collector module area, m²'), '2.5');
    await user.selectOptions(screen.getByLabelText('Orientation'), 'south_west');
    await user.selectOptions(screen.getByLabelText('Obstruction factor F_sh;obst'), 'roof_edge');
    await user.type(screen.getByLabelText('Roof edge height above the panel bottom, m'), '0.8');
    await user.selectOptions(screen.getByLabelText('Storage loss'), 'label');
    await user.selectOptions(screen.getByLabelText('PVT collector (13.7.2.4)'), 'unglazed');
    let solar = current().hotWater.solar[0];
    expect(solar.method.collectors).toMatchObject({ moduleAreaM2: 2.5, orientation: 'south_west',
      obstruction: { method: 'roof_edge', heightM: 0.8, distanceM: null } });
    expect(solar.method.storage.loss).toEqual({ method: 'label', label: 'c' });
    expect(solar.pvt).toBe('unglazed');
    await user.selectOptions(screen.getByLabelText('Method'), 'tested');
    await user.type(screen.getByLabelText('Test: annual demand, kWh'), '2000');
    await user.click(screen.getByRole('button', { name: 'Add test point' }));
    solar = current().hotWater.solar[0];
    expect(solar.method).toMatchObject({ method: 'tested', orientation: 'south', obstruction: { method: 'minimal' } });
    expect(solar.method.testPoints).toEqual([
      { annualDemandKwh: 2000, solarOutputKwh: null, auxiliaryKwh: null },
      { annualDemandKwh: null, solarOutputKwh: null, auxiliaryKwh: null }]);
    expect(solar.method).not.toHaveProperty('collectors');
  });

  it('rates a compression generator with method 1 or 2 (10.5.4/10.5.5)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ cooling: { generators: [{ id: 'c', generator: { kind: 'compression' } }] } }}
      body={(draft, change) => <CoolingPerformanceFields draft={draft} change={change} base={['cooling', 'generators', 0, 'generator']} />} />);
    await user.selectOptions(screen.getByLabelText('Heat rejection (condenser)'), 'dry_cooler');
    await user.selectOptions(screen.getByLabelText('Efficiency method'), 'en14825');
    await user.type(screen.getByLabelText('Nominal EER'), '3.2');
    await user.type(screen.getAllByLabelText('EER')[0], '3.4');
    await user.click(screen.getByLabelText('Fifth test point (10.63)'));
    let generator = current().cooling.generators[0].generator;
    expect(generator.heatRejection).toBe('dry_cooler');
    expect(generator.performance).toMatchObject({ method: 'en14825', nominalEer: 3.2, minimumCapacityKw: null });
    expect(generator.performance.testPoints[0]).toEqual({ partLoadPercent: 100, eer: 3.4, evaporatorOutletC: null, condenserInletC: null });
    expect(generator.performance.fifthPoint).toMatchObject({ partLoadPercent: 47 });
    await user.selectOptions(screen.getByLabelText('Efficiency method'), 'en14511');
    generator = current().cooling.generators[0].generator;
    expect(generator.performance).toEqual({ method: 'en14511', nominalEer: null, nominalCapacityKw: null,
      nominalEvaporatorOutletC: 7, nominalCondenserInletC: 35, sourceReference: '' });
    expect(screen.queryByLabelText('Room unit type (table 10.19)')).toBeNull();
  });

  it('asks the room unit type for a room air conditioner', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ g: { kind: 'room_air_conditioner' } }}
      body={(draft, change) => <CoolingPerformanceFields draft={draft} change={change} base={['g']} />} />);
    expect(screen.queryByLabelText('Heat rejection (condenser)')).toBeNull();
    await user.selectOptions(screen.getByLabelText('Efficiency method'), 'en14511');
    await user.selectOptions(screen.getByLabelText('Room unit type (table 10.19)'), 'multi_split_staged');
    expect(current().g.performance.roomUnitType).toBe('multi_split_staged');
  });

  it('selects window obstruction situations b–g (§17.3)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ windowSolar: { obstruction: { method: 'minimal' } } }}
      body={(draft, change) => <WindowObstructionFields draft={draft} change={change} />} />);
    await user.selectOptions(screen.getByLabelText('Obstruction factor F_sh;obst'), 'overhang');
    await user.type(screen.getByLabelText('Relative height (h/a)'), '0.4');
    expect(current().windowSolar.obstruction).toEqual({ method: 'overhang', relativeHeight: 0.4 });
    await user.selectOptions(screen.getByLabelText('Obstruction factor F_sh;obst'), 'full');
    await user.click(screen.getByLabelText('Conditions for the cooling table met'));
    expect(current().windowSolar.obstruction).toEqual({ method: 'full', coolingConditionsMet: true });
    await user.selectOptions(screen.getByLabelText('Obstruction factor F_sh;obst'), 'declared');
    await user.type(screen.getByLabelText('Heating balance per month 1'), '0.9');
    expect(current().windowSolar.obstruction.heating[0]).toBe(0.9);
    expect(current().windowSolar.obstruction.cooling).toHaveLength(12);
  });
});
