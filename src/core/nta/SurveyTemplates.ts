import residentialExample from '../../../training-data/nta8800-opname-1930-terraced.json';
import utilityExample from '../../../training-data/nta8800-opname-utility-1985-office.json';
import type { NormVersion, OpnameAssessment, ResidentialSurvey, UtilitySurvey } from './KernelClient';
import { assessResidentialSurveyWithRust, assessUtilitySurveyWithRust, DEFAULT_NORM_VERSION } from './KernelClient';

// Starting points of the ISSO 82.1 / 75.1 basisopname editor: the synthetic
// survey fixtures of the kernel tests, edited by the adviser.

export type SurveyKind = 'residential' | 'utility';

export interface StoredSurvey {
  kind: SurveyKind;
  survey: Record<string, unknown>;
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
/**
 * The survey in the project's NTA 8800 edition (`ntaCalculation.normVersion`):
 * the kernel calculates it in that edition; absent is the current one. An
 * older edition is a comparison only (the ISSO protocol is the 2025 one).
 */
function inEdition<T>(survey: T, edition: NormVersion | null | undefined): T {
  const { normVersion: _stored, ...rest } = survey as T & { normVersion?: NormVersion };
  return (edition && edition !== DEFAULT_NORM_VERSION ? { ...rest, normVersion: edition } : rest) as T;
}

export function asResidential(stored: StoredSurvey, edition?: NormVersion | null): ResidentialSurvey {
  return inEdition(stored.survey as unknown as ResidentialSurvey, edition);
}

export function asUtility(stored: StoredSurvey, edition?: NormVersion | null): UtilitySurvey {
  return inEdition(stored.survey as unknown as UtilitySurvey, edition);
}

/**
 * Kernel assessment of the survey kept with the project, for the collapse
 * reasons in the dossier checklist (BRL 9500 Bijlage 3). `null` without a
 * survey or when the kernel cannot assess it.
 */
export async function assessStoredSurvey(
  stored: StoredSurvey | undefined | null,
  edition?: NormVersion | null,
): Promise<OpnameAssessment | null> {
  if (!stored) return null;
  try {
    return stored.kind === 'residential'
      ? await assessResidentialSurveyWithRust(asResidential(stored, edition))
      : await assessUtilitySurveyWithRust(asUtility(stored, edition));
  } catch {
    return null;
  }
}
