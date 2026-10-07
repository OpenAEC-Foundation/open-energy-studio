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

/** Blocks of the main heating chain (`spaceHeating.<key>` → `<key>`). */
const HEATING_CHAIN = ['emission', 'distribution', 'generator', 'collectiveConnection'] as const;

/** Demand blocks of a single-zone survey (`spaceHeating.demand.<key>` → `<key>`). */
const DEMAND = ['usageFunction', 'dwellingType', 'setpoints', 'thermalMass', 'internalGains', 'ventilationFlows', 'ventilation'] as const;

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

/** Writes the derived survey input into a copy of the NTA draft. */
export function surveyTakeover(derived: BuildingPerformanceInput, current: Loose | null): SurveyTakeover {
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
  const heating = (source.spaceHeating ?? {}) as Loose;
  for (const key of HEATING_CHAIN) put(key, heating[key]);
  const demand = (heating.demand ?? {}) as Loose;
  const zones = Array.isArray(heating.additionalZones) ? heating.additionalZones : [];
  if (zones.length === 0) {
    for (const key of DEMAND) put(key, demand[key]);
  } else {
    skipped.push({ path: 'spaceHeating.additionalZones', reason: 'multi_zone' });
  }
  if (Array.isArray(source.additionalHeatingSystems) && source.additionalHeatingSystems.length > 0) {
    skipped.push({ path: 'additionalHeatingSystems', reason: 'additional_heating' });
  }
  skipped.push({ path: 'spaceHeating.demand.transmission', reason: 'geometry' });
  skipped.push({ path: 'totalUsableFloorAreaM2', reason: 'geometry' });
  return { next, keys, skipped };
}

/** A short display of a value in the diff preview. */
export function previewValue(value: unknown): string {
  if (value === undefined) return '—';
  const text = typeof value === 'string' ? value : JSON.stringify(value);
  return text.length > 60 ? `${text.slice(0, 57)}…` : text;
}
