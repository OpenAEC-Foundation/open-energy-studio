import residentialExample from '../../../training-data/nta8800-opname-1930-terraced.json';
import utilityExample from '../../../training-data/nta8800-opname-utility-1985-office.json';
import type { ResidentialSurvey, UtilitySurvey } from './KernelClient';

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

/** Typed views for the kernel calls; the kernel validates the content. */
export function asResidential(stored: StoredSurvey): ResidentialSurvey {
  return stored.survey as unknown as ResidentialSurvey;
}

export function asUtility(stored: StoredSurvey): UtilitySurvey {
  return stored.survey as unknown as UtilitySurvey;
}
