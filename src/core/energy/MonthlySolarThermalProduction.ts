// ============================================================
// Open Energy Studio – Monthly Solar Thermal Production
// Uses NTA 8800 Table 17.2 tilted radiation data
// ============================================================

import type { IProject, MonthlyValues } from './types';
import { getSolarEnergyMonthly } from './ClimateData';

/**
 * Calculate monthly solar thermal production.
 *
 * For each collector per month:
 *   Est[m] = collectorArea × E_sol[orientation, tilt][m] × efficiency
 *
 * Uses actual tilted radiation from NTA 8800 Table 17.2.
 *
 * Efficiency:
 *   - Flat plate: 0.40
 *   - Vacuum tube: 0.55
 *
 * @param project - Project with solar thermal systems
 * @returns Monthly solar thermal production in kWh (12 values)
 */
export function calculateMonthlySolarThermalProduction(project: IProject): MonthlyValues {
  const monthly = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  for (const st of project.solarThermal) {
    const solarEnergy = getSolarEnergyMonthly(st.orientation, st.tilt);
    const efficiency = st.type === 'vacuum_tube' ? 0.55 : 0.40;

    for (let m = 0; m < 12; m++) {
      monthly[m] += st.collectorArea * solarEnergy[m] * efficiency;
    }
  }

  return monthly;
}
