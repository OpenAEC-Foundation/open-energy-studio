import type { BuildingPerformanceInput, OpnameAssessment } from './KernelClient';

/**
 * "Overnemen in projectmodel": the NTA input the kernel derived from the
 * basisopname (ISSO 82.1/75.1) written into the project's NTA draft. Only
 * the blocks with the same meaning in both models are taken over; the
 * geometry stays the project's own (zones, surfaces, windows), so the
 * survey's derived transmission and areas are left out and listed as such.
 */

type Loose = Record<string, unknown>;

/** Top-level blocks with the same shape in the derived input and `ntaCalculation`. */
const SAME_SHAPE = [
  'calculationScope', 'areaSourceReference', 'heatPumpRenewable', 'bacsFactor', 'bacsSourceReference',
  'useInventoryComplete', 'declaredUses', 'declaredRenewableHeat', 'productionInventoryComplete', 'onSiteProduction',
  'pvSystems', 'cooling', 'coolingSystems', 'hotWater', 'additionalHotWaterSystems', 'spaceHeatingSolar', 'lighting',
  'demandUsesFixedC1Ventilation', 'batteryStoragePresent', 'storage', 'externalSupply',
] as const;

/**
 * Blocks of the main heating chain (`spaceHeating.<key>` → `<key>`). The
 * calculated distribution (9.4) needs its `distributionSystem`: without it the
 * kernel refuses the taken-over project with `distribution_system_required`.
 */
const HEATING_CHAIN = ['emission', 'distribution', 'distributionSystem', 'generator', 'collectiveConnection', 'identicalSystems', 'humidifiers'] as const;

/** Demand blocks of a single-zone survey (`spaceHeating.demand.<key>` → `<key>`). */
const DEMAND = ['usageFunction', 'dwellingType', 'setpoints', 'thermalMass', 'internalGains', 'ventilationFlows', 'ventilation'] as const;

/**
 * Building facts with the same meaning in both models, taken over only when the
 * survey gives them: a survey without a label or Bbl function (a dwelling) does
 * not clear the project's. The construction year feeds §5.3.2 and the label.
 */
const BUILDING_FACTS = [
  'constructionYear', 'labelFunction', 'labelFunctions', 'bblFunction', 'bblFunctions', 'activeCooling',
  'fossilAppliancesOutsideCalculation', 'zebHeatDeliveryTemperature',
] as const;

/**
 * Members of the derived input the takeover leaves to the project, with why.
 * `review57-takeover-completeness.test.ts` checks that every member a survey
 * derives is either taken over or listed here.
 */
export const NOT_TAKEN_OVER: Readonly<Record<string, string>> = {
  normVersion: 'the edition is the project\'s own choice (edition switch)',
  totalUsableFloorAreaM2: 'geometry',
  lossAreaM2: 'geometry',
  lossAreaSourceReference: 'geometry',
  permitApplicationAfter20260529: 'a fact of the permit application, not of the survey',
  hotWaterNeedFit: 'no project field',
  additionalHeatingSystems: 'additional heating systems have another shape in the project',
  'spaceHeating.additionalZones': 'multi-zone survey',
  'spaceHeating.demand.transmission': 'geometry',
  'spaceHeating.demand.opaqueElements': 'geometry',
  'spaceHeating.demand.opaqueInventoryComplete': 'geometry',
  'spaceHeating.demand.windows': 'geometry',
  'spaceHeating.demand.windowInventoryComplete': 'geometry',
  'spaceHeating.demand.usableFloorAreaM2': 'geometry',
  'spaceHeating.demand.areaSourceReference': 'geometry',
  'spaceHeating.demand.zoneId': 'geometry',
};

/** Parts of the derived input that are not taken over, with the reason key. */
export type SkippedPart = { path: string; reason: 'geometry' | 'multi_zone' | 'additional_heating' };

export interface SurveyTakeover {
  /** The draft with the survey's blocks written in. */
  next: Loose;
  /** Top-level `ntaCalculation` keys the takeover writes. */
  keys: string[];
  skipped: SkippedPart[];
}

/** True when the survey result holds a derived input that may be taken over. */
export function canTakeOver(assessment: OpnameAssessment | null | undefined): assessment is OpnameAssessment & { derivedInput: BuildingPerformanceInput } {
  return assessment != null && assessment.derivedInput != null
    && (assessment.status === 'calculated_unverified' || assessment.status === 'calculated_legacy_edition');
}

function present(value: unknown): boolean {
  return value !== undefined;
}

/** The project zone a single-zone survey is taken over into: its id and usable floor area. */
export interface TakeoverZone {
  id: string;
  floorArea?: number | null;
  /** A_g of the whole project when it has more zones; the hot water serves the building (13.20). */
  buildingFloorArea?: number | null;
}

/** Every `areaM2` member in a copy of `value`, times `factor`. */
function scaleAreas(value: unknown, factor: number): unknown {
  if (Array.isArray(value)) return value.map((item) => scaleAreas(item, factor));
  if (value == null || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value as Loose).map(([key, inner]) =>
    [key, key === 'areaM2' && typeof inner === 'number' ? inner * factor : scaleAreas(inner, factor)]));
}

/**
 * Lighting (chapter 14) and humidifiers name the survey's zone and carry its
 * areas; the kernel wants the project zone's id and its areas to add up to the
 * zone's area (14.4, `lighting_zone_areas_invalid`). The survey's split is kept
 * and scaled to the project zone.
 */
/** Project zone area over survey area, or null when either is unknown. */
function areaFactor(surveyArea: unknown, zone: TakeoverZone): number | null {
  return typeof zone.floorArea === 'number' && zone.floorArea > 0 && typeof surveyArea === 'number' && surveyArea > 0
    ? zone.floorArea / surveyArea : null;
}

/**
 * A utility hot-water system serves function areas that together are at most
 * the building's A_g (13.20, `hot_water_served_area_exceeds_building`); the
 * survey's areas are scaled to the project zone like the lighting.
 */
function hotWaterToProject(system: unknown, factor: number | null): unknown {
  if (factor == null || system == null || typeof system !== 'object') return system;
  const next = { ...(system as Loose) };
  const need = next.need as Loose | null | undefined;
  if (need != null && typeof need === 'object' && Array.isArray(need.areas)) next.need = { ...need, areas: scaleAreas(need.areas, factor) };
  const circulation = next.circulation as Loose | null | undefined;
  if (circulation != null && typeof circulation === 'object' && typeof circulation.sportHallAreaM2 === 'number') {
    next.circulation = { ...circulation, sportHallAreaM2: circulation.sportHallAreaM2 * factor };
  }
  return next;
}

function toProjectZone(items: unknown, surveyZone: unknown, surveyArea: unknown, zone: TakeoverZone, areaKey: 'areaM2' | 'servedAreaM2'): unknown {
  if (!Array.isArray(items)) return items;
  const factor = areaFactor(surveyArea, zone);
  return items.map((item) => {
    if (item == null || typeof item !== 'object') return item;
    const entry = item as Loose;
    if (entry.zoneId !== surveyZone) return item;
    let next: Loose = { ...entry, zoneId: zone.id };
    if (factor != null) {
      if (areaKey === 'areaM2') next = scaleAreas(next, factor) as Loose;
      else if (typeof next.servedAreaM2 === 'number') next.servedAreaM2 = next.servedAreaM2 * factor;
    }
    return next;
  });
}

/**
 * Writes the derived survey input into a copy of the NTA draft. `zone`: the
 * project zone a single-zone survey goes into (lighting and humidifiers follow it).
 */
export function surveyTakeover(derived: BuildingPerformanceInput, current: Loose | null, zone?: TakeoverZone): SurveyTakeover {
  const source = derived as unknown as Loose;
  const next: Loose = structuredClone(current ?? {});
  const keys: string[] = [];
  const skipped: SkippedPart[] = [];
  const put = (key: string, value: unknown) => {
    if (!present(value)) return;
    next[key] = structuredClone(value);
    keys.push(key);
  };
  for (const key of SAME_SHAPE) put(key, source[key]);
  // A BACS inventory in the draft (§5.5.8) overrides bacsFactor; it describes the
  // installations the takeover replaces, so the survey's f_BACS takes its place.
  if (keys.includes('bacsFactor') && next.bacs != null) {
    next.bacs = null;
    keys.push('bacs');
  }
  for (const key of BUILDING_FACTS) {
    const value = source[key];
    if (value != null && !(Array.isArray(value) && value.length === 0)) put(key, value);
  }
  const heating = (source.spaceHeating ?? {}) as Loose;
  for (const key of HEATING_CHAIN) put(key, heating[key]);
  const demand = (heating.demand ?? {}) as Loose;
  const zones = Array.isArray(heating.additionalZones) ? heating.additionalZones : [];
  if (zones.length === 0) {
    for (const key of DEMAND) put(key, demand[key]);
    if (zone != null) {
      if (keys.includes('lighting')) next.lighting = toProjectZone(next.lighting, demand.zoneId, demand.usableFloorAreaM2, zone, 'areaM2');
      if (keys.includes('humidifiers')) next.humidifiers = toProjectZone(next.humidifiers, demand.zoneId, demand.usableFloorAreaM2, zone, 'servedAreaM2');
      // The hot water serves the building, not only the zone the survey goes into (review 59).
      const building = typeof zone.buildingFloorArea === 'number' && zone.buildingFloorArea > 0
        ? { id: zone.id, floorArea: zone.buildingFloorArea } : zone;
      const factor = areaFactor(demand.usableFloorAreaM2, building);
      if (keys.includes('hotWater')) next.hotWater = hotWaterToProject(next.hotWater, factor);
      if (keys.includes('additionalHotWaterSystems') && Array.isArray(next.additionalHotWaterSystems)) {
        next.additionalHotWaterSystems = next.additionalHotWaterSystems.map((system) => hotWaterToProject(system, factor));
      }
    }
  } else {
    skipped.push({ path: 'spaceHeating.additionalZones', reason: 'multi_zone' });
  }
  if (Array.isArray(source.additionalHeatingSystems) && source.additionalHeatingSystems.length > 0) {
    skipped.push({ path: 'additionalHeatingSystems', reason: 'additional_heating' });
  }
  skipped.push({ path: 'spaceHeating.demand.transmission', reason: 'geometry' });
  skipped.push({ path: 'totalUsableFloorAreaM2', reason: 'geometry' });
  skipped.push({ path: 'lossAreaM2', reason: 'geometry' });
  return { next, keys, skipped };
}

/** The survey rule that adds ΔU_for of 8.3 to the outdoor elements (ISSO 82.1/75.1). */
export const FORFAIT_BRIDGE_RULE = 'thermal_bridges_forfait_delta_u';

/**
 * The thermal-bridge method of 8.2.1 the survey worked with: `forfait` when it
 * added ΔU_for to the outdoor elements or gave a floor a forfait edge; null
 * when the survey says nothing about it.
 */
export function surveyBridgeRoute(assessment: OpnameAssessment & { derivedInput: BuildingPerformanceInput }): 'forfait' | null {
  if ((assessment.appliedDefaults ?? []).some((item) => item.rule === FORFAIT_BRIDGE_RULE)) return 'forfait';
  const forfaitEdge = surveyGroundFloors(assessment.derivedInput)
    .some(({ floor }) => (floor.edgeThermalBridges as Loose | undefined)?.method === 'forfait');
  return forfaitEdge ? 'forfait' : null;
}

/** The building's thermal-bridge method the takeover proposes. */
export interface BridgeRouteItem {
  key: 'thermalBridgeRoute';
  from: unknown;
  to: 'forfait';
  /** `conflict`: the project has another method, a detailed floor edge or ψ-values, so forfait would mix methods. */
  status: 'new' | 'conflict';
}

/**
 * The proposal for `ntaCalculation.thermalBridgeRoute`, or null when the survey
 * implies no method or the draft already has it. `projectHasPsi`: the project
 * zones carry linear thermal bridges (ψ), which the forfait route does not allow.
 */
export function bridgeRouteItem(route: 'forfait' | null, current: Loose | null, projectHasPsi: boolean): BridgeRouteItem | null {
  if (route == null) return null;
  const from = current?.thermalBridgeRoute ?? undefined;
  if (from === route) return null;
  const detailedEdge = (Array.isArray(current?.groundFloors) ? current.groundFloors as Loose[] : [])
    .some((floor) => (floor.edgeThermalBridges as Loose | null | undefined)?.method === 'detailed');
  const status = from === undefined && !detailedEdge && !projectHasPsi ? 'new' : 'conflict';
  return { key: 'thermalBridgeRoute', from, to: route, status };
}

/** Writes the accepted thermal-bridge method into a copy of the draft. */
export function applyBridgeRoute(draft: Loose | null, item: BridgeRouteItem | null, accepted: ReadonlySet<string>): Loose {
  const next: Loose = structuredClone(draft ?? {});
  if (item != null && accepted.has(item.key)) next.thermalBridgeRoute = item.to;
  return next;
}

/** A short display of a value in the diff preview. */
export function previewValue(value: unknown, locale?: string): string {
  if (value === undefined) return '—';
  // A plain number follows the UI language (decimal comma in Dutch); objects stay JSON.
  if (typeof value === 'number' && locale && Number.isFinite(value)) {
    return value.toLocaleString(locale, { maximumFractionDigits: 6 });
  }
  const text = typeof value === 'string' ? value : JSON.stringify(value);
  return text.length > 60 ? `${text.slice(0, 57)}…` : text;
}

/** Members whose values are fine (Ψ, U, ventilation openings, air changes) get three decimals. */
const FINE_NUMBER = /(WPerMk|WPerM2k|PerHour|M2PerM)$/;

/** A number for display: two decimals, three for fine members, in the UI language. */
export function formatTakeoverNumber(value: number, locale?: string, key?: string): string {
  if (!Number.isFinite(value)) return String(value);
  const digits = key && FINE_NUMBER.test(key) ? 3 : 2;
  return value.toLocaleString(locale ?? 'nl-NL', { maximumFractionDigits: digits });
}

/** One line of a ground-floor value: an optional member label and its text. */
export interface TakeoverValueLine { label?: string; text: string }

/**
 * A ground-floor value as readable lines: numbers rounded (R_c, P, areas and
 * depths to two decimals), enum values and member names through `label`
 * (keys `key.<member>` and `value.<enum>`, falling back to the raw name),
 * objects as one line per member, lists numbered. The source reference of a
 * list item is left out; the project keeps it.
 */
export function groundValueLines(
  value: unknown, locale: string | undefined, label: (key: string) => string | undefined, field?: string,
): TakeoverValueLine[] {
  const name = (member: string) => label(`key.${member}`) ?? member;
  const text = (item: unknown, member?: string): string => {
    if (item === undefined || item === null) return '—';
    if (typeof item === 'number') return formatTakeoverNumber(item, locale, member);
    if (typeof item === 'boolean') return label(`value.${item}`) ?? String(item);
    if (typeof item === 'string') return label(`value.${item}`) ?? item;
    if (Array.isArray(item)) return item.length === 0 ? (label('empty') ?? '—') : `${item.length} ×`;
    return Object.entries(item as Record<string, unknown>)
      .filter(([key]) => key !== 'sourceReference')
      .map(([key, inner]) => `${name(key)} ${text(inner, key)}`).join('; ');
  };
  if (value === undefined || value === null || typeof value !== 'object') return [{ text: text(value, field) }];
  if (Array.isArray(value)) {
    if (value.length === 0) return [{ text: label('empty') ?? '—' }];
    return value.map((item, index) => ({ label: `${index + 1}`, text: text(item) }));
  }
  const lines: TakeoverValueLine[] = [];
  for (const [key, inner] of Object.entries(value as Record<string, unknown>)) {
    if (key === 'sourceReference') continue;
    if (Array.isArray(inner) && inner.some((item) => item && typeof item === 'object')) {
      if (inner.length === 0) lines.push({ label: name(key), text: label('empty') ?? '—' });
      inner.forEach((item, index) => lines.push({ label: `${name(key)} ${index + 1}`, text: text(item) }));
    } else {
      lines.push({ label: name(key), text: text(inner, key) });
    }
  }
  return lines;
}

/*
 * Ground floors. The takeover leaves the geometry to the project, so the
 * survey's floors to the ground (perimeter P, crawlspace or unheated cellar,
 * a heated basement from a façade part against the ground) only reach the
 * project through the `ntaCalculation.groundFloors` entry of a project floor.
 * The adviser pairs each survey floor with a project floor and accepts per
 * value; a value the project already holds is never replaced unasked.
 */

/** The `groundFloors` members a survey floor can carry into a project floor. */
export const GROUND_FIELDS = [
  'exposedPerimeterM', 'constructionResistanceM2kPerW', 'edgeThermalBridges', 'edgeInsulation', 'below', 'heatedBasement',
] as const;
export type GroundField = typeof GROUND_FIELDS[number];

/** Members whose absence in the survey means "none" (no crawlspace, no basement). */
const OPTIONAL_GROUND: ReadonlySet<GroundField> = new Set<GroundField>(['edgeInsulation', 'below', 'heatedBasement']);

/** A floor to the ground in the derived survey input. */
export interface SurveyGroundFloor { id: string; areaM2: number | null; floor: Loose }

/** A floor to the ground in the project geometry. */
export interface ProjectGroundSurface { id: string; name: string; areaM2: number }

/** One value of a survey floor offered for a project floor. */
export interface GroundFloorItem {
  key: string;
  surveyId: string;
  surfaceId: string;
  field: GroundField;
  from: unknown;
  to: unknown;
  /** `new`: the project has no value; `conflict`: it has another one. */
  status: 'new' | 'conflict';
}

const demandsOf = (derived: BuildingPerformanceInput): Loose[] => {
  const heating = ((derived as unknown as Loose).spaceHeating ?? {}) as Loose;
  const zones = Array.isArray(heating.additionalZones) ? heating.additionalZones as Loose[] : [];
  return [heating.demand as Loose | undefined, ...zones.map((zone) => zone.demand as Loose | undefined)]
    .filter((demand): demand is Loose => demand != null);
};

/** The survey's floors to the ground, in all its zones. */
export function surveyGroundFloors(derived: BuildingPerformanceInput): SurveyGroundFloor[] {
  return demandsOf(derived).flatMap((demand) => {
    const transmission = (demand.transmission ?? {}) as Loose;
    const floors = Array.isArray(transmission.groundFloors) ? transmission.groundFloors as Loose[] : [];
    return floors.map((floor) => ({
      id: String(floor.id ?? ''),
      areaM2: typeof floor.areaM2 === 'number' ? floor.areaM2 : null,
      floor,
    }));
  });
}

type SurfaceLike = { id?: unknown; name?: unknown; type?: unknown; thermalBoundary?: unknown; area?: unknown };

/** The project's floor surfaces to the ground (walls and roofs to the ground are refused by the kernel). */
export function projectGroundSurfaces(project: { zones?: Array<{ surfaces?: SurfaceLike[] }> } | null | undefined): ProjectGroundSurface[] {
  return (project?.zones ?? []).flatMap((zone) => zone.surfaces ?? [])
    .filter((surface) => surface.thermalBoundary === 'ground' && surface.type !== 'wall' && surface.type !== 'roof')
    .map((surface) => ({ id: String(surface.id), name: String(surface.name || surface.id), areaM2: Number(surface.area) || 0 }));
}

/**
 * The project floor proposed for each survey floor: the same id, else the only
 * floor on both sides, else the one free floor of the same area (±2 %);
 * otherwise none, and the adviser picks.
 */
export function proposeGroundPairs(floors: SurveyGroundFloor[], surfaces: ProjectGroundSurface[]): Record<string, string | null> {
  const pairs: Record<string, string | null> = {};
  const taken = new Set<string>();
  for (const floor of floors) {
    const same = surfaces.find((surface) => surface.id === floor.id);
    if (same) { pairs[floor.id] = same.id; taken.add(same.id); }
  }
  for (const floor of floors) {
    if (floor.id in pairs) continue;
    const free = surfaces.filter((surface) => !taken.has(surface.id));
    let pick: ProjectGroundSurface | undefined;
    if (floors.length === 1 && surfaces.length === 1) pick = free[0];
    else if (floor.areaM2 != null) {
      const area = floor.areaM2;
      const near = free.filter((surface) => Math.abs(surface.areaM2 - area) <= 0.02 * Math.max(area, 1));
      if (near.length === 1) pick = near[0];
    }
    pairs[floor.id] = pick?.id ?? null;
    if (pick) taken.add(pick.id);
  }
  return pairs;
}

/**
 * The value a survey floor gives for a member of a project's `groundFloors`
 * entry. ΔU_for of a heated basement is left out: on the forfait route the
 * kernel takes it from the project's own envelope (8.38 with 8.2.1).
 */
function surveyValue(floor: Loose, field: GroundField, project: unknown): unknown {
  const value = floor[field];
  if (field === 'heatedBasement' && value != null && typeof value === 'object') {
    const rest = { ...(value as Loose) };
    delete rest.forfaitDeltaUWPerM2k;
    // A ΔU_for the adviser entered on the project floor stays theirs.
    const own = project != null && typeof project === 'object' ? (project as Loose).forfaitDeltaUWPerM2k : undefined;
    if (own !== undefined && own !== null) rest.forfaitDeltaUWPerM2k = own;
    return rest;
  }
  return blank(value) ? undefined : value;
}

/** JSON with sorted object keys, so that member order does not count as a difference. */
const canonical = (value: unknown): string => JSON.stringify(value, (_key, item: unknown) =>
  item != null && typeof item === 'object' && !Array.isArray(item)
    ? Object.fromEntries(Object.entries(item as Loose).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)))
    : item);
const sameValue = (a: unknown, b: unknown) => canonical(a) === canonical(b);
/** Absent, null or an empty list (no edge insulation) all mean "none". */
const blank = (value: unknown) => value === undefined || value === null || (Array.isArray(value) && value.length === 0);

const entryOf = (draft: Loose | null, surfaceId: string): Loose | undefined =>
  (Array.isArray(draft?.groundFloors) ? draft.groundFloors as Loose[] : []).find((item) => item.surfaceId === surfaceId);

/** Every value that would change in the paired project floors; unchanged values are left out. */
export function groundFloorItems(derived: BuildingPerformanceInput, current: Loose | null, pairs: Record<string, string | null>): GroundFloorItem[] {
  const items: GroundFloorItem[] = [];
  const used = new Set<string>();
  for (const { id, floor } of surveyGroundFloors(derived)) {
    const surfaceId = pairs[id];
    // One project floor takes one survey floor: a second pairing to it is ignored.
    if (!surfaceId || used.has(surfaceId)) continue;
    used.add(surfaceId);
    const entry = entryOf(current, surfaceId) ?? {};
    for (const field of GROUND_FIELDS) {
      const to = surveyValue(floor, field, entry[field]);
      const from = entry[field] ?? undefined;
      if (blank(to) && !OPTIONAL_GROUND.has(field)) continue;
      if ((blank(from) && blank(to)) || sameValue(from, to)) continue;
      items.push({ key: `${id}→${surfaceId}.${field}`, surveyId: id, surfaceId, field, from, to, status: blank(from) ? 'new' : 'conflict' });
    }
  }
  return items;
}

/** Accepted by default: values the project does not have yet. A conflict needs the adviser's choice. */
export function defaultGroundSelection(items: GroundFloorItem[]): Set<string> {
  return new Set(items.filter((item) => item.status === 'new').map((item) => item.key));
}

/** The source reference of a project floor that took values over from a survey floor. */
function withSurveySource(current: unknown, floor: Loose, surveyId: string): string {
  const marker = `basisopname, vloer ${surveyId}`;
  const note = floor.sourceReference ? `${marker}: ${String(floor.sourceReference)}` : marker;
  const text = typeof current === 'string' ? current.trim() : '';
  const notes = text.split(';').map((part) => part.trim());
  if (notes.some((part) => part === marker || part.startsWith(`${marker}:`))) return text;
  return text ? `${text}; ${note}` : note;
}

/** Writes the accepted survey values into the paired `groundFloors` entries of a copy of the draft. */
export function applyGroundFloors(
  draft: Loose | null, derived: BuildingPerformanceInput, pairs: Record<string, string | null>, accepted: ReadonlySet<string>,
): Loose {
  const next: Loose = structuredClone(draft ?? {});
  const items = groundFloorItems(derived, draft, pairs).filter((item) => accepted.has(item.key));
  if (items.length === 0) return next;
  const list = Array.isArray(next.groundFloors) ? next.groundFloors as Loose[] : [];
  next.groundFloors = list;
  const floors = new Map(surveyGroundFloors(derived).map((item) => [item.id, item.floor]));
  for (const item of items) {
    let entry = list.find((candidate) => candidate.surfaceId === item.surfaceId);
    if (!entry) {
      entry = { surfaceId: item.surfaceId, sourceReference: '' };
      list.push(entry);
    }
    if (item.to === undefined) delete entry[item.field];
    else entry[item.field] = structuredClone(item.to);
    entry.sourceReference = withSurveySource(entry.sourceReference, floors.get(item.surveyId) ?? {}, item.surveyId);
  }
  return next;
}
