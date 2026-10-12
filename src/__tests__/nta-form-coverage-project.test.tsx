import { useState, type ReactElement } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, within } from '@testing-library/react';
import { FieldPathPrefixProvider, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { CommonSpaceFillField, LightingZoneField, SwimmingPoolFields, ZoneDataFields } from '../components/NtaPerformancePanel/NtaZoneData';
import { NtaLightingSection } from '../components/NtaPerformancePanel/NtaExtraSections';
import { CollectiveCoolingField, CollectivePvField } from '../components/NtaPerformancePanel/NtaCollectiveShare';
import { CirculationPsiFields, HotWaterCirculationFields } from '../components/NtaPerformancePanel/NtaPipeGeometry';
import { AdditionalHotWaterSystemsFields, HotWaterExtraGeneratorsFields } from '../components/NtaPerformancePanel/NtaSystemSections';
import { CoolingSystemsFields } from '../components/NtaPerformancePanel/NtaProjectExtras';
import { syncVentilation } from '../core/nta/NtaFormModels';
import { NTA_SECTIONS } from '../components/NtaPerformancePanel/NtaSections';
import type { IProject } from '../core/energy/types';
import { renderWithProviders } from './test-utils';

// Project forms for kernel options that were JSON-only (docs/ui-redesign/formulierdekking.md).
// The kernel reads the same shapes in crates/nta8800-core/tests/project_form_shapes.rs.

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
const select = (label: RegExp | string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });
const setNumber = (label: RegExp | string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });
const project = (ids: string[]) => ({ zones: ids.map((id) => ({ id, name: id === 'z1' ? 'Begane grond' : 'Verdieping',
  floorArea: id === 'z1' ? 400 : 200 })) }) as unknown as IProject;

describe('project forms that had no form', () => {
  it('starts zone data as a copy of the block and writes the zone answers', () => {
    const block = {
      usageFunction: 'office',
      internalGains: { method: 'utility' },
      ventilationFlows: [{ id: 'f1', months: [] }],
      ventilation: { functions: [{ function: 'sport', areaM2: 100, swimmingPool: false }] },
    };
    renderWithProviders(<Harness initial={block}>
      {(d, change) => <ZoneDataFields draft={d} change={change} project={project(['z1', 'z2'])} />}</Harness>);
    fireEvent.click(screen.getAllByText('Own data for this calculation zone')[1]);
    let zone = draft().zoneData[0];
    // The copied chapter 11 ventilation gets this zone's id and area (ventilation_area_mismatch otherwise).
    expect(zone).toEqual({ zoneId: 'z2', internalGains: block.internalGains, ventilationFlows: block.ventilationFlows,
      ventilation: { zoneId: 'z2', usableFloorAreaM2: 200, functions: [{ function: 'sport', areaM2: 200, swimmingPool: false }] } });
    select(/Use function of the calculation zone/, 'retail');
    fireEvent.click(screen.getByText(/Add a part with another use function/));
    setNumber(/Usable floor area of this part/, '80');
    fireEvent.click(screen.getByLabelText(/Own thermal mass for this calculation zone/));
    fireEvent.click(screen.getByText('Add sunroom'));
    fireEvent.click(screen.getByLabelText(/Room with a swimming pool/));
    zone = draft().zoneData[0];
    expect(zone.usageFunction).toBe('retail');
    expect(zone.functionAreas).toEqual([{ function: 'retail', areaM2: 80 }]);
    expect(zone.thermalMass).toEqual({ floor: 'heavy', wall: 'heavy', ceiling: 'open_or_none', sourceReference: '' });
    expect(zone.sunrooms).toHaveLength(1);
    expect(zone.ventilation.functions[0].swimmingPool).toBe(true);
    // The block stays as it was.
    expect(draft().usageFunction).toBe('office');
    expect(draft().ventilation.functions[0].swimmingPool).toBe(false);
    // With several zones removal asks first (zone_data_missing).
    const confirm = vi.spyOn(window, 'confirm').mockReturnValueOnce(false).mockReturnValueOnce(true);
    fireEvent.click(screen.getByText("Remove this calculation zone's own data"));
    expect(draft().zoneData).toHaveLength(1);
    fireEvent.click(screen.getByText("Remove this calculation zone's own data"));
    expect(draft().zoneData).toEqual([]);
    confirm.mockRestore();
  });

  it('writes the common spaces of a zone for the 6.6.4 allocation', () => {
    renderWithProviders(<Harness initial={{ calculationScope: 'utility', usageFunction: 'office', internalGains: { method: 'utility' } }}>
      {(d, change) => <ZoneDataFields draft={d} change={change} project={project(['z1', 'z2'])} />}</Harness>);
    fireEvent.click(screen.getAllByText('Own data for this calculation zone')[0]);
    fireEvent.click(screen.getByText(/Add a common space \(6\.6\.4\)/));
    expect(screen.getByText(/the areas above are the functions' own areas/)).toBeTruthy();
    setNumber(/Usable floor area of the common space/, '49.2');
    fireEvent.change(screen.getByLabelText(/Source of the common space/), { target: { value: 'NEN 2580-meetstaat' } });
    // No function ticked: all functions of the building; ticking writes the list, unticking the last drops it.
    expect('servedFunctions' in draft().zoneData[0].commonSpaces[0]).toBe(false);
    // Review 28: only utility functions can be served; a residential part is a separate calculation.
    const fieldset = screen.getByText(/Usable floor area of the common space/).closest('fieldset')!;
    expect(within(fieldset).queryByLabelText('Residential')).toBeNull();
    const office =screen.getAllByLabelText('Office').find((element) => (element as HTMLInputElement).type === 'checkbox')!;
    fireEvent.click(office);
    expect(draft().zoneData[0].commonSpaces).toEqual([{ areaM2: 49.2, sourceReference: 'NEN 2580-meetstaat', servedFunctions: ['office'] }]);
    fireEvent.click(office);
    expect('servedFunctions' in draft().zoneData[0].commonSpaces[0]).toBe(false);
    fireEvent.click(screen.getAllByText('Remove').slice(-1)[0]);
    expect('commonSpaces' in draft().zoneData[0]).toBe(false);
    // A residential zone offers none: its common spaces belong to the dwellings.
    select(/Use function of the calculation zone/, 'residential');
    expect(screen.queryByText(/Add a common space/)).toBeNull();
  });

  // Review 24: function areas and common spaces live on zone data only, so a single-zone utility project needs
  // the way in too; a zone turning residential keeps no hidden common spaces.
  it('offers own zone data, and so common spaces, with a single zone', () => {
    renderWithProviders(<Harness initial={{ calculationScope: 'utility', usageFunction: 'office', internalGains: { method: 'utility' } }}>
      {(d, change) => <ZoneDataFields draft={d} change={change} project={project(['z1'])} />}</Harness>);
    expect(screen.getByText(/The project has one calculation zone/)).toBeTruthy();
    fireEvent.click(screen.getByText('Own data for this calculation zone'));
    fireEvent.click(screen.getByText(/Add a common space \(6\.6\.4\)/));
    setNumber(/Usable floor area of the common space/, '20');
    expect(draft().zoneData[0].commonSpaces[0].areaM2).toBe(20);
    select(/Use function of the calculation zone/, 'residential');
    expect('commonSpaces' in draft().zoneData[0]).toBe(false);
  });

  it('lets the kernel fill the lighting and hot-water function lists from the common spaces', () => {
    const block = {
      calculationScope: 'utility', usageFunction: 'office', internalGains: { method: 'utility' },
      zoneData: [{ zoneId: 'z1', commonSpaces: [{ areaM2: 20, sourceReference: 'meetstaat' }] }],
      lighting: [{ zoneId: 'z1', functions: [{ function: 'office', areaM2: 400 }], lightingZones: [], sourceReference: '' }],
      hotWater: { need: { method: 'utility', areas: [{ function: 'office', areaM2: 600 }], sourceReference: '' } },
    };
    renderWithProviders(<Harness initial={block}>
      {(d, change) => <>
        <NtaLightingSection draft={d} change={change} project={project(['z1'])} />
        <CommonSpaceFillField draft={d} change={change} path={['hotWater', 'need', 'areas']}>
          <span>hot-water entries</span>
        </CommonSpaceFillField>
      </>}</Harness>);
    const toggles = screen.getAllByLabelText(/Let the kernel split the areas per function from the common spaces/);
    expect(toggles).toHaveLength(2);
    fireEvent.click(toggles[0]);
    fireEvent.click(toggles[1]);
    expect(draft().lighting[0].functions).toEqual([]);
    expect(draft().hotWater.need.areas).toEqual([]);
    expect(screen.queryByText('hot-water entries')).toBeNull();
    expect(screen.getAllByText(/The kernel fills this list with the within-zone split/)).toHaveLength(2);
    // Unticked: one blank entry to fill in by hand.
    fireEvent.click(toggles[1]);
    expect(draft().hotWater.need.areas).toEqual([{ function: null, areaM2: null }]);
    expect(screen.getByText('hot-water entries')).toBeTruthy();
  });

  // Review 25: with several hot-water systems the kernel does not fill (13.20), so the choice is not offered.
  it('offers no hot-water fill choice next to additional hot-water systems', () => {
    const block = {
      calculationScope: 'utility', usageFunction: 'office', internalGains: { method: 'utility' },
      zoneData: [{ zoneId: 'z1', commonSpaces: [{ areaM2: 20, sourceReference: 'meetstaat' }] }],
      hotWater: { need: { method: 'utility', areas: [], sourceReference: '' } },
      additionalHotWaterSystems: [{ need: { method: 'utility', areas: [{ function: 'office', areaM2: 100 }] } }],
    };
    const HotWater = NTA_SECTIONS.find((section) => section.id === 'hotWater')!.Component;
    const view = (initial: Draft) => renderWithProviders(<Harness initial={initial}>
      {(d, change) => <HotWater draft={d} change={change} update={() => undefined} project={project(['z1'])} />}</Harness>);
    const single = view({ ...block, additionalHotWaterSystems: [] });
    expect(screen.getByLabelText(/Let the kernel split the areas/)).toBeTruthy();
    single.unmount();
    view(block);
    expect(screen.queryByLabelText(/Let the kernel split the areas/)).toBeNull();
    expect(screen.getByLabelText(/Use function \(table 13\.1\)/)).toBeTruthy();
  });

  it('offers no fill choice without common spaces', () => {
    renderWithProviders(<Harness initial={{ hotWater: { need: { method: 'utility', areas: [] } } }}>
      {(d, change) => <CommonSpaceFillField draft={d} change={change} path={['hotWater', 'need', 'areas']}>
        <span>hot-water entries</span></CommonSpaceFillField>}</Harness>);
    expect(screen.queryByLabelText(/Let the kernel split the areas/)).toBeNull();
    expect(screen.getByText('hot-water entries')).toBeTruthy();
  });

  it('starts zone data with valid internal gains when the block has none', () => {
    renderWithProviders(<Harness initial={{ calculationScope: 'utility', usageFunction: 'office' }}>
      {(d, change) => <ZoneDataFields draft={d} change={change} project={project(['z1', 'z2'])} />}</Harness>);
    fireEvent.click(screen.getAllByText('Own data for this calculation zone')[0]);
    expect(draft().zoneData[0].internalGains.method).toBe('utility');
  });

  it('keeps the zone dwelling type and function areas consistent with the zone use', () => {
    renderWithProviders(<Harness initial={{ usageFunction: 'office', dwellingType: 'apartment_building', internalGains: { method: 'utility' } }}>
      {(d, change) => <ZoneDataFields draft={d} change={change} project={project(['z1', 'z2'])} />}</Harness>);
    fireEvent.click(screen.getAllByText('Own data for this calculation zone')[0]);
    fireEvent.click(screen.getByText(/Add a part with another use function/));
    // No residential part: the kernel refuses it (function_areas_residential_mixed).
    const functionSelect = screen.getByLabelText(/Use function of this part/) as HTMLSelectElement;
    expect([...functionSelect.options].map((option) => option.value)).not.toContain('residential');
    // Residential: the block's dwelling type is prefilled, the parts go and no part can be added.
    select(/Use function of the calculation zone/, 'residential');
    expect(draft().zoneData[0].dwellingType).toBe('apartment_building');
    expect(draft().zoneData[0].functionAreas).toBeUndefined();
    expect(screen.queryByText(/Add a part with another use function/)).toBeNull();
    // Back to a utility use: no stale dwelling type (dwelling_type_not_applicable).
    select(/Use function of the calculation zone/, 'office');
    expect('dwellingType' in draft().zoneData[0]).toBe(false);
  });

  it('keeps the block swimming pool when Apply syncs the ventilation', () => {
    const block = { usageFunction: 'sport', calculationScope: 'utility', setpoints: { heatingC: 20, coolingC: 24 },
      ventilation: { functions: [{ function: 'sport', areaM2: 400 }] } };
    renderWithProviders(<Harness initial={block}>
      {(d, change) => <SwimmingPoolFields draft={d} change={change} base={['ventilation']} />}</Harness>);
    fireEvent.click(screen.getByLabelText(/Room with a swimming pool/));
    // NtaDraftProvider.apply dispatches syncVentilation(draft, project).
    const applied = syncVentilation(draft(), project(['z1'])) as { ventilation: { functions: Draft[] } };
    expect(applied.ventilation.functions).toEqual([{ function: 'sport', areaM2: 400, swimmingPool: true }]);
  });

  it('ends the series when a second further generator is added', () => {
    renderWithProviders(<Harness initial={{ hotWater: { additionalGenerators: [{}], series: { kind: 'hotfill_electric_boiler' } } }}>
      {(d, change) => <HotWaterExtraGeneratorsFields draft={d} change={change} base={['hotWater']} />}</Harness>);
    fireEvent.click(screen.getByText('Add hot-water generator (13.8.2)'));
    expect(draft().hotWater.additionalGenerators).toHaveLength(2);
    expect(draft().hotWater.series).toBeNull();
  });

  it('shows the collective share per cooling system', () => {
    renderWithProviders(<Harness initial={{ coolingSystems: [{ zoneIds: ['z1'], system: { generators: [] } },
      { zoneIds: ['z2'], system: { generators: [] } }] }}>
      {(d, change) => <CoolingSystemsFields draft={d} change={change} project={project(['z1', 'z2'])} />}</Harness>);
    const boxes = screen.getAllByLabelText(/Cooling collective for the building/);
    expect(boxes).toHaveLength(2);
    fireEvent.click(boxes[1]);
    expect(draft().coolingSystems[1].system.collective).toEqual({ buildingUsableFloorAreaM2: null, sourceReference: '' });
  });

  it('gives the calculated Ψ of a further system a kernel path for "Ga naar"', () => {
    const { container } = renderWithProviders(<FieldPathPrefixProvider value="ntaCalculation">
      <Harness initial={{ additionalHotWaterSystems: [{ circulation: {} }] }}>
        {(d, change) => <CirculationPsiFields draft={d} change={change} system={['additionalHotWaterSystems', 0]} />}</Harness>
    </FieldPathPrefixProvider>);
    expect(container.querySelector('[data-path="ntaCalculation.additionalHotWaterSystems[0].circulation.calculatedPsi"]')).not.toBeNull();
  });

  it('offers the swimming pool only for a sport function', () => {
    renderWithProviders(<Harness initial={{ ventilation: { functions: [{ function: 'office', areaM2: 10 }, { function: 'sport', areaM2: 20 }] } }}>
      {(d, change) => <SwimmingPoolFields draft={d} change={change} base={['ventilation']} />}</Harness>);
    expect(screen.getAllByLabelText(/Room with a swimming pool/)).toHaveLength(1);
    fireEvent.click(screen.getByLabelText(/Room with a swimming pool/));
    expect(draft().ventilation.functions[1].swimmingPool).toBe(true);
  });

  it('writes the circulation of a hot-water system in the kernel shape', () => {
    renderWithProviders(<Harness initial={{ hotWater: {} }}>
      {(d, change) => <HotWaterCirculationFields draft={d} change={change} base={['hotWater']} />}</Harness>);
    fireEvent.click(screen.getByLabelText('Circulation pipe present (§13.5)'));
    expect(draft().hotWater.circulation).toEqual({ insulation: 'unknown', fittingsInsulated: false, floorCount: 1,
      pump: { control: 'uncontrolled_or_unknown' }, sourceReference: '' });
    select(/Insulation of the circulation pipe/, 'mm15');
    setNumber(/Length of the circulation pipe/, '30');
    select(/Control of the circulation pump/, 'constant_pressure');
    setNumber(/Length of the circulation pipe/, '');
    const circulation = draft().hotWater.circulation;
    expect(circulation.insulation).toBe('mm15');
    expect(circulation.pump.control).toBe('constant_pressure');
    expect('lengthM' in circulation).toBe(false);
    // A declared Ψ hides the calculated one: they exclude each other (hot_water_psi_declared_and_calculated).
    setNumber(/Declared Ψ of the circulation pipe/, '0.2');
    expect(screen.queryByLabelText(/calculated/i)).toBeNull();
  });

  it('gives a further hot-water system its circulation, further generators and series', () => {
    renderWithProviders(<Harness initial={{ hotWater: { need: { method: 'residential' } }, additionalHotWaterSystems: [] }}>
      {(d, change) => <AdditionalHotWaterSystemsFields draft={d} change={change} residential />}</Harness>);
    fireEvent.click(screen.getByText(/Add hot-water system/i));
    fireEvent.click(screen.getByLabelText('Circulation pipe present (§13.5)'));
    fireEvent.click(screen.getByText('Add hot-water generator (13.8.2)'));
    select(/Series arrangement/, 'hotfill_electric_boiler');
    const system = draft().additionalHotWaterSystems[0];
    expect(system.circulation.floorCount).toBe(1);
    expect(system.additionalGenerators).toHaveLength(1);
    expect(system.series).toEqual({ kind: 'hotfill_electric_boiler' });
    // Removing the only further generator also removes the series, which needs exactly one.
    const extra = [...document.querySelectorAll('legend')].find((legend) => legend.textContent === 'Additional hot-water generator 1');
    const buttons = [...(extra?.parentElement?.querySelectorAll(':scope > button') ?? [])];
    const remove = buttons[buttons.length - 1] as HTMLElement;
    fireEvent.click(remove);
    expect(draft().additionalHotWaterSystems[0].additionalGenerators).toEqual([]);
    expect(draft().additionalHotWaterSystems[0].series).toBeNull();
  });

  it('writes the collective share of cooling and of a PV system', () => {
    renderWithProviders(<Harness initial={{ cooling: { generators: [] }, pvSystems: [{ id: 'pv' }] }}>
      {(d, change) => <>
        <CollectiveCoolingField draft={d} change={change} />
        <CollectivePvField draft={d} change={change} base={['pvSystems', 0]} />
      </>}</Harness>);
    fireEvent.click(screen.getByLabelText(/Cooling collective for the building/));
    fireEvent.click(screen.getByLabelText(/PV system shared by the building/));
    const [coolingArea, pvArea] = screen.getAllByLabelText(/Usable floor area of the whole building/);
    fireEvent.change(coolingArea, { target: { value: '1200' } });
    fireEvent.change(pvArea, { target: { value: '900' } });
    expect(draft().cooling.collective).toEqual({ buildingUsableFloorAreaM2: 1200, sourceReference: '' });
    expect(draft().pvSystems[0].collective).toEqual({ buildingUsableFloorAreaM2: 900, sourceReference: '' });
  });

  it('asks the zone of the lighting only with several zones', () => {
    const { unmount } = renderWithProviders(<Harness initial={{ lighting: [{ zoneId: 'z1' }] }}>
      {(d, change) => <LightingZoneField draft={d} change={change} project={project(['z1'])} index={0} />}</Harness>);
    expect(screen.queryByLabelText('Calculation zone of the lighting')).toBeNull();
    unmount();
    renderWithProviders(<Harness initial={{ lighting: [{ zoneId: 'z1' }] }}>
      {(d, change) => <LightingZoneField draft={d} change={change} project={project(['z1', 'z2'])} index={0} />}</Harness>);
    select('Calculation zone of the lighting', 'z2');
    expect(draft().lighting[0].zoneId).toBe('z2');
  });
});
