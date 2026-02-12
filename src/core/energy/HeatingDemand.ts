// ============================================================
// Open Energy Studio – Heating Demand Calculation
// ============================================================

/**
 * Calculate the utilization factor for heat gains according to NTA 8800.
 *
 * eta = (1 - gamma^a) / (1 - gamma^(a+1))
 * where gamma = gains / losses and a = time constant parameter (5 for monthly)
 *
 * For gamma = 1: eta = a / (a + 1)
 *
 * @param gainLossRatio - Ratio of total gains to total losses
 * @returns Utilization factor (0..1)
 */
function calculateUtilizationFactor(gainLossRatio: number): number {
  const a = 5; // NTA 8800 time constant parameter

  if (gainLossRatio < 0) {
    return 1;
  }

  // For ratio very close to 1, use the limit value
  if (Math.abs(gainLossRatio - 1) < 1e-6) {
    return a / (a + 1);
  }

  return (1 - Math.pow(gainLossRatio, a)) / (1 - Math.pow(gainLossRatio, a + 1));
}

/**
 * Calculate annual net heating demand using the gain/loss balance method.
 *
 * heatingDemand = losses - eta × gains
 * where eta is the utilization factor for heat gains.
 *
 * @param transmissionLoss - Transmission heat loss in kWh/year
 * @param ventilationLoss - Ventilation heat loss in kWh/year
 * @param infiltrationLoss - Infiltration heat loss in kWh/year
 * @param solarGain - Solar heat gain in kWh/year
 * @param internalGain - Internal heat gain in kWh/year
 * @returns Net heating demand in kWh/year (minimum 0)
 */
export function calculateHeatingDemand(
  transmissionLoss: number,
  ventilationLoss: number,
  infiltrationLoss: number,
  solarGain: number,
  internalGain: number
): number {
  const gains = solarGain + internalGain;
  const losses = transmissionLoss + ventilationLoss + infiltrationLoss;

  if (losses <= 0) {
    return 0;
  }

  const gainLossRatio = gains / losses;
  const utilizationFactor = calculateUtilizationFactor(gainLossRatio);
  const heatingDemand = losses - utilizationFactor * gains;

  return Math.max(0, heatingDemand);
}
