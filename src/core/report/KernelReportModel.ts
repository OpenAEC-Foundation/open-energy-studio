import type { IEnergyBreakdown } from '../energy/types';
import { projectCalculated, type BuildingPerformanceAssessment, type MonthlyDemandAssessment, type ProjectPerformanceAssessment } from '../nta/KernelClient';
import { kernelEnergyBreakdown } from '../nta/KernelBreakdown';

/** One BENG row: value, Bbl limit and the kernel's own toets (null when not testable). */
export interface KernelIndicatorRow {
  key: 'beng1' | 'beng2' | 'beng3';
  value: number | null;
  limit: number | null;
  meets: boolean | null;
  higherIsBetter: boolean;
}

/**
 * Decimals for a BENG value and its limit. Both use the same precision so a
 * row can never print a limit that contradicts its own toets (the kernel
 * compares unrounded values with unrounded limits): EP1/EP2 and TO_juli to
 * 0,01, EP3 to 0,1 (NTA 8800 §5.3.1, p. 72–74; TO_juli p. 120).
 */
export function indicatorDecimals(key: KernelIndicatorRow['key'] | 'tojuli'): number {
  return key === 'beng3' ? 1 : 2;
}

export interface KernelReportModel {
  indicators: KernelIndicatorRow[];
  /** TO_juli for dwellings; null for utility (§5.7 applies to the woonfunctie only). */
  tojuli: { value: number | null; meets: boolean | null } | null;
  labelClass: string | null;
  usableFloorAreaM2: number | null;
  /** Monthly heating-balance terms summed over the calculation zones (chapter 7). */
  monthly: Array<{ month: number; heatingNeedKwh: number; coolingNeedKwh: number; solarGainKwh: number; transmissionKwh: number }>;
  breakdown: IEnergyBreakdown;
  pvSystems: Array<{ id: string; annualKwh: number }>;
}

function zonesOf(performance: BuildingPerformanceAssessment): Array<Partial<MonthlyDemandAssessment>> {
  const heating = performance.spaceHeating;
  return [heating.demand, ...(heating.additionalZoneDemands ?? [])];
}

/**
 * The figures of the BENG report taken from the NTA kernel, so the in-app report, the
 * exported HTML and the results view show the same numbers. Null when the kernel has
 * no calculated result.
 */
export function kernelReportModel(assessment: ProjectPerformanceAssessment | null | undefined): KernelReportModel | null {
  const performance = projectCalculated(assessment?.status) ? assessment?.performance ?? null : null;
  if (!assessment || !performance) return null;
  const bbl = performance.bblCheck ?? null;
  const utility = assessment.derivedInput?.calculationScope === 'utility';
  const zones = zonesOf(performance);
  const monthly = Array.from({ length: 12 }, (_, index) => {
    const rows = zones.map((zone) => zone.monthly?.[index]).filter((row) => row != null);
    const sum = (pick: (row: NonNullable<(typeof rows)[number]>) => number) =>
      rows.reduce((total, row) => total + (Number.isFinite(pick(row)) ? pick(row) : 0), 0);
    return {
      month: index + 1,
      heatingNeedKwh: sum((row) => row.heating.needKwh),
      coolingNeedKwh: sum((row) => row.cooling.needKwh),
      solarGainKwh: sum((row) => row.windowSolarGainsKwh + row.opaqueSolarGainsKwh),
      transmissionKwh: sum((row) => row.heating.transmissionKwh),
    };
  });
  return {
    indicators: [
      { key: 'beng1', value: performance.needIndicatorKwhPerM2Year ?? null, limit: bbl?.limits.energyNeedMaxKwhPerM2 ?? null,
        meets: bbl?.energyNeedMeets ?? null, higherIsBetter: false },
      { key: 'beng2', value: performance.primaryFossilIndicatorKwhPerM2Year ?? null, limit: bbl?.limits.primaryFossilMaxKwhPerM2 ?? null,
        meets: bbl?.primaryFossilMeets ?? null, higherIsBetter: false },
      { key: 'beng3', value: performance.renewableSharePercent ?? null, limit: bbl?.limits.renewableShareMinPercent ?? null,
        meets: bbl?.renewableShareMeets ?? null, higherIsBetter: true },
    ],
    tojuli: utility ? null : { value: performance.tojuliMaxK ?? null, meets: performance.tojuliMeetsBblLimit ?? null },
    labelClass: performance.indicativeLabelClass ?? null,
    usableFloorAreaM2: assessment.geometry?.usableFloorAreaM2 ?? null,
    monthly,
    breakdown: kernelEnergyBreakdown(performance),
    pvSystems: (performance.pvSystems ?? []).map((system) => ({ id: system.id, annualKwh: system.annualKwh })),
  };
}
