import residentialExample from '../../../training-data/nta8800-opname-1930-terraced.json';
import utilityExample from '../../../training-data/nta8800-opname-utility-1985-office.json';
import type { OpnameAssessment, ResidentialSurvey, UtilitySurvey } from './KernelClient';
import { assessResidentialSurveyWithRust, assessUtilitySurveyWithRust } from './KernelClient';

// Starting points of the ISSO 82.1 / 75.1 basisopname editor. A new dwelling
// survey starts empty: no example surfaces, sources or answers that count
// without being visible (found entering ISSO 54 EPWRealB 01). The utility
// survey still starts from the synthetic office fixture of the kernel tests.

export type SurveyKind = 'residential' | 'utility';

export interface StoredSurvey {
  kind: SurveyKind;
  survey: Record<string, unknown>;
  /** Question-flow progress (src/core/survey/surveyFlow.ts); never sent to the kernel. */
  progress?: { done?: string[]; skipped?: string[] };
}

/** An empty dwelling survey: only the answers that are "unknown" by default (ISSO 82.1 defaults apply). */
export function emptyResidentialSurvey(): Record<string, unknown> {
  return {
    id: '',
    areaSourceReference: '',
    envelope: { surfaces: [], windows: [], doors: [] },
    heating: { control: 'unknown', sourceReference: '' },
    hotWater: { served: 'kitchen_and_bathroom', showerHeatRecovery: 'unknown', sourceReference: '' },
    ventilation: { sourceReference: '' },
    pv: [],
    sourceReference: '',
  };
}

/** The start of a new survey: empty for a dwelling, the office example for a utility building. */
export function surveyTemplate(kind: SurveyKind): StoredSurvey {
  return kind === 'residential'
    ? { kind, survey: emptyResidentialSurvey() }
    : { kind, survey: structuredClone(utilityExample) as Record<string, unknown> };
}

/** The synthetic example survey of a kind (kernel test fixture), e.g. for demos and tests. */
export function surveyExample(kind: SurveyKind): StoredSurvey {
  const source = kind === 'residential' ? residentialExample : utilityExample;
  return { kind, survey: structuredClone(source) as Record<string, unknown> };
}

/**
 * The answers a dwelling survey needs before the kernel can read it, as survey
 * paths. The kernel rejects a survey without them as unreadable, so the app
 * names them instead (the question flow leads to each).
 */
export function missingSurveyAnswers(stored: StoredSurvey): string[] {
  if (stored.kind !== 'residential') return [];
  const survey = stored.survey as Record<string, unknown>;
  const at = (path: string): unknown => path.split('.').reduce<unknown>(
    (value, key) => (value != null && typeof value === 'object' ? (value as Record<string, unknown>)[key] : undefined), survey);
  const blank = (value: unknown) => value == null || value === '' || (typeof value === 'number' && !Number.isFinite(value));
  const missing: string[] = [];
  const dwelling = at('dwelling') as Record<string, unknown> | undefined;
  if (!dwelling?.kind) missing.push('dwelling');
  else if (dwelling.kind === 'single_family') {
    if (blank(dwelling.position)) missing.push('dwelling');
    if (blank(dwelling.roofType)) missing.push('dwelling.roofType');
  } else if (dwelling.kind === 'apartment') {
    if (blank(dwelling.floor)) missing.push('dwelling.floor');
  }
  for (const path of ['constructionYear', 'usableFloorAreaM2', 'buildingHeightM', 'construction.floor', 'construction.wall',
    'heating.generator', 'heating.emitters', 'hotWater.generator', 'ventilation.principle']) {
    if (blank(at(path))) missing.push(path);
  }
  return missing;
}

/** Kernel shape of a survey heating generator for a chosen kind. */
export function heatingGeneratorTemplate(kind: string): Record<string, unknown> {
  switch (kind) {
    case 'boiler': return { kind, boilerType: 'hr107', insideThermalBoundary: true };
    case 'heat_pump': return { kind, source: 'outdoor_air' };
    case 'electric': return { kind, connectedDevices: 1 };
    case 'biomass': return { kind, appliance: 'pellet_stove', insideThermalBoundary: true, soleHeatingInServedRooms: false };
    case 'chp': return { kind, electricalPowerKw: 20 };
    case 'local_fired': return { kind, appliance: 'gas_heater', flueGasExhaust: true, electricityConnected: null };
    case 'gas_air_heater': return { kind, heaterType: 'conventional', pilotFlame: null, count: null };
    default: return { kind };
  }
}

/** Kernel shape of a survey hot-water generator for a chosen kind. */
export function hotWaterGeneratorTemplate(kind: string): Record<string, unknown> {
  switch (kind) {
    case 'gas_appliance': return { kind, applianceType: 'combi', gaskeur: 'unknown' };
    case 'heat_pump': return { kind, exhaustAirSource: false };
    case 'gas_storage_heater': return { kind, volumeL: 200 };
    default: return { kind };
  }
}

export function solarTemplate(index: number): Record<string, unknown> {
  return {
    id: `zb${index + 1}`, collector: 'glazed', collectorAreaM2: 4, orientation: 'south', tiltDeg: 45,
    backup: 'separate_heater', storageVolumeL: 200, sourceReference: '',
  };
}

export function pvTemplate(index: number): Record<string, unknown> {
  return {
    id: `pv${index + 1}`, panelAreaM2: 10, moduleType: 'monocrystalline', azimuthDeg: 180, tiltDeg: 35,
    mounting: 'moderately_ventilated', sourceReference: '',
  };
}

export function windowTemplate(index: number, surfaceId: string): Record<string, unknown> {
  return { id: `raam${index + 1}`, surfaceId, areaM2: 2, glass: 'hr_plus_plus', frame: 'wood_or_plastic', sourceReference: '' };
}

/** A calculation zone of the utility survey (ISSO 75.1 §6.5); the functions are added by the adviser. */
export function calculationZoneTemplate(index: number): Record<string, unknown> {
  return { id: `zone${index + 1}`, functions: [] };
}

/** Typed views for the kernel calls; the kernel validates the content. */
/** The lists whose items carry their own source reference in the kernel input. */
const SOURCED_LISTS: string[][] = [
  ['envelope', 'surfaces'], ['envelope', 'windows'], ['envelope', 'doors'], ['pv'], ['hotWater', 'solar'],
];
/** The answer blocks that carry a source reference of their own. */
const SOURCED_BLOCKS = ['construction', 'heating', 'hotWater', 'ventilation', 'cooling'];

/**
 * The survey with the source of the building data ("Bron van de gebouwgegevens",
 * one per survey as on the ISSO opnameformulier) on every surface, window, door
 * and PV system without a source of its own. Without any source the kernel still
 * reports each item, so nothing is invented.
 */
export function withSurveySources(survey: Record<string, unknown>): Record<string, unknown> {
  const source = typeof survey.sourceReference === 'string' ? survey.sourceReference.trim() : '';
  if (!source) return survey;
  const next = structuredClone(survey);
  const empty = (value: unknown) => typeof value !== 'string' || value.trim() === '';
  if ('areaSourceReference' in next && empty(next.areaSourceReference)) next.areaSourceReference = source;
  for (const key of SOURCED_BLOCKS) {
    const block = next[key];
    if (block && typeof block === 'object' && !Array.isArray(block) && empty((block as Record<string, unknown>).sourceReference)) {
      (block as Record<string, unknown>).sourceReference = source;
    }
  }
  for (const path of SOURCED_LISTS) {
    let parent: unknown = next;
    for (const key of path.slice(0, -1)) parent = (parent as Record<string, unknown> | undefined)?.[key];
    const list = (parent as Record<string, unknown> | undefined)?.[path[path.length - 1]];
    if (!Array.isArray(list)) continue;
    for (const item of list) {
      if (item && typeof item === 'object') {
        const own = (item as Record<string, unknown>).sourceReference;
        if (typeof own !== 'string' || own.trim() === '') (item as Record<string, unknown>).sourceReference = source;
      }
    }
  }
  return next;
}

export function asResidential(stored: StoredSurvey): ResidentialSurvey {
  return withSurveySources(stored.survey) as unknown as ResidentialSurvey;
}

export function asUtility(stored: StoredSurvey): UtilitySurvey {
  return withSurveySources(stored.survey) as unknown as UtilitySurvey;
}

/**
 * Kernel assessment of the survey kept with the project, for the collapse
 * reasons in the dossier checklist (BRL 9500 Bijlage 3). `null` without a
 * survey or when the kernel cannot assess it.
 */
export async function assessStoredSurvey(stored: StoredSurvey | undefined | null): Promise<OpnameAssessment | null> {
  if (!stored) return null;
  try {
    return stored.kind === 'residential'
      ? await assessResidentialSurveyWithRust(asResidential(stored))
      : await assessUtilitySurveyWithRust(asUtility(stored));
  } catch {
    return null;
  }
}
