// ============================================================
// Open Energy Studio – Monthly Heating Demand Calculation
// ============================================================

import type { MonthlyValues } from './types';
import { calculateUtilizationFactor } from './HeatingDemand';

export interface MonthlyHeatingResult {
  heatingDemand: MonthlyValues;
  utilizationFactor: MonthlyValues;
}

/**
 * Calculate monthly heating demand using the gain/loss balance method.
 *
 * For each month:
 *   gamma = gains / losses
 *   eta_h = utilization factor for gains
 *   Q_heat[m] = losses[m] - eta_h × gains[m]  (minimum 0)
 *
 * @param transmissionLoss - Monthly transmission losses (12 values)
 * @param ventilationLoss - Monthly ventilation losses (12 values)
 * @param infiltrationLoss - Monthly infiltration losses (12 values)
 * @param solarGain - Monthly solar gains (12 values)
 * @param internalGain - Monthly internal gains (12 values)
 * @returns Monthly heating demand and utilization factors
 */
export function calculateMonthlyHeatingDemand(
  transmissionLoss: MonthlyValues,
  ventilationLoss: MonthlyValues,
  infiltrationLoss: MonthlyValues,
  solarGain: MonthlyValues,
  internalGain: MonthlyValues
): MonthlyHeatingResult {
  const heatingDemand = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;
  const utilizationFactor = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  for (let m = 0; m < 12; m++) {
    const losses = transmissionLoss[m] + ventilationLoss[m] + infiltrationLoss[m];
    const gains = solarGain[m] + internalGain[m];

    if (losses <= 0) {
      heatingDemand[m] = 0;
      utilizationFactor[m] = 1;
      continue;
    }

    const gamma = gains / losses;
    const eta = calculateUtilizationFactor(gamma);
    utilizationFactor[m] = eta;
    heatingDemand[m] = Math.max(0, losses - eta * gains);
  }

  return { heatingDemand, utilizationFactor };
}
