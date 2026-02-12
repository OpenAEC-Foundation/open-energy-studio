// ============================================================
// Open Energy Studio – Internal Heat Gain Calculation
// ============================================================

import type { BuildingFunction } from '../energy/types';
import { INTERNAL_GAIN_POWER, USAGE_HOURS_PER_YEAR } from './Constants';

/**
 * Calculate annual internal heat gains from occupants, equipment, and lighting.
 *
 * Qi = specific_power (W/m2) × floor_area (m2) × usage_hours (h/year) / 1000
 *
 * For residential: 5 W/m2 × 8760 h / 1000 = 43.8 kWh/(m2·year)
 *
 * @param totalFloorArea - Total heated floor area (Ag) in m2
 * @param buildingFunction - Building function type
 * @returns Internal heat gain in kWh/year
 */
export function calculateInternalGain(
  totalFloorArea: number,
  buildingFunction: BuildingFunction
): number {
  const specificPower = INTERNAL_GAIN_POWER[buildingFunction] ?? INTERNAL_GAIN_POWER.residential;
  return specificPower * totalFloorArea * USAGE_HOURS_PER_YEAR / 1000;
}
