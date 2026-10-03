import type { NtaEnergyFunction, NtaServiceEnergyBreakdown } from './KernelClient';

/** Energy functions of 5.20 in report order. */
export const ENERGY_FUNCTIONS: NtaEnergyFunction[] = [
  'heating', 'hotWater', 'cooling', 'humidification', 'ventilation', 'lighting', 'auxiliary', 'heatPumpSource',
];

/** Carriers of 5.20 in report order. */
export const SERVICE_CARRIERS = ['el', 'gas', 'oil', 'bm', 'dh', 'dw', 'dc'] as const;

export interface ServiceEnergyRow {
  service: NtaEnergyFunction;
  /** Annual E_EPus;ci per carrier, kWh. */
  usedKwh: Record<string, number>;
  deliveredKwh: number;
  primaryFossilKwh: number;
  renewablePrimaryKwh: number;
}

export interface ServiceEnergySummary {
  rows: ServiceEnergyRow[];
  /** Carriers that occur in the breakdown. */
  carriers: string[];
  exportedElectricityCreditKwh: number;
  storageCorrectionKwh: number;
  renewableElectricityKwh: number;
  /** Σ primary fossil − export credit − storage correction = EPTot. */
  primaryFossilTotalKwh: number;
  /** Σ renewable + renewable electricity = EPrenTot. */
  renewableTotalKwh: number;
}

/** Annual §5.5.3 table: one row per energy function with any energy. */
export function summarizeServiceEnergy(breakdown: NtaServiceEnergyBreakdown | null | undefined): ServiceEnergySummary | null {
  if (!breakdown || breakdown.months.length === 0) return null;
  const rows = ENERGY_FUNCTIONS.map((service): ServiceEnergyRow => {
    const usedKwh: Record<string, number> = {};
    let deliveredKwh = 0;
    let primaryFossilKwh = 0;
    for (const item of breakdown.annual.filter((row) => row.service === service)) {
      usedKwh[item.carrier] = (usedKwh[item.carrier] ?? 0) + item.usedKwh;
      deliveredKwh += item.deliveredKwh;
      primaryFossilKwh += item.primaryFossilKwh;
    }
    const renewablePrimaryKwh = breakdown.renewable
      .filter((row) => row.service === service)
      .reduce((sum, row) => sum + row.renewablePrimaryKwh, 0);
    return { service, usedKwh, deliveredKwh, primaryFossilKwh, renewablePrimaryKwh };
  }).filter((row) => Object.values(row.usedKwh).some((value) => value !== 0) || row.renewablePrimaryKwh !== 0);
  const carriers = SERVICE_CARRIERS.filter((carrier) => rows.some((row) => (row.usedKwh[carrier] ?? 0) !== 0));
  const sum = (field: 'exportedElectricityCreditKwh' | 'storageCorrectionKwh' | 'renewableElectricityKwh') =>
    breakdown.adjustments.reduce((total, item) => total + item[field], 0);
  const exportedElectricityCreditKwh = sum('exportedElectricityCreditKwh');
  const storageCorrectionKwh = sum('storageCorrectionKwh');
  const renewableElectricityKwh = sum('renewableElectricityKwh');
  return {
    rows,
    carriers,
    exportedElectricityCreditKwh,
    storageCorrectionKwh,
    renewableElectricityKwh,
    primaryFossilTotalKwh: rows.reduce((total, row) => total + row.primaryFossilKwh, 0)
      - exportedElectricityCreditKwh - storageCorrectionKwh,
    renewableTotalKwh: rows.reduce((total, row) => total + row.renewablePrimaryKwh, 0) + renewableElectricityKwh,
  };
}
