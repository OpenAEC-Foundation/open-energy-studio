// ============================================================
// Open Energy Studio – Monthly Cooling Demand (Summer Balance)
// ============================================================

import type { MonthlyValues } from './types';

/**
 * Calculate the cooling utilization factor (inverse of heating utilization).
 *
 * For cooling, the "useful" part is the losses that offset the gains.
 * gamma_c = losses / gains (inverse of heating gamma)
 * eta_c = utilization factor for losses
 *
 * @param lossGainRatio - Ratio of losses to gains (inverse of heating gamma)
 * @returns Cooling utilization factor (0..1)
 */
function calculateCoolingUtilizationFactor(lossGainRatio: number): number {
  const a = 5; // NTA 8800 time constant parameter

  if (lossGainRatio < 0) {
    return 1;
  }

  if (Math.abs(lossGainRatio - 1) < 1e-6) {
    return a / (a + 1);
  }

  return (1 - Math.pow(lossGainRatio, a)) / (1 - Math.pow(lossGainRatio, a + 1));
}

export interface MonthlyCoolingResult {
  coolingDemand: MonthlyValues;
  utilizationFactor: MonthlyValues;
}

/**
 * Calculate monthly cooling demand using the summer gain/loss balance.
 *
 * This includes "free ventilation cooling" — in summer months, ventilation
 * air at outdoor temperature (17-18°C) below the indoor setpoint (25°C)
 * provides significant passive cooling that offsets solar/internal gains.
 *
 * For each month:
 *   totalLosses = HDD-based losses + ventilation free cooling
 *   gamma_c = totalLosses / gains
 *   eta_c = cooling utilization factor
 *   Q_cool[m] = gains[m] - eta_c × totalLosses[m]  (minimum 0)
 *
 * @param transmissionLoss - Monthly transmission losses (HDD-based, 12 values)
 * @param ventilationLoss - Monthly ventilation losses (HDD-based, 12 values)
 * @param infiltrationLoss - Monthly infiltration losses (HDD-based, 12 values)
 * @param solarGain - Monthly solar gains (12 values)
 * @param internalGain - Monthly internal gains (12 values)
 * @param ventilationFreeCooling - Monthly ventilation free cooling (12 values, optional)
 * @returns Monthly cooling demand and utilization factors
 */
export function calculateMonthlyCoolingDemand(
  transmissionLoss: MonthlyValues,
  ventilationLoss: MonthlyValues,
  infiltrationLoss: MonthlyValues,
  solarGain: MonthlyValues,
  internalGain: MonthlyValues,
  ventilationFreeCooling?: MonthlyValues
): MonthlyCoolingResult {
  const coolingDemand = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;
  const utilizationFactor = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  for (let m = 0; m < 12; m++) {
    // Standard HDD-based losses
    const hddLosses = transmissionLoss[m] + ventilationLoss[m] + infiltrationLoss[m];

    // Add free ventilation cooling in summer (ΔT between indoor setpoint and outdoor)
    const freeCooling = ventilationFreeCooling ? ventilationFreeCooling[m] : 0;
    const losses = hddLosses + freeCooling;

    const gains = solarGain[m] + internalGain[m];

    if (gains <= 0) {
      coolingDemand[m] = 0;
      utilizationFactor[m] = 1;
      continue;
    }

    const gamma_c = losses / gains;
    const eta_c = calculateCoolingUtilizationFactor(gamma_c);
    utilizationFactor[m] = eta_c;
    coolingDemand[m] = Math.max(0, gains - eta_c * losses);
  }

  return { coolingDemand, utilizationFactor };
}
