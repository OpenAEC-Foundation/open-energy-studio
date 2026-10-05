/**
 * Data mapping of the results dashboard (UI redesign F7, mockup 05).
 *
 * Every number comes from the kernel answer of the KernelProvider run; nothing
 * is recomputed here except sums and per-m² divisions for display. The monthly
 * series are built so that their sum over all series and months equals the
 * kernel totals they claim to show (tested in results-dashboard.test.tsx):
 * - `primary`: Σ = annualPrimaryFossilKwh (EPTot incl. export credit and storage correction);
 * - `delivered`: Σ of the positive series = Σ delivered energy of §5.5.3 (= Σ carriers);
 * - `carrier`: Σ of the positive series = Σ delivered energy per carrier.
 */
import type {
  BuildingPerformanceAssessment, MonthlyDemandAssessment, NtaEnergyFunction, ProjectPerformanceAssessment,
} from '../../../../core/nta/KernelClient';
import { ENERGY_FUNCTIONS, SERVICE_CARRIERS } from '../../../../core/nta/ServiceEnergy';

export type ChartMode = 'delivered' | 'primary' | 'carrier';

export interface ChartSeries {
  key: string;
  /** i18n key of the series name. */
  labelKey: string;
  /** CSS colour (a `--viz-*` token). */
  color: string;
  /** Twelve monthly values, kWh; negative values are drawn below the zero line. */
  values: number[];
  /** Drawn with a hatch so it never relies on colour alone (PV, export credit). */
  hatched?: boolean;
}

/**
 * Fixed categorical order of the energy functions (validated with the dataviz
 * palette validator in both themes: adjacent CVD ΔE ≥ 8.3). Colour follows the
 * function, never its rank; heat-pump source energy folds into auxiliary energy.
 */
export const SERVICE_COLOR: Record<NtaEnergyFunction, string> = {
  heating: 'var(--viz-heating)',
  hotWater: 'var(--viz-dhw)',
  ventilation: 'var(--viz-fans)',
  lighting: 'var(--viz-lighting)',
  cooling: 'var(--viz-cooling)',
  humidification: 'var(--viz-humid)',
  auxiliary: 'var(--viz-aux)',
  heatPumpSource: 'var(--viz-aux)',
};

/** Stack order (bottom to top) of the energy functions. */
export const SERVICE_STACK: NtaEnergyFunction[] = [
  'heating', 'hotWater', 'ventilation', 'lighting', 'cooling', 'humidification', 'auxiliary',
];

/** Carriers in the same fixed categorical slots. */
export const CARRIER_COLOR: Record<string, string> = {
  el: 'var(--viz-cooling)',
  gas: 'var(--viz-heating)',
  oil: 'var(--viz-fans)',
  bm: 'var(--viz-dhw)',
  dh: 'var(--viz-humid)',
  dw: 'var(--viz-lighting)',
  dc: 'var(--viz-aux)',
};

const zeros = () => Array.from({ length: 12 }, () => 0);
const monthIndex = (month: number) => Math.min(11, Math.max(0, Math.round(month) - 1));
const finite = (value: number | null | undefined) => (value != null && Number.isFinite(value) ? value : 0);

function serviceOf(service: NtaEnergyFunction): NtaEnergyFunction {
  return service === 'heatPumpSource' ? 'auxiliary' : service;
}

/** Monthly PV production (chapter 16), kWh; positive numbers. */
export function monthlyPv(performance: BuildingPerformanceAssessment): number[] {
  const values = zeros();
  for (const system of performance.pvSystems ?? []) {
    system.monthlyKwh.forEach((value, index) => { if (index < 12) values[index] += finite(value); });
  }
  return values;
}

/** The monthly series of the energy chart for one mode. */
export function energySeries(performance: BuildingPerformanceAssessment, mode: ChartMode): ChartSeries[] {
  const breakdown = performance.energyByService;
  const months = breakdown?.months ?? [];
  const series: ChartSeries[] = [];

  if (mode === 'carrier') {
    for (const carrier of SERVICE_CARRIERS) {
      const values = zeros();
      for (const row of months) if (row.carrier === carrier) values[monthIndex(row.month)] += finite(row.deliveredKwh);
      if (values.some((value) => value !== 0)) {
        series.push({ key: carrier, labelKey: `results.carrier.${carrier}`, color: CARRIER_COLOR[carrier] ?? 'var(--viz-aux)', values });
      }
    }
  } else {
    for (const service of SERVICE_STACK) {
      const values = zeros();
      for (const row of months) {
        if (serviceOf(row.service) !== service) continue;
        values[monthIndex(row.month)] += finite(mode === 'primary' ? row.primaryFossilKwh : row.deliveredKwh);
      }
      if (values.some((value) => value !== 0)) {
        series.push({ key: service, labelKey: `nta.performance.service.${service}`, color: SERVICE_COLOR[service], values });
      }
    }
  }

  if (mode === 'primary') {
    // 5.10/5.13 export credit and 5.14a storage correction lower EPTot; drawn below zero.
    const credit = zeros();
    for (const row of breakdown?.adjustments ?? []) {
      credit[monthIndex(row.month)] -= finite(row.exportedElectricityCreditKwh) + finite(row.storageCorrectionKwh);
    }
    if (credit.some((value) => value !== 0)) {
      series.push({ key: 'credit', labelKey: 'results.series.exportCredit', color: 'var(--viz-pv)', values: credit, hatched: true });
    }
  } else {
    const pv = monthlyPv(performance).map((value) => -value);
    if (pv.some((value) => value !== 0)) {
      series.push({ key: 'pv', labelKey: 'results.series.pv', color: 'var(--viz-pv)', values: pv, hatched: true });
    }
  }
  return series;
}

/** Sum over every series and month (the chart's annual total). */
export function seriesTotal(series: ChartSeries[], sign: 'all' | 'positive' = 'all'): number {
  return series.reduce((total, item) => total + item.values.reduce((sum, value) =>
    sum + (sign === 'positive' && value < 0 ? 0 : value), 0), 0);
}

/** All zone demand assessments of the heating chain (main zone first). */
export function zoneDemands(performance: BuildingPerformanceAssessment): MonthlyDemandAssessment[] {
  const heating = performance.spaceHeating;
  return [heating.demand, ...(heating.additionalZoneDemands ?? [])].filter(Boolean);
}

/** Net heat and cold need per month (chapter 7), summed over zones, kWh. */
export function monthlyNeed(performance: BuildingPerformanceAssessment): { heat: number[]; cold: number[] } {
  const heat = zeros();
  const cold = zeros();
  for (const zone of zoneDemands(performance)) {
    for (const row of zone.monthly ?? []) {
      heat[monthIndex(row.month)] += finite(row.heating?.needKwh);
      cold[monthIndex(row.month)] += finite(row.cooling?.needKwh);
    }
  }
  return { heat, cold };
}

export interface Gauge {
  key: 'beng1' | 'beng2' | 'beng3' | 'tojuli';
  value: number | null;
  limit: number | null;
  /** BENG 3: higher is better. */
  higherIsBetter: boolean;
  /** true meets, false fails, null without a limit or value. */
  meets: boolean | null;
  /** Distance to the limit in the favourable direction (positive = margin, negative = excess). */
  margin: number | null;
  digits: number;
}

/**
 * The indicator gauges with the Bbl limits (art. 4.149, table 4.148A) and
 * TOjuli (art. 4.149b, 1,20 K, dwellings only). The verdict is the kernel's own
 * Bbl check where it has one, so the card never disagrees with the report.
 */
export function gauges(assessment: ProjectPerformanceAssessment): Gauge[] {
  const performance = assessment.performance;
  if (!performance) return [];
  const bbl = performance.bblCheck;
  const make = (key: Gauge['key'], value: number | null, limit: number | null, higherIsBetter: boolean,
    kernelMeets: boolean | null | undefined, digits: number): Gauge => {
    const known = value != null && limit != null && Number.isFinite(value) && Number.isFinite(limit);
    const margin = known ? (higherIsBetter ? value - limit : limit - value) : null;
    const meets = kernelMeets ?? (margin == null ? null : margin >= 0);
    return { key, value, limit, higherIsBetter, meets, margin, digits };
  };
  const result = [
    make('beng1', performance.needIndicatorKwhPerM2Year, bbl?.limits.energyNeedMaxKwhPerM2 ?? null, false, bbl?.energyNeedMeets, 2),
    make('beng2', performance.primaryFossilIndicatorKwhPerM2Year, bbl?.limits.primaryFossilMaxKwhPerM2 ?? null, false, bbl?.primaryFossilMeets, 2),
    make('beng3', performance.renewableSharePercent, bbl?.limits.renewableShareMinPercent ?? null, true, bbl?.renewableShareMeets, 1),
  ];
  if (isResidential(assessment)) {
    result.push(make('tojuli', performance.tojuliMaxK, 1.2, false, performance.tojuliMeetsBblLimit, 2));
  }
  return result;
}

export function isResidential(assessment: ProjectPerformanceAssessment): boolean {
  return assessment.derivedInput?.calculationScope !== 'utility';
}

/** Label classes, best first (Omgevingsregeling bijlage IX/X). */
export const LABEL_CLASSES = ['A++++', 'A+++', 'A++', 'A+', 'A', 'B', 'C', 'D', 'E', 'F', 'G'] as const;

/** Background of a label class on the scale: green (best) via yellow to red. */
export function labelColor(labelClass: string): string {
  const index = LABEL_CLASSES.indexOf(labelClass as typeof LABEL_CLASSES[number]);
  const palette = ['#0b7a3b', '#14873f', '#1f9a45', '#36a84a', '#6cba3f', '#a6c834', '#e0cf2a', '#f2b322', '#ee8a1f', '#e2591c', '#c8281a'];
  return palette[index] ?? 'var(--surface-raised)';
}

/** White or near-black text on a label colour, whichever contrasts more (F10 contrast audit). */
export function labelTextColor(background: string): string {
  const hex = /^#([0-9a-f]{6})$/i.exec(background);
  if (!hex) return 'var(--fg-1)';
  const channel = (offset: number) => {
    const value = parseInt(hex[1].slice(offset, offset + 2), 16) / 255;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  };
  const luminance = 0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4);
  const onWhite = 1.05 / (luminance + 0.05);
  const onDark = (luminance + 0.05) / (0.0137 + 0.05); // #1d1d1d
  return onWhite >= onDark ? '#ffffff' : '#1d1d1d';
}

/** Month numbers 1..12 for the axis labels. */
export const MONTHS = Array.from({ length: 12 }, (_, index) => index + 1);

/** Sum helpers for the key-figure cards. */
export function keyFigures(assessment: ProjectPerformanceAssessment) {
  const performance = assessment.performance;
  return {
    primaryFossilKwh: performance?.annualPrimaryFossilKwh ?? null,
    renewableKwh: performance?.annualRenewablePrimaryKwh ?? null,
    finalEnergyKwh: performance?.annualFinalEnergyKwh ?? null,
    co2KgPerM2: performance?.co2KgPerM2 ?? null,
    zebKwhPerM2: performance?.zebPrimaryTotalIndicatorKwhPerM2 ?? null,
  };
}

export { ENERGY_FUNCTIONS };
