import { useState, type ReactElement } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import { SpaceGeneratorFields } from '../components/NtaPerformancePanel/NtaSystemSections';
import { DeclaredHeatingTableTool, GroundFloorDetailFields } from '../components/NtaPerformancePanel/NtaProductGenerators';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import type { DeclaredHeatingTableAssessment } from '../core/nta/KernelClient';
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
const generatorFields: Body = (draft, change) =>
  <SpaceGeneratorFields draft={draft} change={change} base={['generator']} project={project} />;

describe('NTA generators with product data', () => {
  it('edits an annex Q heat pump with part-load series, switch-off criteria and a boiler backup', { timeout: 60000 }, async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ generator: { kind: 'gas_boiler' } }} body={generatorFields} />);
    await user.selectOptions(screen.getByLabelText('Generator type'), 'heat_pump_annex_q');
    await user.selectOptions(screen.getByLabelText('Source / delivery (annex Q)'), 'brine_water');
    await user.type(screen.getByLabelText('Design supply temperature θ_sup, °C (≤ 75)'), '35');
    await user.type(screen.getAllByLabelText('Evaporator in, °C')[0], '0');
    await user.type(screen.getAllByLabelText('Condenser out, °C')[0], '35');
    await user.type(screen.getAllByLabelText('Heating power, kW')[0], '6.1');
    await user.type(screen.getAllByLabelText('COP')[0], '4.5');
    await user.selectOptions(screen.getByLabelText('Capacity control'), 'modulating');
    expect(screen.getAllByLabelText('COP')).toHaveLength(6);
    await user.type(screen.getByLabelText('Minimum power P_H;hp;min, kW'), '2');
    await user.type(screen.getByLabelText('Minimum COP (≥ 1)'), '1.5');
    await user.click(screen.getByLabelText('Source pump (Q.4.4)'));
    await user.type(screen.getByLabelText('Source pump overrun t_ev;xt, s'), '60');
    await user.selectOptions(screen.getByLabelText('Evaporator inlet temperature (Q.2.14.2)'), 'forfait');
    await user.selectOptions(screen.getByLabelText('Backup heater (required when F_H;gen < 1)'), 'gas_boiler');
    await user.selectOptions(screen.getByLabelText('Boiler type'), 'hr107');
    await user.type(screen.getByLabelText('Test report (EN 14511/14825)'), 'TR-1');
    // A brine/water source offers annex V regeneration.
    expect(screen.getByLabelText('Ground-source regeneration (annex V)')).toBeTruthy();

    const generator = current().generator;
    expect(generator).toMatchObject({
      kind: 'heat_pump_annex_q', designSupplyTemperatureC: 35,
      backup: { kind: 'gas_boiler', boiler: { generatorId: 'backup', kind: 'hr107', fuel: 'natural_gas' } },
      heatPump: {
        source: 'brine_water', testReportReference: 'TR-1',
        maximumPower: { condition1: { evaporatorInC: 0, condenserOutC: 35, heatingPowerKw: 6.1, cop: 4.5 }, condition4: null },
        modulation: { method: 'modulating', minimumPowerKw: 2, condenserPumpModulating: false, sourcePumpModulating: false },
        switchOff: { minCop: 1.5 },
        sourcePump: { nominalPowerW: null, overrunS: 60 },
        evaporatorInlet: { method: 'forfait' },
      },
    });
    expect(generator.heatPump.modulation.lowRange).toHaveLength(5);
  });

  it('edits an annex M boiler with condensing efficiencies', { timeout: 60000 }, async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ generator: { kind: 'gas_boiler' } }} body={generatorFields} />);
    await user.selectOptions(screen.getByLabelText('Generator type'), 'product_boiler');
    await user.selectOptions(screen.getByLabelText('Boiler technology (annex M)'), 'condensing_gas');
    await user.type(screen.getByLabelText('Nominal power, kW'), '24');
    await user.selectOptions(screen.getByLabelText('Full-load efficiency'), 'condensing');
    await user.type(screen.getByLabelText('Efficiency at 80/60 °C (fraction)'), '0.88');
    await user.type(screen.getByLabelText('Efficiency at 30 °C return (fraction)'), '0.97');
    await user.selectOptions(screen.getByLabelText('Design temperature class'), '55_47');
    expect(current().generator).toMatchObject({
      kind: 'product_boiler', designTemperatureClass: '55_47',
      boiler: { technology: 'condensing_gas', fuel: 'natural_gas',
        product: { nominalPowerKw: 24, fullLoad: { method: 'condensing', efficiencyAt60: 0.88, efficiencyAt30: 0.97 } } },
    });
  });

  it('edits an annex N heater and a table 9.25 air heater', { timeout: 60000 }, async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ generator: { kind: 'gas_boiler' } }} body={generatorFields} />);
    await user.selectOptions(screen.getByLabelText('Generator type'), 'local_heater');
    await user.selectOptions(screen.getByLabelText('Heater type (annex N)'), 'air_heater_fan_burner');
    await user.selectOptions(screen.getByLabelText('Fan'), 'axial');
    await user.type(screen.getByLabelText('Combustion efficiency, %'), '92');
    expect(current().generator).toMatchObject({
      kind: 'local_heater', fuel: 'natural_gas',
      heater: { heaterType: 'air_heater_fan_burner', fan: 'axial', product: { combustionEfficiencyPercent: 92 } },
    });
    await user.selectOptions(screen.getByLabelText('Generator type'), 'forfait_heater');
    await user.selectOptions(screen.getByLabelText('Heater (table 9.25)'), 'air_heater_hr107');
    await user.type(screen.getByLabelText('Pilot flames (695 kWh/year each)'), '2');
    expect(current().generator).toMatchObject({
      kind: 'forfait_heater', heaterKind: 'air_heater_hr107', fuel: 'natural_gas', pilotFlames: 2,
      auxiliary: { electricallyConnectedDevices: null },
    });
  });

  it('edits a crawlspace, a heated basement and edge insulation of a ground floor (8.3)', { timeout: 60000 }, async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ groundFloors: [{ surfaceId: 'f1' }] }}
      body={(draft, change) => <GroundFloorDetailFields draft={draft} change={change} base={['groundFloors', 0]} />} />);
    await user.click(screen.getByRole('button', { name: 'Add edge insulation' }));
    await user.selectOptions(screen.getByLabelText('Edge insulation (table D.1)'), 'vertical');
    await user.type(screen.getByLabelText('R_n, m²K/W'), '2.5');
    await user.type(screen.getByLabelText('Thickness d_n, m'), '0.1');
    expect(current().groundFloors[0].edgeInsulation).toEqual([
      { kind: 'vertical', resistanceM2kPerW: 2.5, thicknessM: 0.1, sourceReference: '' }]);
    await user.click(screen.getByRole('button', { name: 'Remove' }));
    await user.selectOptions(screen.getByLabelText('Space below the floor (8.3.4.2)'), 'crawlspace');
    await user.type(screen.getByLabelText('R_bw of the wall, m²K/W'), '2.7');
    await user.type(screen.getByLabelText('U of the wall above ground, W/m²K'), '0.35');
    expect(current().groundFloors[0].below).toEqual({
      kind: 'crawlspace', floorResistanceM2kPerW: null, depthClass: 'other', wallResistanceM2kPerW: 2.7,
      wallUValueWPerM2k: 0.35, ventilationOpeningM2PerM: null });
    // Edge insulation belongs to a slab on ground only.
    expect(screen.queryByRole('button', { name: 'Add edge insulation' })).toBeNull();
    await user.selectOptions(screen.getByLabelText('Space below the floor (8.3.4.2)'), 'unheated_basement');
    expect(current().groundFloors[0].below).toMatchObject({ kind: 'unheated_basement', volumeM3: null, airChangesPerHour: null });
    await user.selectOptions(screen.getByLabelText('Space below the floor (8.3.4.2)'), '');
    await user.click(screen.getByLabelText('Heated room below ground level (8.3.3.2)'));
    await user.type(screen.getByLabelText('Depth of the floor below ground level z, m'), '2.5');
    await user.type(screen.getByLabelText('R_c of the basement walls, m²K/W'), '3.5');
    expect(current().groundFloors[0]).toMatchObject({ below: null, heatedBasement: { depthM: 2.5, wallResistanceM2kPerW: 3.5 } });
  });

  it('interpolates a BCRG declaration table through the kernel', { timeout: 60000 }, async () => {
    const user = userEvent.setup();
    const diagnose = vi.fn(async () => ({
      status: 'input_valid', generationEfficiency: 3.9, preferredEnergyFraction: 0.95, auxiliaryElectricityKwhPerYear: 120, issues: [],
    }) as unknown as DeclaredHeatingTableAssessment);
    renderWithProviders(<DeclaredHeatingTableTool diagnose={diagnose} />);
    await user.type(screen.getByLabelText('BCRG declaration code'), 'BCRG-0042');
    await user.type(screen.getAllByLabelText('Gross heat demand, kWh/year')[0], '8000');
    await user.type(screen.getByLabelText('Design supply temperature, °C'), '35');
    await user.type(screen.getByLabelText('Supply temperature of the row, °C'), '35');
    await user.type(screen.getAllByLabelText('Gross heat demand, kWh/year')[1], '8000');
    await user.type(screen.getByLabelText('Generation efficiency η'), '3.9');
    await user.click(screen.getByRole('button', { name: 'Interpolate in the table' }));
    expect(diagnose).toHaveBeenCalledWith(expect.objectContaining({
      declarationId: 'BCRG-0042', grossHeatDemandKwhPerYear: 8000, designSupplyTemperatureC: 35,
      rows: [{ supplyTemperatureC: 35, points: [{ grossHeatDemandKwhPerYear: 8000, generationEfficiency: 3.9 }] }],
    }));
    expect((await screen.findByRole('status')).textContent).toContain('η 3.9');
  });
});
