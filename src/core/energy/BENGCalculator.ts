// ============================================================
// Open Energy Studio – BENG Calculator (Main Orchestrator)
// ============================================================

import type {
  IProject,
  IBENGResult,
  IEnergyBreakdown,
} from '../energy/types';
import {
  BENG_LIMITS,
  HOT_WATER_DEMAND,
  LIGHTING_ENERGY,
  SOLAR_RADIATION,
  PV_PERFORMANCE_RATIO,
  USAGE_HOURS_PER_YEAR,
} from './Constants';
import { calculateTransmissionLoss } from './TransmissionLoss';
import { calculateVentilationLoss } from './VentilationLoss';
import { calculateSolarGain } from './SolarGain';
import { calculateInternalGain } from './InternalGain';
import { calculateHeatingDemand } from './HeatingDemand';
import { calculateCoolingDemand } from './CoolingDemand';
import { calculatePrimaryEnergy } from './PrimaryEnergy';
import { calculateRenewableShare } from './RenewableShare';

/**
 * Calculate ventilation fan energy based on SFP, airflow and operating hours.
 */
function calculateVentilationFanEnergy(project: IProject, totalFloorArea: number): number {
  if (project.ventilationSystems.length === 0) return 0;

  // Design airflow in dm3/s (0.9 dm3/(s·m2) for residential)
  const designAirflow = 0.9 * totalFloorArea;

  // Average SFP across systems
  const avgSfp = project.ventilationSystems.reduce((s, v) => s + v.sfp, 0) /
    project.ventilationSystems.length;

  // Fan energy: SFP (W/(dm3/s)) × airflow (dm3/s) × hours / 1000
  return avgSfp * designAirflow * USAGE_HOURS_PER_YEAR / 1000;
}

/**
 * Calculate annual PV electricity production.
 */
function calculatePVProduction(project: IProject): number {
  let total = 0;
  for (const pv of project.solarPV) {
    const radiation = SOLAR_RADIATION[pv.orientation] ?? SOLAR_RADIATION.S;
    // Production = peak power × (radiation / 1000) × performance ratio
    // radiation is for vertical surface; adjust for tilt (simplified)
    const tiltFactor = pv.orientation === 'horizontal' ? 1.0 : 1.0; // simplified
    total += pv.peakPower * (radiation / 1000) * PV_PERFORMANCE_RATIO * tiltFactor;
  }
  return total;
}

/**
 * Calculate annual solar thermal production.
 */
function calculateSolarThermalProduction(project: IProject): number {
  let total = 0;
  for (const st of project.solarThermal) {
    const radiation = SOLAR_RADIATION[st.orientation] ?? SOLAR_RADIATION.S;
    // Collector efficiency: flat plate ~0.40, vacuum tube ~0.55
    const efficiency = st.type === 'vacuum_tube' ? 0.55 : 0.40;
    total += st.collectorArea * radiation * efficiency;
  }
  return total;
}

/**
 * Calculate heat pump renewable energy (energy extracted from environment).
 */
function calculateHeatPumpRenewable(project: IProject, heatingDemand: number): number {
  let hpRenewable = 0;
  for (const hs of project.heatingSystems) {
    if (
      (hs.type === 'heat_pump_air' || hs.type === 'heat_pump_ground') &&
      hs.cop > 1
    ) {
      // Renewable fraction = heatingDemand × coverage × (1 - 1/COP)
      hpRenewable += heatingDemand * hs.coverageFraction * (1 - 1 / hs.cop);
    }
  }
  return hpRenewable;
}

/**
 * Main BENG calculation function.
 *
 * Orchestrates all sub-calculations and produces the complete BENG result
 * including BENG 1 (energy demand), BENG 2 (primary fossil energy),
 * BENG 3 (renewable share), and a detailed energy breakdown.
 *
 * @param project - Complete project definition
 * @returns Full BENG calculation result with pass/fail per indicator
 */
export function calculateBENG(project: IProject): IBENGResult {
  // --- Total floor area ---
  const totalFloorArea = project.zones.reduce((sum, z) => sum + z.floorArea, 0);
  const safeFloorArea = Math.max(totalFloorArea, 1); // prevent division by zero

  // --- Transmission loss ---
  const transmissionLoss = calculateTransmissionLoss(project.zones, project.constructions);

  // --- Ventilation & infiltration loss ---
  const { ventilationLoss, infiltrationLoss } = calculateVentilationLoss(
    project.zones,
    project.ventilationSystems
  );

  // --- Solar gain ---
  const solarGain = calculateSolarGain(project.zones);

  // --- Internal gain ---
  const internalGain = calculateInternalGain(totalFloorArea, project.buildingFunction);

  // --- Heating demand ---
  const heatingDemand = calculateHeatingDemand(
    transmissionLoss,
    ventilationLoss,
    infiltrationLoss,
    solarGain,
    internalGain
  );

  // --- Cooling demand ---
  const coolingDemand = calculateCoolingDemand(
    transmissionLoss,
    ventilationLoss,
    infiltrationLoss,
    solarGain,
    internalGain
  );

  // --- Hot water demand ---
  const hotWaterDemand = HOT_WATER_DEMAND[project.buildingFunction] ?? HOT_WATER_DEMAND.residential;

  // --- Ventilation fan energy ---
  const ventilationEnergy = calculateVentilationFanEnergy(project, totalFloorArea);

  // --- Lighting energy ---
  const lightingSpecific = LIGHTING_ENERGY[project.buildingFunction] ?? LIGHTING_ENERGY.residential;
  const lightingEnergy = lightingSpecific * totalFloorArea;

  // --- Primary energy ---
  const primaryResult = calculatePrimaryEnergy(
    heatingDemand,
    coolingDemand,
    hotWaterDemand,
    ventilationEnergy,
    lightingEnergy,
    project.heatingSystems,
    project.coolingSystems,
    project.hotWaterSystems,
    project.ventilationSystems
  );

  // --- Renewable energy ---
  const pvProduction = calculatePVProduction(project);
  const solarThermalProduction = calculateSolarThermalProduction(project);
  const heatPumpRenewable = calculateHeatPumpRenewable(project, heatingDemand);

  const renewableShare = calculateRenewableShare(
    pvProduction,
    solarThermalProduction,
    heatPumpRenewable,
    primaryResult.totalPrimaryEnergy
  );

  const totalRenewableEnergy = pvProduction + solarThermalProduction + heatPumpRenewable;

  // --- BENG indicators ---
  const beng1 = (heatingDemand + coolingDemand) / safeFloorArea;
  const beng2 = primaryResult.totalPrimaryEnergy / safeFloorArea;
  const beng3 = renewableShare;

  // --- Limits ---
  const limits = BENG_LIMITS[project.buildingFunction] ?? BENG_LIMITS.residential;

  // --- Energy breakdown ---
  const breakdown: IEnergyBreakdown = {
    transmissionLoss,
    ventilationLoss,
    infiltrationLoss,
    solarGain,
    internalGain,
    heatingDemand,
    coolingDemand,
    heatingEnergy: primaryResult.heatingEnergy,
    coolingEnergy: primaryResult.coolingEnergy,
    ventilationEnergy: primaryResult.ventilationEnergy,
    hotWaterEnergy: primaryResult.hotWaterEnergy,
    lightingEnergy: primaryResult.lightingEnergy,
    auxiliaryEnergy: 0,
    totalPrimaryEnergy: primaryResult.totalPrimaryEnergy,
    renewableEnergy: totalRenewableEnergy,
    pvProduction,
    solarThermalProduction,
  };

  return {
    beng1,
    beng2,
    beng3,
    beng1Limit: limits.beng1,
    beng2Limit: limits.beng2,
    beng3Limit: limits.beng3,
    beng1Pass: beng1 <= limits.beng1,
    beng2Pass: beng2 <= limits.beng2,
    beng3Pass: beng3 >= limits.beng3,
    breakdown,
    totalFloorArea,
  };
}
