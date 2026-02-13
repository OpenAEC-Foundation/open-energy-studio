// ============================================================
// Open Energy Studio – Monthly Ventilation & Infiltration Loss
// NTA 8800: Q_v[m] = H_v × (θ_int - θ_e[m]) × t_mi / 1000
// ============================================================

import type { IZone, IVentilationSystem, MonthlyValues } from './types';
import {
  VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL,
  QV10_CORRECTION_FACTOR,
} from './Constants';
import {
  MONTHLY_AVERAGE_TEMPERATURE,
  HOURS_PER_MONTH,
  HEATING_SETPOINT,
} from './ClimateData';

export interface MonthlyVentilationLossResult {
  ventilationLoss: MonthlyValues;    // kWh per month
  infiltrationLoss: MonthlyValues;   // kWh per month
}

/**
 * Calculate monthly ventilation and infiltration heat losses.
 *
 * NTA 8800 monthly method:
 *   Qv[m] = Hv × (θ_int - θ_e[m]) × t_mi / 1000 × (1 - η_wtw)
 *   Qi[m] = Hi × (θ_int - θ_e[m]) × t_mi / 1000
 *
 * where:
 *   Hv = 0.34 × qv_design × 3.6  (ventilation heat transfer coeff, W/K)
 *   Hi = 0.34 × qv10_corr × 3.6  (infiltration heat transfer coeff, W/K)
 *   θ_int = 20°C (heating setpoint)
 *   η_wtw = heat recovery efficiency (only for type D systems)
 *
 * @param zones - All building zones
 * @param ventilationSystems - Ventilation systems
 * @returns Monthly ventilation and infiltration losses
 */
export function calculateMonthlyVentilationLoss(
  zones: IZone[],
  ventilationSystems: IVentilationSystem[]
): MonthlyVentilationLossResult {
  // Determine effective heat recovery efficiency based on system type
  let heatRecoveryEfficiency = 0;

  if (ventilationSystems.length > 0) {
    // Only type D (balanced) systems can have heat recovery
    const typeDSystems = ventilationSystems.filter(v => v.type === 'type_d');
    if (typeDSystems.length > 0) {
      heatRecoveryEfficiency = Math.max(
        ...typeDSystems.map(v => v.heatRecoveryEfficiency)
      );
    }
  }

  const ventLoss = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;
  const infLoss = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  for (const zone of zones) {
    const designFlowRate = VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL * zone.floorArea;
    const qv10 = zone.airTightness.qv10;
    const qv10Corrected = QV10_CORRECTION_FACTOR * qv10 * zone.volume;

    // Heat transfer coefficients (W/K)
    const hv = 0.34 * designFlowRate * 3.6; // ventilation
    const hi = 0.34 * qv10Corrected * 3.6;  // infiltration

    for (let m = 0; m < 12; m++) {
      const deltaT = Math.max(0, HEATING_SETPOINT - MONTHLY_AVERAGE_TEMPERATURE[m]);
      const factor = deltaT * HOURS_PER_MONTH[m] / 1000;

      // Ventilation loss (reduced by heat recovery)
      ventLoss[m] += hv * factor * (1 - heatRecoveryEfficiency);

      // Infiltration loss (no heat recovery)
      infLoss[m] += hi * factor;
    }
  }

  return {
    ventilationLoss: ventLoss,
    infiltrationLoss: infLoss,
  };
}
