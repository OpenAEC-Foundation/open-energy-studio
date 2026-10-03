import type { IProject } from '../energy/types';
import type { MwaMeasureCategory, MwaPatchOperation } from './KernelClient';
import { hotWaterGeneratorTemplate, solarWaterHeaterTemplate, spaceGeneratorTemplate } from './NtaSystemTemplates';
import { heatRecoveryTemplate, ventilationUnit } from './NtaFormModels';

// Maatwerkadvies measure templates (ISSO 82.2 §4.3, p. 54–62; ISSO 75.2
// §4.3, p. 65–78, lighting on p. 78). A template holds the adviser's
// choices and generates the RFC 6902 patch the kernel applies to the
// project (`apply_patch`, target `project`). The patch is regenerated from
// the current project, so array indices in its paths stay valid when the
// project changes.

type Block = Record<string, unknown>;

export type InsulationPart = 'roof' | 'facade' | 'floor';

export type MwaMeasureTemplate =
  /** Pick surfaces; new Rc (U follows) or new U. */
  | { kind: 'insulation'; part: InsulationPart; surfaces: string[]; rcValue: number | null; uValue: number | null }
  /** Pick windows; new U and, optionally, g. */
  | { kind: 'glazing'; windows: string[]; uValue: number | null; gValue: number | null }
  /** Target q_v10 after sealing (to be verified by a measurement after execution). */
  | { kind: 'airtightness'; qv10DmPerSM2: number | null; sourceReference: string }
  /** Chapter 11 ventilation: the edited `ventilation` block (system, fans, …). */
  | { kind: 'ventilation'; ventilation: Block }
  /** Space-heating generator replacing the current one (forfait or annex Q product route). */
  | { kind: 'heat_pump'; generator: Block; renewable?: Block | null }
  /** Hot-water generator replacing the current one. */
  | { kind: 'hot_water'; generator: Block }
  /** PV systems added to the building (chapter 16). */
  | { kind: 'pv'; systems: Block[] }
  /** Solar water heaters added to the hot-water system (§13.7). */
  | { kind: 'solar_water_heater'; systems: Block[] }
  /** Shower heat recovery (annex U / §13.6) on the hot-water system. */
  | { kind: 'shower_heat_recovery'; recovery: Block }
  /** Chapter 14 lighting of a utility building: the edited `lighting` list. */
  | { kind: 'lighting'; lighting: Block[] };

export type MwaTemplateKind = MwaMeasureTemplate['kind'];

export const TEMPLATE_KINDS: MwaTemplateKind[] = [
  'insulation', 'glazing', 'airtightness', 'ventilation', 'heat_pump', 'hot_water', 'pv',
  'solar_water_heater', 'shower_heat_recovery', 'lighting',
];

export const TEMPLATE_CATEGORY: Record<MwaTemplateKind, MwaMeasureCategory> = {
  insulation: 'insulation',
  glazing: 'glazing',
  airtightness: 'airtightness',
  ventilation: 'ventilation',
  heat_pump: 'heat_pump',
  hot_water: 'hot_water',
  pv: 'pv',
  solar_water_heater: 'solar_thermal',
  shower_heat_recovery: 'hot_water',
  lighting: 'lighting',
};

/**
 * Starting lifetime in years for a new measure. ISSO 82.2/75.2 give no
 * lifetime table (only boilers: 15–20 years, 82.2 p. 90); these are
 * editable suggestions, not norm values.
 */
export const TEMPLATE_LIFETIME: Record<MwaTemplateKind, number> = {
  insulation: 40,
  glazing: 30,
  airtightness: 20,
  ventilation: 15,
  heat_pump: 15,
  hot_water: 15,
  pv: 25,
  solar_water_heater: 20,
  shower_heat_recovery: 20,
  lighting: 15,
};

/** Table C.2 surface resistances: R_si by heat-flow direction, R_se 0,04. */
const R_SI: Record<InsulationPart, number> = { roof: 0.10, facade: 0.13, floor: 0.17 };
const R_SE = 0.04;
const SURFACE_TYPE: Record<InsulationPart, string> = { roof: 'roof', facade: 'wall', floor: 'floor' };
const OUTER_BOUNDARIES = new Set(['outdoor', 'ground', 'unheated_space', undefined]);

function clone<T>(value: T): T {
  return value === undefined ? value : structuredClone(value);
}

function nta(project: IProject): Block | undefined {
  return project.ntaCalculation as unknown as Block | undefined;
}

/** JSON pointer segment (RFC 6901). */
function seg(key: string | number): string {
  return String(key).replace(/~/g, '~0').replace(/\//g, '~1');
}

function pointer(...keys: Array<string | number>): string {
  return keys.map((key) => `/${seg(key)}`).join('');
}

export interface SurfaceOption { key: string; label: string; zoneIndex: number; surfaceIndex: number }
export interface WindowOption extends SurfaceOption { windowIndex: number }

/** Surfaces a part can insulate: walls, roofs or floors on the outside, ground or an unheated space. */
export function insulationOptions(project: IProject, part: InsulationPart): SurfaceOption[] {
  return project.zones.flatMap((zone, zoneIndex) => zone.surfaces.flatMap((surface, surfaceIndex) =>
    surface.type === SURFACE_TYPE[part] && OUTER_BOUNDARIES.has(surface.thermalBoundary)
      ? [{ key: `${zone.id}/${surface.id}`, label: `${zone.name || zone.id} — ${surface.name || surface.id}`, zoneIndex, surfaceIndex }]
      : []));
}

export function windowOptions(project: IProject): WindowOption[] {
  return project.zones.flatMap((zone, zoneIndex) => zone.surfaces.flatMap((surface, surfaceIndex) =>
    (surface.windows ?? []).map((window, windowIndex) => ({
      key: `${zone.id}/${surface.id}/${window.id}`,
      label: `${zone.name || zone.id} — ${surface.name || surface.id} — ${window.name || window.id}`,
      zoneIndex, surfaceIndex, windowIndex,
    }))));
}

/** Rc and U of an insulated part: U = 1/(R_si + Rc + R_se), or Rc back from U. */
export function insulationValues(part: InsulationPart, rcValue: number | null, uValue: number | null): { rc: number; u: number } | null {
  if (rcValue != null && Number.isFinite(rcValue) && rcValue >= 0) {
    return { rc: rcValue, u: 1 / (R_SI[part] + rcValue + R_SE) };
  }
  if (uValue != null && Number.isFinite(uValue) && uValue > 0) {
    return { rc: Math.max(0, 1 / uValue - R_SI[part] - R_SE), u: uValue };
  }
  return null;
}

const round = (value: number, digits: number) => Number(value.toFixed(digits));

/** A new measure template, started from the project where that helps. */
export function initialTemplate(kind: MwaTemplateKind, project: IProject): MwaMeasureTemplate {
  const block = nta(project);
  switch (kind) {
    case 'insulation': return { kind, part: 'facade', surfaces: [], rcValue: null, uValue: null };
    case 'glazing': return { kind, windows: [], uValue: null, gValue: null };
    case 'airtightness': return { kind, qv10DmPerSM2: null, sourceReference: '' };
    case 'ventilation': {
      const current = (block?.ventilation as Block | undefined) ?? {};
      const unit = { ...ventilationUnit('d2', ((current.system as Block | undefined)?.unit as Block | undefined)), heatRecovery: heatRecoveryTemplate() };
      return { kind, ventilation: { ...clone(current), system: { kind: 'single', unit } } };
    }
    case 'heat_pump': return { kind, generator: heatPumpGeneratorTemplate(project), renewable: heatPumpRenewableTemplate() };
    case 'hot_water': return { kind, generator: hotWaterGeneratorTemplate('heat_pump') };
    case 'pv': return { kind, systems: [pvSystemTemplate(0)] };
    case 'solar_water_heater': return { kind, systems: [solarWaterHeaterTemplate(0)] };
    case 'shower_heat_recovery': return { kind, recovery: {
      showers: [{ unit: 'vertical' }], connection: 'mixer_and_heater', sourceReference: '' } };
    case 'lighting': return { kind, lighting: clone((block?.lighting as Block[] | undefined) ?? []) };
  }
}

/** Generator kinds whose renewable heat needs 5.31/5.32 evidence (`heatPumpRenewable`). */
export const HEAT_PUMP_KINDS = new Set(['heat_pump_forfait', 'heat_pump_annex_q', 'hybrid_heat_pump', 'gas_heat_pump']);

/** 5.31/5.32 source evidence of a heat pump. */
export function heatPumpRenewableTemplate(): Block {
  return { sourceBelow20C: true, exhaustAirSource: false, sourceReference: '' };
}

/** A forfait heat pump (tables 9.27/9.29) with its table row inputs; the adviser completes them. */
export function forfaitHeatPumpTemplate(project: IProject): Block {
  const residential = (nta(project)?.calculationScope ?? 'residential') === 'residential';
  return {
    generatorId: 'heat-pump',
    classificationSourceReference: '',
    scope: residential ? 'residential_at_most25_kw' : 'utility_collective_or_over25_kw',
    source: 'outdoor_air',
    sink: 'hydronic',
    designSupplyTemperatureC: null,
    sourceCorrectionFactor: null,
    sourceCorrectionReference: null,
  };
}

export function heatPumpGeneratorTemplate(project: IProject): Block {
  const generator = spaceGeneratorTemplate('heat_pump_forfait', project) ?? { kind: 'heat_pump_forfait' };
  return { ...generator, forfait: generator.forfait ?? forfaitHeatPumpTemplate(project), sourceSystemReference: '' };
}

export function pvSystemTemplate(index: number): Block {
  return {
    id: `pv-${index + 1}`,
    peakPower: { method: 'panels', panelPeakPowerW: null, panelCount: null },
    azimuthDeg: 180,
    tiltDeg: 35,
    mounting: 'moderately_ventilated',
    obstruction: { method: 'minimal' },
    sourceReference: '',
  };
}

const isObject = (value: unknown): value is Block => typeof value === 'object' && value !== null && !Array.isArray(value);

/**
 * The replace/add/remove operations that turn `before` into `after` at
 * `path`, recursing `depth` levels into objects (and equally long arrays).
 */
export function diffOperations(path: string, before: unknown, after: unknown, depth: number): MwaPatchOperation[] {
  if (JSON.stringify(before) === JSON.stringify(after)) return [];
  if (before === undefined) return after === undefined ? [] : [{ op: 'add', path, value: after }];
  if (after === undefined) return [{ op: 'remove', path }];
  if (depth > 0 && isObject(before) && isObject(after)) {
    const keys = [...new Set([...Object.keys(before), ...Object.keys(after)])];
    return keys.flatMap((key) => diffOperations(`${path}/${seg(key)}`, before[key], after[key], depth - 1));
  }
  if (depth > 0 && Array.isArray(before) && Array.isArray(after) && before.length === after.length) {
    return before.flatMap((item, index) => diffOperations(`${path}/${index}`, item, after[index], depth - 1));
  }
  return [{ op: 'replace', path, value: after }];
}

export interface TemplatePatch {
  patch: MwaPatchOperation[];
  /** i18n keys (mwa.template.problem.*) of what keeps the template from a complete patch. */
  problems: string[];
}

/** Appends `items` to the list at `path`, creating it when absent. */
function append(path: string, current: unknown, items: Block[]): MwaPatchOperation[] {
  if (items.length === 0) return [];
  if (!Array.isArray(current)) return [{ op: 'add', path, value: items }];
  return items.map((item) => ({ op: 'add', path: `${path}/-`, value: item }));
}

/**
 * §17.3 obstruction of a PV system: entered factors (16.2) replace the
 * situation (exclusive in the kernel); without factors the situation stays.
 */
function pvObstruction(system: Block): Block {
  const factors = Array.isArray(system.obstructionFactors) ? system.obstructionFactors : [];
  const { obstruction, obstructionFactors: _drop, ...rest } = system;
  void _drop;
  if (factors.length > 0) return { ...rest, obstructionFactors: factors };
  return obstruction == null ? rest : { ...rest, obstruction };
}

/** Prefixes the ids of added systems with the measure id, so they never clash with existing ones. */
function withIds(items: Block[], measureId: string, prefix: string): Block[] {
  return items.map((item, index) => ({ ...item, id: `${measureId}-${String(item.id ?? `${prefix}-${index + 1}`)}` }));
}

/** The patch of a template against the current project. */
export function buildTemplatePatch(project: IProject, template: MwaMeasureTemplate, measureId: string): TemplatePatch {
  const block = nta(project);
  const problems: string[] = [];
  const patch: MwaPatchOperation[] = [];
  switch (template.kind) {
    case 'insulation': {
      const values = insulationValues(template.part, template.rcValue, template.uValue);
      if (!values) problems.push('valueRequired');
      const options = insulationOptions(project, template.part).filter((option) => template.surfaces.includes(option.key));
      if (options.length === 0) problems.push('selectionRequired');
      if (!values) break;
      const rc = round(values.rc, 3);
      const u = round(values.u, 4);
      const created = new Map<string, string>();
      const groundFloors = (block?.groundFloors as Block[] | undefined) ?? [];
      for (const option of options) {
        const surface = project.zones[option.zoneIndex].surfaces[option.surfaceIndex];
        const original = project.constructions.find((item) => item.id === surface.constructionId);
        const key = surface.constructionId ?? '';
        let id = created.get(key);
        if (!id) {
          id = `${measureId}-${key || template.part}`;
          created.set(key, id);
          patch.push({ op: 'add', path: '/constructions/-', value: {
            id, name: `${original?.name ?? template.part} (${measureId})`, layers: [], rcValue: rc, uValue: u } });
        }
        patch.push({ op: 'replace', path: pointer('zones', option.zoneIndex, 'surfaces', option.surfaceIndex, 'constructionId'), value: id });
        // A floor on the ground or above a crawlspace/basement is
        // calculated from its R_si + R_c (§8.3, 8.43).
        const groundIndex = groundFloors.findIndex((item) => item.surfaceId === surface.id);
        if (groundIndex >= 0) {
          patch.push({ op: 'replace', path: pointer('ntaCalculation', 'groundFloors', groundIndex, 'constructionResistanceM2kPerW'),
            value: round(rc + R_SI.floor, 3) });
        }
      }
      break;
    }
    case 'glazing': {
      const options = windowOptions(project).filter((option) => template.windows.includes(option.key));
      if (options.length === 0) problems.push('selectionRequired');
      const u = template.uValue;
      if (u == null || !Number.isFinite(u) || u <= 0) { problems.push('valueRequired'); break; }
      for (const option of options) {
        const base = ['zones', option.zoneIndex, 'surfaces', option.surfaceIndex, 'windows', option.windowIndex] as const;
        patch.push({ op: 'replace', path: pointer(...base, 'uValue'), value: u });
        if (template.gValue != null && Number.isFinite(template.gValue)) {
          patch.push({ op: 'replace', path: pointer(...base, 'gValue'), value: template.gValue });
        }
      }
      break;
    }
    case 'airtightness': {
      const qv10 = template.qv10DmPerSM2;
      if (qv10 == null || !Number.isFinite(qv10) || qv10 <= 0) problems.push('valueRequired');
      if (!template.sourceReference.trim()) problems.push('sourceRequired');
      const infiltration = { method: 'measured', qv10DmPerSM2: qv10, sourceReference: template.sourceReference };
      const targets: string[] = [];
      if (block?.ventilation) targets.push(pointer('ntaCalculation', 'ventilation', 'infiltration'));
      ((block?.zoneData as Block[] | undefined) ?? []).forEach((zone, index) => {
        if (zone.ventilation) targets.push(pointer('ntaCalculation', 'zoneData', index, 'ventilation', 'infiltration'));
      });
      if (targets.length === 0) problems.push('ventilationRequired');
      if (problems.length > 0) break;
      for (const path of targets) patch.push({ op: 'replace', path, value: infiltration });
      project.zones.forEach((zone, index) => {
        const tightness = (zone as unknown as Block).airTightness as Block | undefined;
        if (tightness && 'qv10' in tightness) patch.push({ op: 'replace', path: pointer('zones', index, 'airTightness', 'qv10'), value: qv10 });
      });
      break;
    }
    case 'ventilation': {
      const current = block?.ventilation as Block | undefined;
      if (!current) { problems.push('ventilationRequired'); break; }
      // Infiltration belongs to the airtightness template; a package with
      // both keeps the sealed q_v10.
      const { infiltration: _skip, ...after } = template.ventilation;
      void _skip;
      const { infiltration: _keep, ...before } = current;
      void _keep;
      patch.push(...diffOperations(pointer('ntaCalculation', 'ventilation'), before, after, 1));
      ((block?.zoneData as Block[] | undefined) ?? []).forEach((zone, index) => {
        const ventilation = zone.ventilation as Block | undefined;
        if (ventilation && after.system !== undefined && JSON.stringify(ventilation.system) !== JSON.stringify(after.system)) {
          patch.push({ op: 'replace', path: pointer('ntaCalculation', 'zoneData', index, 'ventilation', 'system'), value: after.system });
        }
      });
      if (patch.length === 0) problems.push('noChange');
      break;
    }
    case 'heat_pump': {
      if (!block) { problems.push('calculationRequired'); break; }
      let generator = template.generator;
      const forfait = generator.forfait as Block | null | undefined;
      // An individual heat pump with a stated capacity is not part of a
      // collective building installation (9.6.3 table row choice).
      if (forfait && forfait.thermalCapacityKw != null && forfait.collectiveBuildingInstallation == null) {
        generator = { ...generator, forfait: { ...forfait, collectiveBuildingInstallation: generator.sourceSystem !== 'individual' } };
      }
      patch.push({ op: 'replace', path: pointer('ntaCalculation', 'generator'), value: generator });
      // 5.31/5.32: a heat pump's renewable heat needs the source evidence.
      if (HEAT_PUMP_KINDS.has(String(generator.kind))) {
        const renewable = template.renewable;
        if (!renewable || !String(renewable.sourceReference ?? '').trim()) problems.push('sourceRequired');
        if (renewable) {
          patch.push({ op: block.heatPumpRenewable == null ? 'add' : 'replace', path: pointer('ntaCalculation', 'heatPumpRenewable'), value: renewable });
        }
      }
      break;
    }
    case 'hot_water': {
      if (!block?.hotWater) { problems.push('hotWaterRequired'); break; }
      patch.push({ op: 'replace', path: pointer('ntaCalculation', 'hotWater', 'generator'), value: template.generator });
      break;
    }
    case 'pv': {
      if (!block) { problems.push('calculationRequired'); break; }
      if (template.systems.length === 0) problems.push('selectionRequired');
      patch.push(...append(pointer('ntaCalculation', 'pvSystems'), block.pvSystems, withIds(template.systems.map(pvObstruction), measureId, 'pv')));
      break;
    }
    case 'solar_water_heater': {
      const hotWater = block?.hotWater as Block | undefined;
      if (!hotWater) { problems.push('hotWaterRequired'); break; }
      if (template.systems.length === 0) problems.push('selectionRequired');
      patch.push(...append(pointer('ntaCalculation', 'hotWater', 'solar'), hotWater.solar, withIds(template.systems, measureId, 'solar')));
      break;
    }
    case 'shower_heat_recovery': {
      const hotWater = block?.hotWater as Block | undefined;
      if (!hotWater) { problems.push('hotWaterRequired'); break; }
      if (!String(template.recovery.sourceReference ?? '').trim()) problems.push('sourceRequired');
      patch.push({ op: hotWater.showerHeatRecovery == null ? 'add' : 'replace',
        path: pointer('ntaCalculation', 'hotWater', 'showerHeatRecovery'), value: template.recovery });
      break;
    }
    case 'lighting': {
      const current = block?.lighting as Block[] | undefined;
      if (!current || current.length === 0) { problems.push('lightingRequired'); break; }
      // Per lighting zone: the changed power, occupancy, daylight, … blocks.
      patch.push(...diffOperations(pointer('ntaCalculation', 'lighting'), current, template.lighting, 4));
      if (patch.length === 0) problems.push('noChange');
      break;
    }
  }
  return { patch, problems };
}

/** RFC 6902 subset as the kernel's `apply_patch` (maatwerkadvies.rs), for previews and tests. */
export function applyPatchOperation(target: unknown, operation: MwaPatchOperation): string | null {
  const path = operation.path;
  if (!path.startsWith('/')) return `path must start with '/': ${path}`;
  const keys = path.slice(1).split('/').map((key) => key.replace(/~1/g, '/').replace(/~0/g, '~'));
  const last = keys.pop() as string;
  let container: unknown = target;
  for (const key of keys) {
    if (Array.isArray(container)) container = container[Number(key)];
    else if (isObject(container)) container = container[key];
    else return `parent not found: ${path}`;
    if (container === undefined) return `parent not found: ${path}`;
  }
  if (operation.op === 'replace') {
    if (Array.isArray(container)) {
      const index = Number(last);
      if (!(index in container)) return `path not found: ${path}`;
      container[index] = operation.value;
      return null;
    }
    if (!isObject(container) || !(last in container)) return `path not found: ${path}`;
    container[last] = operation.value;
    return null;
  }
  if (operation.op === 'add') {
    if (Array.isArray(container)) {
      if (last === '-') { container.push(operation.value); return null; }
      const index = Number(last);
      if (!Number.isInteger(index) || index > container.length) return `index out of range: ${path}`;
      container.splice(index, 0, operation.value);
      return null;
    }
    if (!isObject(container)) return `parent is not a container: ${path}`;
    container[last] = operation.value;
    return null;
  }
  if (Array.isArray(container)) {
    const index = Number(last);
    if (!(index in container)) return `path not found: ${path}`;
    container.splice(index, 1);
    return null;
  }
  if (!isObject(container) || !(last in container)) return `path not found: ${path}`;
  delete container[last];
  return null;
}
