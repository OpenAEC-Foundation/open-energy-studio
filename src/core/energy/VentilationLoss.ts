// ============================================================
// Open Energy Studio – Ventilation & Infiltration Heat Loss
// ============================================================

import type { IZone, IVentilationSystem } from '../energy/types';
import {
  HEATING_DEGREE_DAYS,
  VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL,
  QV10_CORRECTION_FACTOR,
} from './Constants';

export interface VentilationLossResult {
  ventilationLoss: number;    // kWh/year
  infiltrationLoss: number;   // kWh/year
}

/**
 * Calculate annual ventilation and infiltration heat losses.
 *
 * Ventilation loss:
 *   Qv = 0.34 × qv × volume × degree_days × 0.024
 *   qv = design flow rate (0.9 dm3/(s·m2) for residential, converted to 1/s per m3)
 *   Corrected for heat recovery: multiply by (1 - recovery_efficiency)
 *
 * Infiltration loss:
 *   Qi = 0.34 × qv10_corr × volume × degree_days × 0.024
 *   qv10_corr = 0.067 × qv10 × volume  (simplified NTA 8800 approach)
 *
 * @param zones - All building zones
 * @param ventilationSystems - Ventilation systems with WTW efficiency
 * @returns Ventilation and infiltration losses in kWh/year
 */
export function calculateVentilationLoss(
  zones: IZone[],
  ventilationSystems: IVentilationSystem[]
): VentilationLossResult {
  const degreeDayFactor = HEATING_DEGREE_DAYS * 0.024;

  // Use the best available heat recovery efficiency
  // If multiple systems exist, take the weighted average (simplified: max)
  const heatRecoveryEfficiency = ventilationSystems.length > 0
    ? Math.max(...ventilationSystems.map(v => v.heatRecoveryEfficiency))
    : 0;

  let totalVentilationLoss = 0;
  let totalInfiltrationLoss = 0;

  for (const zone of zones) {
    // --- Ventilation loss ---
    // Design flow rate in dm3/s based on floor area
    const designFlowRate = VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL * zone.floorArea;
    // Convert dm3/s to m3/s: divide by 1000
    // Heat capacity of air: 0.34 Wh/(m3·K) = rho_air × cp_air / 3600
    // Q = 0.34 × qv(dm3/s) × degree_days × 0.024 × 3.6
    // Simplified: 0.34 × qv(dm3/s) × degreeDayFactor × 3.6
    // Or equivalently: 0.34 Wh/(m3·K) × (qv/1000 m3/s × 3600 s/h) × DD × 24h/d / 1000 kWh
    // = 0.34 × qv × 3.6 × DD × 0.024
    // = 0.34 × qv × DD × 0.0864
    const ventLoss =
      0.34 * designFlowRate * degreeDayFactor * 3.6 * (1 - heatRecoveryEfficiency);
    totalVentilationLoss += ventLoss;

    // --- Infiltration loss ---
    // qv10 in dm3/(s·m2) at 10 Pa
    const qv10 = zone.airTightness.qv10;
    // Corrected average infiltration volume flow in dm3/s
    const qv10Corrected = QV10_CORRECTION_FACTOR * qv10 * zone.volume;
    const infLoss = 0.34 * qv10Corrected * degreeDayFactor * 3.6;
    totalInfiltrationLoss += infLoss;
  }

  return {
    ventilationLoss: totalVentilationLoss,
    infiltrationLoss: totalInfiltrationLoss,
  };
}
