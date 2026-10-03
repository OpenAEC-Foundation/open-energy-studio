import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import type { IProject } from '../core/energy/types';
import { buildMaatwerkadviesInput, type MwaMeasure, type NtaMaatwerkadvies } from '../core/nta/KernelClient';
import {
  applyLightingChanges, applyPatchOperation, buildTemplatePatch, diffOperations, heatPumpGeneratorTemplate, initialTemplate, insulationOptions,
  insulationValues, lightingChanges, measureEvidenceNotes, normalizeTemplate, pvSystemTemplate, PV_OBSTRUCTION_SOURCE,
  regenerateTemplatePatches, TEMPLATE_CATEGORY, TEMPLATE_LIFETIME, windowOptions,
  type LegacyTemplate, type MwaMeasureTemplate,
} from '../core/nta/MwaTemplates';
import { templateMeasure } from '../components/MaatwerkadviesPanel/MwaTemplateEditor';
import { solarWaterHeaterTemplate } from '../core/nta/NtaSystemTemplates';

// The measures below are generated from the templates against the two
// example projects and kept in training-data; the Rust test
// `template_measures_run_on_the_example_projects` (maatwerkadvies.rs) runs
// them through the kernel. Regenerate with MWA_WRITE_FIXTURE=1.
const FIXTURE = resolve(process.cwd(), 'training-data/nta8800-mwa-template-measures.json');

function load(name: string): IProject {
  return JSON.parse(readFileSync(resolve(process.cwd(), `training-data/${name}`), 'utf8')) as IProject;
}

type Block = Record<string, unknown>;

function measure(project: IProject, id: string, name: string, template: MwaMeasureTemplate): MwaMeasure {
  const { patch, problems } = buildTemplatePatch(project, template, id);
  expect(problems, `${id}: ${problems.join(', ')}`).toEqual([]);
  return {
    id, name, category: TEMPLATE_CATEGORY[template.kind], target: 'project', patch,
    investmentEur: 1000, costSource: 'test: kostenkentallen', lifetimeYears: TEMPLATE_LIFETIME[template.kind], template,
  };
}

/** Balanced ventilation with heat recovery η 0,9 (declared), as a ventilation template. */
function heatRecoveryVentilation(project: IProject): Extract<MwaMeasureTemplate, { kind: 'ventilation' }> {
  const ventilation = initialTemplate('ventilation', project) as Extract<MwaMeasureTemplate, { kind: 'ventilation' }>;
  const unit = (ventilation.system as Block).unit as Block;
  unit.equipmentReference = 'test: WTW-unit productblad';
  unit.ducts = 'luka_a_b_c';
  unit.heatRecovery = { ...(unit.heatRecovery as Block),
    efficiency: { method: 'declared', value: 0.9, standard: 'en13141_7', sourceReference: 'test: productblad' },
    supplyDuctInsulation: { kind: 'insulated' }, manufactureYear: 2026, equipmentReference: 'test: WTW-unit' };
  return ventilation;
}

function dwellingMeasures(project: IProject): MwaMeasure[] {
  const keys = (part: 'roof' | 'facade' | 'floor') => insulationOptions(project, part).map((option) => option.key);
  const ventilation = heatRecoveryVentilation(project);
  const generator = heatPumpGeneratorTemplate(project);
  generator.forfait = { ...(generator.forfait as Block), designSupplyTemperatureC: 45, classificationSourceReference: 'test: offerte',
    thermalCapacityKw: 6, capacitySourceReference: 'test: offerte' };
  const solar = solarWaterHeaterTemplate(0);
  const method = solar.method as Block;
  (method.collectors as Block).moduleAreaM2 = 2.5;
  (method.storage as Block).totalVolumeL = 150;
  solar.sourceReference = 'test: zonneboiler productblad';
  return [
    measure(project, 'roof', 'Dakisolatie Rc 8', { kind: 'insulation', part: 'roof', surfaces: keys('roof'), rcValue: 8, uValue: null }),
    measure(project, 'facade', 'Gevelisolatie Rc 6', { kind: 'insulation', part: 'facade', surfaces: keys('facade'), rcValue: 6, uValue: null }),
    measure(project, 'floor', 'Vloerisolatie Rc 5', { kind: 'insulation', part: 'floor', surfaces: keys('floor'), rcValue: 5, uValue: null }),
    measure(project, 'glazing', 'Triple glas', { kind: 'glazing', windows: windowOptions(project).map((option) => option.key), uValue: 0.7, gValue: 0.5 }),
    measure(project, 'air', 'Kierdichting', { kind: 'airtightness', qv10DmPerSM2: 0.25, sourceReference: 'test: streefwaarde' }),
    measure(project, 'vent', 'Balansventilatie met WTW', ventilation),
    measure(project, 'hp', 'Lucht/water-warmtepomp', { kind: 'heat_pump', generator: { ...generator, sourceSystemReference: 'test: individuele buitenunit' },
      renewable: { sourceBelow20C: true, exhaustAirSource: false, sourceReference: 'test: buitenlucht' } }),
    measure(project, 'dhw', 'Tapwaterwarmtepomp', { kind: 'hot_water', generator: { kind: 'heat_pump', exhaustAirSource: false, measuredClass: 'class4' } }),
    measure(project, 'pv', 'PV 8 × 400 Wp', { kind: 'pv', systems: [{
      id: 'pv-1', peakPower: { method: 'panels', panelPeakPowerW: 400, panelCount: 8 }, azimuthDeg: 180, tiltDeg: 35,
      mounting: 'moderately_ventilated', obstruction: { method: 'minimal' }, sourceReference: 'test: offerte',
      obstructionSourceReference: 'test: foto vrij dakvlak' }] }),
    measure(project, 'solar', 'Zonneboiler', { kind: 'solar_water_heater', systems: [solar] }),
    measure(project, 'shower', 'Douche-WTW', { kind: 'shower_heat_recovery', recovery: {
      showers: [{ unit: 'vertical' }], connection: 'mixer_and_heater', sourceReference: 'test: plattegrond' } }),
  ];
}

function officeMeasures(project: IProject): MwaMeasure[] {
  const current = (project.ntaCalculation as unknown as { lighting: Block[] }).lighting;
  const edited = structuredClone(current);
  const zone = ((edited[0] as Block).lightingZones as Block[])[0];
  zone.occupancy = { ...(zone.occupancy as Block), control: 'auto_on_auto_off' };
  zone.daylight = { method: 'forfait', daylightControl: true };
  zone.power = { method: 'installed', sourceReference: 'test: lichtplan LED', luminaires: [{ count: 100, power: { method: 'system', powerW: 18 } }] };
  const lighting: MwaMeasureTemplate = { kind: 'lighting', zones: lightingChanges(current, edited) };
  const generator = heatPumpGeneratorTemplate(project);
  generator.forfait = { ...(generator.forfait as Block), designSupplyTemperatureC: 45, classificationSourceReference: 'test: offerte',
    thermalCapacityKw: 40, capacitySourceReference: 'test: offerte' };
  return [
    measure(project, 'glazing', 'Triple glas', { kind: 'glazing', windows: windowOptions(project).map((option) => option.key), uValue: 0.8, gValue: null }),
    measure(project, 'roof', 'Dakisolatie Rc 8', { kind: 'insulation', part: 'roof', surfaces: insulationOptions(project, 'roof').map((o) => o.key), rcValue: 8, uValue: null }),
    measure(project, 'light', 'Aanwezigheids- en daglichtregeling', lighting),
    measure(project, 'hp', 'Lucht/water-warmtepomp', { kind: 'heat_pump', generator: { ...generator, sourceSystemReference: 'test: individuele buitenunit' },
      renewable: { sourceBelow20C: true, exhaustAirSource: false, sourceReference: 'test: buitenlucht' } }),
    measure(project, 'pv', 'PV 40 × 400 Wp', { kind: 'pv', systems: [{
      id: 'pv-1', peakPower: { method: 'panels', panelPeakPowerW: 400, panelCount: 40 }, azimuthDeg: 180, tiltDeg: 15,
      mounting: 'moderately_ventilated', obstruction: { method: 'minimal' }, sourceReference: 'test: offerte',
      obstructionSourceReference: 'test: foto vrij dakvlak' }] }),
    measure(project, 'air', 'Kierdichting', { kind: 'airtightness', qv10DmPerSM2: 0.2, sourceReference: 'test: streefwaarde' }),
    measure(project, 'vent', 'Balansventilatie met WTW', heatRecoveryVentilation(project)),
    // The forfait tap-water heat pump carries its storage in the table
    // efficiency, so the template removes the office's electric-boiler vessel.
    measure(project, 'dhw', 'Tapwaterwarmtepomp', { kind: 'hot_water', generator: { kind: 'heat_pump', exhaustAirSource: false, measuredClass: 'class4' } }),
  ];
}

describe('maatwerkadvies measure templates', () => {
  const dwelling = load('nta8800-example-terraced-dwelling.json');
  const office = load('nta8800-example-office.json');
  const generated = { terracedDwelling: dwellingMeasures(dwelling), office: officeMeasures(office) };

  it('match the fixture the kernel test runs', () => {
    if (process.env.MWA_WRITE_FIXTURE) writeFileSync(FIXTURE, `${JSON.stringify(generated, null, 2)}\n`);
    expect(JSON.parse(readFileSync(FIXTURE, 'utf8'))).toEqual(JSON.parse(JSON.stringify(generated)));
  });

  it('apply cleanly to their project, also all together', () => {
    for (const [project, measures] of [[dwelling, generated.terracedDwelling], [office, generated.office]] as const) {
      const all = structuredClone(project) as unknown;
      for (const item of measures) {
        const single = structuredClone(project) as unknown;
        for (const operation of item.patch) {
          expect(applyPatchOperation(single, operation), `${item.id} ${operation.path}`).toBeNull();
          expect(applyPatchOperation(all, operation), `${item.id} ${operation.path} (all)`).toBeNull();
        }
      }
    }
  });

  it('insulates with a new construction and the table C.2 resistances', () => {
    const values = insulationValues('facade', 4, null)!;
    expect(values.u).toBeCloseTo(1 / (0.13 + 4 + 0.04), 12);
    expect(insulationValues('roof', null, 0.2)!.rc).toBeCloseTo(5 - 0.10 - 0.04, 12);
    expect(insulationValues('floor', null, null)).toBeNull();
    const floor = generated.terracedDwelling.find((item) => item.id === 'floor')!;
    expect(floor.patch[0]).toMatchObject({ op: 'add', path: '/constructions/-', value: { id: 'floor-c-floor', rcValue: 5 } });
    expect(floor.patch.some((operation) => operation.path === '/ntaCalculation/groundFloors/0/constructionResistanceM2kPerW'
      && 'value' in operation && operation.value === 5.17)).toBe(true);
  });

  it('keeps infiltration with the airtightness measure in a package', () => {
    const vent = generated.terracedDwelling.find((item) => item.id === 'vent')!;
    expect(vent.patch.every((operation) => !operation.path.includes('infiltration'))).toBe(true);
    const air = generated.terracedDwelling.find((item) => item.id === 'air')!;
    const project = structuredClone(dwelling) as unknown as { ntaCalculation: { ventilation: Block } };
    for (const operation of [...air.patch, ...vent.patch]) expect(applyPatchOperation(project, operation)).toBeNull();
    expect(project.ntaCalculation.ventilation.infiltration).toMatchObject({ qv10DmPerSM2: 0.25 });
    expect((project.ntaCalculation.ventilation.system as Block).unit).toMatchObject({ variant: 'd2' });
  });

  it('flags blanks the kernel would refuse and handles the storage vessel', () => {
    const ventilation = heatRecoveryVentilation(office);
    const unit = (ventilation.system as Block).unit as Block;
    (unit.heatRecovery as Block).efficiency = { method: 'declared', value: null, standard: 'en13141_7', sourceReference: 'x' };
    expect(buildTemplatePatch(office, ventilation, 'm1').problems).toContain('valueRequired');
    const forfait = buildTemplatePatch(office, { kind: 'hot_water', generator: { kind: 'heat_pump', exhaustAirSource: false, measuredClass: 'class4' } }, 'm1');
    expect(forfait.problems).toEqual([]);
    expect(forfait.patch).toContainEqual({ op: 'replace', path: '/ntaCalculation/hotWater/storage', value: [] });
    const boiler = buildTemplatePatch(office, { kind: 'hot_water', generator: { kind: 'electric_boiler' } }, 'm1');
    expect(boiler.patch.some((operation) => operation.path.endsWith('/storage'))).toBe(false);
    const tested = buildTemplatePatch(office, { kind: 'hot_water', generator: { kind: 'heat_pump_en16147', profile: 'l',
      deliveredKwhPerDay: 11.655, inputKwhPerDay: null, exhaustAirSource: false, sourceReference: 'x' } }, 'm1');
    expect(tested.problems).toContain('valueRequired');
  });

  it('reports what keeps a template from a patch', () => {
    expect(buildTemplatePatch(dwelling, initialTemplate('insulation', dwelling), 'm1').problems)
      .toEqual(['valueRequired', 'selectionRequired']);
    expect(buildTemplatePatch(dwelling, initialTemplate('lighting', dwelling), 'm1').problems).toEqual(['lightingRequired']);
    expect(buildTemplatePatch(dwelling, { kind: 'airtightness', qv10DmPerSM2: 0.3, sourceReference: '' }, 'm1').problems)
      .toEqual(['sourceRequired']);
  });

  it('marks chosen parts that no longer exist and roofs steeper than 60°', () => {
    const template: MwaMeasureTemplate = { kind: 'insulation', part: 'facade', surfaces: ['z1/wall-N', 'z1/gone'], rcValue: 4, uValue: null };
    expect(buildTemplatePatch(dwelling, template, 'm1').problems).toEqual(['selectionStale']);
    expect(buildTemplatePatch(dwelling, { kind: 'glazing', windows: ['z1/x/y'], uValue: 1, gValue: null }, 'm1').problems)
      .toEqual(['selectionStale', 'selectionRequired']);
    // Table C.2 note 3: a roof steeper than 60° takes R_si 0,13.
    const roof = insulationOptions(dwelling, 'roof')[0];
    const surfaceId = roof.key.split('/')[1];
    const steep = structuredClone(dwelling);
    (steep.ntaCalculation as unknown as Block).surfaceTilts = [{ surfaceId, tiltDeg: 70, sourceReference: 'test' }];
    const { patch } = buildTemplatePatch(steep, { kind: 'insulation', part: 'roof', surfaces: [roof.key], rcValue: 6, uValue: null }, 'm1');
    expect(patch[0]).toMatchObject({ op: 'add', value: { uValue: Number((1 / (0.13 + 6 + 0.04)).toFixed(4)) } });
    expect(String((patch[0] as { value: Block }).value.id)).toMatch(/-steep$/);
    // A wall against the ground is not offered: its construction is not used (8.38).
    const ground = structuredClone(dwelling);
    ground.zones[0].surfaces[0].thermalBoundary = 'ground' as never;
    expect(insulationOptions(ground, 'facade').map((option) => option.key)).not.toContain(`z1/${dwelling.zones[0].surfaces[0].id}`);
  });

  it('blocks incomplete heat pumps and PV and derives the source flags', () => {
    const generator = heatPumpGeneratorTemplate(dwelling);
    const problems = buildTemplatePatch(dwelling, { kind: 'heat_pump', generator, renewable: { sourceBelow20C: true, exhaustAirSource: false, sourceReference: 'x' } }, 'm1').problems;
    expect(problems).toContain('supplyTemperatureRequired');
    const warm = { ...generator, forfait: { ...(generator.forfait as Block), source: 'collective20_to40_c', designSupplyTemperatureC: 45 } };
    const { patch } = buildTemplatePatch(dwelling, { kind: 'heat_pump', generator: warm,
      renewable: { sourceBelow20C: true, exhaustAirSource: false, sourceReference: 'x' } }, 'm1');
    expect(patch.find((operation) => operation.path === '/ntaCalculation/heatPumpRenewable'))
      .toMatchObject({ value: { sourceBelow20C: false, exhaustAirSource: false } });
    // PV starts on the conservative situation e); a blank peak power and situation a) without evidence block.
    const pv = pvSystemTemplate(0);
    expect(pv.obstruction).toEqual({ method: 'full' });
    expect(buildTemplatePatch(dwelling, { kind: 'pv', systems: [pv] }, 'm1').problems).toEqual(['peakPowerRequired']);
    const minimal = { ...pv, peakPower: { method: 'panels', panelPeakPowerW: 400, panelCount: 4 }, obstruction: { method: 'minimal' } };
    expect(buildTemplatePatch(dwelling, { kind: 'pv', systems: [minimal] }, 'm1').problems).toEqual(['obstructionEvidenceRequired']);
    const evidenced = buildTemplatePatch(dwelling, { kind: 'pv', systems: [{ ...minimal, obstructionSourceReference: 'foto' }] }, 'm1');
    expect(evidenced.problems).toEqual([]);
    expect(JSON.stringify(evidenced.patch)).not.toContain('obstructionSourceReference');
  });

  it('applies ventilation and lighting changes on top of later project changes', () => {
    const vent = generated.terracedDwelling.find((item) => item.id === 'vent')!;
    const changed = structuredClone(dwelling);
    const ventilation = (changed.ntaCalculation as unknown as { ventilation: Block }).ventilation;
    ventilation.changedLater = true;
    const { patch } = buildTemplatePatch(changed, vent.template!, 'vent');
    expect(patch.map((operation) => operation.path)).toEqual(['/ntaCalculation/ventilation/system']);
    // Lighting: a later area change of the base stays; only the measure members are patched.
    const light = generated.office.find((item) => item.id === 'light')!;
    const later = structuredClone(office);
    const lightingZone = ((later.ntaCalculation as unknown as { lighting: Block[] }).lighting[0].lightingZones as Block[])[0];
    lightingZone.areaM2 = 123;
    const result = buildTemplatePatch(later, light.template!, 'light');
    expect(result.problems).toEqual([]);
    expect(result.patch.every((operation) => /\/(power|occupancy|daylight)$/.test(operation.path))).toBe(true);
    const applied = applyLightingChanges((later.ntaCalculation as unknown as { lighting: Block[] }).lighting,
      (light.template as Extract<MwaMeasureTemplate, { kind: 'lighting' }>).zones);
    expect((applied[0].lightingZones as Block[])[0].areaM2).toBe(123);
  });

  it('migrates saved snapshot templates', () => {
    const legacyVentilation = { kind: 'ventilation', ventilation: { system: { kind: 'single', unit: { variant: 'd2' } }, other: 1 } } as LegacyTemplate;
    const vent = buildTemplatePatch(dwelling, legacyVentilation, 'm1');
    expect(vent.problems).toEqual([]);
    expect(vent.patch).toEqual([{ op: 'replace', path: '/ntaCalculation/ventilation/system', value: { kind: 'single', unit: { variant: 'd2' } } }]);
    const current = (office.ntaCalculation as unknown as { lighting: Block[] }).lighting;
    const snapshot = structuredClone(current);
    ((snapshot[0].lightingZones as Block[])[0]).occupancy = { control: 'auto_on_auto_off', centralOnControl: false };
    const light = buildTemplatePatch(office, { kind: 'lighting', lighting: snapshot } as LegacyTemplate, 'm1');
    expect(light.problems).toEqual(['migrationReview']);
    expect(light.patch.map((operation) => operation.path)).toEqual(['/ntaCalculation/lighting/0/lightingZones/0/occupancy']);

    // Editing the migrated measure keeps the review open; only the
    // adviser's confirmation (`reviewed: true`) clears it.
    const { template: migrated } = normalizeTemplate(office, { kind: 'lighting', lighting: snapshot } as LegacyTemplate);
    expect(migrated).toMatchObject({ kind: 'lighting', reviewed: false });
    const edited = { ...(migrated as Extract<MwaMeasureTemplate, { kind: 'lighting' }>), zones: {} };
    expect(buildTemplatePatch(office, edited, 'm1').problems).toContain('migrationReview');
    expect(buildTemplatePatch(office, { ...edited, reviewed: true }, 'm1').problems).not.toContain('migrationReview');

    // A ventilation snapshot without a system never writes `{}`.
    const empty = buildTemplatePatch(dwelling, { kind: 'ventilation', ventilation: { other: 1 } } as LegacyTemplate, 'm1');
    expect(empty.problems).toEqual(['ventilationSystemRequired']);
    expect(empty.patch).toEqual([]);
  });

  it('keeps the minimal-obstruction evidence of PV for the report and dossier (table 17.3 a)', () => {
    const system = { ...pvSystemTemplate(0), obstruction: { method: 'minimal' }, [PV_OBSTRUCTION_SOURCE]: 'foto zuiddak' };
    const measure = { template: { kind: 'pv' as const, systems: [system, pvSystemTemplate(1)] } };
    expect(measureEvidenceNotes(measure)).toEqual([{ id: 'pv-1', source: 'foto zuiddak' }]);
    expect(measureEvidenceNotes({ template: { kind: 'pv', systems: [pvSystemTemplate(0)] } })).toEqual([]);
  });

  it('marks incomplete measures for the kernel and regenerates in the kernel input', () => {
    const definition = {
      measures: [{ id: 'm1', name: 'PV', category: 'pv', target: 'project', patch: [], investmentEur: 0, costSource: 'x',
        lifetimeYears: 25, template: { kind: 'pv', systems: [pvSystemTemplate(0)] } }],
      packages: [], tariffs: { gasEurPerM3: 1, electricityEurPerKwh: 0.3, sourceReference: 'x' },
    } as unknown as NtaMaatwerkadvies;
    expect(regenerateTemplatePatches(dwelling, definition).measures[0].incomplete).toEqual(['peakPowerRequired']);
    const input = buildMaatwerkadviesInput(dwelling, definition) as { measures: MwaMeasure[] };
    expect(input.measures[0].incomplete).toEqual(['peakPowerRequired']);
    expect(input.measures[0].patch[0]).toMatchObject({ op: 'add', path: '/ntaCalculation/pvSystems/-' });
  });

  it('keeps a lifetime and category the adviser entered when the kind changes', () => {
    const base: MwaMeasure = { id: 'm1', name: '', category: 'insulation', target: 'project', patch: [], investmentEur: 0, costSource: '', lifetimeYears: 30 };
    const insulation = templateMeasure(dwelling, base, 'insulation');
    expect(insulation.lifetimeYears).toBe(TEMPLATE_LIFETIME.insulation);
    const glazing = templateMeasure(dwelling, insulation, 'glazing');
    expect(glazing).toMatchObject({ category: 'glazing', lifetimeYears: TEMPLATE_LIFETIME.glazing });
    const custom = templateMeasure(dwelling, { ...glazing, lifetimeYears: 27, category: 'other' }, 'pv');
    expect(custom).toMatchObject({ category: 'other', lifetimeYears: 27 });
  });

  it('adds systems under the measure id and diffs only changed members', () => {
    const pv = generated.terracedDwelling.find((item) => item.id === 'pv')!;
    expect(pv.patch).toEqual([expect.objectContaining({ op: 'add', path: '/ntaCalculation/pvSystems/-', value: expect.objectContaining({ id: 'pv-pv-1' }) })]);
    expect(diffOperations('/a', { x: 1, y: { z: 2 } }, { x: 1, y: { z: 3 }, w: 4 }, 2))
      .toEqual([{ op: 'replace', path: '/a/y/z', value: 3 }, { op: 'add', path: '/a/w', value: 4 }]);
  });
});
