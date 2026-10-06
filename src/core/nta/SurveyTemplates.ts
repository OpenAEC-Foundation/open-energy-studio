import residentialExample from '../../../training-data/nta8800-opname-1930-terraced.json';
import utilityExample from '../../../training-data/nta8800-opname-utility-1985-office.json';
import type { OpnameAssessment, ResidentialSurvey, UtilitySurvey } from './KernelClient';
import { assessResidentialSurveyWithRust, assessUtilitySurveyWithRust } from './KernelClient';

// Starting points of the ISSO 82.1 / 75.1 basisopname editor: the synthetic
// survey fixtures of the kernel tests, edited by the adviser.

export type SurveyKind = 'residential' | 'utility';

export interface StoredSurvey {
  kind: SurveyKind;
  survey: Record<string, unknown>;
  /** Question-flow progress (src/core/survey/surveyFlow.ts); never sent to the kernel. */
  progress?: { done?: string[]; skipped?: string[] };
}

export function surveyTemplate(kind: SurveyKind): StoredSurvey {
  const source = kind === 'residential' ? residentialExample : utilityExample;
  return { kind, survey: structuredClone(source) as Record<string, unknown> };
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
