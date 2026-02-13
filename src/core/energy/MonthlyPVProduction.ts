// ============================================================
// Open Energy Studio – Monthly PV Production (NTA 8800 §16)
//
// Formula 16.2: E_PV = E_sol × P_pk × f_perf × c_sh × f_prac / I_ref
// ============================================================

import type { IProject, MonthlyValues } from './types';
import { getSolarEnergyMonthly } from './ClimateData';

// NTA 8800 Table 16.2 – Performance factor f_perf
// Depends on ventilation behind panels:
//   Not ventilated (integrated):  0.76
//   Moderate ventilation:         0.80
//   Well ventilated (free-standing): 0.82
const F_PERF = 0.80; // Default: moderate ventilation (typical roof-mount)

// NTA 8800 §16.3.2 – Practical reduction factor
const F_PRAC = 0.95;

// NTA 8800 Table 16.3 – Shading correction c_sh;PV
// For F_sh = 1.00 (no shading): c_sh = 1.00
const C_SH = 1.00;

// I_ref = 1 kW/m² (STC reference irradiance)
// Since P_pk is in kW and E_sol in kWh/m², the division by I_ref = 1 is implicit

/**
 * Calculate monthly PV electricity production per NTA 8800 §16.
 *
 * For each PV array:
 *   E_PV[m] = E_sol[m] × P_pk × f_perf × c_sh × f_prac
 *
 * where E_sol[m] is the monthly solar energy on the tilted panel surface
 * from NTA 8800 Table 17.2 (kWh/m²), and P_pk is the peak power in kW.
 *
 * @param project - Project with solar PV systems
 * @returns Monthly PV production in kWh (12 values)
 */
export function calculateMonthlyPVProduction(project: IProject): MonthlyValues {
  const monthly = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  for (const pv of project.solarPV) {
    // Get tilted radiation from NTA 8800 Table 17.2 (kWh/m² per month)
    const solarEnergy = getSolarEnergyMonthly(pv.orientation, pv.tilt);

    for (let m = 0; m < 12; m++) {
      // E_PV[m] = E_sol[m] × P_pk × f_perf × c_sh × f_prac
      monthly[m] += solarEnergy[m] * pv.peakPower * F_PERF * C_SH * F_PRAC;
    }
  }

  return monthly;
}
