import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import type { IProject } from '../core/energy/types';
import type { MwaMeasure } from '../core/nta/KernelClient';
import {
  applyPatchOperation, buildTemplatePatch, diffOperations, heatPumpGeneratorTemplate, initialTemplate, insulationOptions,
  insulationValues, TEMPLATE_CATEGORY, TEMPLATE_LIFETIME, windowOptions, type MwaMeasureTemplate,
} from '../core/nta/MwaTemplates';
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

function dwellingMeasures(project: IProject): MwaMeasure[] {
  const keys = (part: 'roof' | 'facade' | 'floor') => insulationOptions(project, part).map((option) => option.key);
  const ventilation = initialTemplate('ventilation', project) as Extract<MwaMeasureTemplate, { kind: 'ventilation' }>;
  const unit = (ventilation.ventilation.system as Block).unit as Block;
  unit.equipmentReference = 'test: WTW-unit productblad';
  unit.ducts = 'luka_a_b_c';
  unit.heatRecovery = { ...(unit.heatRecovery as Block),
    efficiency: { method: 'declared', value: 0.9, standard: 'en13141_7', sourceReference: 'test: productblad' },
    supplyDuctInsulation: { kind: 'insulated' }, manufactureYear: 2026, equipmentReference: 'test: WTW-unit' };
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
      mounting: 'moderately_ventilated', obstruction: { method: 'minimal' }, sourceReference: 'test: offerte' }] }),
    measure(project, 'solar', 'Zonneboiler', { kind: 'solar_water_heater', systems: [solar] }),
    measure(project, 'shower', 'Douche-WTW', { kind: 'shower_heat_recovery', recovery: {
      showers: [{ unit: 'vertical' }], connection: 'mixer_and_heater', sourceReference: 'test: plattegrond' } }),
  ];
}

function officeMeasures(project: IProject): MwaMeasure[] {
  const lighting = initialTemplate('lighting', project) as Extract<MwaMeasureTemplate, { kind: 'lighting' }>;
  const zone = ((lighting.lighting[0] as Block).lightingZones as Block[])[0];
  zone.occupancy = { ...(zone.occupancy as Block), control: 'auto_on_auto_off' };
  zone.daylight = { method: 'forfait', daylightControl: true };
  zone.power = { method: 'installed', sourceReference: 'test: lichtplan LED', luminaires: [{ count: 100, power: { method: 'system', powerW: 18 } }] };
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
      mounting: 'moderately_ventilated', obstruction: { method: 'minimal' }, sourceReference: 'test: offerte' }] }),
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

  it('reports what keeps a template from a patch', () => {
    expect(buildTemplatePatch(dwelling, initialTemplate('insulation', dwelling), 'm1').problems)
      .toEqual(['valueRequired', 'selectionRequired']);
    expect(buildTemplatePatch(dwelling, initialTemplate('lighting', dwelling), 'm1').problems).toEqual(['lightingRequired']);
    expect(buildTemplatePatch(dwelling, { kind: 'airtightness', qv10DmPerSM2: 0.3, sourceReference: '' }, 'm1').problems)
      .toEqual(['sourceRequired']);
  });

  it('adds systems under the measure id and diffs only changed members', () => {
    const pv = generated.terracedDwelling.find((item) => item.id === 'pv')!;
    expect(pv.patch).toEqual([expect.objectContaining({ op: 'add', path: '/ntaCalculation/pvSystems/-', value: expect.objectContaining({ id: 'pv-pv-1' }) })]);
    expect(diffOperations('/a', { x: 1, y: { z: 2 } }, { x: 1, y: { z: 3 }, w: 4 }, 2))
      .toEqual([{ op: 'replace', path: '/a/y/z', value: 3 }, { op: 'add', path: '/a/w', value: 4 }]);
  });
});
