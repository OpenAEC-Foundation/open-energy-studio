import type { IProject } from '../energy/types';
import type { MwaMeasure, MwaMeasureCategory, MwaPatchOperation, NtaMaatwerkadvies } from './KernelClient';
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
  /**
   * Chapter 11 ventilation: the new `ventilation.system` (unit, heat
   * recovery, fans). Only the system is stored, so later changes to the
   * project's flows, controls or infiltration stay in the variant.
   */
  | { kind: 'ventilation'; system: Block }
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
  /**
   * Chapter 14 lighting of a utility building: per lighting zone
   * (`zoneId/lightingZoneId`) only the changed members (power, parasitic,
   * occupancy, daylight, extracted luminaires), applied on top of the
   * project's current lighting. `reviewed: false` marks a measure migrated
   * from a snapshot; only the adviser's confirmation sets it to true.
   */
  | { kind: 'lighting'; zones: Record<string, Block>; reviewed?: boolean };

/**
 * Template forms saved before 3 October 2026: ventilation and lighting kept
 * a snapshot of the whole block. `normalizeTemplate` migrates them.
 */
export type LegacyTemplate =
  | { kind: 'ventilation'; ventilation: Block }
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

/**
 * Table C.2 surface resistances (p. 778): R_si by heat-flow direction,
 * R_se 0,04. Note 3: a roof steeper than 60° counts as horizontal heat flow.
 */
const R_SI: Record<InsulationPart, number> = { roof: 0.10, facade: 0.13, floor: 0.17 };
const R_SI_STEEP_ROOF = 0.13;
const R_SE = 0.04;
const SURFACE_TYPE: Record<InsulationPart, string> = { roof: 'roof', facade: 'wall', floor: 'floor' };
/**
 * Boundaries a part can insulate. A wall against the ground is left out:
 * the kernel takes a heated basement's walls from
 * `groundFloors[].heatedBasement.wallResistanceM2kPerW` (8.38), not from the
 * wall's construction.
 */
const OUTER_BOUNDARIES: Record<InsulationPart, Set<string | undefined>> = {
  roof: new Set(['outdoor', 'unheated_space', undefined]),
  facade: new Set(['outdoor', 'unheated_space', undefined]),
  floor: new Set(['outdoor', 'ground', 'unheated_space', undefined]),
};

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
    surface.type === SURFACE_TYPE[part] && OUTER_BOUNDARIES[part].has(surface.thermalBoundary)
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

/** R_si of a surface of `part`: a roof steeper than 60° takes 0,13 (table C.2 note 3). */
export function insulationRsi(project: IProject, part: InsulationPart, surfaceId?: string): number {
  if (part !== 'roof' || !surfaceId) return R_SI[part];
  const tilts = (nta(project)?.surfaceTilts as Block[] | undefined) ?? [];
  const tilt = tilts.find((item) => item.surfaceId === surfaceId)?.tiltDeg;
  return typeof tilt === 'number' && tilt > 60 ? R_SI_STEEP_ROOF : R_SI[part];
}

/** Rc and U of an insulated part: U = 1/(R_si + Rc + R_se), or Rc back from U. */
export function insulationValues(part: InsulationPart, rcValue: number | null, uValue: number | null, rsi = R_SI[part]): { rc: number; u: number } | null {
  if (rcValue != null && Number.isFinite(rcValue) && rcValue >= 0) {
    return { rc: rcValue, u: 1 / (rsi + rcValue + R_SE) };
  }
  if (uValue != null && Number.isFinite(uValue) && uValue > 0) {
    return { rc: Math.max(0, 1 / uValue - rsi - R_SE), u: uValue };
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
      return { kind, system: { kind: 'single', unit } };
    }
    case 'heat_pump': return { kind, generator: heatPumpGeneratorTemplate(project), renewable: heatPumpRenewableTemplate() };
    case 'hot_water': return { kind, generator: hotWaterGeneratorTemplate('heat_pump') };
    case 'pv': return { kind, systems: [pvSystemTemplate(0)] };
    case 'solar_water_heater': return { kind, systems: [solarWaterHeaterTemplate(0)] };
    case 'shower_heat_recovery': return { kind, recovery: {
      showers: [{ unit: 'vertical' }], connection: 'mixer_and_heater', sourceReference: '' } };
    case 'lighting': return { kind, zones: {} };
  }
}

/** Members of a lighting zone a lighting measure may change (§14.4–14.6). */
export const LIGHTING_MEASURE_FIELDS = ['power', 'parasitic', 'occupancy', 'daylight', 'extractedLuminaires'] as const;

const lightingKey = (entry: Block, zone: Block) => `${String(entry.zoneId ?? '')}/${String(zone.id ?? '')}`;

/** The current lighting with a lighting template's changes applied (the editor's draft). */
export function applyLightingChanges(current: Block[], zones: Record<string, Block>): Block[] {
  return current.map((entry) => ({
    ...entry,
    lightingZones: ((entry.lightingZones as Block[] | undefined) ?? []).map((zone) => {
      const change = zones[lightingKey(entry, zone)];
      return change ? { ...zone, ...clone(change) } : zone;
    }),
  }));
}

/**
 * Per lighting zone, the measure members of `edited` that differ from
 * `current`. Added or removed lighting zones and changed functions or areas
 * are not part of a lighting measure and are ignored.
 */
export function lightingChanges(current: Block[], edited: Block[]): Record<string, Block> {
  const before = new Map<string, Block>();
  for (const entry of current) {
    for (const zone of (entry.lightingZones as Block[] | undefined) ?? []) before.set(lightingKey(entry, zone), zone);
  }
  const changes: Record<string, Block> = {};
  for (const entry of edited) {
    for (const zone of (entry.lightingZones as Block[] | undefined) ?? []) {
      const key = lightingKey(entry, zone);
      const base = before.get(key);
      if (!base) continue;
      const changed: Block = {};
      for (const field of LIGHTING_MEASURE_FIELDS) {
        if (zone[field] !== undefined && JSON.stringify(zone[field]) !== JSON.stringify(base[field])) changed[field] = clone(zone[field]);
      }
      if (Object.keys(changed).length > 0) changes[key] = changed;
    }
  }
  return changes;
}

/**
 * The current form of a saved template. A ventilation snapshot keeps only
 * its `system` (the editor only edited the unit); a lighting snapshot is
 * converted to its changes against the current project and needs the
 * adviser's review (`migrationReview`), because the snapshot cannot tell a
 * measure's change from a later change of the base project.
 */
export function normalizeTemplate(project: IProject, template: MwaMeasureTemplate | LegacyTemplate): { template: MwaMeasureTemplate; migrated: boolean } {
  if (template.kind === 'ventilation' && 'ventilation' in template && !('system' in template)) {
    // A snapshot without a system has no unit to apply: it stays empty and
    // `buildTemplatePatch` reports it instead of writing `{}`.
    const system = template.ventilation.system as Block | undefined;
    return { template: { kind: 'ventilation', system: isObject(system) ? clone(system) : {} }, migrated: true };
  }
  if (template.kind === 'lighting' && 'lighting' in template && !('zones' in template)) {
    const current = (nta(project)?.lighting as Block[] | undefined) ?? [];
    return { template: { kind: 'lighting', zones: lightingChanges(current, template.lighting), reviewed: false }, migrated: true };
  }
  return { template: template as MwaMeasureTemplate, migrated: false };
}

/** Generator kinds whose renewable heat needs 5.31/5.32 evidence (`heatPumpRenewable`). */
export const HEAT_PUMP_KINDS = new Set(['heat_pump_forfait', 'heat_pump_annex_q', 'hybrid_heat_pump', 'gas_heat_pump']);

/** 5.31/5.32 source evidence of a heat pump. */
export function heatPumpRenewableTemplate(): Block {
  return { sourceBelow20C: true, exhaustAirSource: false, sourceReference: '' };
}

/**
 * The 5.31/5.32 flags that follow from the generator's own source, or null
 * when the source does not fix them. Forfait: exhaust air iff the table row
 * is exhaust air; the collective 20–40 °C and ≥ 40 °C rows are not below
 * 20 °C. Annex Q: exhaust air iff the source is exhaust-air/water.
 */
export function derivedRenewableFlags(generator: Block): { sourceBelow20C?: boolean; exhaustAirSource: boolean } | null {
  const forfait = generator.forfait as Block | null | undefined;
  if (generator.kind === 'heat_pump_forfait' && forfait?.source) {
    const source = String(forfait.source);
    return {
      exhaustAirSource: source === 'exhaust_air',
      sourceBelow20C: source !== 'collective20_to40_c' && source !== 'collective_at_least40_c',
    };
  }
  const annexQ = generator.heatPump as Block | undefined;
  if (generator.kind === 'heat_pump_annex_q' && annexQ?.source) {
    return { exhaustAirSource: annexQ.source === 'exhaust_air_water' };
  }
  return null;
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
    // Table 17.3 with remark 23 (p. 706–707): situation e) is the
    // conservative choice for PV; situation a) needs evidence
    // (`obstructionSourceReference`, template only).
    obstruction: { method: 'full' },
    sourceReference: '',
  };
}

/** Template-only member of a PV system: evidence for the minimal-obstruction situation a). */
export const PV_OBSTRUCTION_SOURCE = 'obstructionSourceReference';

/**
 * Evidence a measure's template keeps outside its patch: the source of each
 * PV system on minimal obstruction (table 17.3 situation a), p. 706–707),
 * so the report and the dossier can show it.
 */
export function measureEvidenceNotes(measure: { template?: MwaMeasureTemplate | LegacyTemplate | null }): { id: string; source: string }[] {
  const template = measure.template;
  if (!template || template.kind !== 'pv' || !('systems' in template)) return [];
  return template.systems.flatMap((system) => {
    const obstruction = system.obstruction as Block | undefined;
    const source = String(system[PV_OBSTRUCTION_SOURCE] ?? '').trim();
    return obstruction?.method === 'minimal' && source ? [{ id: String(system.id ?? ''), source }] : [];
  });
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
  /** i18n keys (mwa.template.warning.*) of points to check; they do not block the measure. */
  warnings?: string[];
  /** Field names (mwa.template.field.*) behind a `valueRequired` problem. */
  missingFields?: string[];
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
  const { obstruction, obstructionFactors: _drop, [PV_OBSTRUCTION_SOURCE]: _evidence, ...rest } = system;
  void _drop;
  void _evidence;
  if (factors.length > 0) return { ...rest, obstructionFactors: factors };
  return obstruction == null ? rest : { ...rest, obstruction };
}

/** Prefixes the ids of added systems with the measure id, so they never clash with existing ones. */
function withIds(items: Block[], measureId: string, prefix: string): Block[] {
  return items.map((item, index) => ({ ...item, id: `${measureId}-${String(item.id ?? `${prefix}-${index + 1}`)}` }));
}

/** Whether any leaf of `value` is null (a field the adviser left blank). */
function hasBlank(value: unknown): boolean {
  if (value === null) return true;
  if (Array.isArray(value)) return value.some(hasBlank);
  if (isObject(value)) return Object.values(value).some(hasBlank);
  return false;
}

/** Hot-water generators the kernel expects a separate storage vessel with (§13.6.2). */
const STORAGE_GENERATORS = new Set(['electric_boiler', 'indirect_boiler', 'indirect_heat_pump']);

/**
 * Members that the kernel requires whenever their parent object is present: a
 * blank (null) one there is a missing value, not an optional field. Paths are
 * member names from the parent down; `*` matches any member or array element.
 */
const VENTILATION_REQUIRED: string[][] = [
  ['heatRecovery', 'efficiency', 'value'],
  ['heatRecovery', 'efficiency', 'exchanger'],
  ['heatRecovery', 'bypass', 'fraction'],
  ['heatRecovery', 'supplyDuctInsulation', 'thicknessM'],
  ['heatRecovery', 'supplyDuctInsulation', 'conductivityWPerMK'],
  ['*', 'heatRecovery', 'efficiency', 'value'],
  ['*', 'heatRecovery', 'efficiency', 'exchanger'],
  ['*', 'heatRecovery', 'bypass', 'fraction'],
];
const HOT_WATER_REQUIRED: string[][] = [
  ['appliance'], ['boiler'], ['inputKwhPerDay'], ['deliveredKwhPerDay'],
  ['low', 'inputKwhPerDay'], ['high', 'inputKwhPerDay'],
];

/** Required members (see the lists above) that are present but blank, as dotted paths without `*`. */
export function requiredBlanks(value: unknown, paths: string[][]): string[] {
  const found: string[] = [];
  const walk = (node: unknown, path: string[], trail: string[]) => {
    if (path.length === 0 || !isObject(node)) return;
    const [head, ...rest] = path;
    const keys = head === '*' ? Object.keys(node) : head in node ? [head] : [];
    for (const key of keys) {
      const child = node[key];
      const named = head === '*' ? trail : [...trail, key];
      if (rest.length === 0) { if (child === null) found.push(named.join('.')); }
      else walk(child, rest, named);
    }
  };
  for (const path of paths) walk(value, path, []);
  return [...new Set(found)];
}

/** Whether a required member (see the lists above) is present but blank. */
export function hasRequiredBlank(value: unknown, paths: string[][]): boolean {
  return requiredBlanks(value, paths).length > 0;
}

/** The patch of a template against the current project. */
export function buildTemplatePatch(project: IProject, saved: MwaMeasureTemplate | LegacyTemplate, measureId: string): TemplatePatch {
  const { template } = normalizeTemplate(project, saved);
  const block = nta(project);
  const problems: string[] = template.kind === 'lighting' && template.reviewed === false ? ['migrationReview'] : [];
  const warnings: string[] = [];
  const missing: string[] = [];
  const requireValue = (...fields: string[]) => { problems.push('valueRequired'); missing.push(...fields); };
  const patch: MwaPatchOperation[] = [];
  switch (template.kind) {
    case 'insulation': {
      const values = insulationValues(template.part, template.rcValue, template.uValue);
      if (!values) requireValue('rcOrU');
      const all = insulationOptions(project, template.part);
      const options = all.filter((option) => template.surfaces.includes(option.key));
      if (template.surfaces.some((key) => !all.some((option) => option.key === key))) problems.push('selectionStale');
      if (options.length === 0) problems.push('selectionRequired');
      if (!values) break;
      const created = new Map<string, string>();
      const groundFloors = (block?.groundFloors as Block[] | undefined) ?? [];
      for (const option of options) {
        const surface = project.zones[option.zoneIndex].surfaces[option.surfaceIndex];
        const original = project.constructions.find((item) => item.id === surface.constructionId);
        const rsi = insulationRsi(project, template.part, surface.id);
        const steep = rsi !== R_SI[template.part];
        const surfaceValues = insulationValues(template.part, template.rcValue, template.uValue, rsi)!;
        const rc = round(surfaceValues.rc, 3);
        const u = round(surfaceValues.u, 4);
        // The new Rc should improve on the current construction (lower U).
        if (original && Number.isFinite(original.uValue) && original.uValue > 0 && u >= original.uValue - 1e-9) warnings.push('notImproved');
        const key = `${surface.constructionId ?? ''}${steep ? '#steep' : ''}`;
        let id = created.get(key);
        if (!id) {
          id = `${measureId}-${surface.constructionId || template.part}${steep ? '-steep' : ''}`;
          created.set(key, id);
          const rcText = rc.toLocaleString('nl-NL', { maximumFractionDigits: 2 });
          patch.push({ op: 'add', path: '/constructions/-', value: {
            id, name: `${original?.name ?? template.part} → Rc ${rcText} (${measureId})`, layers: [], rcValue: rc, uValue: u } });
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
      const all = windowOptions(project);
      const options = all.filter((option) => template.windows.includes(option.key));
      if (template.windows.some((key) => !all.some((option) => option.key === key))) problems.push('selectionStale');
      if (options.length === 0) problems.push('selectionRequired');
      const u = template.uValue;
      if (u == null || !Number.isFinite(u) || u <= 0) { requireValue('uValue'); break; }
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
      if (qv10 == null || !Number.isFinite(qv10) || qv10 <= 0) requireValue('qv10');
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
      // A migrated snapshot without a system: never replace the system with `{}`.
      if (!isObject(template.system) || Object.keys(template.system).length === 0) { problems.push('ventilationSystemRequired'); break; }
      { const blanks = requiredBlanks(template.system, VENTILATION_REQUIRED); if (blanks.length > 0) requireValue(...blanks); }
      // Only the system is replaced: the project's flows, controls and
      // infiltration (the airtightness measure) stay as they are.
      if (JSON.stringify(current.system) !== JSON.stringify(template.system)) {
        patch.push({ op: current.system === undefined ? 'add' : 'replace', path: pointer('ntaCalculation', 'ventilation', 'system'), value: template.system });
      }
      ((block?.zoneData as Block[] | undefined) ?? []).forEach((zone, index) => {
        const ventilation = zone.ventilation as Block | undefined;
        if (ventilation && JSON.stringify(ventilation.system) !== JSON.stringify(template.system)) {
          patch.push({ op: ventilation.system === undefined ? 'add' : 'replace',
            path: pointer('ntaCalculation', 'zoneData', index, 'ventilation', 'system'), value: template.system });
        }
      });
      if (patch.length === 0) problems.push('noChange');
      break;
    }
    case 'heat_pump': {
      if (!block) { problems.push('calculationRequired'); break; }
      let generator = template.generator;
      const forfait = generator.forfait as Block | null | undefined;
      // Hydronic emission needs the design supply temperature (tables
      // 9.27/9.29 columns; annex Q θ_sup).
      if ((generator.kind === 'heat_pump_forfait' && forfait?.sink === 'hydronic' && forfait.designSupplyTemperatureC == null)
        || (generator.kind === 'heat_pump_annex_q' && generator.designSupplyTemperatureC == null)) {
        problems.push('supplyTemperatureRequired');
      }
      // An individual heat pump with a stated capacity is not part of a
      // collective building installation (9.6.3 table row choice).
      if (forfait && forfait.thermalCapacityKw != null && forfait.collectiveBuildingInstallation == null) {
        generator = { ...generator, forfait: { ...forfait, collectiveBuildingInstallation: generator.sourceSystem !== 'individual' } };
      }
      patch.push({ op: 'replace', path: pointer('ntaCalculation', 'generator'), value: generator });
      // 5.31/5.32: a heat pump's renewable heat needs the source evidence.
      if (HEAT_PUMP_KINDS.has(String(generator.kind))) {
        const flags = derivedRenewableFlags(generator);
        const renewable = template.renewable && flags ? { ...template.renewable, ...flags } : template.renewable;
        if (!renewable || !String(renewable.sourceReference ?? '').trim()) problems.push('sourceRequired');
        if (renewable) {
          patch.push({ op: block.heatPumpRenewable == null ? 'add' : 'replace', path: pointer('ntaCalculation', 'heatPumpRenewable'), value: renewable });
        }
      }
      break;
    }
    case 'hot_water': {
      const hotWater = block?.hotWater as Block | undefined;
      if (!hotWater) { problems.push('hotWaterRequired'); break; }
      { const blanks = requiredBlanks(template.generator, HOT_WATER_REQUIRED); if (blanks.length > 0) requireValue(...blanks); }
      patch.push({ op: 'replace', path: pointer('ntaCalculation', 'hotWater', 'generator'), value: template.generator });
      // §13.6.2 with the kernel's storage rule (domestic_hot_water.rs),
      // decided over the main and the additional generators: a separate
      // vessel goes with an electric/indirect boiler, an indirect heat pump
      // or external heat; other generators carry their storage in the
      // generator efficiency, so the old vessel is removed.
      const storage = (hotWater.storage as Block[] | undefined) ?? [];
      const generators: Block[] = [template.generator as Block, ...((hotWater.additionalGenerators as Block[] | undefined) ?? [])
        .map((item) => (item.generator as Block | undefined) ?? {})];
      const kindOf = (generator: Block) => String(generator.kind ?? '');
      const needsStorage = generators.some((generator) => STORAGE_GENERATORS.has(kindOf(generator)));
      const external = generators.some((generator) => kindOf(generator) === 'external_heat');
      const tested = generators.some((generator) => kindOf(generator) === 'measured_two_profiles'
        || kindOf(generator) === 'heat_pump_en16147'
        || (kindOf(generator) === 'gas_appliance' && generator.annexT != null));
      if (needsStorage && storage.length === 0) problems.push('storageRequired');
      if (!needsStorage && !external && storage.length > 0) {
        // Note 1 of §13.6.2 (p. 566): with a tested appliance, vessels outside
        // its test stay and are calculated; the rest is in the efficiency.
        const kept = tested ? storage.filter((vessel) => vessel.notInApplianceTest === true) : [];
        if (kept.length !== storage.length) {
          patch.push({ op: 'replace', path: pointer('ntaCalculation', 'hotWater', 'storage'), value: kept });
        }
      }
      break;
    }
    case 'pv': {
      if (!block) { problems.push('calculationRequired'); break; }
      if (template.systems.length === 0) problems.push('selectionRequired');
      for (const system of template.systems) {
        if (hasBlank(system.peakPower)) problems.push('peakPowerRequired');
        if (system.azimuthDeg == null) requireValue('azimuthDeg');
        if (system.tiltDeg == null) requireValue('tiltDeg');
        const obstruction = system.obstruction as Block | undefined;
        if (obstruction?.method === 'minimal' && !String(system[PV_OBSTRUCTION_SOURCE] ?? '').trim()) problems.push('obstructionEvidenceRequired');
      }
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
      // Per lighting zone: the changed members on top of the current zone.
      const matched = new Set<string>();
      current.forEach((entry, entryIndex) => {
        ((entry.lightingZones as Block[] | undefined) ?? []).forEach((zone, zoneIndex) => {
          const key = lightingKey(entry, zone);
          const change = template.zones[key];
          if (!change) return;
          matched.add(key);
          for (const field of LIGHTING_MEASURE_FIELDS) {
            if (change[field] === undefined || JSON.stringify(change[field]) === JSON.stringify(zone[field])) continue;
            patch.push({ op: zone[field] === undefined ? 'add' : 'replace',
              path: pointer('ntaCalculation', 'lighting', entryIndex, 'lightingZones', zoneIndex, field), value: change[field] });
          }
        });
      });
      if (Object.keys(template.zones).some((key) => !matched.has(key))) problems.push('selectionStale');
      if (patch.length === 0) problems.push('noChange');
      break;
    }
  }
  return { patch, problems: [...new Set(problems)], warnings: [...new Set(warnings)], missingFields: [...new Set(missing)] };
}

/**
 * Every template measure with its patch rebuilt against the current project
 * (array indices in the paths follow the project) and its open problems in
 * `incomplete`; the kernel refuses a variant with an incomplete measure.
 */
export function regenerateTemplatePatches(project: IProject, definition: NtaMaatwerkadvies): NtaMaatwerkadvies {
  return {
    ...definition,
    measures: definition.measures.map((measure): MwaMeasure => {
      const { incomplete: _old, ...rest } = measure;
      void _old;
      if (!measure.template) return rest;
      const { patch, problems } = buildTemplatePatch(project, measure.template, measure.id);
      return problems.length > 0 ? { ...rest, patch, incomplete: problems } : { ...rest, patch };
    }),
  };
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
