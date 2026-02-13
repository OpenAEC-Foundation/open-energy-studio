// ============================================================
// Open Energy Studio – Monthly Solar Gain Calculation
// ============================================================

import type { IZone, MonthlyValues } from './types';
import { FRAME_FACTOR } from './Constants';
import { MONTHLY_SOLAR_RADIATION } from './ClimateData';

/**
 * Calculate monthly solar heat gains through all windows.
 *
 * For each window per month:
 *   Qs[m] = area × g_value × I[orientation][m] × frame_factor
 *
 * Frame factor (0.9) accounts for the opaque frame.
 *
 * @param zones - All building zones with surfaces and windows
 * @returns Monthly solar gains in kWh (12 values)
 */
export function calculateMonthlySolarGain(zones: IZone[]): MonthlyValues {
  const monthly = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  for (const zone of zones) {
    for (const surface of zone.surfaces) {
      for (const window of surface.windows) {
        const radiation = MONTHLY_SOLAR_RADIATION[window.orientation]
          ?? MONTHLY_SOLAR_RADIATION.S;

        for (let m = 0; m < 12; m++) {
          monthly[m] += window.area * window.gValue * radiation[m] * FRAME_FACTOR;
        }
      }
    }
  }

  return monthly;
}
