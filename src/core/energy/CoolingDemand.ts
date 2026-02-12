// ============================================================
// Open Energy Studio – Cooling Demand Calculation
// ============================================================

/**
 * Calculate annual cooling demand.
 *
 * Simplified approach for residential buildings:
 *   coolingDemand = 0.1 × (solarGain + internalGain)
 *
 * This is a simplified NTA 8800 approach; a full calculation would use
 * monthly gain/loss balances with cooling utilization factors for the
 * summer period.
 *
 * @param _transmissionLoss - Transmission heat loss in kWh/year (reserved for future use)
 * @param _ventilationLoss - Ventilation heat loss in kWh/year (reserved for future use)
 * @param _infiltrationLoss - Infiltration heat loss in kWh/year (reserved for future use)
 * @param solarGain - Solar heat gain in kWh/year
 * @param internalGain - Internal heat gain in kWh/year
 * @returns Cooling demand in kWh/year
 */
export function calculateCoolingDemand(
  _transmissionLoss: number,
  _ventilationLoss: number,
  _infiltrationLoss: number,
  solarGain: number,
  internalGain: number
): number {
  // Simplified: 10% of total gains contribute to cooling demand
  const coolingDemand = 0.1 * (solarGain + internalGain);
  return Math.max(0, coolingDemand);
}
