import type { BuildingPerformanceAssessment, MonthlyDemandAssessment } from './KernelClient';

export interface NtaVentilationSummary {
  zoneId: string;
  requiredJanuaryM3PerH: number;
  infiltrationJanuaryM3PerH: number;
  conductanceJanuaryWPerK: number;
  fanKwh: number;
  frostProtectionKwh: number;
  grillePreheatingKwh: number;
}

export interface NtaResultExtras {
  ventilation: NtaVentilationSummary[];
  /** How BENG 1 was obtained: the §5.4.2 C1 run, a confirmed C1 input, or not at all. */
  beng1Basis: 'fixed_c1' | 'confirmed' | null;
  fixedC1NeedKwh: number | null;
  recoverableLossesApplied: boolean;
  recoverableLossKwh: number;
  lightingKwh: number | null;
  co2Kg: number | null;
  co2KgPerM2: number | null;
  storageCorrectionKwh: number | null;
}

/** Outputs beyond BENG 1/2/3 that the panel and report show. Tolerates older payloads. */
export function summarizeExtras(performance: BuildingPerformanceAssessment): NtaResultExtras {
  const heating = performance.spaceHeating;
  const zones: Array<Partial<MonthlyDemandAssessment>> = [heating.demand, ...(heating.additionalZoneDemands ?? [])];
  const ventilation = zones.flatMap((zone) => {
    const result = zone.ventilation;
    if (!result) return [];
    const january = result.months[0];
    return [{
      zoneId: result.zoneId,
      requiredJanuaryM3PerH: january?.heating.requiredOutdoorAirM3PerH ?? 0,
      infiltrationJanuaryM3PerH: january?.heating.infiltrationM3PerH ?? 0,
      conductanceJanuaryWPerK: january?.heating.conductanceWPerK ?? 0,
      fanKwh: result.annualFanElectricityKwh,
      frostProtectionKwh: result.annualFrostProtectionElectricityKwh,
      grillePreheatingKwh: result.annualGrillePreheatingElectricityKwh,
    }];
  });
  const runs = zones.map((zone) => zone.fixedC1);
  const allFixed = runs.length > 0 && runs.every((run) => run?.status === 'calculated_unverified');
  const fixedC1NeedKwh = allFixed
    ? runs.reduce((sum, run) => sum + (run?.annualHeatingNeedKwh ?? 0) + (run?.annualCoolingNeedKwh ?? 0), 0)
    : null;
  const beng1Basis = performance.needIndicatorKwhPerM2Year == null ? null : allFixed ? 'fixed_c1' : 'confirmed';
  const recoverableLossKwh = (heating.monthly ?? []).reduce((sum, row) => sum + (row.recoverableLossKwh ?? 0), 0);
  return {
    ventilation,
    beng1Basis,
    fixedC1NeedKwh,
    recoverableLossesApplied: zones.some((zone) => zone.recoverableLossesApplied === true),
    recoverableLossKwh,
    lightingKwh: performance.lighting && performance.lighting.length > 0
      ? performance.lighting.reduce((sum, zone) => sum + zone.annualKwh, 0) : null,
    co2Kg: performance.annualCo2Kg ?? null,
    co2KgPerM2: performance.co2KgPerM2 ?? null,
    storageCorrectionKwh: performance.annualStorageCorrectionKwh ?? null,
  };
}
