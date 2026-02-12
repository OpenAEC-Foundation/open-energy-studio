// ============================================================
// Open Energy Studio – Transmission Heat Loss Calculation
// ============================================================

import type { IZone, IConstruction } from '../energy/types';
import { HEATING_DEGREE_DAYS } from './Constants';

/**
 * Calculate annual transmission heat loss through building envelope.
 *
 * For each surface: Q = U × A_net × degree_days × 0.024
 * A_net = gross area minus window areas in that surface.
 * Window losses are calculated separately using window U-values.
 * Thermal bridges are added as: psi × length × degree_days × 0.024.
 *
 * @param zones - All building zones
 * @param constructions - All construction definitions
 * @returns Total transmission heat loss in kWh/year
 */
export function calculateTransmissionLoss(
  zones: IZone[],
  constructions: IConstruction[]
): number {
  const constructionMap = new Map<string, IConstruction>();
  for (const c of constructions) {
    constructionMap.set(c.id, c);
  }

  const degreeDayFactor = HEATING_DEGREE_DAYS * 0.024; // K·d → kWh/(W) conversion
  let totalLoss = 0;

  for (const zone of zones) {
    // --- Surface losses ---
    for (const surface of zone.surfaces) {
      const construction = constructionMap.get(surface.constructionId);
      if (!construction) continue;

      // Net area = gross area minus all windows in this surface
      const windowArea = surface.windows.reduce(
        (sum, w) => sum + w.area,
        0
      );
      const netArea = Math.max(0, surface.area - windowArea);

      // Opaque surface loss: U × A_net × degree_days × 0.024
      const opaqueLoss = construction.uValue * netArea * degreeDayFactor;
      totalLoss += opaqueLoss;

      // Window losses: Uw × Aw × degree_days × 0.024
      for (const window of surface.windows) {
        const windowLoss = window.uValue * window.area * degreeDayFactor;
        totalLoss += windowLoss;
      }
    }

    // --- Thermal bridge losses ---
    for (const tb of zone.thermalBridges) {
      const tbLoss = tb.psiValue * tb.length * degreeDayFactor;
      totalLoss += tbLoss;
    }
  }

  return totalLoss;
}
