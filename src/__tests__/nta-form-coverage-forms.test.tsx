import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { HotWaterGeneratorFields, HotWaterSystemExtrasFields, SpaceGeneratorFields } from '../components/NtaPerformancePanel/NtaSystemSections';
import { NtaDistributionFields } from '../components/NtaPerformancePanel/NtaExtraSections';
import { CoolingSystemFields } from '../components/NtaPerformancePanel/NtaProjectExtras';
import {
  DwellingTypeFields, UtilityInstallationExtrasFields, UtilityLightingFields, VentilationBasicsFields,
} from '../components/BasisopnamePanel/SurveyBasics';
import { hotWaterGeneratorTemplate } from '../core/nta/NtaSystemTemplates';
import { routeForPath } from '../core/nta/gapRoutes';
import type { IProject } from '../core/energy/types';
import { en } from '../i18n/en';
import { renderWithProviders } from './test-utils';

// Forms for routes that were JSON-only (docs/ui-redesign/formulierdekking.md):
// each writes the kernel's own shape.

function Harness({ initial, children }: {
  initial: Draft;
  children: (draft: Draft, change: (path: Path, value: unknown) => void) => ReactElement;
}) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    {children(draft, change)}
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');
const t = (key: string) => (en as Record<string, string>)[key] ?? key;
const setNumber = (label: RegExp | string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });
const select = (label: RegExp | string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });

describe('hot-water generators without a form before', () => {
  it('builds the annex W booster heat pump with its tests and monthly source temperatures', () => {
    renderWithProviders(<Harness initial={{ hotWater: { generator: { kind: 'electric_boiler' } } }}>{(d, change) =>
      <HotWaterGeneratorFields draft={d} change={change} base={['hotWater', 'generator']} />}</Harness>);
    select('Hot-water generator', 'booster_heat_pump');
    expect(draft().hotWater.generator).toEqual(hotWaterGeneratorTemplate('booster_heat_pump'));
    fireEvent.change(screen.getByLabelText('Measured COP — test 1 (low source temperature)'), { target: { value: '3.2' } });
    fireEvent.change(screen.getByLabelText('Measured COP — test 2 (40 °C)'), { target: { value: '4.4' } });
    setNumber(/Standing loss P_ls/, '0.05');
    fireEvent.click(screen.getByLabelText('Enter the source temperature per month'));
    fireEvent.change(screen.getByLabelText('Mean source inlet temperature θ_evap 3'), { target: { value: '30' } });
    fireEvent.change(screen.getByLabelText('Test report (source)'), { target: { value: 'rapport 12' } });
    const generator = draft().hotWater.generator;
    expect(generator.lowTest).toEqual({ sourceTemperatureC: 24, cop: 3.2 });
    expect(generator.highTest).toEqual({ sourceTemperatureC: 40, cop: 4.4 });
    expect(generator.standingLossKw).toBe(0.05);
    expect(generator.sourceTemperaturesC).toHaveLength(12);
    expect(generator.sourceTemperaturesC[2]).toBe(30);
    expect(generator.heatSource).toEqual({ kind: 'heating_system' });
    expect(generator.testReportReference).toBe('rapport 12');
  });

  it('offers the series, indirect, biomass and large-storage routes in their kernel shape', () => {
    renderWithProviders(<Harness initial={{ hotWater: { generator: { kind: 'electric_boiler' } } }}>{(d, change) =>
      <HotWaterGeneratorFields draft={d} change={change} base={['hotWater', 'generator']} />}</Harness>);
    select('Hot-water generator', 'heat_pump_series');
    select(/Source of the last heat pump/, 'groundwater_below15_c');
    expect(draft().hotWater.generator).toEqual({ kind: 'heat_pump_series', lastSource: 'groundwater_below15_c' });
    select('Hot-water generator', 'biomass_combi');
    select('Vessel insulation (annex R)', 'at_least10_mm');
    expect(draft().hotWater.generator).toEqual({ kind: 'biomass_combi', insulation: 'at_least10_mm', insideBoundary: true });
    select('Hot-water generator', 'indirect_heat_pump');
    expect(draft().hotWater.generator).toEqual({ kind: 'indirect_heat_pump', alsoSpaceHeating: true });
    select('Hot-water generator', 'large_direct_storage');
    fireEvent.click(screen.getByLabelText(/Gas-fired/));
    expect(draft().hotWater.generator).toEqual({ kind: 'large_direct_storage', gasFired: false });
  });

  it('enters delivery sets and the b_U of a hot-water system', () => {
    renderWithProviders(<Harness initial={{ hotWater: {} }}>{(d, change) =>
      <HotWaterSystemExtrasFields draft={d} change={change} base={['hotWater']} />}</Harness>);
    fireEvent.click(screen.getByLabelText(/Delivery sets of external heat supply/));
    setNumber('Number of delivery sets', '24');
    setNumber(/Reduction factor b_U of the unheated space with pipes/, '0.6');
    expect(draft().hotWater).toEqual({ deliverySets: { count: 24, sourceReference: '' }, unheatedReductionFactor: 0.6 });
  });

  // The kernel accepts these shapes: a project probe calculated each one
  // (`calculated_unverified`, no gaps) when the forms were added.
  it('enters a boiling-water tap and shower heat recovery per shower', () => {
    renderWithProviders(<Harness initial={{ hotWater: { need: { method: 'residential' } } }}>{(d, change) =>
      <HotWaterSystemExtrasFields draft={d} change={change} base={['hotWater']} />}</Harness>);
    fireEvent.click(screen.getByLabelText(t('nta.form.dhwBoilingWaterTap')));
    fireEvent.click(screen.getByLabelText(t('nta.form.shower')));
    // Residential systems have no "which shower on which unit" question (utility only).
    expect(screen.queryByLabelText(t('nta.form.shower.assignmentUnknown'))).toBeNull();
    fireEvent.click(screen.getByText(t('nta.form.shower.add')));
    const units = screen.getAllByLabelText(t('nta.form.shower.unitKind'));
    fireEvent.change(units[1], { target: { value: 'declared' } });
    setNumber(t('nta.form.shower.efficiency'), '0.55');
    select(t('nta.form.shower.connection'), 'mixer_only');
    expect(draft().hotWater).toMatchObject({
      boilingWaterTap: true,
      showerHeatRecovery: { connection: 'mixer_only', sourceReference: '',
        showers: [{ unit: 'vertical' }, { unit: 'declared', efficiency: 0.55, sourceReference: '' }] },
    });
  });

  it('builds the gas-fired storage heater of 13.165–13.175', () => {
    renderWithProviders(<Harness initial={{ hotWater: { generator: { kind: 'electric_boiler' } } }}>{(d, change) =>
      <HotWaterGeneratorFields draft={d} change={change} base={['hotWater', 'generator']} />}</Harness>);
    select('Hot-water generator', 'gas_storage_heater');
    setNumber(/Volume of the storage vessel/, '120');
    fireEvent.click(screen.getByLabelText(/Appliance from before 1985/));
    expect(draft().hotWater.generator).toEqual({ kind: 'gas_storage_heater', volumeL: 120, before1985: true, inHeatedZone: true });
  });
});

describe('heating generator and distribution details', () => {
  const project = { heatingSystems: [] } as unknown as IProject;

  it('sets the boiler role and the source system of a forfait heat pump', () => {
    renderWithProviders(<Harness initial={{ generator: { kind: 'gas_boiler', boiler: { role: 'individual_main' } } }}>{(d, change) =>
      <SpaceGeneratorFields draft={d} change={change} base={['generator']} project={project} />}</Harness>);
    select('Role of the boiler', 'individual_supplementary');
    expect(draft().generator.boiler.role).toBe('individual_supplementary');
  });

  it('sets the collective source system of a forfait heat pump', () => {
    renderWithProviders(<Harness initial={{ generator: { kind: 'heat_pump_forfait', forfait: null, sourceSystem: 'individual', sourceSystemReference: '' } }}>
      {(d, change) => <SpaceGeneratorFields draft={d} change={change} base={['generator']} project={project} />}</Harness>);
    select('Source system of the heat pump (9.62)', 'collective_ground');
    expect(draft().generator.sourceSystem).toBe('collective_ground');
  });

  it('enters the pump, a buffer vessel and the combined pipes of a calculated distribution', () => {
    renderWithProviders(<Harness initial={{
      distribution: { method: 'calculated', sourceReference: '' },
      distributionSystem: { installation: 'individual', usageFunction: 'residential', connectedStoreys: 1, pipeTransmittance: { method: 'forfait', insulation: { state: 'unknown' } },
        valvesInsulated: false, pump: { method: 'calculated', heatMeterPresent: false, sourceReference: '' }, sourceReference: '' },
    }}>{(d, change) => <NtaDistributionFields draft={d} change={change} />}</Harness>);
    setNumber(/Energy efficiency index/, '0.23');
    setNumber(/One-pipe system/, '6');
    fireEvent.click(screen.getByLabelText('The pipes also carry hot water (combined pipes)'));
    // An individual installation has neither a delivery set (9.2.6) nor a
    // buffer vessel, and a forfait Ψ already includes buffer losses
    // (table 9.16 note 2): the kernel refuses both, so the form hides them.
    expect(screen.queryByLabelText(t('nta.dist.deliverySet'))).toBeNull();
    expect(screen.queryByLabelText(t('nta.dist.buffer'))).toBeNull();
    select(t('nta.dist.installation'), 'collective');
    expect(screen.getByLabelText(t('nta.dist.deliverySet'))).toBeTruthy();
    expect(screen.queryByLabelText(t('nta.dist.buffer'))).toBeNull();
    select(t('nta.dist.psiRoute'), 'calculated');
    setNumber(t('nta.form.pipeGeometry.pipeDiameter'), '0.022');
    setNumber(t('nta.form.pipeGeometry.insulatedDiameter'), '0.062');
    setNumber(t('nta.form.pipeGeometry.insulationLambda'), '0.035');
    fireEvent.click(screen.getByLabelText(t('nta.dist.buffer')));
    setNumber(/Buffer vessel volume/, '200');
    select('Energy label of the vessel', 'b');
    const system = draft().distributionSystem;
    expect(system.installation).toBe('collective');
    expect(system.pipeTransmittance).toEqual({ method: 'insulated_in_air', pipeOuterDiameterM: 0.022,
      insulatedDiameterM: 0.062, insulationLambdaWPerMK: 0.035 });
    expect(system.pump).toMatchObject({ method: 'calculated', energyEfficiencyIndex: 0.23, onePipeEmitterCount: 6 });
    expect(system.pipesAlsoForHotWater).toBe(true);
    expect(system.bufferVessel).toMatchObject({ volumeL: 200, label: 'b', producedFrom2018: false, inHeatedSpace: true });
    // Back to the forfait Ψ: the buffer vessel fields disappear again.
    select(t('nta.dist.psiRoute'), 'forfait');
    expect(draft().distributionSystem.pipeTransmittance.method).toBe('forfait');
    expect(screen.queryByLabelText(t('nta.dist.buffer'))).toBeNull();
  });

  it('enters the circulation pump of a water-based cooling system', () => {
    renderWithProviders(<Harness initial={{ cooling: { generators: [{ generator: { kind: 'compression' } }],
      distribution: { designTemperature: 't6_to12_or_unknown', pipe: { kind: 'uninsulated' }, fittingsInsulated: false, sourceReference: '' } } }}>
      {(d, change) => <CoolingSystemFields draft={d} change={change} base={['cooling']} allowNone />}</Harness>);
    fireEvent.click(screen.getByLabelText(/Circulation pump of the cooling system/));
    fireEvent.click(screen.getByLabelText('Hydraulically balanced'));
    setNumber('Number of storeys', '3');
    setNumber(/Pipe length outside the cooled zone/, '12');
    expect(draft().cooling.distribution).toMatchObject({ unconditionedPipeLengthM: 12,
      pump: { hydraulicallyBalanced: true, floorCount: 3, heatMeter: false, individualDwellingInstallation: false } });
  });

  it('routes gaps of the new fields to their installation step', () => {
    for (const path of ['ntaCalculation.distributionSystem.bufferVessel.volumeL', 'ntaCalculation.distributionSystem.pump.onePipeEmitterCount',
      'ntaCalculation.hotWater.generator.lowTest.cop', 'ntaCalculation.hotWater.deliverySets.count',
      'ntaCalculation.cooling.distribution.pump.floorCount']) {
      expect(routeForPath(path), path).toMatchObject({ step: 'installations' });
    }
  });
});

describe('survey answers without a form before', () => {
  it('chooses the dwelling type, position and roof', () => {
    renderWithProviders(<Harness initial={{ dwelling: { kind: 'single_family', position: 'terraced', roofType: 'pitched' } }}>
      {(d, change) => <DwellingTypeFields draft={d} change={change} t={t} />}</Harness>);
    select('Position', 'end_or_corner');
    select('Roof type', 'flat');
    expect(draft().dwelling).toEqual({ kind: 'single_family', position: 'end_or_corner', roofType: 'flat' });
    select('Dwelling type', 'apartment');
    select('Position in the block', 'end_or_corner');
    expect(draft().dwelling).toEqual({ kind: 'apartment', floor: 'ground_or_intermediate', side: 'end_or_corner' });
  });

  it('chooses the ventilation principle and the unit', () => {
    renderWithProviders(<Harness initial={{ ventilation: { principle: 'natural', sourceReference: '' } }}>
      {(d, change) => <VentilationBasicsFields draft={d} change={change} t={t} />}</Harness>);
    select('Pressure class of the vents', 'from1_to5_pa');
    select('Ventilation principle', 'balanced');
    setNumber('Manufacture year of the ventilation unit', '2012');
    select('Fan motor', 'dc');
    expect(draft().ventilation).toMatchObject({ principle: 'balanced', pressureClass: 'from1_to5_pa', unitManufactureYear: 2012, motor: 'dc' });
  });

  it('enters a utility lighting zone with lamps, presence and daylight control', () => {
    renderWithProviders(<Harness initial={{ lighting: [] }}>
      {(d, change) => <UtilityLightingFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByText('Add lighting zone'));
    setNumber(/Area of the lighting zone/, '120');
    select('Installed power', 'lamps');
    setNumber(/Lamp power/, '28');
    select('Lamp type', 't5');
    select('Occupancy control', 'auto_on_auto_off');
    select('Daylight control', 'dimming');
    expect(draft().lighting[0]).toMatchObject({ id: 'verlichting-1', areaM2: 120,
      power: { method: 'lamps', lamps: [{ count: 1, lampPowerW: 28, lampType: 't5' }] },
      presence: 'auto_on_auto_off', daylight: 'dimming' });
  });

  it('enters humidification and flow control, and leaves an absent cooling system alone', () => {
    renderWithProviders(<Harness initial={{ ventilation: { principle: 'balanced', sourceReference: '' }, heatingInstallation: { collective: false } }}>
      {(d, change) => <UtilityInstallationExtrasFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByLabelText('Humidification present'));
    select('Humidifier', 'electric_steam');
    fireEvent.click(screen.getByLabelText('Flow control present'));
    select('Control method', 'inlet_vane');
    expect(draft().humidification).toEqual({ humidifier: 'electric_steam', absorptionWheel: false, sourceReference: '' });
    expect(draft().ventilation.flowControl).toEqual({ method: 'inlet_vane', minimumPercent: null });
    expect(screen.queryByLabelText('Year of the cooling pipe insulation')).toBeNull();
    expect(draft().cooling).toBeUndefined();
  });
});
