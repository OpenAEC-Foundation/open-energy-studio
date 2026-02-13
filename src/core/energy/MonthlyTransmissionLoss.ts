// ============================================================
// Open Energy Studio – Monthly Transmission Loss Calculation
// NTA 8800: Q_tr[m] = H_tr × (θ_int - θ_e[m]) × t_mi / 1000
// ============================================================

import type { IZone, IConstruction, MonthlyValues } from './types';
import {
  MONTHLY_AVERAGE_TEMPERATURE,
  HOURS_PER_MONTH,
  HEATING_SETPOINT,
} from './ClimateData';

/**
 * Calculate the transmission loss coefficient Ht (W/K).
 * This is the steady-state heat loss per degree of temperature difference.
 *
 * Ht = sum(U_surface × A_net) + sum(U_window × A_window) + sum(psi × length)
 *
 * @param zones - All building zones
 * @param constructions - All construction definitions
 * @returns Ht in W/K
 */
export function calculateTransmissionLossCoefficient(
  zones: IZone[],
  constructions: IConstruction[]
): number {
  const constructionMap = new Map<string, IConstruction>();
  for (const c of constructions) {
    constructionMap.set(c.id, c);
  }

  let ht = 0;

  for (const zone of zones) {
    for (const surface of zone.surfaces) {
      const construction = constructionMap.get(surface.constructionId);
      if (!construction) continue;

      const windowArea = surface.windows.reduce((sum, w) => sum + w.area, 0);
      const netArea = Math.max(0, surface.area - windowArea);

      // Opaque surface: U × A_net
      ht += construction.uValue * netArea;

      // Windows: Uw × Aw
      for (const window of surface.windows) {
        ht += window.uValue * window.area;
      }
    }

    // Thermal bridges: psi × length
    for (const tb of zone.thermalBridges) {
      ht += tb.psiValue * tb.length;
    }
  }

  return ht;
}

/**
 * Calculate monthly transmission losses.
 *
 * NTA 8800 monthly method:
 *   Q_tr[m] = H_tr × (θ_int - θ_e[m]) × t_mi / 1000
 *
 * where θ_int = 20°C (heating setpoint), θ_e = monthly avg outdoor temp,
 * t_mi = hours per month.
 *
 * @param zones - All building zones
 * @param constructions - All construction definitions
 * @returns Monthly transmission losses in kWh (12 values)
 */
export function calculateMonthlyTransmissionLoss(
  zones: IZone[],
  constructions: IConstruction[]
): MonthlyValues {
  const ht = calculateTransmissionLossCoefficient(zones, constructions);

  const monthly = new Array(12) as MonthlyValues;
  for (let m = 0; m < 12; m++) {
    const deltaT = Math.max(0, HEATING_SETPOINT - MONTHLY_AVERAGE_TEMPERATURE[m]);
    monthly[m] = ht * deltaT * HOURS_PER_MONTH[m] / 1000;
  }

  return monthly;
}
