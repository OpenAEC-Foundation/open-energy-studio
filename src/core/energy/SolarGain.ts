// ============================================================
// Open Energy Studio – Solar Heat Gain Calculation
// ============================================================

import type { IZone } from '../energy/types';
import { SOLAR_RADIATION, FRAME_FACTOR } from './Constants';

/**
 * Calculate annual solar heat gain through all windows.
 *
 * For each window:
 *   Qs = area × g_value × radiation_for_orientation × frame_factor
 *
 * The frame factor (0.9) accounts for the opaque frame reducing the
 * transparent area relative to the total window area.
 *
 * @param zones - All building zones with their surfaces and windows
 * @returns Total solar gain in kWh/year
 */
export function calculateSolarGain(zones: IZone[]): number {
  let totalSolarGain = 0;

  for (const zone of zones) {
    for (const surface of zone.surfaces) {
      for (const window of surface.windows) {
        const radiation = SOLAR_RADIATION[window.orientation] ?? SOLAR_RADIATION.S;
        const gain = window.area * window.gValue * radiation * FRAME_FACTOR;
        totalSolarGain += gain;
      }
    }
  }

  return totalSolarGain;
}
