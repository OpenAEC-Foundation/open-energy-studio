import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import {
  CollectiveAndRenewableFields, CoolingSystemsFields, DeclaredFlowsFields, HumidifiersFields, SpaceHeatingSolarFields, SunroomsFields,
} from '../components/NtaPerformancePanel/NtaProjectExtras';
import { ExternalSupplyFields } from '../components/NtaPerformancePanel/NtaExternalSupply';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { withoutNulls } from '../core/nta/KernelInput';
import { annexPGeneratorKindTemplate, annexPRouteTemplate, coolingSystemTemplate } from '../core/nta/NtaSystemTemplates';
import { renderWithProviders, userEvent } from './test-utils';

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'office', address: '', city: '',
  zones: [{ id: 'zone-a', name: 'Kantoor', surfaces: [] }, { id: 'zone-b', name: 'Sporthal', surfaces: [] }],
  heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [],
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

describe('NTA project input templates', () => {
  it('use the kernel field names of annex P', () => {
    expect(annexPRouteTemplate('declared', 'heating')).toEqual({
      method: 'declared', primaryFactor: null, renewableFactor: null, co2KgPerKwh: null, declarationReference: '', measuredOnly: false,
    });
    expect(withoutNulls(annexPRouteTemplate('calculated', 'hot_water'))).toEqual({
      method: 'calculated', function: 'hot_water',
      distribution: { method: 'flows', sourceReference: '' },
      generators: [{ id: 'opwekker-1', kind: { kind: 'boiler', carrier: { kind: 'natural_gas' }, efficiency: { method: 'table_p3' } } }],
      hotWaterStorage: { method: 'forfait' },
      sourceReference: '',
    });
    expect(annexPGeneratorKindTemplate('heat_pump')).toEqual({
      kind: 'heat_pump', efficiency: { method: 'table_p5', source: null, supplyTemperatureC: null }, drive: { kind: 'electricity' },
    });
    expect(coolingSystemTemplate('compression')).toMatchObject({ generators: [{ id: 'cold-1', generator: { kind: 'compression' } }] });
  });
});

describe('NTA project input forms', () => {
  it('edits a calculated annex P heat supply with two generators', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}} body={(draft, change) => <ExternalSupplyFields draft={draft} change={change} />} />);
    await user.selectOptions(screen.getByRole('combobox', { name: 'External heat for space heating (dh)' }), 'calculated');
    await user.type(screen.getByRole('spinbutton', { name: 'Delivered energy Q_XD;out;tot, kWh/year' }), '120000');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Distribution' }), 'small_system_forfait');
    await user.type(screen.getByRole('spinbutton', { name: 'Number of connections' }), '40');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Boiler type' }), 'hr107');
    await user.click(screen.getByRole('button', { name: 'Add generator' }));
    await user.selectOptions(screen.getAllByRole('combobox', { name: 'Generator kind' })[1], 'heat_pump');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source (table P.5)' }), 'electric_ground');
    await user.type(screen.getByRole('spinbutton', { name: 'Network design supply temperature, °C' }), '55');
    await user.type(screen.getAllByRole('spinbutton', { name: 'Priority (1 first)' })[1], '1');
    await user.click(screen.getByRole('checkbox', { name: 'Calculate the auxiliary energy (P.56–P.70)' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Network' }), 'secondary');
    // P.6.8.4.3: a declared efficiency that includes the source pump.
    await user.selectOptions(screen.getAllByRole('combobox', { name: 'Heat-pump efficiency' })[0], 'declared');
    await user.type(screen.getByRole('spinbutton', { name: 'Efficiency' }), '4.2');
    await user.click(screen.getByRole('checkbox', {
      name: 'Source pump or fan included in the declared efficiency (P.6.8.4.3: 0 W/kW)',
    }));
    expect(withoutNulls(current().externalSupply.heating.generators[1].kind.efficiency)).toEqual({
      method: 'declared', value: 4.2, sourceReference: '', sourcePumpIncluded: true,
    });
    await user.selectOptions(screen.getAllByRole('combobox', { name: 'Heat-pump efficiency' })[0], 'table_p5');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source (table P.5)' }), 'electric_ground');
    await user.type(screen.getByRole('spinbutton', { name: 'Network design supply temperature, °C' }), '55');
    expect(withoutNulls(current().externalSupply.heating)).toEqual({
      method: 'calculated', function: 'heating', deliveredKwh: 120000,
      distribution: { method: 'small_system_forfait', connections: 40, connectionType: 'ground_bound' },
      generators: [
        { id: 'opwekker-1', kind: { kind: 'boiler', carrier: { kind: 'natural_gas' }, efficiency: { method: 'table_p3', boiler: 'hr107' } } },
        { id: 'opwekker-2', priority: 1, kind: {
          kind: 'heat_pump', efficiency: { method: 'table_p5', source: 'electric_ground', supplyTemperatureC: 55 }, drive: { kind: 'electricity' },
        } },
      ],
      auxiliary: { distribution: { method: 'forfait', network: 'secondary' } },
      sourceReference: '',
    });
  }, 60000);

  it('edits measured cold, a collective source and area electricity', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}} body={(draft, change) => <ExternalSupplyFields draft={draft} change={change} />} />);
    await user.selectOptions(screen.getByRole('combobox', { name: 'External cold (dc)' }), 'measured');
    await user.type(screen.getByRole('spinbutton', { name: 'Delivered energy Q_XD;out;tot, kWh/year' }), '5000');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Energy carrier' }), 'electricity');
    await user.type(screen.getByRole('spinbutton', { name: 'Input, kWh/year' }), '1200');
    await user.type(screen.getByRole('spinbutton', { name: 'Renewable factor f_Pren' }), '0');
    expect(withoutNulls(current().externalSupply.cooling)).toEqual({
      method: 'measured', function: 'cooling', deliveredKwh: 5000, inputs: [{ carrier: { kind: 'electricity' }, kwh: 1200 }],
      exportedElectricityKwh: 0, renewableFactor: 0, sourceReference: '',
    });
    await user.click(screen.getByRole('checkbox', { name: 'Collective heat-pump source (9.6.8.1.1.2.3)' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Source temperature class' }), 'below20_c');
    await user.type(screen.getByRole('textbox', { name: 'Invoices or design data (reference)' }), 'factuur 12');
    await user.click(screen.getByRole('button', { name: 'Add area producer (wind, hydro, other)' }));
    await user.type(screen.getByRole('spinbutton', { name: 'Annual production, kWh' }), '800');
    expect(withoutNulls(current().externalSupply)).toMatchObject({
      collectiveHeatPumpSource: { temperatureClass: 'below20_c', supplierReference: 'factuur 12' },
      areaElectricity: [{ kind: 'declared', id: 'gebied-1', annualKwh: 800, sourceReference: '' }],
    });
  }, 60000);

  it('edits a sunroom and a steam humidifier', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}} body={(draft, change) => <>
      <SunroomsFields draft={draft} change={change} />
      <HumidifiersFields draft={draft} change={change} project={project} />
    </>} />);
    await user.click(screen.getByRole('button', { name: 'Add sunroom' }));
    await user.type(screen.getByRole('spinbutton', { name: 'g_gl;ue heating' }), '0.6');
    await user.type(screen.getByRole('spinbutton', { name: 'Reduction factor b_U' }), '0.8');
    await user.type(screen.getByRole('spinbutton', { name: 'Surface area, m²' }), '12');
    await user.click(screen.getByRole('button', { name: 'Add humidifier' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Calculation zone' }), 'zone-b');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Humidifier type' }), 'steam');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Steam generator' }), 'gas_or_oil');
    await user.type(screen.getByRole('spinbutton', {
      name: 'Area served by the steam humidifier, m² (12.2.1; empty: heating-system area)',
    }), '650');
    const draft = withoutNulls(current());
    expect(draft.sunrooms).toEqual([{
      id: 'serre-1', glazingGHeating: 0.6, reductionFactor: 0.8, distributionFactor: 1, surfaces: [{ areaM2: 12 }], sourceReference: '',
    }]);
    expect(draft.humidifiers).toEqual([{
      zoneId: 'zone-b', humidification: { humidifier: { kind: 'steam', carrier: 'gas_or_oil' }, rotaryWheel: false, equipmentReference: '' },
      servedAreaM2: 650,
    }]);
  }, 60000);

  it('switches between one cooling system and several per zone', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ cooling: coolingSystemTemplate('compression') }}
      body={(draft, change) => <CoolingSystemsFields draft={draft} change={change} project={project} />} />);
    await user.click(screen.getByRole('checkbox', { name: 'Several cooling systems, each for its own zones (§10.2)' }));
    expect(current().cooling).toBeNull();
    expect(current().coolingSystems).toHaveLength(1);
    expect(current().coolingSystems[0].zoneIds).toEqual(['zone-a', 'zone-b']);
    const first = screen.getAllByRole('group').find((group) => within(group).queryByText('Cooling system 1'))!;
    await user.click(within(first).getByRole('checkbox', { name: 'Sporthal' }));
    await user.click(screen.getByRole('button', { name: 'Add cooling system' }));
    const second = screen.getAllByRole('group').find((group) => within(group).queryByText('Cooling system 2'))!;
    expect(within(second).getByRole('checkbox', { name: 'Kantoor' })).toBeDisabled();
    await user.click(within(second).getByRole('checkbox', { name: 'Sporthal' }));
    await user.selectOptions(within(second).getByRole('combobox', { name: 'Cold generator' }), 'room_air_conditioner');
    expect(current().coolingSystems.map((item: Draft) => item.zoneIds)).toEqual([['zone-a'], ['zone-b']]);
    expect(current().coolingSystems[1].system.generators[0].generator.kind).toBe('room_air_conditioner');
    await user.click(screen.getByRole('checkbox', { name: 'Several cooling systems, each for its own zones (§10.2)' }));
    expect(current().coolingSystems).toEqual([]);
    expect(current().cooling.generators[0].generator.kind).toBe('compression');
  }, 60000);

  it('keeps further cooling generators and clears hidden outdoor-air values', async () => {
    const user = userEvent.setup();
    const cooling = coolingSystemTemplate('compression') as Draft;
    const generators = cooling.generators as Draft[];
    const initial = {
      cooling: { ...cooling, generators: [...generators, { id: 'cold-2', generator: { kind: 'external_cold' }, capacityKw: 40, equipmentReference: '' }] },
      heatPumpRenewable: {
        sourceBelow20C: false, exhaustAirSource: false, combinedOutdoorAndExhaustAir: true,
        outdoorAirHeatFraction: 0.6, outdoorAirFractionReference: 'kwaliteitsverklaring', sourceReference: '',
      },
    };
    renderWithProviders(<Harness initial={initial} body={(draft, change) => <>
      <CoolingSystemsFields draft={draft} change={change} project={project} />
      <CollectiveAndRenewableFields draft={draft} change={change} />
    </>} />);
    await user.selectOptions(screen.getByRole('combobox', { name: 'Cold generator' }), 'absorption_chp');
    let draft = current();
    expect(draft.cooling.generators).toHaveLength(2);
    expect(draft.cooling.generators[0].generator).toEqual({
      kind: 'absorption_chp', chp: { powerKw: null, builtAfter2006: true, hreDeclared: false, lowTemperature: false },
    });
    expect(draft.cooling.generators[1].id).toBe('cold-2');
    await user.type(screen.getByRole('spinbutton', { name: 'Electric power P_el, kW' }), '50');
    expect(current().cooling.generators[0].generator.chp.powerKw).toBe(50);
    await user.selectOptions(screen.getByRole('combobox', { name: 'Cold generator' }), 'gas_engine_compression');
    expect(current().cooling.generators[0].generator).toMatchObject({ kind: 'gas_engine_compression', gasEngine: { powerKw: null } });
    await user.click(screen.getByRole('checkbox', { name: 'Outdoor air and exhaust air combined' }));
    draft = current();
    expect(draft.heatPumpRenewable.combinedOutdoorAndExhaustAir).toBe(false);
    expect(withoutNulls(draft.heatPumpRenewable)).toEqual({
      sourceBelow20C: false, exhaustAirSource: false, combinedOutdoorAndExhaustAir: false, sourceReference: '',
    });
  }, 60000);

  it('edits solar space heating, the collective installation and declared flows', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ declaredUses: [] }} body={(draft, change) => <>
      <SpaceHeatingSolarFields draft={draft} change={change} />
      <CollectiveAndRenewableFields draft={draft} change={change} />
      <DeclaredFlowsFields draft={draft} change={change} />
    </>} />);
    await user.click(screen.getByRole('button', { name: /Add solar/ }));
    expect(current().spaceHeatingSolar[0]).toMatchObject({ id: 'solar-1', solarUse: 'space_heating' });
    await user.click(screen.getByRole('checkbox', { name: 'Building on a collective installation (9.6.1)' }));
    await user.type(screen.getByRole('spinbutton', { name: 'A_g of the whole building on the installation, m²' }), '2400');
    await user.click(screen.getByRole('checkbox', { name: 'Renewable share of the heat pump (5.31/5.32)' }));
    await user.click(screen.getByRole('checkbox', { name: 'Source temperature below 20 °C' }));
    await user.click(screen.getByRole('button', { name: 'Add declared use (§5.5)' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Service' }), 'lighting');
    await user.type(screen.getByRole('spinbutton', { name: 'kWh per month 1' }), '40');
    await user.click(screen.getByRole('button', { name: 'Add on-site production (chapter 16)' }));
    const draft = current();
    expect(draft.collectiveConnection).toEqual({ connectedUsableAreaM2: 2400, sourceReference: '' });
    expect(draft.heatPumpRenewable).toEqual({
      sourceBelow20C: true, exhaustAirSource: false, combinedOutdoorAndExhaustAir: false, sourceReference: '',
    });
    expect(draft.declaredUses).toEqual([{ id: 'gebruik-1', service: 'lighting', carrier: 'el', monthlyKwh: [40], sourceReference: '' }]);
    expect(draft.onSiteProduction).toEqual([{ id: 'opwekking-1', kind: 'pv', monthlyKwh: [], sourceReference: '' }]);
  }, 60000);
});
