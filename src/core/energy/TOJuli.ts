// ============================================================
// Open Energy Studio – TO-juli (Summer Comfort) Calculation
// GTO: Gewogen Temperatuur Overschrijding
// ============================================================

import type { IProject, ITOJuliResult, MonthlyValues } from './types';
import { TO_JULI_LIMIT, TO_JULI_BASE_TEMP } from './Constants';
import { MONTHLY_AVERAGE_TEMPERATURE, HOURS_PER_MONTH } from './ClimateData';

/**
 * Calculate the GTO (Gewogen Temperatuur Overschrijding) for summer comfort.
 *
 * The GTO estimates the risk of overheating during summer months.
 * It considers the indoor temperature rise above a base temperature (25°C),
 * weighted by the number of hours in each month.
 *
 * Simplified NTA 8800 approach:
 *   For each month, estimate indoor temperature based on:
 *   - Outdoor temperature
 *   - Solar gains (how much heat enters)
 *   - Thermal mass / ventilation capacity (how well heat is dissipated)
 *
 *   GTO = sum of weighted overheating hours / total hours
 *
 * For residential: GTO limit = 1.20
 *
 * @param project - Project data
 * @param solarGain - Monthly solar gains in kWh
 * @param totalFloorArea - Total heated floor area (Ag)
 * @returns TO-juli result with GTO value, limit, and pass/fail
 */
export function calculateTOJuli(
  _project: IProject,
  solarGain: MonthlyValues,
  totalFloorArea: number
): ITOJuliResult {
  const safeArea = Math.max(totalFloorArea, 1);
  const monthlyRisk = new Array(12).fill(0);

  // Summer months (May-Sep, indices 4-8) are most relevant
  let totalWeightedOverheating = 0;
  let totalSummerHours = 0;

  for (let m = 0; m < 12; m++) {
    const tOutdoor = MONTHLY_AVERAGE_TEMPERATURE[m];
    const hours = HOURS_PER_MONTH[m];

    // Estimate indoor temperature contribution from solar gains
    // Solar gain in kWh/month → average power in W = gain * 1000 / hours
    const solarPower = (solarGain[m] * 1000) / hours; // W
    // Temperature rise due to solar = power / (effective thermal capacity)
    // Simplified: assume ~50 W/K effective dissipation capacity per 100m²
    const thermalCapacity = safeArea * 0.5; // W/K (simplified)
    const solarTempRise = thermalCapacity > 0 ? solarPower / thermalCapacity : 0;

    // Estimated indoor temperature
    const tIndoor = tOutdoor + solarTempRise + 2; // +2°C for internal gains

    // Overheating: excess above base temperature
    const excess = Math.max(0, tIndoor - TO_JULI_BASE_TEMP);
    monthlyRisk[m] = excess;

    // Weight by hours (summer months May-Sep)
    if (m >= 4 && m <= 8) {
      totalWeightedOverheating += excess * hours;
      totalSummerHours += hours;
    }
  }

  // GTO = weighted average overheating during summer period
  const gto = totalSummerHours > 0
    ? totalWeightedOverheating / totalSummerHours
    : 0;

  return {
    gto: Math.round(gto * 100) / 100,
    limit: TO_JULI_LIMIT,
    pass: gto <= TO_JULI_LIMIT,
    monthlyRisk,
  };
}
