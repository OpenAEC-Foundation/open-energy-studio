import { describe, expect, it } from 'vitest';
import { fireEvent, screen, within } from '@testing-library/react';
import { createDefaultProject, documentManagerReducer, useEnergy } from '../context/EnergyContext';
import { editAction } from '../core/energy/projectItems';
import { normalizeProject } from '../core/energy/normalizeProject';
import { describeCascade } from '../core/energy/cascadeLabels';
import { deleteZoneFromProject } from '../core/energy/projectDelete';
import { buildTemplatePatch } from '../core/nta/MwaTemplates';
import { dutchDecimals, dutchSource } from '../core/nta/OpnameValueText';
import { ConstructionEditorDialog } from '../components/dialogs/ConstructionEditorDialog/ConstructionEditorDialog';
import { WindowEditorDialog } from '../components/dialogs/WindowEditorDialog/WindowEditorDialog';
import { SurfaceEditorDialog } from '../components/dialogs/SurfaceEditorDialog/SurfaceEditorDialog';
import { ZoneEditorDialog } from '../components/dialogs/ZoneEditorDialog/ZoneEditorDialog';
import { HeatingSystemDialog } from '../components/dialogs/HeatingSystemDialog/HeatingSystemDialog';
import { VentilationSystemDialog } from '../components/dialogs/VentilationSystemDialog/VentilationSystemDialog';
import { HotWaterSystemDialog } from '../components/dialogs/HotWaterSystemDialog/HotWaterSystemDialog';
import { CoolingSystemDialog } from '../components/dialogs/CoolingSystemDialog/CoolingSystemDialog';
import { SolarPVDialog } from '../components/dialogs/SolarPVDialog/SolarPVDialog';
import { SolarThermalDialog } from '../components/dialogs/SolarThermalDialog/SolarThermalDialog';
import { ErrorBoundary } from '../components/ErrorBoundary/ErrorBoundary';
import type { IConstruction, ICoolingSystem, IHeatingSystem, IHotWaterSystem, IProject, ISolarPV, ISolarThermal, IVentilationSystem } from '../core/energy/types';
import { exampleProject } from '../core/nta/ExampleProjects';
import { renderWithProviders, userEvent } from './test-utils';

/** Construction without layers, as in both example projects: only Rc/U are stored. */
const BARE: IConstruction = { id: 'con-test-bare', name: 'Gevel Rc 4,7', layers: [], rcValue: 4.7, uValue: 0.207, exteriorSurfaceResistance: 0 };
/** Heat pump with saved draft inputs the dialog does not edit itself. */
const HEATING: IHeatingSystem = {
  id: 'hs-test', name: 'Warmtepomp', type: 'heat_pump_air', cop: 4.2, coverageFraction: 0.29,
  ntaHeatPump: {
    id: 'hs-test', source: 'outdoor_air', sink: 'hydronic', drive: 'electric_compression',
    reversible: false, hybrid: false, booster: false,
    performanceEvidence: { kind: 'normative_default', reference: null },
    forfaitHeatPumpDraft: { kept: true } as never,
  },
};
const VENTILATION: IVentilationSystem = { id: 'vs-test', name: 'WTW', type: 'type_d', heatRecoveryEfficiency: 0.29, sfp: 0.45 };
const HOT_WATER: IHotWaterSystem = { id: 'hw-test', name: 'Boiler', type: 'electric_boiler', efficiency: 0.9, hasSolarBoiler: true, solarBoilerFraction: 0.29 };

const COOLING: ICoolingSystem = { id: 'cs-test', name: 'Split', type: 'split_unit', eer: 3.4 };
const PV: ISolarPV = { id: 'pv-test', name: 'PV dak', peakPower: 2.1, orientation: 'S', tilt: 35, area: 11.3 };
const SOLAR_THERMAL: ISolarThermal = { id: 'st-test', name: 'Zonneboiler', collectorArea: 2.6, type: 'flat_plate', orientation: 'SW', tilt: 40 };

/** The live project of the harness, for a strict comparison of the object itself. */
let latest: IProject | null = null;

function Harness({ initial }: { initial?: IProject }) {
  const { state, dispatch } = useEnergy();
  const close = () => dispatch({ type: 'CLOSE_DIALOG' });
  const { dialog, project } = state;
  latest = project;
  const zone = project.zones[0];
  const surface = zone?.surfaces[0];
  const window = zone?.surfaces.flatMap((s) => s.windows)[0];
  const open = (type: string, id: string | undefined) => {
    const action = id ? editAction(type, id) : null;
    if (action) dispatch(action);
  };
  return <>
    <button type="button" onClick={() => {
      if (initial) dispatch({ type: 'SET_PROJECT', payload: initial });
      dispatch({ type: 'ADD_CONSTRUCTION', payload: BARE });
      dispatch({ type: 'ADD_HEATING_SYSTEM', payload: HEATING });
      dispatch({ type: 'ADD_VENTILATION_SYSTEM', payload: VENTILATION });
      dispatch({ type: 'ADD_HOT_WATER_SYSTEM', payload: HOT_WATER });
      dispatch({ type: 'ADD_COOLING_SYSTEM', payload: COOLING });
      dispatch({ type: 'ADD_SOLAR_PV', payload: PV });
      dispatch({ type: 'ADD_SOLAR_THERMAL', payload: SOLAR_THERMAL });
    }}>setup</button>
    <button type="button" onClick={() => open('construction', 'con-test-bare')}>open construction</button>
    <button type="button" onClick={() => open('zone', zone?.id)}>open zone</button>
    <button type="button" onClick={() => open('surface', surface?.id)}>open surface</button>
    <button type="button" onClick={() => open('window', window?.id)}>open window</button>
    <button type="button" onClick={() => open('heatingSystem', 'hs-test')}>open heating</button>
    <button type="button" onClick={() => open('ventilationSystem', 'vs-test')}>open ventilation</button>
    <button type="button" onClick={() => open('hotWaterSystem', 'hw-test')}>open hot water</button>
    <button type="button" onClick={() => open('coolingSystem', 'cs-test')}>open cooling</button>
    <button type="button" onClick={() => open('solarPV', 'pv-test')}>open pv</button>
    <button type="button" onClick={() => open('solarThermal', 'st-test')}>open solar thermal</button>
    {dialog.type === 'construction-editor' && <ConstructionEditorDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'zone-editor' && <ZoneEditorDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'surface-editor' && <SurfaceEditorDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'window-editor' && <WindowEditorDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'heating-system' && <HeatingSystemDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'ventilation-system' && <VentilationSystemDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'hot-water-system' && <HotWaterSystemDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'cooling-system' && <CoolingSystemDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'solar-pv' && <SolarPVDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'solar-thermal' && <SolarThermalDialog editId={dialog.editId} onClose={close} />}
    <output data-testid="project">{JSON.stringify({ ...project, id: undefined })}</output>
  </>;
}

const projectJson = () => JSON.parse(screen.getByTestId('project').textContent ?? '{}') as IProject;

const EDITORS = ['open construction', 'open zone', 'open surface', 'open window', 'open heating',
  'open ventilation', 'open hot water', 'open cooling', 'open pv', 'open solar thermal'];

/** Opens and saves every editor without a change; the project object must stay strictly equal. */
async function roundTripEveryEditor(initial?: IProject) {
  const user = userEvent.setup();
  renderWithProviders(<Harness initial={initial} />);
  await user.click(screen.getByRole('button', { name: 'setup' }));
  if (initial?.ntaCalculation) expect(latest?.ntaCalculation).toBeDefined();
  for (const name of EDITORS) {
    const before = latest!;
    await user.click(screen.getByRole('button', { name }));
    const dialog = screen.queryByRole('dialog');
    expect(dialog, `${name} must open a dialog`).not.toBeNull();
    await user.click(within(dialog!).getByRole('button', { name: 'Save' }));
    expect(latest, name).toStrictEqual(before);
  }
}

describe('opening and saving an editor without changes is a no-op', () => {
  it('keeps every stored value of each editor', async () => {
    await roundTripEveryEditor();
  }, 60000);

  it('keeps every stored value, including the NTA input, on a project with ntaCalculation', async () => {
    await roundTripEveryEditor(exampleProject('terraced_dwelling'));
  }, 60000);

  it('shows the stored Rc and U of a construction without layers and only recomputes after a layer edit', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(screen.getByRole('button', { name: 'setup' }));
    await user.click(screen.getByRole('button', { name: 'open construction' }));
    const dialog = screen.getByRole('dialog');
    expect(within(dialog).getByDisplayValue('4.70')).toBeInTheDocument();
    expect(within(dialog).getByDisplayValue('0.207')).toBeInTheDocument();
    fireEvent.change(within(dialog).getByLabelText('Name'), { target: { value: 'Gevel hernoemd' } });
    await user.click(within(dialog).getByRole('button', { name: 'Save' }));
    const saved = projectJson().constructions.find((c) => c.id === 'con-test-bare')!;
    expect(saved).toMatchObject({ name: 'Gevel hernoemd', rcValue: 4.7, uValue: 0.207, exteriorSurfaceResistance: 0 });
  }, 60000);
});

describe('normalising an opened project', () => {
  it('fills the arrays the editor relies on and leaves a complete project untouched', () => {
    const complete = createDefaultProject();
    expect(normalizeProject(complete)).toBe(complete);
    const broken = JSON.parse(JSON.stringify(complete)) as IProject;
    delete (broken.zones[0] as Partial<IProject['zones'][number]>).thermalBridges;
    delete (broken.zones[0].surfaces[0] as Partial<IProject['zones'][number]['surfaces'][number]>).windows;
    delete (broken as Partial<IProject>).solarThermal;
    const fixed = normalizeProject(broken);
    expect(fixed.zones[0].thermalBridges).toEqual([]);
    expect(fixed.zones[0].surfaces[0].windows).toEqual([]);
    expect(fixed.solarThermal).toEqual([]);
  });
});

function Thrower(): never {
  throw new Error('boom');
}

describe('error boundary', () => {
  it('shows a recoverable message instead of blanking the app', () => {
    const errors = console.error;
    console.error = () => {};
    try {
      renderWithProviders(<><ErrorBoundary><Thrower /></ErrorBoundary><p>still here</p></>);
    } finally {
      console.error = errors;
    }
    expect(screen.getByRole('alert')).toHaveTextContent('boom');
    expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
    expect(screen.getByText('still here')).toBeInTheDocument();
  });
});

describe('delete cascade labels', () => {
  it('names the zone and the kind of input instead of the JSON path', () => {
    const project = createDefaultProject();
    const zone = project.zones[0];
    const withNta = {
      ...project,
      ntaCalculation: { zoneData: [{ zoneId: zone.id }], lighting: [{ zoneId: zone.id }] },
      ntaHeatPumps: [{ id: 'hp', servedZoneIds: [zone.id] }],
    } as unknown as IProject;
    const { cascade } = deleteZoneFromProject(withNta, zone.id);
    const t = (key: string, params?: Record<string, unknown>) => {
      const texts: Record<string, string> = {
        'item.cascade.removed': '{{kind}} of {{name}}',
        'item.cascade.servedZone': '{{name}} from {{system}}',
        'item.cascadeKind.zoneData': 'zone data',
        'item.cascadeKind.lighting': 'lighting zone',
      };
      const text = texts[key] ?? key;
      return text.replace(/\{\{(\w+)\}\}/g, (_, name) => String(params?.[name] ?? ''));
    };
    const labels = cascade.map((entry) => describeCascade(entry, withNta, t));
    expect(labels).toContain(`zone data of ${zone.name}`);
    expect(labels).toContain(`lighting zone of ${zone.name}`);
    expect(labels).toContain(`${zone.name} from hp`);
    expect(labels.some((label) => label.includes('ntaCalculation'))).toBe(false);
  });
});

describe('template problems name the missing field', () => {
  it('lists the blank glazing U-value and PV tilt', () => {
    const project = createDefaultProject();
    const zone = project.zones[0];
    const surface = zone.surfaces.find((s) => s.windows.length > 0)!;
    const windowKey = `${zone.id}/${surface.id}/${surface.windows[0].id}`;
    const glazing = buildTemplatePatch(project, { kind: 'glazing', windows: [windowKey], uValue: null, gValue: null } as never, 'm1');
    expect(glazing.problems).toContain('valueRequired');
    expect(glazing.missingFields).toContain('uValue');
  });
});

describe('Dutch value texts', () => {
  it('keeps formula and table numbers but writes decimals with a comma', () => {
    expect(dutchDecimals('kernel default for a central system (11.109)')).toBe('kernel default for a central system (11.109)');
    expect(dutchDecimals('R_c 1.47 m²K/W (table 8.9)')).toBe('R_c 1,47 m²K/W (table 8.9)');
    expect(dutchDecimals('share (0.35)')).toBe('share (0,35)');
    expect(dutchSource('ISSO 82.1 p. 51 (positions 4/8); NTA table 11.14')).toBe('ISSO 82.1 p. 51 (posities 4/8); NTA tabel 11.14');
  });
});


describe('opening a project in the browser', () => {
  it('opens a project without a saved location as a new document', () => {
    const project = createDefaultProject();
    const one = documentManagerReducer({ documents: [], activeDocumentId: null },
      { type: 'DOC_OPEN', payload: { id: 'a', project, filePath: null } });
    const two = documentManagerReducer(one, { type: 'DOC_OPEN', payload: { id: 'b', project, filePath: null } });
    expect(two.documents.map((doc) => doc.id)).toEqual(['a', 'b']);
    expect(two.activeDocumentId).toBe('b');
    const same = documentManagerReducer(
      documentManagerReducer(two, { type: 'DOC_OPEN', payload: { id: 'c', project, filePath: '/x.oes.json' } }),
      { type: 'DOC_OPEN', payload: { id: 'd', project, filePath: '/x.oes.json' } },
    );
    expect(same.documents.map((doc) => doc.id)).toEqual(['a', 'b', 'c']);
  });
});
