import type { ProjectPerformanceAssessment } from './KernelClient';

/** Figures the live preview shows; all come from the Rust kernel. */
export interface PreviewSummary {
  status: ProjectPerformanceAssessment['status'];
  beng1: number | null;
  beng2: number | null;
  beng3: number | null;
  beng1Limit: number | null;
  beng2Limit: number | null;
  beng3Limit: number | null;
  tojuliMaxK: number | null;
  tojuliMeetsLimit: boolean | null;
  /** TOjuli has a Bbl limit for dwellings only (§5.7); the preview hides it for utility. */
  tojuliApplies: boolean;
  labelClass: string | null;
  zebIndicator: number | null;
  finalEnergyKwh: number | null;
  co2KgPerM2: number | null;
  monthlyHeatingKwh: number[];
  monthlyCoolingKwh: number[];
  gapCount: number;
  issueCount: number;
}

/** Summarise a project-performance result for the preview panel. */
export function summarizeForPreview(assessment: ProjectPerformanceAssessment): PreviewSummary {
  const performance = assessment.status === 'calculated_unverified' ? assessment.performance : null;
  const limits = performance?.bblCheck?.limits ?? null;
  const zones = performance
    ? [performance.spaceHeating.demand, ...performance.spaceHeating.additionalZoneDemands]
    : [];
  const monthly = (pick: 'heating' | 'cooling') => Array.from({ length: 12 }, (_, index) =>
    zones.reduce((sum, zone) => sum + (zone.monthly[index]?.[pick].needKwh ?? 0), 0));
  return {
    status: assessment.status,
    beng1: performance?.needIndicatorKwhPerM2Year ?? null,
    beng2: performance?.primaryFossilIndicatorKwhPerM2Year ?? null,
    beng3: performance?.renewableSharePercent ?? null,
    beng1Limit: limits?.energyNeedMaxKwhPerM2 ?? null,
    beng2Limit: limits?.primaryFossilMaxKwhPerM2 ?? null,
    beng3Limit: limits?.renewableShareMinPercent ?? null,
    tojuliMaxK: performance?.tojuliMaxK ?? null,
    tojuliMeetsLimit: performance?.tojuliMeetsBblLimit ?? null,
    tojuliApplies: assessment.derivedInput?.calculationScope !== 'utility',
    labelClass: performance?.indicativeLabelClass ?? null,
    zebIndicator: performance?.zebPrimaryTotalIndicatorKwhPerM2 ?? null,
    finalEnergyKwh: performance?.annualFinalEnergyKwh ?? null,
    co2KgPerM2: performance?.co2KgPerM2 ?? null,
    monthlyHeatingKwh: monthly('heating'),
    monthlyCoolingKwh: monthly('cooling'),
    gapCount: assessment.gaps.length,
    issueCount: assessment.performance?.issues.length ?? 0,
  };
}
