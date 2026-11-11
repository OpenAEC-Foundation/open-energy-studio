/**
 * The `energy_*` tools for the assistant of OpenAEC SuperCloud (browser build only, `window.__OPDS_APP__`).
 *
 * The site sends the tool calls of its assistant through its bridge to this interface in the frame of the application.
 * The tools read the open documents and change them with the actions of the app (EnergyContext), through the store
 * that EnergyProvider registers (context/documentStore.ts); the calculation is the one behind "Bereken": the NTA 8800
 * kernel (WebAssembly in the browser build) and the indicative BENG engine. No network, no popups, no messages to other
 * windows. Opening and saving stay with the user. The app has no undo, so there is no energy_undo.
 *
 * Contract: OpenAEC-Blueprint docs/supercloud-koppeling.md. An error is `throw new Error("short_code: ...")`.
 */
import definitions from './siteAppTools.json';
import type { DocumentStore } from '../../context/documentStore';
import { getDocumentStore } from '../../context/documentStore';
import type { EnergyAction, EnergyState } from '../../context/EnergyContext';
import type { IBENGResult, IBENGResultMonthly, IProject, IZone, SurfaceType } from '../energy/types';
import type { ProjectPerformanceAssessment } from '../nta/KernelClient';
import { projectCalculated } from '../nta/KernelClient';
import { kernelVerdict } from '../nta/KernelVerdict';
import { calculateProjectPerformanceShared } from '../nta/useProjectPerformance';
import { indicativeCalculation } from '../energy/indicativeCalculation';
import { SURFACE_RESISTANCE } from '../energy/Constants';
import { nl } from '../../i18n/nl';

export interface SiteAppToolDefinition { name: string; description: string; inputSchema: object }
export interface SiteAppTools {
  tools(): Promise<string[]>;
  definitions(): Promise<SiteAppToolDefinition[]>;
  run(name: string, args?: Record<string, unknown>): Promise<unknown>;
}

type Args = Record<string, unknown>;
type Row = Record<string, unknown>;
type Kernel = (project: IProject) => Promise<ProjectPerformanceAssessment>;

/** A tool answer goes into the context of a model; a bigger answer has to be narrowed down. */
const MAX_RESULT_CHARS = 30000;
const GAPS_PER_ANSWER = 20;

interface Context { store: DocumentStore; kernel: Kernel }

function active(store: DocumentStore): { id: string; state: EnergyState } {
  const docs = store.getState();
  const doc = docs.documents.find(d => d.id === docs.activeDocumentId);
  if (!doc) throw new Error('geen_document: open of maak eerst een project');
  return { id: doc.id, state: doc.state };
}

/** One action of the app on the active document; returns the state after React committed it. */
function apply(store: DocumentStore, action: EnergyAction): EnergyState {
  const { id } = active(store);
  store.dispatch({ type: 'DOC_DISPATCH', payload: { id, action } });
  return active(store).state;
}

const pick = (args: Args, keys: readonly string[]) => Object.fromEntries(keys.filter(k => args[k] !== undefined).map(k => [k, args[k]]));
const round = (value: number | null | undefined, digits = 2) => value == null || !Number.isFinite(value) ? null : Number(value.toFixed(digits));

function zoneOf(project: IProject, id: unknown): IZone {
  const zone = project.zones.find(z => z.id === id);
  if (!zone) throw new Error(`zone_onbekend: ${id}`);
  return zone;
}

function surfaceOf(project: IProject, zoneId: unknown, surfaceId: unknown) {
  const surface = zoneOf(project, zoneId).surfaces.find(s => s.id === surfaceId);
  if (!surface) throw new Error(`vlak_onbekend: ${surfaceId}`);
  return surface;
}

function stand({ store }: Context) {
  const docs = store.getState();
  const documents = docs.documents.map(d => ({ id: d.id, name: d.state.project.name, file: d.filePath, isDirty: d.state.isDirty }));
  const doc = docs.documents.find(d => d.id === docs.activeDocumentId);
  if (!doc) return { documents, activeDocumentId: null };
  const { project, route, result, isDirty } = doc.state;
  return { documents, activeDocumentId: doc.id,
    project: { name: project.name, description: project.description, buildingFunction: project.buildingFunction, address: project.address, city: project.city,
      normVersion: project.ntaCalculation?.normVersion ?? null, hasNtaInput: Boolean(project.ntaCalculation) },
    building: { floorArea: round(project.zones.reduce((sum, z) => sum + (z.floorArea || 0), 0)),
      zones: project.zones.map(z => ({ id: z.id, name: z.name, floorArea: z.floorArea, surfaces: z.surfaces.length })),
      constructions: project.constructions.length,
      installations: { verwarming: project.heatingSystems.length, ventilatie: project.ventilationSystems.length, koeling: project.coolingSystems.length,
        tapwater: project.hotWaterSystems.length, pv: project.solarPV.length, zonthermisch: project.solarThermal.length } },
    screen: route, isDirty, indicativeResult: result ? indicative(result) : null };
}

function zones({ store }: Context, args: Args) {
  const { project } = active(store).state;
  const list = typeof args.zoneId === 'string' ? [zoneOf(project, args.zoneId)] : project.zones;
  const construction = new Map(project.constructions.map(c => [c.id, c.name]));
  return { zones: list.map(z => ({ id: z.id, name: z.name, floorArea: z.floorArea, volume: z.volume, height: z.height,
    qv10: z.airTightness?.qv10 ?? null, qv10Assumed: z.airTightness?.assumed || undefined,
    thermalBridges: z.thermalBridges.length, pointThermalBridges: z.pointThermalBridges?.length || undefined,
    surfaces: z.surfaces.map(s => ({ id: s.id, name: s.name, type: s.type, thermalBoundary: s.thermalBoundary ?? 'outdoor', area: s.area,
      orientation: s.orientation, constructionId: s.constructionId, construction: construction.get(s.constructionId) ?? null,
      windows: s.windows.map(w => ({ id: w.id, name: w.name, area: w.area, uValue: w.uValue, gValue: w.gValue, orientation: w.orientation,
        typeId: w.typeId, count: w.count })) })) })) };
}

function constructions({ store }: Context) {
  const { project } = active(store).state;
  const uses = new Map<string, number>();
  for (const z of project.zones) for (const s of z.surfaces) uses.set(s.constructionId, (uses.get(s.constructionId) ?? 0) + 1);
  return { constructions: project.constructions.map(c => ({ id: c.id, name: c.name, rcValue: c.rcValue, uValue: c.uValue,
      layers: c.layers.map(l => ({ material: l.material, thickness: l.thickness, lambda: l.lambda })), surfaces: uses.get(c.id) ?? 0 })),
    windowTypes: (project.windowTypes ?? []).map(t => ({ id: t.id, name: t.name, kind: t.kind, uValue: t.uValue, gValue: t.gValue, unitArea: t.unitArea })) };
}

/** Per kind: the list in the project, the fields the tool may set with their type, the defaults of the dialog, the actions. */
const INSTALLATIONS = {
  verwarming: { list: 'heatingSystems', add: 'ADD_HEATING_SYSTEM', update: 'UPDATE_HEATING_SYSTEM',
    fields: { name: 'string', type: ['hr107', 'hr_combi', 'heat_pump_air', 'heat_pump_ground', 'district_heating', 'electric', 'biomass'], cop: 'number', coverageFraction: 'fraction' },
    defaults: { name: 'Verwarming', type: 'hr107', cop: 0.95, coverageFraction: 1 } },
  ventilatie: { list: 'ventilationSystems', add: 'ADD_VENTILATION_SYSTEM', update: 'UPDATE_VENTILATION_SYSTEM',
    fields: { name: 'string', type: ['natural', 'type_c', 'type_d'], heatRecoveryEfficiency: 'fraction', sfp: 'number' },
    defaults: { name: 'Ventilatie', type: 'natural', heatRecoveryEfficiency: 0, sfp: 0 } },
  koeling: { list: 'coolingSystems', add: 'ADD_COOLING_SYSTEM', update: 'UPDATE_COOLING_SYSTEM',
    fields: { name: 'string', type: ['none', 'split_unit', 'central_chiller', 'heat_pump_reversible'], eer: 'number' },
    defaults: { name: 'Koeling', type: 'split_unit', eer: 3 } },
  tapwater: { list: 'hotWaterSystems', add: 'ADD_HOT_WATER_SYSTEM', update: 'UPDATE_HOT_WATER_SYSTEM',
    fields: { name: 'string', type: ['hr_combi', 'heat_pump', 'electric_boiler', 'solar_boiler', 'district_heating'], efficiency: 'number', hasSolarBoiler: 'boolean', solarBoilerFraction: 'fraction' },
    defaults: { name: 'Warm tapwater', type: 'hr_combi', efficiency: 0.95, hasSolarBoiler: false, solarBoilerFraction: 0 } },
  pv: { list: 'solarPV', add: 'ADD_SOLAR_PV', update: 'UPDATE_SOLAR_PV',
    fields: { name: 'string', peakPower: 'number', orientation: ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'], tilt: 'number', area: 'number' },
    defaults: { name: 'PV', peakPower: 0, orientation: 'S', tilt: 35, area: 0 } },
  zonthermisch: { list: 'solarThermal', add: 'ADD_SOLAR_THERMAL', update: 'UPDATE_SOLAR_THERMAL',
    fields: { name: 'string', type: ['flat_plate', 'vacuum_tube'], collectorArea: 'number', orientation: ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'], tilt: 'number' },
    defaults: { name: 'Zonthermisch', type: 'flat_plate', collectorArea: 0, orientation: 'S', tilt: 45 } },
} as const;
type InstallationKind = keyof typeof INSTALLATIONS;

function installations({ store }: Context) {
  const { project } = active(store).state;
  return Object.fromEntries(Object.entries(INSTALLATIONS).map(([kind, def]) => [kind,
    (project[def.list] as unknown as Row[]).map(item => ({ id: item.id, ...Object.fromEntries(Object.keys(def.fields).map(k => [k, item[k]])),
      ntaHeatPump: item.ntaHeatPump ? true : undefined }))]));
}

function updateProject({ store }: Context, args: Args) {
  const state = apply(store, { type: 'UPDATE_PROJECT_INFO', payload: pick(args, ['name', 'description', 'buildingFunction', 'address', 'city']) });
  const { name, description, buildingFunction, address, city } = state.project;
  return { name, description, buildingFunction, address, city };
}

function updateZone({ store }: Context, args: Args) {
  const zone = zoneOf(active(store).state.project, args.zoneId);
  const data = pick(args, ['name', 'floorArea', 'volume', 'height']);
  if (Object.keys(data).length) apply(store, { type: 'UPDATE_ZONE', payload: { id: zone.id, data } });
  // A measured or chosen qv10 is no longer the assumed default.
  if (typeof args.qv10 === 'number') apply(store, { type: 'UPDATE_AIR_TIGHTNESS', payload: { zoneId: zone.id, airTightness: { qv10: args.qv10 } } });
  return zones({ store } as Context, { zoneId: zone.id }).zones[0];
}

function updateSurface({ store }: Context, args: Args) {
  const project = active(store).state.project;
  surfaceOf(project, args.zoneId, args.surfaceId);
  if (typeof args.constructionId === 'string' && !project.constructions.some(c => c.id === args.constructionId)) {
    throw new Error(`constructie_onbekend: ${args.constructionId}`);
  }
  const data = pick(args, ['name', 'area', 'orientation', 'constructionId', 'thermalBoundary']);
  const state = apply(store, { type: 'UPDATE_SURFACE', payload: { zoneId: String(args.zoneId), surfaceId: String(args.surfaceId), data } });
  const { windows, ...surface } = surfaceOf(state.project, args.zoneId, args.surfaceId);
  return { ...surface, windows: windows.length };
}

function updateWindow({ store }: Context, args: Args) {
  const project = active(store).state.project;
  const win = surfaceOf(project, args.zoneId, args.surfaceId).windows.find(w => w.id === args.windowId);
  if (!win) throw new Error(`raam_onbekend: ${args.windowId}`);
  const type = win.typeId ? project.windowTypes?.find(t => t.id === win.typeId) : undefined;
  if (type && (args.uValue !== undefined || args.gValue !== undefined || (args.area !== undefined && type.unitArea))) {
    throw new Error(`raam_heeft_type: U, g${type.unitArea ? ' en oppervlakte' : ''} komen van kozijntype "${type.name}" (${type.id})`);
  }
  const data = pick(args, ['name', 'area', 'uValue', 'gValue', 'orientation']);
  const state = apply(store, { type: 'UPDATE_WINDOW', payload: { zoneId: String(args.zoneId), surfaceId: String(args.surfaceId), windowId: win.id, data } });
  return surfaceOf(state.project, args.zoneId, args.surfaceId).windows.find(w => w.id === win.id);
}

function updateConstruction({ store }: Context, args: Args) {
  const project = active(store).state.project;
  const construction = project.constructions.find(c => c.id === args.constructionId);
  if (!construction) throw new Error(`constructie_onbekend: ${args.constructionId}`);
  // Rsi + Rse of the surfaces that use it (the editor uses those of a wall).
  const surfaceType: SurfaceType = project.zones.flatMap(z => z.surfaces).find(s => s.constructionId === construction.id)?.type ?? 'wall';
  const { rsi, rse } = SURFACE_RESISTANCE[surfaceType] ?? SURFACE_RESISTANCE.wall;
  const data: Row = pick(args, ['name']);
  const rc = args.rcValue as number | undefined;
  const u = args.uValue as number | undefined;
  if (rc !== undefined && u !== undefined) Object.assign(data, { rcValue: rc, uValue: u });
  else if (rc !== undefined) Object.assign(data, { rcValue: round(rc, 2), uValue: round(1 / (rsi + rc + rse), 3) });
  else if (u !== undefined) {
    const derived = 1 / u - rsi - rse;
    if (derived < 0) throw new Error(`u_te_hoog: bij U ${u} is Rc negatief`);
    Object.assign(data, { rcValue: round(derived, 2), uValue: u });
  }
  const state = apply(store, { type: 'UPDATE_CONSTRUCTION', payload: { id: construction.id, data } });
  const { id, name, rcValue, uValue } = state.project.constructions.find(c => c.id === construction.id)!;
  return { id, name, rcValue, uValue, surfaceType };
}

function updateInstallation({ store }: Context, args: Args) {
  const kind = args.kind as InstallationKind;
  const def = INSTALLATIONS[kind];
  const fields = args.fields as Row;
  for (const [key, value] of Object.entries(fields)) {
    const rule = (def.fields as Record<string, unknown>)[key];
    if (rule === undefined) throw new Error(`ongeldig_argument: fields.${key} hoort niet bij ${kind}; mogelijk: ${Object.keys(def.fields).join(', ')}`);
    const ok = Array.isArray(rule) ? rule.includes(value)
      : rule === 'fraction' ? typeof value === 'number' && value >= 0 && value <= 1
        : rule === 'number' ? typeof value === 'number' && Number.isFinite(value) && value >= 0
          : typeof value === rule;
    if (!ok) throw new Error(`ongeldig_argument: fields.${key} moet ${Array.isArray(rule) ? `een van ${rule.join(', ')}` : rule === 'fraction' ? 'een getal van 0 tot 1' : rule} zijn`);
  }
  const list = active(store).state.project[def.list] as unknown as Row[];
  let id = args.id as string | undefined;
  if (id !== undefined) {
    if (!list.some(item => item.id === id)) throw new Error(`installatie_onbekend: ${id}`);
    apply(store, { type: def.update, payload: { id, data: fields } } as EnergyAction);
  } else {
    id = crypto.randomUUID();
    apply(store, { type: def.add, payload: { ...def.defaults, ...fields, id } } as EnergyAction);
  }
  const saved = (active(store).state.project[def.list] as unknown as Row[]).find(item => item.id === id)!;
  return { kind, added: args.id === undefined, ...saved };
}

function indicative(result: IBENGResult) {
  const toJuli = (result as Partial<IBENGResultMonthly>).toJuli;
  return { beng1: round(result.beng1), beng1Limit: result.beng1Limit, beng1Pass: result.beng1Pass,
    beng2: round(result.beng2), beng2Limit: result.beng2Limit, beng2Pass: result.beng2Pass,
    beng3: round(result.beng3, 1), beng3Limit: result.beng3Limit, beng3Pass: result.beng3Pass,
    toJuli: toJuli ? { gto: round(toJuli.gto), limit: toJuli.limit, pass: toJuli.pass } : null, floorArea: round(result.totalFloorArea) };
}

/** The kernel answer in the terms of the results view: label, BENG 1/2/3 against the BBL limits, TOjuli. */
function kernelSummary(assessment: ProjectPerformanceAssessment) {
  const p = assessment.performance;
  const calculated = projectCalculated(assessment.status) && p;
  return { status: assessment.status, verdict: kernelVerdict(assessment), normVersion: assessment.normVersion ?? assessment.targetNormVersion,
    registrationEligible: assessment.registrationEligible ?? null,
    ...(calculated ? {
      energyLabel: p.indicativeLabelClass, labelSource: p.labelSource,
      beng1: { value: round(p.needIndicatorKwhPerM2Year), limit: p.bblCheck?.limits.energyNeedMaxKwhPerM2 ?? null, meets: p.bblCheck?.energyNeedMeets ?? null, unit: 'kWh/m²·jr' },
      beng2: { value: round(p.primaryFossilIndicatorKwhPerM2Year), limit: p.bblCheck?.limits.primaryFossilMaxKwhPerM2 ?? null, meets: p.bblCheck?.primaryFossilMeets ?? null, unit: 'kWh/m²·jr' },
      beng3: { value: round(p.renewableSharePercent, 1), limit: p.bblCheck?.limits.renewableShareMinPercent ?? null, meets: p.bblCheck?.renewableShareMeets ?? null, unit: '%' },
      toJuli: { max: round(p.tojuliMaxK), limit: 1.2, meets: p.tojuliMeetsBblLimit, unit: 'K',
        zones: p.tojuli.map(z => ({ zoneId: z.zoneId, max: round(z.maxTojuliK), activeCooling: z.activeCooling })) },
      usableFloorArea: round(assessment.geometry?.usableFloorAreaM2), co2KgPerM2: round(p.co2KgPerM2),
    } : {}),
    gaps: assessment.gaps.length,
    firstGaps: assessment.gaps.slice(0, GAPS_PER_ANSWER).map(g => ({ code: g.code, path: g.path, detail: g.detail })),
    warnings: (assessment.warnings?.length ?? 0) + (p?.warnings?.length ?? 0) };
}

async function outcome({ store, kernel }: Context) {
  const { project, result } = active(store).state;
  let nta: unknown;
  try { nta = kernelSummary(await kernel(project)); } catch (error) {
    nta = { status: 'kernel_niet_beschikbaar', error: error instanceof Error ? error.message : String(error) };
  }
  return { nta8800: nta, indicative: result ? indicative(result) : null };
}

async function calculate(ctx: Context) {
  const { project } = active(ctx.store).state;
  const run = indicativeCalculation(project);
  apply(ctx.store, { type: 'SET_RESULT', payload: run.result });
  if (run.result) apply(ctx.store, { type: 'NAVIGATE', payload: { step: 'results' } });
  const refused = run.errorKey ? (nl as Record<string, string>)[run.errorKey] ?? run.errorKey : run.error;
  return { ...await outcome(ctx), ...(refused ? { indicativeRefused: refused } : {}) };
}

const handlers: Record<string, (ctx: Context, args: Args) => unknown> = {
  energy_stand: stand,
  energy_zones: zones,
  energy_constructies: constructions,
  energy_installaties: installations,
  energy_project_bijwerken: updateProject,
  energy_zone_wijzigen: updateZone,
  energy_vlak_wijzigen: updateSurface,
  energy_raam_wijzigen: updateWindow,
  energy_constructie_wijzigen: updateConstruction,
  energy_installatie_wijzigen: updateInstallation,
  energy_bereken: calculate,
  energy_uitkomst: outcome,
};

type Schema = { type?: string | string[]; enum?: unknown[]; minimum?: number; maximum?: number; exclusiveMinimum?: number;
  required?: string[]; properties?: Record<string, Schema>; additionalProperties?: boolean };

/** Enough of JSON Schema for these definitions: type, enum, required, (exclusive) minimum/maximum, additionalProperties. */
export function validate(value: unknown, schema: Schema, path = 'argumenten'): void {
  const types = schema.type === undefined ? [] : ([] as string[]).concat(schema.type);
  const kind = value === null ? 'null' : Array.isArray(value) ? 'array' : typeof value;
  const ok = !types.length || types.some(t => t === kind || (t === 'integer' && Number.isInteger(value)) || (t === 'number' && kind === 'number' && Number.isFinite(value)));
  if (!ok) throw new Error(`ongeldig_argument: ${path} moet ${types.join(' of ')} zijn`);
  if (schema.enum && !schema.enum.includes(value)) throw new Error(`ongeldig_argument: ${path} moet een van ${schema.enum.join(', ')} zijn`);
  if (typeof value === 'number') {
    if (schema.minimum !== undefined && value < schema.minimum) throw new Error(`ongeldig_argument: ${path} is kleiner dan ${schema.minimum}`);
    if (schema.exclusiveMinimum !== undefined && value <= schema.exclusiveMinimum) throw new Error(`ongeldig_argument: ${path} moet groter zijn dan ${schema.exclusiveMinimum}`);
    if (schema.maximum !== undefined && value > schema.maximum) throw new Error(`ongeldig_argument: ${path} is groter dan ${schema.maximum}`);
  }
  if (kind === 'object' && schema.properties) {
    const obj = value as Row;
    for (const key of schema.required ?? []) if (obj[key] === undefined) throw new Error(`ongeldig_argument: ${path}.${key} ontbreekt`);
    for (const [key, v] of Object.entries(obj)) {
      const sub = schema.properties[key];
      if (!sub) { if (schema.additionalProperties === false) throw new Error(`ongeldig_argument: ${path}.${key} is onbekend`); continue; }
      if (v !== undefined) validate(v, sub, `${path}.${key}`);
    }
  }
}

function bounded(name: string, result: unknown): unknown {
  const json = JSON.stringify(result ?? null);
  if (json.length > MAX_RESULT_CHARS) throw new Error(`te_groot: het antwoord van ${name} heeft ${json.length} tekens; vraag één zone op met zoneId`);
  return JSON.parse(json);
}

export function createSiteAppTools(options: { store?: () => DocumentStore | null; kernel?: Kernel } = {}): SiteAppTools {
  const getStore = options.store ?? getDocumentStore;
  const kernel = options.kernel ?? calculateProjectPerformanceShared;
  return {
    async tools() { return definitions.tools.map(def => def.name); },
    async definitions() { return definitions.tools; },
    async run(name, args = {}) {
      if (!Object.prototype.hasOwnProperty.call(handlers, name)) throw new Error('onbekende_tool');
      validate(args, definitions.tools.find(def => def.name === name)!.inputSchema as Schema);
      const store = getStore();
      if (!store) throw new Error('app_niet_klaar');
      return bounded(name, await handlers[name]({ store, kernel }, args));
    },
  };
}

/** Put `window.__OPDS_APP__` in place (browser build only, main.tsx). */
export function installSiteAppTools(target: { __OPDS_APP__?: SiteAppTools } = window as never): void {
  target.__OPDS_APP__ = createSiteAppTools();
}
