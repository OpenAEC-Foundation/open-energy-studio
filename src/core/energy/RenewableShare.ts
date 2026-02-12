// ============================================================
// Open Energy Studio – Renewable Energy Share Calculation
// ============================================================

import { PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL } from './Constants';

/**
 * Calculate the renewable energy share (BENG 3).
 *
 * Renewable sources:
 * - PV production: avoided fossil primary energy = pvProduction × fp_electricity_fossil
 * - Heat pump renewable fraction: heating extracted from environment = heatingDemand × (1 - 1/COP)
 * - Solar thermal: production × primary_factor_of_avoided_carrier
 *
 * Share = totalRenewable / (totalPrimaryEnergy + totalRenewable) × 100
 *
 * @param pvProduction - Annual PV electricity production in kWh/year
 * @param solarThermalProduction - Annual solar thermal production in kWh/year
 * @param heatPumpRenewable - Heat pump renewable fraction in kWh/year
 * @param totalPrimaryEnergy - Total fossil primary energy in kWh/year
 * @returns Renewable energy share as percentage (0-100)
 */
export function calculateRenewableShare(
  pvProduction: number,
  solarThermalProduction: number,
  heatPumpRenewable: number,
  totalPrimaryEnergy: number
): number {
  // PV: avoided fossil electricity → counted as renewable primary energy
  const pvRenewable = pvProduction * PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL;

  // Solar thermal: avoided fossil energy for hot water heating
  // Assume it replaces gas (factor 1.0), so the renewable value equals production
  const solarThermalRenewable = solarThermalProduction * 1.0;

  // Heat pump renewable: environmental energy already in kWh/year
  // No primary factor applied (it's free energy from the environment)
  const hpRenewable = heatPumpRenewable;

  const totalRenewable = pvRenewable + solarThermalRenewable + hpRenewable;

  const denominator = totalPrimaryEnergy + totalRenewable;
  if (denominator <= 0) {
    return 0;
  }

  const share = (totalRenewable / denominator) * 100;
  return Math.min(100, Math.max(0, share));
}
