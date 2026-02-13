// ============================================================
// Open Energy Studio – Primary Energy Calculation
// ============================================================

import type {
  IHeatingSystem,
  ICoolingSystem,
  IHotWaterSystem,
  IVentilationSystem,
  HeatingSystemType,
  HotWaterSystemType,
} from '../energy/types';
import {
  PRIMARY_ENERGY_FACTOR_GAS,
  PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL,
  PRIMARY_ENERGY_FACTOR_DISTRICT_HEATING,
  PRIMARY_ENERGY_FACTOR_BIOMASS,
  DISTRIBUTION_LOSS_FACTOR_HEATING,
  DISTRIBUTION_LOSS_FACTOR_HOT_WATER,
} from './Constants';

export interface PrimaryEnergyResult {
  heatingEnergy: number;      // kWh/year delivered
  coolingEnergy: number;      // kWh/year delivered
  ventilationEnergy: number;  // kWh/year fan energy
  hotWaterEnergy: number;     // kWh/year delivered
  lightingEnergy: number;     // kWh/year
  totalPrimaryEnergy: number; // kWh/year fossil primary
}

/**
 * Determine the primary energy factor for a heating system type.
 * Heat pumps and electric systems use electricity; HR boilers use gas.
 */
function getHeatingPrimaryFactor(systemType: HeatingSystemType): number {
  switch (systemType) {
    case 'heat_pump_air':
    case 'heat_pump_ground':
    case 'electric':
      return PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL;
    case 'district_heating':
      return PRIMARY_ENERGY_FACTOR_DISTRICT_HEATING;
    case 'biomass':
      return PRIMARY_ENERGY_FACTOR_BIOMASS;
    case 'hr107':
    case 'hr_combi':
    default:
      return PRIMARY_ENERGY_FACTOR_GAS;
  }
}

/**
 * Determine the primary energy factor for a hot water system type.
 */
function getHotWaterPrimaryFactor(systemType: HotWaterSystemType): number {
  switch (systemType) {
    case 'heat_pump':
    case 'electric_boiler':
      return PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL;
    case 'district_heating':
      return PRIMARY_ENERGY_FACTOR_DISTRICT_HEATING;
    case 'solar_boiler':
    case 'hr_combi':
    default:
      return PRIMARY_ENERGY_FACTOR_GAS;
  }
}

/**
 * Calculate delivered and primary fossil energy for all end uses.
 *
 * - Heating: heatingDemand / system COP, weighted by coverage fraction
 * - Cooling: coolingDemand / system EER (0 if no cooling system)
 * - Hot water: hotWaterDemand / system efficiency
 * - Ventilation: fan energy from SFP and airflow
 * - Lighting: direct input
 *
 * Total primary = sum of (delivered energy × primary_energy_factor) per carrier.
 *
 * @returns Delivered energies and total fossil primary energy in kWh/year
 */
export function calculatePrimaryEnergy(
  heatingDemand: number,
  coolingDemand: number,
  hotWaterDemand: number,
  ventilationEnergy: number,
  lightingEnergy: number,
  heatingSystems: IHeatingSystem[],
  coolingSystems: ICoolingSystem[],
  hotWaterSystems: IHotWaterSystem[],
  _ventilationSystems: IVentilationSystem[]
): PrimaryEnergyResult {
  // --- Heating delivered energy ---
  let heatingDelivered = 0;
  let heatingPrimary = 0;

  // Include distribution losses (NTA 8800 §10: pipe/duct losses)
  const heatingWithDistribution = heatingDemand * (1 + DISTRIBUTION_LOSS_FACTOR_HEATING);

  if (heatingSystems.length > 0) {
    // Normalize coverage fractions
    const totalCoverage = heatingSystems.reduce((s, h) => s + h.coverageFraction, 0);

    for (const system of heatingSystems) {
      const fraction = totalCoverage > 0 ? system.coverageFraction / totalCoverage : 1 / heatingSystems.length;
      const cop = Math.max(system.cop, 0.1); // prevent division by zero
      const delivered = (heatingWithDistribution * fraction) / cop;
      heatingDelivered += delivered;
      heatingPrimary += delivered * getHeatingPrimaryFactor(system.type);
    }
  } else {
    // Default: assume gas HR boiler with COP 0.95
    heatingDelivered = heatingWithDistribution / 0.95;
    heatingPrimary = heatingDelivered * PRIMARY_ENERGY_FACTOR_GAS;
  }

  // --- Cooling delivered energy ---
  let coolingDelivered = 0;
  let coolingPrimary = 0;

  const activeCoolingSystems = coolingSystems.filter(c => c.type !== 'none');
  if (activeCoolingSystems.length > 0) {
    // Average EER across systems
    const avgEer = activeCoolingSystems.reduce((s, c) => s + c.eer, 0) / activeCoolingSystems.length;
    const eer = Math.max(avgEer, 0.1);
    coolingDelivered = coolingDemand / eer;
    // Cooling systems are assumed to be electric
    coolingPrimary = coolingDelivered * PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL;
  }

  // --- Hot water delivered energy ---
  let hotWaterDelivered = 0;
  let hotWaterPrimary = 0;

  // Include hot water distribution losses (NTA 8800 §13.3: pipe losses)
  const hotWaterWithDistribution = hotWaterDemand * (1 + DISTRIBUTION_LOSS_FACTOR_HOT_WATER);

  if (hotWaterSystems.length > 0) {
    // Each system covers an equal share of the hot water demand
    const sharePerSystem = 1 / hotWaterSystems.length;
    for (const system of hotWaterSystems) {
      const efficiency = Math.max(system.efficiency, 0.1);
      // Reduce demand by solar boiler fraction
      const systemDemand = hotWaterWithDistribution * sharePerSystem * (1 - system.solarBoilerFraction);
      const delivered = systemDemand / efficiency;
      hotWaterDelivered += delivered;
      hotWaterPrimary += delivered * getHotWaterPrimaryFactor(system.type);
    }
  } else {
    // Default: gas HR combi with efficiency 0.85
    hotWaterDelivered = hotWaterWithDistribution / 0.85;
    hotWaterPrimary = hotWaterDelivered * PRIMARY_ENERGY_FACTOR_GAS;
  }

  // --- Ventilation fan energy ---
  // ventilationEnergy is pre-calculated externally
  const ventilationPrimary = ventilationEnergy * PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL;

  // --- Lighting energy ---
  const lightingPrimary = lightingEnergy * PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL;

  // --- Total fossil primary energy ---
  const totalPrimaryEnergy =
    heatingPrimary +
    coolingPrimary +
    hotWaterPrimary +
    ventilationPrimary +
    lightingPrimary;

  return {
    heatingEnergy: heatingDelivered,
    coolingEnergy: coolingDelivered,
    ventilationEnergy,
    hotWaterEnergy: hotWaterDelivered,
    lightingEnergy,
    totalPrimaryEnergy,
  };
}
