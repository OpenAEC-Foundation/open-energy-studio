import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import { ExternalSupplyFields } from '../components/NtaPerformancePanel/NtaExternalSupply';
import { NtaLightingSection } from '../components/NtaPerformancePanel/NtaExtraSections';
import { GroundEdgeBridgesFields } from '../components/NtaPerformancePanel/NtaCalculationForm';
import { MeasurePatchFields, parsePatchValue } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import type { MwaPatchOperation } from '../core/nta/KernelClient';
import { withoutNulls } from '../core/nta/KernelInput';
import { renderWithProviders, userEvent } from './test-utils';

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

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'office', address: '', city: '',
  zones: [{ id: 'zone-a', name: 'Kantoor', floorArea: 400, surfaces: [] }],
  heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [],
  solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

describe('annex P details without JSON editors', () => {
  it('edits pipe segments, a buffer and the calculated hot-water storage', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}} body={(draft, change) => <ExternalSupplyFields draft={draft} change={change} />} />);
    await user.selectOptions(screen.getByLabelText('External heat for hot water (dw)'), 'calculated');
    await user.selectOptions(screen.getByLabelText('Distribution'), 'pipes');
    await user.click(screen.getByRole('button', { name: 'Add pipe segment' }));
    await user.type(screen.getByLabelText('Length, m'), '120');
    await user.click(screen.getByRole('button', { name: 'Add layer' }));
    await user.type(screen.getByLabelText('Layer 1 — λ W/(m·K)'), '0.03');
    await user.type(screen.getByLabelText('Inner diameter, m'), '0.06');
    await user.type(screen.getByLabelText('Outer diameter, m'), '0.12');
    await user.type(screen.getByLabelText('Cover depth h_j, m'), '0.8');
    await user.selectOptions(screen.getByLabelText('Surroundings'), 'crawlspace');
    await user.selectOptions(screen.getByLabelText('Correction f_x;j (table P.1)'), 'two_pipes_in_trench');
    await user.selectOptions(screen.getByLabelText('Network water temperature θ_XD;circ'), 'constant');
    await user.type(screen.getByLabelText('Water temperature, °C'), '70');
    await user.click(screen.getByRole('button', { name: 'Add buffer vessel' }));
    await user.type(screen.getByLabelText('Volume, l'), '500');
    await user.selectOptions(screen.getByLabelText('Insulation (P.6.6.4.1)'), 'at_least20_mm');
    await user.type(screen.getAllByLabelText('Source')[0], 'leidingtekening');
    expect(withoutNulls(current().externalSupply.hotWater.distribution)).toEqual({
      method: 'pipes', sourceReference: 'leidingtekening',
      segments: [{
        lengthM: 120, layers: [{ conductivityWPerMk: 0.03, innerDiameterM: 0.06, outerDiameterM: 0.12 }],
        placement: { kind: 'buried', coverDepthM: 0.8, ambient: { kind: 'crawlspace' } }, correction: 'two_pipes_in_trench',
      }],
      waterTemperature: { method: 'constant', temperatureC: 70 },
      buffers: [{ volumeL: 500, insulation: 'at_least20_mm' }],
    });
    // Other losses are optional: clearing drops the member (kernel default 0).
    const other = screen.getByLabelText('Other losses, kWh/year');
    await user.type(other, '50');
    expect(current().externalSupply.hotWater.distribution.otherLossKwh).toBe(50);
    await user.clear(other);
    expect('otherLossKwh' in current().externalSupply.hotWater.distribution
      && current().externalSupply.hotWater.distribution.otherLossKwh !== undefined).toBe(false);

    // Switching the placement drops the buried-only fields.
    await user.selectOptions(screen.getByLabelText('Placement'), 'in_air');
    expect(current().externalSupply.hotWater.distribution.segments[0].placement)
      .toEqual({ kind: 'in_air', ambient: { kind: 'indoor', temperatureC: null } });

    // The calculated route starts with one vessel: without components η would be 1 (P.35).
    await user.selectOptions(screen.getByLabelText('Hot-water storage η_WD;gen;sto (P.34/P.35)'), 'calculated');
    await user.type(screen.getAllByLabelText('Volume, l')[1], '300');
    await user.click(screen.getByRole('button', { name: 'Add charging pipe' }));
    await user.type(screen.getAllByLabelText('Length, m')[1], '10');
    await user.click(screen.getByLabelText('External plate heat exchanger (P.46)'));
    const powers = screen.getAllByLabelText('Nominal power, kW');
    await user.type(powers[powers.length - 1], '150');
    await user.click(screen.getByLabelText('Insulated at least 20 mm all round'));
    expect(withoutNulls(current().externalSupply.hotWater.hotWaterStorage)).toEqual({
      method: 'calculated', sourceReference: '', vessels: [{ volumeL: 300 }], pipes: [{ lengthM: 10 }],
      exchanger: { nominalPowerKw: 150, insulated: true },
    });
  }, 60000);

  it('edits collective solar, flex and sorption generators and clears stale fields', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}} body={(draft, change) => <ExternalSupplyFields draft={draft} change={change} />} />);
    await user.selectOptions(screen.getByLabelText('External heat for space heating (dh)'), 'calculated');
    await user.click(screen.getByRole('button', { name: 'Add generator' }));
    const kind = () => screen.getAllByLabelText('Generator kind')[1];
    const generator = () => current().externalSupply.heating.generators[1].kind;

    await user.selectOptions(kind(), 'collective_solar');
    await user.type(screen.getByLabelText('Annual solar contribution, kWh'), '4000');
    expect(withoutNulls(generator())).toEqual({
      kind: 'collective_solar', contribution: { method: 'declared', annualKwh: 4000, sourceReference: '' },
    });
    await user.click(screen.getByLabelText('Twelve monthly values'));
    await user.type(screen.getByLabelText('Monthly solar contribution, kWh 1'), '300');
    expect(generator().contribution.annualKwh).toBeUndefined();
    expect(generator().contribution.monthlyKwh[0]).toBe(300);
    await user.selectOptions(screen.getByLabelText('Solar contribution Q_XD;sol;mi'), 'calculated');
    await user.type(screen.getByLabelText('Collector module area, m²'), '2.5');
    await user.type(screen.getByLabelText('Network design supply temperature, °C'), '70');
    expect(generator().contribution).toMatchObject({
      method: 'calculated', solarType: 'preheater', networkSupplyC: 70,
      collectors: { moduleAreaM2: 2.5, orientation: 'south' }, storage: { loss: { method: 'unknown_label' } },
    });
    expect(generator().contribution.monthlyKwh).toBeUndefined();

    await user.selectOptions(kind(), 'electric_flex');
    await user.selectOptions(screen.getByLabelText('Flex-mode generator (5.8)'), 'heat_pump');
    await user.selectOptions(screen.getByLabelText('Source (table P.5)'), 'electric_surface_water');
    await user.type(screen.getByLabelText('Connections of the heat network (at least 500)'), '600');
    await user.click(screen.getByLabelText('Heat buffer decouples production from demand'));
    expect(withoutNulls(generator())).toEqual({
      kind: 'electric_flex', connections: 600, heatBuffer: true, registrationReference: '',
      generator: { kind: 'heat_pump', efficiency: { method: 'table_p5', source: 'electric_surface_water' } },
    });

    await user.selectOptions(kind(), 'sorption_chiller');
    await user.selectOptions(screen.getByLabelText('Heat source (table P.10)'), 'chp');
    expect(withoutNulls(generator())).toEqual({
      kind: 'sorption_chiller', heat: { source: 'chp', carrier: { kind: 'natural_gas' }, tableP6: { installedAfter2006: true } },
    });
    await user.selectOptions(screen.getByLabelText('Heat source (table P.10)'), 'collective_heat');
    expect(withoutNulls(generator())).toEqual({ kind: 'sorption_chiller', heat: { source: 'collective_heat', sourceReference: '' } });
  }, 60000);

  it('edits an area PV system with the chapter 16 fields', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{}} body={(draft, change) => <ExternalSupplyFields draft={draft} change={change} />} />);
    await user.click(screen.getByRole('button', { name: 'Add area PV system' }));
    await user.type(screen.getByLabelText('panel peak power W'), '400');
    await user.type(screen.getByLabelText('number of panels'), '10');
    await user.type(screen.getByLabelText('azimuth ° (0 = north)'), '180');
    expect(withoutNulls(current().externalSupply.areaElectricity[0])).toEqual({
      kind: 'pv', id: 'gebied-pv-1', peakPower: { method: 'panels', panelPeakPowerW: 400, panelCount: 10 }, azimuthDeg: 180,
      mounting: 'unknown', obstructionFactors: [], sourceReference: '',
    });
    // The obstruction factor is optional: clearing it leaves no blank.
    const obstruction = screen.getByLabelText(/obstruction/i);
    await user.type(obstruction, '0.9');
    expect(current().externalSupply.areaElectricity[0].obstructionFactors).toEqual([0.9]);
    await user.clear(obstruction);
    expect(current().externalSupply.areaElectricity[0].obstructionFactors).toEqual([]);
    await user.selectOptions(screen.getByLabelText('gebied-pv-1 — peak power P_pk'), 'table16_1');
    expect(current().externalSupply.areaElectricity[0].peakPower).toEqual({ method: 'table16_1', moduleType: null, panelAreaM2: null });
  }, 60000);
});

describe('lighting, floor edges and measure patches without JSON', () => {
  it('edits luminaire groups, parasitic power and daylight sectors', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ labelFunction: 'office' }}
      body={(draft, change) => <NtaLightingSection draft={draft} change={change} project={project} />} />);
    await user.click(screen.getByLabelText('Calculate lighting'));
    await user.selectOptions(screen.getByLabelText('Power'), 'installed');
    await user.type(screen.getByLabelText('Luminaire group 1 — Number of luminaires'), '20');
    await user.type(screen.getByLabelText('System power per luminaire, W'), '36');
    await user.click(screen.getByRole('button', { name: 'Add luminaire group' }));
    await user.type(screen.getByLabelText('Luminaire group 2 — Number of luminaires'), '4');
    await user.selectOptions(screen.getAllByLabelText('Power per luminaire')[1], 'lamps');
    await user.type(screen.getByLabelText('Lamp power, W'), '28');
    await user.type(screen.getByLabelText('Lamps per luminaire'), '2');
    await user.selectOptions(screen.getByLabelText('Lamp technology (table 14.2)'), 'fluorescent_t5');
    await user.selectOptions(screen.getByLabelText('Parasitic power'), 'installed');
    await user.type(screen.getByLabelText('Σ emergency charging power P_ei, W'), '12');
    await user.selectOptions(screen.getByLabelText('Daylight'), 'sectors');
    await user.selectOptions(screen.getByLabelText('Kind'), 'tilted_window');
    await user.type(screen.getByLabelText('Tilt γ from the horizontal, °'), '45');
    await user.click(screen.getByLabelText('Data for the rooflight projection (below 75°)'));
    await user.type(screen.getAllByLabelText('Distance to wall, m — side 1')[0], '2');
    const zone = withoutNulls(current().lighting[0].lightingZones[0]);
    expect(zone.power).toEqual({
      method: 'installed', sourceReference: '', luminaires: [
        { count: 20, power: { method: 'system', powerW: 36 } },
        { count: 4, power: { method: 'lamps', lampPowerW: 28, lampCount: 2, technology: 'fluorescent_t5' } },
      ],
    });
    expect(zone.parasitic).toEqual({ method: 'installed', emergencyChargingW: 12, sourceReference: '' });
    expect(zone.daylight).toEqual({
      method: 'sectors', sourceReference: '', sectors: [{
        kind: 'tilted_window', tiltDeg: 45, control: 'automatic_switching_or_unknown',
        rooflight: { wallDistancesM: [2, null, null, null] },
      }],
    });
    await user.click(screen.getByRole('button', { name: 'Add lighting zone' }));
    expect(current().lighting[0].lightingZones.map((item: Draft) => item.id)).toEqual(['lighting-1', 'lighting-2']);
  }, 60000);

  it('edits the floor-edge ψ values of 8.36', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ groundFloors: [{ edgeThermalBridges: { method: 'detailed', bridges: [] } }] }}
      body={(draft, change) => <GroundEdgeBridgesFields draft={draft} change={change}
        base={['groundFloors', 0, 'edgeThermalBridges', 'bridges']} />} />);
    await user.click(screen.getByRole('button', { name: 'Add edge part' }));
    await user.type(screen.getByLabelText('Edge part length ℓ_j, m 1'), '12.5');
    await user.type(screen.getByLabelText('ψ_gr;j, W/(m·K)'), '0.08');
    expect(current().groundFloors[0].edgeThermalBridges.bridges).toEqual([{ lengthM: 12.5, psiWPerMk: 0.08, sourceReference: '' }]);
  }, 60000);

  it('edits measure patches row by row', async () => {
    expect(parsePatchValue('0.15')).toBe(0.15);
    expect(parsePatchValue('true')).toBe(true);
    expect(parsePatchValue('{"method":"forfait"}')).toEqual({ method: 'forfait' });
    expect(parsePatchValue('hr107')).toBe('hr107');
    const user = userEvent.setup();
    function PatchHarness() {
      const [patch, setPatch] = useState<MwaPatchOperation[]>([]);
      return <>
        <MeasurePatchFields patch={patch} onChange={setPatch} />
        <output data-testid="draft">{JSON.stringify({ patch })}</output>
      </>;
    }
    renderWithProviders(<PatchHarness />);
    await user.click(screen.getByRole('button', { name: 'Add change' }));
    await user.type(screen.getByLabelText('Path (JSON pointer)'), '/constructions/0/uValue');
    await user.type(screen.getByLabelText('New value'), '0.15');
    expect(current().patch).toEqual([{ op: 'replace', path: '/constructions/0/uValue', value: 0.15 }]);
    // The type is explicit: text "2" stays a string, true a boolean.
    await user.selectOptions(screen.getByLabelText('Value type'), 'text');
    await user.clear(screen.getByLabelText('New value'));
    await user.type(screen.getByLabelText('New value'), '2');
    expect(current().patch[0].value).toBe('2');
    await user.selectOptions(screen.getByLabelText('Value type'), 'boolean');
    await user.selectOptions(screen.getByLabelText('New value'), 'true');
    expect(current().patch[0].value).toBe(true);
    await user.selectOptions(screen.getByLabelText('Value type'), 'json');
    expect(current().patch[0].value).toEqual({});
    await user.selectOptions(screen.getByLabelText('Value type'), 'number');
    await user.type(screen.getByLabelText('New value'), '7');
    expect(current().patch[0].value).toBe(7);
    await user.selectOptions(screen.getByLabelText('Operation'), 'remove');
    expect(current().patch).toEqual([{ op: 'remove', path: '/constructions/0/uValue' }]);
  }, 60000);

  it('keeps each patch row its own value after a removal and reopens a saved null as null', async () => {
    const user = userEvent.setup();
    function PatchHarness() {
      const [patch, setPatch] = useState<MwaPatchOperation[]>([
        { op: 'replace', path: '/a', value: 'first' },
        { op: 'replace', path: '/b', value: 'second' },
        { op: 'replace', path: '/c', value: null },
      ]);
      return <>
        <MeasurePatchFields patch={patch} onChange={setPatch} />
        <output data-testid="draft">{JSON.stringify({ patch })}</output>
      </>;
    }
    renderWithProviders(<PatchHarness />);
    // The saved null row shows the null type, not "number".
    const types = () => screen.getAllByLabelText('Value type') as HTMLSelectElement[];
    expect(types().map((select) => select.value)).toEqual(['text', 'text', 'null']);
    await user.click(screen.getAllByRole('button', { name: 'Remove' })[0]);
    expect(current().patch.map((row: MwaPatchOperation) => row.path)).toEqual(['/b', '/c']);
    // The remaining rows keep their own values and types.
    expect((screen.getByLabelText('New value') as HTMLInputElement).value).toBe('second');
    expect(types().map((select) => select.value)).toEqual(['text', 'null']);
  }, 60000);
});
