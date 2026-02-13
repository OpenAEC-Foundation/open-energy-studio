// ============================================================
// Open Energy Studio – Monthly Internal Gain Calculation
// ============================================================

import type { BuildingFunction, MonthlyValues } from './types';
import { INTERNAL_GAIN_POWER } from './Constants';
import { HOURS_PER_MONTH } from './ClimateData';

/**
 * Calculate monthly internal heat gains.
 *
 * Qi[m] = specific_power × floor_area × hours[m] / 1000
 *
 * @param totalFloorArea - Total heated floor area (Ag) in m²
 * @param buildingFunction - Building function type
 * @returns Monthly internal gains in kWh (12 values)
 */
export function calculateMonthlyInternalGain(
  totalFloorArea: number,
  buildingFunction: BuildingFunction
): MonthlyValues {
  const specificPower = INTERNAL_GAIN_POWER[buildingFunction]
    ?? INTERNAL_GAIN_POWER.residential;

  const monthly = new Array(12) as MonthlyValues;
  for (let m = 0; m < 12; m++) {
    monthly[m] = specificPower * totalFloorArea * HOURS_PER_MONTH[m] / 1000;
  }

  return monthly;
}
